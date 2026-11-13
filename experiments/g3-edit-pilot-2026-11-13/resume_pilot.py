#!/usr/bin/env python3
"""Continue only not-yet-started G3 slots after an interrupted one-shot run."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from llama_cpp import Llama, llama_chat_format

from run_pilot import check_manifest, hash_file, order_bit, pair_seed


ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
EXTERNAL = Path("/tmp/aic-g3-pilot-2026-11-13")
MAX_SECONDS = 4 * 60 * 60


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def schedule() -> list[dict]:
    prompts = {(entry["task_id"], entry["format"]): entry for entry in json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))["trial_prompts"]}
    rows = []
    index = 0
    for task_id in ("E1", "E2", "E3", "E4", "E5", "E6"):
        for replicate in range(10):
            seed = pair_seed(task_id, replicate)
            bit = order_bit(task_id, replicate)
            formats = ("aic", "json") if bit == 0 else ("json", "aic")
            for form in formats:
                rows.append({
                    "call_index": index,
                    "task_id": task_id,
                    "replicate": replicate,
                    "format": form,
                    "pair_seed": seed,
                    "order_bit": bit,
                    "pair_order": list(formats),
                    "prompt_entry": prompts[(task_id, form)],
                })
                index += 1
    return rows


def append_record(path: Path, row: dict) -> None:
    encoded = (json.dumps(row, ensure_ascii=True, separators=(",", ":")) + "\n").encode("utf-8")
    with path.open("ab") as handle:
        handle.write(encoded)
        handle.flush()
        os.fsync(handle.fileno())


def atomic_json(path: Path, value: dict) -> None:
    temp = path.with_suffix(path.suffix + ".tmp")
    temp.write_text(json.dumps(value, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
    with temp.open("rb") as handle:
        os.fsync(handle.fileno())
    os.replace(temp, path)
    dir_fd = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(dir_fd)
    finally:
        os.close(dir_fd)


def unknown_row(slot: dict, detection_reason: str) -> dict:
    entry = slot["prompt_entry"]
    prompt_bytes = (ROOT / entry["path"]).read_bytes()
    return {
        "record_type": "trial",
        **{key: slot[key] for key in ("call_index", "task_id", "replicate", "format", "pair_seed", "order_bit", "pair_order")},
        "system_message": "/no_think",
        "user_prompt": prompt_bytes.decode("utf-8"),
        "user_prompt_utf8_hex": prompt_bytes.hex(),
        "prompt_sha256": entry["sha256"],
        "prompt_byte_count": len(prompt_bytes),
        "model": json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))["model"],
        "sampling": {"temperature": 0.7, "top_p": 0.8, "top_k": 20, "min_p": 0.0, "presence_penalty": 1.5, "frequency_penalty": 0.0, "repeat_penalty": 1.0, "mirostat_mode": 0, "max_tokens": 1024, "stream": False, "tools": False, "seed": slot["pair_seed"]},
        "status": "interrupted_unknown",
        "response_text": None,
        "response_utf8_hex": None,
        "raw_response": None,
        "usage": None,
        "finish_reason": None,
        "exception_type": "ExternalInterruption",
        "exception_message": detection_reason,
        "elapsed_seconds": None,
        "started_utc": None,
        "finished_utc": datetime.now(timezone.utc).isoformat(),
        "response_unavailable": True,
        "retry_performed": False,
    }


def load_rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


def main() -> int:
    manifest_path = ROOT / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    addendum_path = ROOT / "recovery_addendum.json"
    addendum = json.loads(addendum_path.read_text(encoding="utf-8"))
    baseline = addendum["baseline"]
    if baseline["recovery_script_sha256"] != hash_file(Path(__file__)):
        raise SystemExit("recovery script hash differs from recovery addendum")
    if baseline["original_manifest_sha256"] != sha256(manifest_path.read_bytes()):
        raise SystemExit("original preregistration changed")
    original_runner_hash = sha256((ROOT / "run_pilot.py").read_bytes())
    if baseline["original_runner_sha256"] != original_runner_hash:
        raise SystemExit("original one-shot runner changed")
    model_path = Path(manifest["model"]["path"])
    check_manifest(manifest, model_path)

    trials_path = ROOT / "trials.jsonl"
    rows = load_rows(trials_path)
    if len(rows) < 24:
        raise SystemExit("fewer than 24 baseline rows; recovery protocol does not match this run")
    first_24 = ("\n".join(trials_path.read_text(encoding="utf-8").splitlines()[:24]) + "\n").encode("utf-8")
    if sha256(first_24) != baseline["initial_trials_prefix_sha256"]:
        raise SystemExit("the first 24 raw trial records differ from the interrupted baseline")
    indices = [row["call_index"] for row in rows]
    if len(indices) != len(set(indices)) or indices != list(range(len(indices))):
        raise SystemExit("raw trial indices are duplicated or non-contiguous")
    slots = schedule()
    for index, row in enumerate(rows[:24]):
        expected = slots[index]
        if any(row.get(key) != expected[key] for key in ("call_index", "task_id", "replicate", "format", "pair_seed", "order_bit")):
            raise SystemExit(f"baseline schedule mismatch at call {index}")
        if row.get("status") != "completed":
            raise SystemExit(f"baseline call {index} is not a completed response")

    state_path = ROOT / "inflight.json"
    if state_path.exists():
        state = json.loads(state_path.read_text(encoding="utf-8"))
        index = state["call_index"]
        if state.get("state") == "active" and index not in indices:
            append_record(trials_path, unknown_row(slots[index], "A later recovery process found a durable active-call record without a durable response"))
            rows.append(load_rows(trials_path)[-1])
            indices.append(index)
        state_path.unlink(missing_ok=True)

    if len(rows) == 24:
        # The original runner had no durable pre-call marker; conservatively reserve
        # the next slot as possibly started and never send it again.
        append_record(trials_path, unknown_row(slots[24], "Original run process ended while call 24 may have been in progress; response was not durably recorded; slot is not retried"))
        rows.append(load_rows(trials_path)[-1])
        indices.append(24)
    if indices != list(range(len(indices))):
        raise SystemExit("recovery has a missing or non-contiguous call index")

    base_finished = datetime.fromisoformat(baseline["last_completed_finished_utc"])
    downtime = max(0.0, (datetime.now(timezone.utc) - base_finished).total_seconds())
    spent_before_recovery = float(baseline["prior_elapsed_run_seconds"]) + downtime
    remaining_seconds = max(0.0, MAX_SECONDS - spent_before_recovery)
    if remaining_seconds < 60:
        (ROOT / "recovery_status.json").write_text(json.dumps({"status": "budget_exhausted_before_resume", "spent_seconds": spent_before_recovery, "remaining_seconds": remaining_seconds}, indent=2) + "\n", encoding="utf-8")
        return 124
    budget_start = time.monotonic()
    deadline = budget_start + remaining_seconds

    started_utc = datetime.now(timezone.utc).isoformat()
    environment = {
        "python": sys.version,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cpu_count": os.cpu_count(),
        "threads": 5,
        "model_path": str(model_path),
        "model_sha256": hash_file(model_path),
        "llama_cpp_python": __import__("importlib.metadata", fromlist=["version"]).version("llama-cpp-python"),
    }
    model = Llama(model_path=str(model_path), n_ctx=4096, n_threads=5, n_threads_batch=5, n_gpu_layers=0, n_batch=512, verbose=False)
    template = model.metadata.get("tokenizer.chat_template")
    if not template or model.chat_format != "chat_template.default":
        raise SystemExit("pinned model chat template is unavailable")
    formatter = llama_chat_format.Jinja2ChatFormatter(
        template=template,
        eos_token=model._model.token_get_text(model.token_eos()),
        bos_token=model._model.token_get_text(model.token_bos()),
        stop_token_ids=[model.token_eos()],
    )
    prompt_token_counts = {}
    for entry in manifest["trial_prompts"]:
        messages = [{"role": "system", "content": "/no_think"}, {"role": "user", "content": (ROOT / entry["path"]).read_text(encoding="utf-8")}]
        formatted = formatter(messages=messages, tools=None)
        prompt_token_counts[entry["task_id"] + "/" + entry["format"]] = len(model.tokenize(formatted.prompt.encode("utf-8"), add_bos=not formatted.added_special, special=True))
    if not all(count + 1024 <= 4096 for count in prompt_token_counts.values()):
        raise SystemExit("recovery context preflight exceeds fixed context/output limits")
    environment["chat_template_sha256"] = sha256(template.encode("utf-8"))
    (ROOT / "recovery_context_preflight.json").write_text(json.dumps({"context_tokens": 4096, "max_output_tokens": 1024, "formatted_prompt_tokens": prompt_token_counts, "all_fit": True, "chat_template_sha256": environment["chat_template_sha256"]}, indent=2) + "\n", encoding="utf-8")

    task_formats = {(entry["task_id"], entry["format"]): entry for entry in manifest["trial_prompts"]}
    start_index = len(indices)
    with trials_path.open("ab") as handle:
        for slot in slots[start_index:]:
            if time.monotonic() >= deadline:
                break
            entry = task_formats[(slot["task_id"], slot["format"])]
            prompt_bytes = (ROOT / entry["path"]).read_bytes()
            prompt = prompt_bytes.decode("utf-8")
            record = {
                "record_type": "trial",
                **{key: slot[key] for key in ("call_index", "task_id", "replicate", "format", "pair_seed", "order_bit", "pair_order")},
                "system_message": "/no_think",
                "user_prompt": prompt,
                "user_prompt_utf8_hex": prompt_bytes.hex(),
                "prompt_sha256": entry["sha256"],
                "prompt_byte_count": len(prompt_bytes),
                "formatted_prompt_tokens": prompt_token_counts[slot["task_id"] + "/" + slot["format"]],
                "model": manifest["model"],
                "sampling": {"temperature": 0.7, "top_p": 0.8, "top_k": 20, "min_p": 0.0, "presence_penalty": 1.5, "frequency_penalty": 0.0, "repeat_penalty": 1.0, "mirostat_mode": 0, "max_tokens": 1024, "stream": False, "tools": False, "seed": slot["pair_seed"]},
                "environment": environment,
                "started_utc": datetime.now(timezone.utc).isoformat(),
            }
            atomic_json(state_path, {"state": "active", "call_index": slot["call_index"], "task_id": slot["task_id"], "replicate": slot["replicate"], "format": slot["format"], "prompt_sha256": entry["sha256"], "started_utc": record["started_utc"]})
            call_started = time.monotonic()
            try:
                result = model.create_chat_completion(
                    messages=[{"role": "system", "content": "/no_think"}, {"role": "user", "content": prompt}],
                    temperature=0.7, top_p=0.8, top_k=20, min_p=0.0,
                    presence_penalty=1.5, frequency_penalty=0.0, repeat_penalty=1.0,
                    mirostat_mode=0, max_tokens=1024, seed=slot["pair_seed"], stream=False, tools=None,
                )
                message = result["choices"][0]["message"]
                response = message.get("content")
                record.update({"status": "completed", "response_text": response, "response_utf8_hex": response.encode("utf-8").hex() if isinstance(response, str) else None, "raw_response": result, "usage": result.get("usage"), "finish_reason": result["choices"][0].get("finish_reason")})
            except Exception as error:
                record.update({"status": "inference_error", "exception_type": type(error).__name__, "exception_message": str(error), "response_text": None, "raw_response": None, "usage": None, "finish_reason": None})
            record["elapsed_seconds"] = time.monotonic() - call_started
            record["finished_utc"] = datetime.now(timezone.utc).isoformat()
            record["elapsed_run_seconds"] = float(baseline["prior_elapsed_run_seconds"]) + max(0.0, (datetime.fromisoformat(record["finished_utc"]) - base_finished).total_seconds())
            line = (json.dumps(record, ensure_ascii=True, separators=(",", ":")) + "\n").encode("utf-8")
            handle.write(line)
            handle.flush()
            os.fsync(handle.fileno())
            atomic_json(state_path, {"state": "completed", "call_index": slot["call_index"], "finished_utc": record["finished_utc"]})
            if record["status"] == "inference_error":
                continue
    final_rows = load_rows(trials_path)
    done = len(final_rows) == 120 and [row["call_index"] for row in final_rows] == list(range(120))
    runtime = {
        "status": "complete_with_interruption_recorded" if done else "partial_after_recovery",
        "calls_recorded": len(final_rows),
        "completed_responses": sum(row.get("status") == "completed" for row in final_rows),
        "interrupted_unknown": sum(row.get("status") == "interrupted_unknown" for row in final_rows),
        "inference_errors": sum(row.get("status") == "inference_error" for row in final_rows),
        "started_utc": started_utc,
        "remaining_budget_at_resume_seconds": remaining_seconds,
        "elapsed_recovery_process_seconds": time.monotonic() - budget_start,
        "environment": environment,
    }
    (ROOT / "run_metadata.json").write_text(json.dumps(runtime, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
    atomic_json(ROOT / "recovery_status.json", runtime)
    return 0 if done else 124


if __name__ == "__main__":
    raise SystemExit(main())
