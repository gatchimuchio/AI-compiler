#!/usr/bin/env python3
"""Strict candidate adapter, production-parser bridge, and independent G3 scorer."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
EXTERNAL = Path("/tmp/aic-g3-pilot-2026-11-13")
ID_RE = re.compile(r"[a-z][a-z0-9_]{0,62}\Z")
DECIMAL_RE = re.compile(r"(?:0|[1-9][0-9]{0,19})\Z")
U64_MAX = 2**64 - 1


class CandidateError(ValueError):
    pass


def _unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise CandidateError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_candidate_json(text: str) -> tuple[bool, dict | None, str | None]:
    try:
        value = json.loads(text, object_pairs_hook=_unique_pairs, parse_constant=lambda value: (_ for _ in ()).throw(CandidateError(f"invalid JSON constant: {value}")))
    except (json.JSONDecodeError, CandidateError) as error:
        return False, None, str(error)
    if not isinstance(value, dict):
        return True, None, "top-level value must be an object"
    return True, value, None


def _exact_keys(value: object, expected: set[str], where: str) -> dict:
    if not isinstance(value, dict):
        raise CandidateError(f"{where} must be an object")
    actual = set(value)
    if actual != expected:
        raise CandidateError(f"{where} keys differ: missing={sorted(expected - actual)}, unknown={sorted(actual - expected)}")
    return value


def _string(value: object, where: str) -> str:
    if not isinstance(value, str):
        raise CandidateError(f"{where} must be a string")
    return value


def _check_id(value: object, where: str) -> str:
    ident = _string(value, where)
    if not ID_RE.fullmatch(ident):
        raise CandidateError(f"{where} is not a valid value ID")
    return ident


def _check_decimal(value: object, where: str) -> str:
    decimal = _string(value, where)
    if not DECIMAL_RE.fullmatch(decimal) or int(decimal) > U64_MAX:
        raise CandidateError(f"{where} is not a u64 decimal string")
    return decimal


def validate_json_profile(value: dict) -> tuple[list[dict], str]:
    _exact_keys(value, {"format", "function"}, "top-level")
    if value["format"] != "AIC-J/0.1" or type(value["format"]) is not str:
        raise CandidateError('top-level format must be "AIC-J/0.1"')
    function = _exact_keys(value["function"], {"name", "params", "result", "blocks"}, "function")
    if function["name"] != "aic_main" or type(function["name"]) is not str:
        raise CandidateError('function.name must be "aic_main"')
    if function["result"] != "u64" or type(function["result"]) is not str:
        raise CandidateError('function.result must be "u64"')
    params = function["params"]
    if not isinstance(params, list) or len(params) != 1:
        raise CandidateError("function.params must contain exactly one parameter")
    param = _exact_keys(params[0], {"id", "type"}, "parameter")
    if param["id"] != "x" or type(param["id"]) is not str or param["type"] != "u64" or type(param["type"]) is not str:
        raise CandidateError('parameter must be {"id":"x","type":"u64"}')
    blocks = function["blocks"]
    if not isinstance(blocks, list) or len(blocks) != 1:
        raise CandidateError("function.blocks must contain exactly one block")
    block = _exact_keys(blocks[0], {"id", "ops", "term"}, "block")
    if block["id"] != "entry" or type(block["id"]) is not str:
        raise CandidateError('block.id must be "entry"')
    if not isinstance(block["ops"], list):
        raise CandidateError("block.ops must be an array")

    defined = {"x"}
    ops: list[dict] = []
    lines = ["AIC 0.2", "fn aic_main(%x: u64) -> u64 {", "^entry:"]
    for index, raw in enumerate(block["ops"]):
        if not isinstance(raw, dict):
            raise CandidateError(f"ops[{index}] must be an object")
        op_name = raw.get("op")
        ident = _check_id(raw.get("id"), f"ops[{index}].id")
        if ident in defined:
            raise CandidateError(f"ops[{index}].id duplicates a previous definition")
        if op_name == "const.u64":
            op = _exact_keys(raw, {"id", "op", "value"}, f"ops[{index}]")
            decimal = _check_decimal(op["value"], f"ops[{index}].value")
            ops.append({"id": ident, "op": "const.u64", "value": decimal})
            lines.append(f"  %{ident} = const.u64 {decimal}")
        elif op_name in ("add.wrap.u64", "sub.wrap.u64"):
            op = _exact_keys(raw, {"id", "op", "args"}, f"ops[{index}]")
            args = op["args"]
            if not isinstance(args, list) or len(args) != 2:
                raise CandidateError(f"ops[{index}].args must contain exactly two IDs")
            lhs = _check_id(args[0], f"ops[{index}].args[0]")
            rhs = _check_id(args[1], f"ops[{index}].args[1]")
            if lhs not in defined or rhs not in defined:
                raise CandidateError(f"ops[{index}] uses an undefined or forward value")
            ops.append({"id": ident, "op": op_name, "args": [lhs, rhs]})
            lines.append(f"  %{ident} = {op_name} %{lhs}, %{rhs}")
        else:
            raise CandidateError(f"ops[{index}].op is unknown")
        defined.add(ident)

    term = _exact_keys(block["term"], {"op", "value"}, "block.term")
    if term["op"] != "ret.u64" or type(term["op"]) is not str:
        raise CandidateError('block.term.op must be "ret.u64"')
    term_value = _check_id(term["value"], "block.term.value")
    if term_value not in defined:
        raise CandidateError("block.term.value is undefined")
    ops.append({"op": "ret.u64", "value": term_value})
    lines.extend([f"  ret.u64 %{term_value}", "}"])
    return ops, "\n".join(lines)


def run_helper(source: str, stem: str) -> dict:
    scratch = EXTERNAL / "normalized"
    scratch.mkdir(parents=True, exist_ok=True)
    source_path = scratch / f"{stem}.aic"
    source_path.write_bytes(source.encode("utf-8"))
    helper = EXTERNAL / "helper-target" / "release" / "g3-parser-adapter"
    completed = subprocess.run([str(helper), str(source_path)], capture_output=True, text=True, timeout=30)
    lines = completed.stdout.splitlines()
    stages = set(lines)
    result = {
        "returncode": completed.returncode,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
        "common_parser_ok": "PARSE_OK" in stages,
        "common_validator_ok": "VALIDATION_OK" in stages,
        "candidate_profile_ok": "PROFILE_OK" in stages,
        "operations": None,
    }
    if result["candidate_profile_ok"]:
        operations = []
        for line in lines:
            parts = line.split("\t")
            if parts[0] == "CONST":
                operations.append({"id": parts[1], "op": "const.u64", "value": parts[2]})
            elif parts[0] in ("ADD", "SUB"):
                operations.append({"id": parts[1], "op": {"ADD": "add.wrap.u64", "SUB": "sub.wrap.u64"}[parts[0]], "args": [parts[2], parts[3]]})
            elif parts[0] == "RET":
                operations.append({"op": "ret.u64", "value": parts[1]})
        result["operations"] = operations
    return result


def independent_oracle(operations: list[dict], x: int) -> int:
    values = {"x": x}
    for operation in operations:
        kind = operation["op"]
        if kind == "const.u64":
            values[operation["id"]] = int(operation["value"])
        elif kind == "add.wrap.u64":
            lhs, rhs = operation["args"]
            values[operation["id"]] = (values[lhs] + values[rhs]) % 2**64
        elif kind == "sub.wrap.u64":
            lhs, rhs = operation["args"]
            values[operation["id"]] = (values[lhs] - values[rhs]) % 2**64
        elif kind == "ret.u64":
            return values[operation["value"]]
        else:
            raise ValueError(f"unsupported normalized op: {kind}")
    raise ValueError("missing return")


def unrequested_changes(actual: list[dict], expected: list[dict], allowed: list[str]) -> int:
    if "*" in allowed:
        return 0
    allowed_set = set(allowed)
    expected_ops = [op for op in expected if op["op"] != "ret.u64"]
    actual_ops = [op for op in actual if op["op"] != "ret.u64"]
    expected_term = next((op for op in expected if op["op"] == "ret.u64"), None)
    actual_term = next((op for op in actual if op["op"] == "ret.u64"), None)
    changed = 0
    for index in range(max(len(expected_ops), len(actual_ops))):
        path = f"i{index}"
        exp = expected_ops[index] if index < len(expected_ops) else None
        got = actual_ops[index] if index < len(actual_ops) else None
        if exp is None or got is None:
            if f"{path}.*" not in allowed_set:
                changed += 1
            continue
        if f"{path}.*" in allowed_set:
            continue
        for field in set(exp) | set(got):
            if exp.get(field) != got.get(field) and f"{path}.{field}" not in allowed_set:
                changed += 1
    if expected_term != actual_term and "term.value" not in allowed_set:
        changed += 1
    return changed


def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def compile_and_run(source: str, stem: str, candidate_ops: list[dict], expected_ops: list[dict], inputs: list[int], aic: Path, repo: Path) -> dict:
    artifacts = EXTERNAL / "artifacts"
    artifacts.mkdir(parents=True, exist_ok=True)
    source_path = artifacts / f"{stem}.aic"
    object_path = artifacts / f"{stem}.o"
    consumer_path = artifacts / f"{stem}.consumer"
    source_path.write_bytes(source.encode("ascii"))
    compiled = subprocess.run([str(aic), "compile", "--target", "x86_64-linux-gnu", str(source_path), "-o", str(object_path)], capture_output=True, text=True, timeout=60)
    result = {
        "compiler_returncode": compiled.returncode,
        "compiler_stdout": compiled.stdout,
        "compiler_stderr": compiled.stderr,
        "source_path": str(source_path),
        "source_sha256": hash_file(source_path),
        "object_path": str(object_path) if object_path.exists() else None,
        "object_sha256": hash_file(object_path) if object_path.exists() else None,
    }
    if compiled.returncode:
        return result
    link = subprocess.run([
        "g++", "-std=c++20", "-O2", "-mno-red-zone", "-Wall", "-Wextra", "-Werror",
        str(repo / "tests/harness.cpp"), str(repo / "tests/abi_probe.S"), str(object_path), "-o", str(consumer_path),
    ], capture_output=True, text=True, timeout=60)
    result.update({
        "consumer_returncode": link.returncode,
        "consumer_stdout": link.stdout,
        "consumer_stderr": link.stderr,
        "consumer_path": str(consumer_path) if consumer_path.exists() else None,
        "consumer_sha256": hash_file(consumer_path) if consumer_path.exists() else None,
    })
    if link.returncode:
        return result
    input_text = "".join(f"{item}\n" for item in inputs)
    run = subprocess.run([str(consumer_path)], input=input_text, capture_output=True, text=True, timeout=10)
    observed = []
    parse_error = None
    try:
        observed = [int(line) for line in run.stdout.splitlines()]
    except ValueError as error:
        parse_error = str(error)
    candidate_expected = [independent_oracle(candidate_ops, item) for item in inputs]
    task_expected = [independent_oracle(expected_ops, item) for item in inputs]
    result.update({
        "run_returncode": run.returncode,
        "run_stdout": run.stdout,
        "run_stderr": run.stderr,
        "run_parse_error": parse_error,
        "input_values": inputs,
        "observed_outputs": observed,
        "candidate_oracle_outputs": candidate_expected,
        "task_oracle_outputs": task_expected,
        "binary_matches_candidate_oracle": run.returncode == 0 and parse_error is None and observed == candidate_expected,
        "binary_matches_task_expected": run.returncode == 0 and parse_error is None and observed == task_expected,
    })
    return result


def score_trial(trial: dict, oracle: dict, inputs: list[int], aic: Path, repo: Path) -> dict:
    task_id = trial["task_id"]
    form = trial["format"]
    task = oracle["tasks"][task_id]
    response = trial.get("response_text") if trial.get("status") == "completed" else None
    row = {
        "call_index": trial["call_index"],
        "task_id": task_id,
        "replicate": trial["replicate"],
        "format": form,
        "status": trial.get("status"),
        "pair_seed": trial["pair_seed"],
        "order_bit": trial["order_bit"],
        "usage": trial.get("usage"),
        "finish_reason": trial.get("finish_reason"),
        "elapsed_seconds": trial.get("elapsed_seconds"),
        "error": trial.get("exception_message"),
        "format_syntax_ok": False,
        "profile_shape_ok": False,
        "common_parser_ok": False,
        "common_validator_ok": False,
        "edit_pass": False,
        "unrequested_node_field_changes": None,
        "object_check": None,
        "pilot_success": False,
    }
    if response is None or not isinstance(response, str):
        row["format_error"] = "no response text"
        return row

    common_source = None
    adapter_ops = None
    if form == "json":
        syntax_ok, parsed, syntax_error = parse_candidate_json(response)
        row["format_syntax_ok"] = syntax_ok
        row["format_error"] = syntax_error
        if syntax_ok and parsed is not None:
            try:
                adapter_ops, common_source = validate_json_profile(parsed)
                row["profile_shape_ok"] = True
                row["format_error"] = None
            except CandidateError as error:
                row["format_error"] = str(error)
    elif form == "aic":
        common_source = response
    else:
        row["format_error"] = f"unknown format {form}"
        return row

    if common_source is None:
        return row
    stem = f"{task_id}-r{trial['replicate']:02d}-{form}"
    helper = run_helper(common_source, stem)
    row["common_parser_ok"] = helper["common_parser_ok"]
    row["common_validator_ok"] = helper["common_validator_ok"]
    if form == "aic":
        row["format_syntax_ok"] = helper["common_parser_ok"]
        row["profile_shape_ok"] = helper["candidate_profile_ok"]
    elif not helper["candidate_profile_ok"]:
        row["profile_shape_ok"] = False
    row["adapter_error"] = helper["stderr"].strip() if helper["stderr"].strip() else None
    operations = helper["operations"]
    row["normalized_operations"] = operations
    if form == "json" and adapter_ops is not None:
        row["adapter_consistent"] = operations == adapter_ops
        if not row["adapter_consistent"]:
            row["profile_shape_ok"] = False
            row["format_error"] = "JSON adapter output differs from production parser normalization"
    row["parse_accepted"] = row["format_syntax_ok"] and row["profile_shape_ok"] and row["common_parser_ok"]
    row["validator_accepted"] = row["parse_accepted"] and row["common_validator_ok"]
    if operations is not None:
        expected = task["expected_operations"]
        row["edit_pass"] = operations == expected
        row["unrequested_node_field_changes"] = unrequested_changes(operations, expected, task["allowed_target_fields"])

    if row["validator_accepted"] and operations is not None:
        row["object_check"] = compile_and_run(common_source, stem, operations, task["expected_operations"], inputs, aic, repo)
        obj = row["object_check"]
        row["pilot_success"] = bool(row["edit_pass"] and obj.get("compiler_returncode") == 0 and obj.get("consumer_returncode") == 0 and obj.get("binary_matches_candidate_oracle") and obj.get("binary_matches_task_expected"))
    return row


def _summary(scored: list[dict], trial_count: int) -> dict:
    rows_by = {}
    for form in ("aic", "json"):
        form_rows = [row for row in scored if row["format"] == form]
        rows_by[form] = {
            "calls_scored": len(form_rows),
            "inference_errors": sum(row["status"] != "completed" for row in form_rows),
            "parse_accepted": sum(bool(row.get("parse_accepted")) for row in form_rows),
            "validator_accepted": sum(bool(row.get("validator_accepted")) for row in form_rows),
            "edit_pass": sum(bool(row.get("edit_pass")) for row in form_rows),
            "compiler_success": sum(bool(row.get("object_check") and row["object_check"].get("compiler_returncode") == 0) for row in form_rows),
            "consumer_success": sum(bool(row.get("object_check") and row["object_check"].get("consumer_returncode") == 0) for row in form_rows),
            "binary_matches_candidate_oracle": sum(bool(row.get("object_check") and row["object_check"].get("binary_matches_candidate_oracle")) for row in form_rows),
            "pilot_success": sum(bool(row.get("pilot_success")) for row in form_rows),
        }
    tasks = {}
    for task_id in ("E1", "E2", "E3", "E4", "E5", "E6"):
        task_rows = [row for row in scored if row["task_id"] == task_id]
        tasks[task_id] = {}
        for form in ("aic", "json"):
            selected = [row for row in task_rows if row["format"] == form]
            tasks[task_id][form] = {
                "denominator": len(selected),
                "parse_accepted": sum(bool(row.get("parse_accepted")) for row in selected),
                "validator_accepted": sum(bool(row.get("validator_accepted")) for row in selected),
                "edit_pass": sum(bool(row.get("edit_pass")) for row in selected),
                "pilot_success": sum(bool(row.get("pilot_success")) for row in selected),
            }
    paired = {}
    for task_id in ("E1", "E2", "E3", "E4", "E5", "E6"):
        by_key = {(row["task_id"], row["replicate"], row["format"]): row for row in scored}
        pair_diffs = []
        for replicate in range(10):
            aic = by_key.get((task_id, replicate, "aic"))
            js = by_key.get((task_id, replicate, "json"))
            if aic is not None and js is not None:
                pair_diffs.append(int(bool(aic.get("pilot_success"))) - int(bool(js.get("pilot_success"))))
        paired[task_id] = {"paired_rows": len(pair_diffs), "aic_minus_json_pilot_success": pair_diffs, "sum": sum(pair_diffs)}
    return {
        "status": "scored",
        "trials_read": trial_count,
        "scored_rows": len(scored),
        "missing_or_unrecorded_calls": max(0, 120 - trial_count),
        "by_format": rows_by,
        "by_task": tasks,
        "paired_descriptive_difference": paired,
        "interpretation": "Exploratory pilot only; no winner, candidate adoption, QA threshold, product path, or general performance claim is determined.",
    }


def main() -> int:
    global EXTERNAL
    parser = argparse.ArgumentParser()
    parser.add_argument("--trials", type=Path, default=ROOT / "trials.jsonl")
    parser.add_argument("--aic", type=Path, default=EXTERNAL / "cargo-target" / "release" / "aic")
    parser.add_argument("--helper-target", type=Path, default=EXTERNAL / "helper-target")
    args = parser.parse_args()
    EXTERNAL = args.helper_target.parent
    oracle = json.loads((ROOT / "oracle.json").read_text(encoding="utf-8"))
    fixed_inputs = json.loads((ROOT / "fixed_inputs.json").read_text(encoding="utf-8"))["values_u64"]
    if not args.trials.exists():
        raise SystemExit("no raw trials file exists")
    trials = [json.loads(line) for line in args.trials.read_text(encoding="utf-8").splitlines() if line]
    if not args.aic.is_file():
        raise SystemExit(f"production compiler is missing: {args.aic}")
    helper_bin = args.helper_target / "release" / "g3-parser-adapter"
    if not helper_bin.is_file():
        raise SystemExit(f"production parser adapter is missing: {helper_bin}")
    scored = [score_trial(trial, oracle, fixed_inputs, args.aic, REPO) for trial in trials]
    with (ROOT / "scored.jsonl").open("w", encoding="utf-8", newline="\n") as handle:
        for row in scored:
            handle.write(json.dumps(row, ensure_ascii=True, separators=(",", ":")) + "\n")
    summary = _summary(scored, len(trials))
    summary["compiler_sha256"] = hashlib.sha256(args.aic.read_bytes()).hexdigest()
    summary["helper_sha256"] = hashlib.sha256(helper_bin.read_bytes()).hexdigest()
    summary["g++_version"] = subprocess.run(["g++", "--version"], capture_output=True, text=True, check=False).stdout.splitlines()[0]
    summary["trials_sha256"] = hashlib.sha256(args.trials.read_bytes()).hexdigest()
    summary["scored_sha256"] = hashlib.sha256((ROOT / "scored.jsonl").read_bytes()).hexdigest()
    (ROOT / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
