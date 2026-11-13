#!/usr/bin/env python3
"""One non-task probe of Qwen3's pinned GGUF chat-template switch.

This does not use an E1-E6 task, score AIC/JSON, or alter the original pilot.
"""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import platform
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from llama_cpp import Llama, llama_chat_format


HERE = Path(__file__).resolve().parent
MODEL = Path("/tmp/aic-g3-pilot-2026-11-13/Qwen3-4B-Q4_K_M.gguf")
MODEL_SHA256 = "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5"
SYSTEM = "/no_think"
USER = "Reply with the exact text OK and nothing else."


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    if MODEL.stat().st_size != 2_497_280_256 or file_sha256(MODEL) != MODEL_SHA256:
        raise SystemExit("pinned model artifact mismatch; no generation performed")

    model = Llama(
        model_path=str(MODEL),
        n_ctx=4096,
        n_threads=5,
        n_threads_batch=5,
        n_gpu_layers=0,
        verbose=False,
    )
    template = model.metadata.get("tokenizer.chat_template")
    if not template or model.chat_format != "chat_template.default":
        raise SystemExit("expected embedded default chat template; no generation performed")

    formatter = llama_chat_format.Jinja2ChatFormatter(
        template=template,
        eos_token=model._model.token_get_text(model.token_eos()),
        bos_token=model._model.token_get_text(model.token_bos()),
        stop_token_ids=[model.token_eos()],
    )
    # llama-cpp-python 0.3.16's Jinja2ChatFormatter accepts **kwargs but does
    # not forward them to Jinja. Set the template variable explicitly so that
    # Qwen3 receives its documented non-thinking assistant prefix.
    formatter._environment.globals["enable_thinking"] = False

    model.chat_handler = llama_chat_format.chat_formatter_to_chat_completion_handler(formatter)
    prompt = formatter(
        messages=[{"role": "system", "content": SYSTEM}, {"role": "user", "content": USER}],
        tools=None,
    ).prompt
    if not prompt.endswith("<think>\n\n</think>\n\n"):
        raise SystemExit("Qwen3 non-thinking template prefix missing; no generation performed")

    started = time.monotonic()
    started_utc = datetime.now(timezone.utc).isoformat()
    response = model.create_chat_completion(
        messages=[{"role": "system", "content": SYSTEM}, {"role": "user", "content": USER}],
        temperature=0.0,
        top_p=1.0,
        top_k=1,
        min_p=0.0,
        max_tokens=16,
        seed=131113,
        stream=False,
        tools=None,
    )
    message = response["choices"][0]["message"]
    content = message.get("content")
    result = {
        "kind": "non_task_template_feasibility_probe",
        "scoring_experiment": False,
        "started_utc": started_utc,
        "finished_utc": datetime.now(timezone.utc).isoformat(),
        "elapsed_seconds": time.monotonic() - started,
        "model": {
            "repo_id": "Qwen/Qwen3-4B-GGUF",
            "revision": "bc640142c66e1fdd12af0bd68f40445458f3869b",
            "path": str(MODEL),
            "size_bytes": MODEL.stat().st_size,
            "sha256": file_sha256(MODEL),
            "llama_cpp_python": importlib.metadata.version("llama-cpp-python"),
        },
        "chat_template": {
            "source": "GGUF tokenizer.chat_template",
            "sha256": sha256(template.encode("utf-8")),
            "enable_thinking": False,
            "prompt_sha256": sha256(prompt.encode("utf-8")),
            "prompt_utf8_hex": prompt.encode("utf-8").hex(),
        },
        "system_message": SYSTEM,
        "user_prompt": USER,
        "sampling": {"temperature": 0.0, "top_p": 1.0, "top_k": 1, "min_p": 0.0, "max_tokens": 16, "seed": 131113},
        "environment": {"python": sys.version, "platform": platform.platform(), "machine": platform.machine(), "threads": 5},
        "status": "completed",
        "response": response,
        "content": content,
        "content_utf8_hex": content.encode("utf-8").hex() if isinstance(content, str) else None,
        "content_is_exact_ok": content == "OK",
        "empty_think_wrapper_in_generated_content": isinstance(content, str) and content.startswith("<think>\n\n</think>\n\n"),
    }
    output = HERE / "template_probe.json"
    output.write_text(json.dumps(result, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: result[k] for k in ("kind", "elapsed_seconds", "content", "content_is_exact_ok", "empty_think_wrapper_in_generated_content")}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
