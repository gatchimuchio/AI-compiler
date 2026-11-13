#!/usr/bin/env python3
"""Run preregistered G3 pilot run 02 exactly once; never strips or retries output."""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from llama_cpp import Llama, llama_chat_format


ROOT = Path(__file__).resolve().parent
EXPERIMENT = ROOT.parent
REPO = ROOT.parents[2]
MODEL = Path("/tmp/aic-g3-pilot-2026-11-13/Qwen3-4B-Q4_K_M.gguf")
MODEL_SHA256 = "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5"
EXPECTED_TEMPLATE_SHA256 = "57f1fd00f0013a2be96aa79b857391f27e23df5b5f847072b524c897e24d0361"
MAX_SECONDS = 4 * 60 * 60


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def pair_seed(task_id: str, replicate: int) -> int:
    payload = b"AIC-G3-run02-seed-v1\0" + task_id.encode("ascii") + b"\0" + replicate.to_bytes(4, "big")
    return int.from_bytes(hashlib.sha256(payload).digest()[:4], "big")


def order_bit(task_id: str, replicate: int) -> int:
    payload = b"AIC-G3-run02-order-v1\0" + task_id.encode("ascii") + b"\0" + replicate.to_bytes(4, "big")
    return hashlib.sha256(payload).digest()[0] & 1


def write_json(path: Path, value: dict) -> None:
    with path.open("x", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(value, ensure_ascii=True, indent=2) + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def append_record(handle, record: dict) -> None:
    handle.write(json.dumps(record, ensure_ascii=True, separators=(",", ":")) + "\n")
    handle.flush()
    os.fsync(handle.fileno())


def check_frozen_inputs(manifest: dict) -> None:
    if MODEL.stat().st_size != manifest["model"]["size_bytes"] or hash_file(MODEL) != MODEL_SHA256:
        raise SystemExit("pinned model artifact mismatch; no task calls made")
    for relative, expected in manifest["repository_files_sha256"].items():
        path = REPO / relative
        if not path.is_file() or hash_file(path) != expected:
            raise SystemExit(f"frozen repository input changed: {relative}; no task calls made")
    current = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if current != manifest["base_head"]:
        raise SystemExit("repository HEAD differs from frozen run-02 base; no task calls made")
    for relative, expected in manifest["task_bank"]["prompt_packets_sha256"].items():
        path = EXPERIMENT / relative
        if not path.is_file() or hash_file(path) != expected:
            raise SystemExit(f"fixed prompt packet changed: {relative}; no task calls made")
    if hash_file(ROOT / "preregistration.md") != manifest["preregistration_sha256"]:
        raise SystemExit("preregistration changed; no task calls made")
    if hash_file(Path(__file__)) != manifest["runner_sha256"]:
        raise SystemExit("runner differs from frozen preregistration; no task calls made")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-seconds", type=int, default=MAX_SECONDS)
    args = parser.parse_args()
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    check_frozen_inputs(manifest)

    trials_path = ROOT / "trials.jsonl"
    inflight_path = ROOT / "inflight.json"
    preflight_path = ROOT / "preflight.json"
    if any(path.exists() for path in (trials_path, inflight_path, preflight_path, ROOT / "run_metadata.json")):
        raise SystemExit("run-02 already has output/in-flight state; the runner never resumes or retries")

    started = time.monotonic()
    started_utc = datetime.now(timezone.utc).isoformat()
    model = Llama(
        model_path=str(MODEL),
        n_ctx=4096,
        n_threads=5,
        n_threads_batch=5,
        n_gpu_layers=0,
        n_batch=512,
        verbose=False,
    )
    template = model.metadata.get("tokenizer.chat_template")
    if not template or model.chat_format != "chat_template.default" or sha256(template.encode("utf-8")) != EXPECTED_TEMPLATE_SHA256:
        raise SystemExit("pinned GGUF chat-template identity differs; no task calls made")
    formatter = llama_chat_format.Jinja2ChatFormatter(
        template=template,
        eos_token=model._model.token_get_text(model.token_eos()),
        bos_token=model._model.token_get_text(model.token_bos()),
        stop_token_ids=[model.token_eos()],
    )
    formatter._environment.globals["enable_thinking"] = False
    model.chat_handler = llama_chat_format.chat_formatter_to_chat_completion_handler(formatter)

    manifest_parent = json.loads((ROOT / "run01_manifest.json").read_text(encoding="utf-8"))
    prompts = {(entry["task_id"], entry["format"]): entry for entry in manifest_parent["trial_prompts"]}
    prompt_tokens = {}
    formatted_hashes = {}
    for entry in manifest_parent["trial_prompts"]:
        user_text = (EXPERIMENT / entry["path"]).read_text(encoding="utf-8")
        messages = [{"role": "system", "content": "/no_think"}, {"role": "user", "content": user_text}]
        formatted = formatter(messages=messages, tools=None)
        if not formatted.prompt.endswith("<think>\n\n</think>\n\n"):
            raise SystemExit("non-thinking assistant prefix missing; no task calls made")
        tokenized = model.tokenize(formatted.prompt.encode("utf-8"), add_bos=not formatted.added_special, special=True)
        key = entry["task_id"] + "/" + entry["format"]
        prompt_tokens[key] = len(tokenized)
        formatted_hashes[key] = sha256(formatted.prompt.encode("utf-8"))
    all_fit = all(count + manifest["sampling"]["max_tokens"] <= manifest["model"]["context_tokens"] for count in prompt_tokens.values())
    preflight = {
        "status": "ready" if all_fit else "blocked_context_limit",
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "base_head": manifest["base_head"],
        "manifest_sha256": hash_file(ROOT / "manifest.json"),
        "runner_sha256": hash_file(Path(__file__)),
        "model_sha256": hash_file(MODEL),
        "chat_template_sha256": sha256(template.encode("utf-8")),
        "enable_thinking": False,
        "system_message": "/no_think",
        "context_tokens": manifest["model"]["context_tokens"],
        "max_output_tokens": manifest["sampling"]["max_tokens"],
        "formatted_prompt_tokens": prompt_tokens,
        "formatted_prompt_sha256": formatted_hashes,
        "all_fit": all_fit,
    }
    write_json(preflight_path, preflight)
    if not all_fit:
        return 2

    environment = {
        "python": sys.version,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cpu_count": os.cpu_count(),
        "threads": 5,
        "model_path": str(MODEL),
        "model_sha256": hash_file(MODEL),
        "llama_cpp_python": importlib.metadata.version("llama-cpp-python"),
        "chat_format": model.chat_format,
        "chat_template_sha256": sha256(template.encode("utf-8")),
        "enable_thinking": False,
    }
    if environment["llama_cpp_python"] != manifest["model"]["runner_version"]:
        raise SystemExit("llama-cpp-python version differs; no task calls made")

    deadline = started + min(args.max_seconds, MAX_SECONDS)
    call_index = 0
    with trials_path.open("x", encoding="utf-8", newline="\n") as handle:
        for task_id in ("E1", "E2", "E3", "E4", "E5", "E6"):
            for replicate in range(10):
                seed = pair_seed(task_id, replicate)
                bit = order_bit(task_id, replicate)
                formats = ("aic", "json") if bit == 0 else ("json", "aic")
                for form in formats:
                    if time.monotonic() >= deadline:
                        write_json(ROOT / "run_metadata.json", {"status": "wall_clock_limit_before_next_slot", "calls_recorded": call_index, "started_utc": started_utc, "elapsed_seconds": time.monotonic() - started, "environment": environment})
                        return 124
                    entry = prompts[(task_id, form)]
                    packet_path = EXPERIMENT / entry["path"]
                    prompt_bytes = packet_path.read_bytes()
                    prompt = prompt_bytes.decode("utf-8")
                    messages = [{"role": "system", "content": "/no_think"}, {"role": "user", "content": prompt}]
                    slot = {
                        "call_index": call_index,
                        "task_id": task_id,
                        "replicate": replicate,
                        "format": form,
                        "pair_seed": seed,
                        "order_bit": bit,
                        "pair_order": list(formats),
                        "prompt_sha256": entry["sha256"],
                        "formatted_prompt_sha256": formatted_hashes[task_id + "/" + form],
                    }
                    write_json(inflight_path, {"status": "in_progress", **slot, "started_utc": datetime.now(timezone.utc).isoformat()})
                    call_started = time.monotonic()
                    record = {
                        "record_type": "trial",
                        **slot,
                        "system_message": "/no_think",
                        "enable_thinking": False,
                        "user_prompt": prompt,
                        "user_prompt_utf8_hex": prompt_bytes.hex(),
                        "prompt_byte_count": len(prompt_bytes),
                        "formatted_prompt_tokens": prompt_tokens[task_id + "/" + form],
                        "model": manifest["model"],
                        "sampling": {**manifest["sampling"], "seed": seed, "stream": False, "tools": False},
                        "environment": environment,
                        "started_utc": datetime.now(timezone.utc).isoformat(),
                    }
                    call_index += 1
                    try:
                        result = model.create_chat_completion(
                            messages=messages,
                            temperature=0.7,
                            top_p=0.8,
                            top_k=20,
                            min_p=0.0,
                            presence_penalty=1.5,
                            frequency_penalty=0.0,
                            repeat_penalty=1.0,
                            mirostat_mode=0,
                            max_tokens=1024,
                            seed=seed,
                            stream=False,
                            tools=None,
                        )
                        message = result["choices"][0]["message"]
                        response = message.get("content")
                        record.update({
                            "status": "completed",
                            "response_text": response,
                            "response_utf8_hex": response.encode("utf-8").hex() if isinstance(response, str) else None,
                            "raw_response": result,
                            "usage": result.get("usage"),
                            "finish_reason": result["choices"][0].get("finish_reason"),
                        })
                    except Exception as error:
                        record.update({"status": "inference_error", "exception_type": type(error).__name__, "exception_message": str(error)})
                    record["elapsed_seconds"] = time.monotonic() - call_started
                    record["finished_utc"] = datetime.now(timezone.utc).isoformat()
                    record["elapsed_run_seconds"] = time.monotonic() - started
                    append_record(handle, record)
                    inflight_path.unlink()
    write_json(ROOT / "run_metadata.json", {"status": "complete", "calls_recorded": call_index, "started_utc": started_utc, "elapsed_seconds": time.monotonic() - started, "environment": environment})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
