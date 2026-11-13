#!/usr/bin/env python3
"""Materialize the preregistered G3 pilot prompts and scoring oracle."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent
MODEL_PATH = Path("/tmp/aic-g3-pilot-2026-11-13/Qwen3-4B-Q4_K_M.gguf")
MODEL_REVISION = "bc640142c66e1fdd12af0bd68f40445458f3869b"
MODEL_SHA256 = "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5"

COMMON_PROMPT = (
    "提示されたprogramに指定taskを適用してください。要求された変更を実施し、要求に含まれない意味、操作、識別子は保ってください。\n"
    "回答は指定形式の完全なprogramを1つだけ返し、説明文やMarkdown fenceを付けないでください。"
)

AIC_CARD = """AIC/0.2の完全なmoduleを一つ返してください。出力形は次のとおりです。
AIC 0.2
fn aic_main(%x: u64) -> u64 {
^entry:
  %id = const.u64 DECIMAL
  %id = add.wrap.u64 %lhs, %rhs
  %id = sub.wrap.u64 %lhs, %rhs
  ret.u64 %id
}
const.u64行、add.wrap.u64行、sub.wrap.u64行は必要な数だけ使い、各instruction/terminatorは1物理行に一つ置いてください。add.wrap.u64とsub.wrap.u64はmodulo 2^64です。value IDは[a-z][a-z0-9_]{0,62}、parameterとinstruction definitionは互いに異なる一意なID、参照では%を付け、definitionより前のuseを作らないでください。id/lhs/rhs/DECIMALは説明用placeholderなので実際のID/数値へ置き換え、placeholderそのものは出力しないでください。DECIMALは`0`、または1〜20桁で1〜9から始まり、値が18446744073709551615以下の符号なし10進数です。追加のfunction、block、operation、commentは出力しないでください。"""

JSON_CARD = """AIC-J/0.1の完全なJSON objectを一つ返してください。top-levelはformat="AIC-J/0.1"とfunctionだけです。function keyはname、params、result、blocksだけで、name="aic_main"、paramsは[{"id":"x","type":"u64"}]、result="u64"、blocksはid="entry"の一要素です。block keyはid、ops、termだけです。opsは以下のshapeを使います：const `{ "id": ID, "op": "const.u64", "value": DECIMAL_STRING }`、add `{ "id": ID, "op": "add.wrap.u64", "args": [ID, ID] }`、sub `{ "id": ID, "op": "sub.wrap.u64", "args": [ID, ID] }`。add.wrap.u64とsub.wrap.u64はmodulo 2^64です。termは`{ "op": "ret.u64", "value": ID }`だけです。IDは[a-z][a-z0-9_]{0,62}、parameterとinstruction definitionは互いに異なる一意なID、JSONでは%や^を付けません。IDとDECIMAL_STRINGは説明用placeholderなので実際の値に置き換え、placeholderそのものは出力しないでください。DECIMAL_STRINGは`0`、または1〜20桁で1〜9から始まり、値が18446744073709551615以下の符号なし10進数です。array順を保ち、definitionより前のuseを作らないでください。duplicate key、unknown key/operation、型やarityの誤り、重複ID、未解決参照、範囲外定数は不正です。"""

INITIAL_AIC = """AIC 0.2
fn aic_main(%x: u64) -> u64 {
^entry:
  %one = const.u64 1
  %sum = add.wrap.u64 %x, %one
  ret.u64 %sum
}"""

INITIAL_JSON = '''{
  "format": "AIC-J/0.1",
  "function": {
    "name": "aic_main",
    "params": [{"id": "x", "type": "u64"}],
    "result": "u64",
    "blocks": [
      {
        "id": "entry",
        "ops": [
          {"id": "one", "op": "const.u64", "value": "1"},
          {"id": "sum", "op": "add.wrap.u64", "args": ["x", "one"]}
        ],
        "term": {"op": "ret.u64", "value": "sum"}
      }
    ]
  }
}'''

TASKS = [
    ("E1", "identifier oneで定義されている定数を1から17に変更してください。ほかの操作は変更しないでください。"),
    ("E2", "sumの演算をwrap減算に変えてください。operandと定数は変更しないでください。"),
    ("E3", "現在のsumの値から2を引いた値を返してください。定数をidentifier two、結果をidentifier resultで追加してください。"),
    ("E4", "sumという値identifierをanswerへ変更し、その定義とすべての参照を更新してください。演算と定数は変えないでください。"),
    ("E5", "入力値に関係なく7を返すprogramに変更し、計算に不要になった操作を削除してください。定数identifierはsevenにしてください。"),
    ("E6", "定数oneを4に変え、現在のsumに2を足した値を返してください。追加する定数と結果のidentifierはtwoとresultにしてください。"),
]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_bytes(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def json_bytes(value: object) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")


def expected_for(task_id: str) -> tuple[list[dict[str, object]], list[str]]:
    const = lambda ident, value: {"id": ident, "op": "const.u64", "value": str(value)}
    add = lambda ident, lhs, rhs: {"id": ident, "op": "add.wrap.u64", "args": [lhs, rhs]}
    sub = lambda ident, lhs, rhs: {"id": ident, "op": "sub.wrap.u64", "args": [lhs, rhs]}
    ret = lambda value: {"op": "ret.u64", "value": value}
    specs: dict[str, tuple[list[dict[str, object]], list[str]]] = {
        "E1": ([const("one", 17), add("sum", "x", "one")], ["i0.value"]),
        "E2": ([const("one", 1), sub("sum", "x", "one")], ["i1.op"]),
        "E3": ([const("one", 1), add("sum", "x", "one"), const("two", 2), sub("result", "sum", "two")], ["i2.*", "i3.*", "term.value"]),
        "E4": ([const("one", 1), add("answer", "x", "one")], ["i1.id", "term.value"]),
        "E5": ([const("seven", 7)], ["*"]),
        "E6": ([const("one", 4), add("sum", "x", "one"), const("two", 2), add("result", "sum", "two")], ["i0.value", "i2.*", "i3.*", "term.value"]),
    }
    ops, allowed = specs[task_id]
    term = {"op": "ret.u64", "value": {"E1": "sum", "E2": "sum", "E3": "result", "E4": "answer", "E5": "seven", "E6": "result"}[task_id]}
    return ops + [term], allowed


def make_fixed_inputs() -> list[int]:
    values = [0, 1, 2, 17, 2**64 - 2, 2**64 - 1]
    for i in range(16):
        payload = b"AIC-G3-edit-pilot-v1\0" + i.to_bytes(4, "big")
        values.append(int.from_bytes(hashlib.sha256(payload).digest()[:8], "big"))
    assert len(values) == 22 and len(set(values)) == 22
    return values


def main() -> None:
    assets = ROOT / "prompt_assets"
    packet_dir = ROOT / "prompt_packets"
    raw_assets = {
        "common_prompt.txt": COMMON_PROMPT,
        "aic_schema_card.txt": AIC_CARD,
        "json_schema_card.txt": JSON_CARD,
        "initial_aic.aic": INITIAL_AIC,
        "initial_json.json": INITIAL_JSON,
        "system_message.txt": "/no_think",
    }
    for name, text in raw_assets.items():
        write_bytes(assets / name, text.encode("utf-8"))

    prompt_entries = []
    for task_id, request in TASKS:
        for form, card, initial in (("aic", AIC_CARD, INITIAL_AIC), ("json", JSON_CARD, INITIAL_JSON)):
            packet = f"{COMMON_PROMPT}\n\n{card}\n\n{request}\n\n{initial}".encode("utf-8")
            path = packet_dir / task_id / f"{form}.txt"
            write_bytes(path, packet)
            prompt_entries.append({
                "task_id": task_id,
                "format": form,
                "path": str(path.relative_to(ROOT)),
                "request": request,
                "sha256": sha256(packet),
                "byte_count": len(packet),
            })

    oracle_tasks = {}
    for task_id, request in TASKS:
        expected, allowed = expected_for(task_id)
        oracle_tasks[task_id] = {"request": request, "expected_operations": expected, "allowed_target_fields": allowed}
    oracle = {"oracle_version": "g3-modular-u64-v1", "tasks": oracle_tasks}
    oracle_path = ROOT / "oracle.json"
    write_bytes(oracle_path, json_bytes(oracle))
    fixed_inputs = {
        "formula": "fixed {0,1,2,17,2^64-2,2^64-1} plus SHA256(ASCII(AIC-G3-edit-pilot-v1)||0x00||uint32_be(i))[:8] as unsigned big-endian, i=0..15",
        "values_u64": make_fixed_inputs(),
    }
    inputs_path = ROOT / "fixed_inputs.json"
    write_bytes(inputs_path, json_bytes(fixed_inputs))

    tracked = [
        *[assets / name for name in raw_assets],
        *[ROOT / entry["path"] for entry in prompt_entries],
        oracle_path,
        inputs_path,
        ROOT / "prepare_pilot.py",
        ROOT / "run_pilot.py",
        ROOT / "score_pilot.py",
        ROOT / "rust-helper" / "Cargo.toml",
        ROOT / "rust-helper" / "Cargo.lock",
        ROOT / "rust-helper" / "src" / "main.rs",
    ]
    file_hashes = {str(path.relative_to(ROOT)): sha256(path.read_bytes()) for path in sorted(tracked)}
    production_files = [
        "Cargo.toml", "Cargo.lock", "src/parser.rs", "src/validate.rs", "src/ir.rs",
        "src/driver.rs", "src/x86_64.rs", "tests/harness.cpp", "tests/abi_probe.S",
    ]
    manifest = {
        "experiment": "G3 single-block local-edit pilot",
        "date": "2026-11-13",
        "status": "preregistered; ready for first generation call",
        "scope": {
            "control": "adopted AIC/0.2",
            "candidate": "unadopted AIC-J/0.1 pilot-only profile",
            "tasks": [task_id for task_id, _ in TASKS],
            "replicates_per_task": 10,
            "paired_runs": 60,
            "max_first_response_calls": 120,
            "repair_turns": 0,
        },
        "model": {
            "repository": "Qwen/Qwen3-4B-GGUF",
            "revision": MODEL_REVISION,
            "filename": MODEL_PATH.name,
            "path": str(MODEL_PATH),
            "license": "Apache-2.0 (pinned repository model card)",
            "sha256": MODEL_SHA256,
            "size_bytes": 2497280256,
            "provider": "local CPU; no remote inference or token billing",
            "runner": "llama-cpp-python==0.3.16",
            "threads": 5,
            "context_tokens": 4096,
            "max_output_tokens": 1024,
            "system_message": "/no_think",
            "temperature": 0.7,
            "top_p": 0.8,
            "top_k": 20,
            "min_p": 0.0,
            "presence_penalty": 1.5,
            "frequency_penalty": 0.0,
            "repeat_penalty": 1.0,
            "mirostat_mode": 0,
            "stream": False,
            "tools": False,
            "web_access": False,
        },
        "randomization": {
            "pair_seed": "uint32_be(SHA256(ASCII(AIC-G3-pilot-seed-v1)||0x00||task_id||0x00||uint32_be(replicate))[:4])",
            "order_bit": "SHA256(ASCII(AIC-G3-pilot-order-v1)||0x00||task_id||0x00||uint32_be(replicate))[0] & 1; 0=AIC first, 1=JSON first",
            "same_seed_within_pair": True,
        },
        "budget": {"generation_calls_max": 120, "wall_clock_hours_max": 4, "model_download_bytes_max": 2497280256, "inference_charge": 0},
        "compiler": {
            "repository_commit": subprocess.run(
                ["git", "-C", str(ROOT.parent.parent), "rev-parse", "HEAD"],
                capture_output=True,
                text=True,
                check=True,
            ).stdout.strip(),
            "target": "x86_64-linux-gnu",
            "source_parser": "production ai-compiler::parse",
            "validator": "production ai-compiler::validate",
            "candidate_adapter": "isolated JSON-to-AIC adapter in score_pilot.py",
            "oracle": "independent Python modulo-2^64 evaluator",
            "production_source_sha256": {
                relative: sha256((ROOT.parent.parent / relative).read_bytes())
                for relative in production_files
            },
        },
        "tokenizer": "GGUF tokenizer/chat template from pinned model; llama-cpp-python chat completion usage fields",
        "trial_prompts": prompt_entries,
        "input_set_sha256": sha256(inputs_path.read_bytes()),
        "oracle_sha256": sha256(oracle_path.read_bytes()),
        "preregistered_file_sha256": file_hashes,
        "storage": {
            "repository_raw_records": "trials.jsonl; complete prompt, response, metadata, and errors",
            "repository_scored_records": "scored.jsonl and summary.json",
            "external_model_and_builds": "/tmp/aic-g3-pilot-2026-11-13/; model weights are not committed",
        },
    }
    write_bytes(ROOT / "manifest.json", json_bytes(manifest))


if __name__ == "__main__":
    main()
