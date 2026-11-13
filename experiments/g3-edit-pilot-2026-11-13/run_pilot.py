#!/usr/bin/env python3
"""Run the fixed local-model G3 paired pilot once; never retries a generation."""

from __future__ import annotations

import argparse
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


ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
DEFAULT_MODEL = Path("/tmp/aic-g3-pilot-2026-11-13/Qwen3-4B-Q4_K_M.gguf")
MODEL_SHA256 = "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5"
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
    payload = b"AIC-G3-pilot-seed-v1\0" + task_id.encode("ascii") + b"\0" + replicate.to_bytes(4, "big")
    return int.from_bytes(hashlib.sha256(payload).digest()[:4], "big")


def order_bit(task_id: str, replicate: int) -> int:
    payload = b"AIC-G3-pilot-order-v1\0" + task_id.encode("ascii") + b"\0" + replicate.to_bytes(4, "big")
    return hashlib.sha256(payload).digest()[0] & 1


def check_manifest(manifest: dict, model_path: Path) -> None:
    if model_path.stat().st_size != 2_497_280_256 or hash_file(model_path) != MODEL_SHA256:
        raise SystemExit("model size/hash differs from the preregistered artifact; no calls made")
    for relative, expected in manifest["preregistered_file_sha256"].items():
        path = ROOT / relative
        if not path.is_file() or hash_file(path) != expected:
            raise SystemExit(f"preregistered input changed: {relative}; no calls made")
    current_commit = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if current_commit != manifest["compiler"]["repository_commit"]:
        raise SystemExit("production compiler commit differs from pre-registration; no calls made")
    for relative, expected in manifest["compiler"]["production_source_sha256"].items():
        path = REPO / relative
        if not path.is_file() or hash_file(path) != expected:
            raise SystemExit(f"production source changed: {relative}; no calls made")
    for entry in manifest["trial_prompts"]:
        path = ROOT / entry["path"]
        data = path.read_bytes()
        if len(data) != entry["byte_count"] or sha256(data) != entry["sha256"]:
            raise SystemExit(f"prompt packet changed: {entry['path']}; no calls made")


def write_record(handle, record: dict) -> None:
    line = json.dumps(record, ensure_ascii=True, separators=(",", ":")) + "\n"
    handle.write(line)
    handle.flush()
    os.fsync(handle.fileno())


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    parser.add_argument("--max-seconds", type=int, default=MAX_SECONDS)
    args = parser.parse_args()
    manifest_path = ROOT / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    check_manifest(manifest, args.model)
    output_path = ROOT / "trials.jsonl"
    if output_path.exists():
        raise SystemExit("trials.jsonl already exists; this one-shot runner never resumes or retries")

    started = time.monotonic()
    started_utc = datetime.now(timezone.utc).isoformat()
    task_formats = {(entry["task_id"], entry["format"]): entry for entry in manifest["trial_prompts"]}
    environment = {
        "python": sys.version,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cpu_count": os.cpu_count(),
        "threads": 5,
        "model_path": str(args.model),
        "model_sha256": hash_file(args.model),
        "llama_cpp_python": __import__("importlib.metadata", fromlist=["version"]).version("llama-cpp-python"),
        "chat_template": "GGUF tokenizer.chat_template via llama-cpp-python chat_format.default",
    }
    model = None
    try:
        model = Llama(
            model_path=str(args.model),
            n_ctx=4096,
            n_threads=5,
            n_threads_batch=5,
            n_gpu_layers=0,
            n_batch=512,
            verbose=False,
        )
    except Exception as error:
        # Record setup failure as a run-level artifact without counting an inference call.
        (ROOT / "runner_setup_error.json").write_text(
            json.dumps({"stage": "model_load", "exception_type": type(error).__name__, "message": str(error), "environment": environment}, ensure_ascii=True, indent=2) + "\n",
            encoding="utf-8",
        )
        return 2

    template = model.metadata.get("tokenizer.chat_template")
    if not template or model.chat_format != "chat_template.default":
        (ROOT / "runner_setup_error.json").write_text(
            json.dumps({"stage": "chat_template", "chat_format": model.chat_format, "template_present": bool(template)}, ensure_ascii=True, indent=2) + "\n",
            encoding="utf-8",
        )
        return 2
    formatter = llama_chat_format.Jinja2ChatFormatter(
        template=template,
        eos_token=model._model.token_get_text(model.token_eos()),
        bos_token=model._model.token_get_text(model.token_bos()),
        stop_token_ids=[model.token_eos()],
    )
    prompt_token_counts = {}
    for entry in manifest["trial_prompts"]:
        messages = [
            {"role": "system", "content": "/no_think"},
            {"role": "user", "content": (ROOT / entry["path"]).read_text(encoding="utf-8")},
        ]
        formatted = formatter(messages=messages, tools=None)
        tokens = model.tokenize(formatted.prompt.encode("utf-8"), add_bos=not formatted.added_special, special=True)
        prompt_token_counts[entry["task_id"] + "/" + entry["format"]] = len(tokens)
    (ROOT / "context_preflight.json").write_text(
        json.dumps({"context_tokens": 4096, "max_output_tokens": 1024, "formatted_prompt_tokens": prompt_token_counts, "all_fit": all(count + 1024 <= 4096 for count in prompt_token_counts.values()), "chat_template_sha256": sha256(template.encode("utf-8"))}, ensure_ascii=True, indent=2) + "\n",
        encoding="utf-8",
    )
    if not all(count + 1024 <= 4096 for count in prompt_token_counts.values()):
        (ROOT / "runner_setup_error.json").write_text(
            json.dumps({"stage": "context_limit", "context_tokens": 4096, "max_output_tokens": 1024, "formatted_prompt_tokens": prompt_token_counts}, ensure_ascii=True, indent=2) + "\n",
            encoding="utf-8",
        )
        return 2
    environment["chat_template_sha256"] = sha256(template.encode("utf-8"))

    call_index = 0
    deadline = started + min(args.max_seconds, MAX_SECONDS)
    with output_path.open("x", encoding="utf-8", newline="\n") as handle:
        for task_id in ("E1", "E2", "E3", "E4", "E5", "E6"):
            for replicate in range(10):
                seed = pair_seed(task_id, replicate)
                bit = order_bit(task_id, replicate)
                formats = ("aic", "json") if bit == 0 else ("json", "aic")
                for form in formats:
                    if time.monotonic() >= deadline:
                        return 124
                    entry = task_formats[(task_id, form)]
                    prompt_bytes = (ROOT / entry["path"]).read_bytes()
                    prompt = prompt_bytes.decode("utf-8")
                    record = {
                        "record_type": "trial",
                        "call_index": call_index,
                        "task_id": task_id,
                        "replicate": replicate,
                        "format": form,
                        "pair_seed": seed,
                        "order_bit": bit,
                        "pair_order": list(formats),
                        "system_message": "/no_think",
                        "user_prompt": prompt,
                        "user_prompt_utf8_hex": prompt_bytes.hex(),
                        "prompt_sha256": entry["sha256"],
                        "prompt_byte_count": len(prompt_bytes),
                        "formatted_prompt_tokens": prompt_token_counts[task_id + "/" + form],
                        "model": manifest["model"],
                        "sampling": {"temperature": 0.7, "top_p": 0.8, "top_k": 20, "min_p": 0.0, "presence_penalty": 1.5, "frequency_penalty": 0.0, "repeat_penalty": 1.0, "mirostat_mode": 0, "max_tokens": 1024, "stream": False, "tools": False, "seed": seed},
                        "environment": environment,
                        "started_utc": datetime.now(timezone.utc).isoformat(),
                    }
                    call_index += 1
                    call_started = time.monotonic()
                    try:
                        result = model.create_chat_completion(
                            messages=[{"role": "system", "content": "/no_think"}, {"role": "user", "content": prompt}],
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
                    write_record(handle, record)
    (ROOT / "run_metadata.json").write_text(
        json.dumps({"status": "complete", "calls_recorded": call_index, "started_utc": started_utc, "elapsed_seconds": time.monotonic() - started, "environment": environment}, ensure_ascii=True, indent=2) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
