#!/usr/bin/env python3
"""固定した2条件についてGCC 15.2.0のext_dce RTL dumpを取得する。"""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import shutil
import subprocess

ROOT = Path('/tmp/aic-g2-gcc-pr116488-ext-dce-2026-11-13')
SOURCE = ROOT / 'source/reporter-small.c'
RUNS = ROOT / 'runs'
RESULTS = ROOT / 'observations/results.json'
EXPECTED_SOURCE_SHA256 = '77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2'
GCC15_BUILD = Path('/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0')
GCC15 = GCC15_BUILD / 'gcc/xgcc'
GCC15_CC1 = GCC15_BUILD / 'gcc/cc1'
EXPECTED_XGCC_SHA256 = 'd11bd49769c79c84969134620c9c7607fb8cf53d0be074e8e012b9cc47c4b297'
EXPECTED_CC1_SHA256 = '987e7a1b2eb0b3103fa427bc5d7d4b05f9beeae4c2176d794216bbbc28648118'
GCC_SOURCE = Path('/tmp/aic-gcc15.2-pr116488-2026-11-13/source/gcc-15.2.0/gcc')
GCC_SOURCE_HASHES = {
    'ext-dce.cc': 'f6b40d2ec541e522d98d9c9801c7427c6a1c0fc6f74dddbdc976b0036a07d76f',
    'passes.def': '1f1b546ebf20092c126a5c681fa20d2a66e7ed97d1f621bc0cc3764edceecbe5',
    'common.opt': 'e9b54a8ae36c17201dd708fe238613df1acdc1ae7c9a6daed801514d3be1a989',
}
TOOLS = Path('/tmp/aic-gcc15-pr116488-2026-11-13/tools')
GCC15_LD_LIBRARY_PATH = ':'.join([
    '', str(GCC15_BUILD / 'gcc'),
    str(GCC15_BUILD / 'gmp/.libs'), str(GCC15_BUILD / 'prev-gmp/.libs'),
    str(GCC15_BUILD / 'mpfr/src/.libs'), str(GCC15_BUILD / 'prev-mpfr/src/.libs'),
    str(GCC15_BUILD / 'mpc/src/.libs'), str(GCC15_BUILD / 'prev-mpc/src/.libs'),
    str(GCC15_BUILD / 'isl/.libs'), str(GCC15_BUILD / 'prev-isl/.libs'),
    str(TOOLS / 'lib'),
])
CASES = [
    ('gcc-15.2-baseline', ['-O3']),
    ('gcc-15.2-reporter-order', ['-fno-forward-propagate', '-O3']),
]


def utc_now():
    return datetime.now(timezone.utc).isoformat(timespec='microseconds').replace('+00:00', 'Z')


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    return sha256(path.read_bytes())


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def run(argv, cwd, env):
    started = utc_now()
    try:
        proc = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, timeout=65, check=False)
        out, err, code, timed_out = proc.stdout, proc.stderr, proc.returncode, False
    except subprocess.TimeoutExpired as exc:
        out = exc.stdout or b''
        err = exc.stderr or b''
        code, timed_out = None, True
    except OSError as exc:
        out, err, code, timed_out = b'', str(exc).encode('utf-8', 'replace'), None, False
    completed = utc_now()
    return {
        'argv': argv, 'cwd': str(cwd), 'environment': env,
        'started_utc': started, 'completed_utc': completed,
        'outer_timeout_seconds': 65, 'exit_status': code, 'timed_out': timed_out,
        'stdout_bytes': len(out), 'stdout_sha256': sha256(out),
        'stderr_bytes': len(err), 'stderr_sha256': sha256(err),
    }, out, err


def main():
    if RUNS.exists() or RESULTS.exists():
        raise SystemExit('refusing to reuse or overwrite B63 run outputs')
    if not SOURCE.is_file() or sha256_file(SOURCE) != EXPECTED_SOURCE_SHA256:
        raise SystemExit('reporter source missing or hash mismatch; no compile started')
    if not GCC15.is_file() or sha256_file(GCC15) != EXPECTED_XGCC_SHA256:
        raise SystemExit('GCC 15.2.0 driver missing or hash mismatch; no compile started')
    if not GCC15_CC1.is_file() or sha256_file(GCC15_CC1) != EXPECTED_CC1_SHA256:
        raise SystemExit('GCC 15.2.0 cc1 missing or hash mismatch; no compile started')
    for name, expected in GCC_SOURCE_HASHES.items():
        path = GCC_SOURCE / name
        if not path.is_file() or sha256_file(path) != expected:
            raise SystemExit(f'GCC source identity mismatch for {name}; no compile started')

    RUNS.mkdir()
    result = {
        'schema_version': 1,
        'record_id': 'G2-HISTORY-2026-11-13-11',
        'batch': 'B63',
        'source_sha256': EXPECTED_SOURCE_SHA256,
        'consumer_execution': 'NOT_RUN',
        'AICompiler_project_tests': 'NOT_RUN',
        'run_started_utc': utc_now(),
        'case_results': [],
        'overall': 'IN_PROGRESS',
    }
    env = {
        'PATH': f'{TOOLS}/bin:/usr/bin:/bin',
        'LC_ALL': 'C',
        'LANG': 'C',
        'LD_LIBRARY_PATH': GCC15_LD_LIBRARY_PATH,
    }

    for case_id, opts in CASES:
        cwd = RUNS / case_id
        cwd.mkdir()
        shutil.copyfile(SOURCE, cwd / 'small.c')
        source_hash = sha256_file(cwd / 'small.c')
        case = {
            'case_id': case_id,
            'options': opts,
            'source_copy_sha256': source_hash,
            'compile': None,
            'assembly': None,
            'rtl_dump_files': [],
            'status': 'IN_PROGRESS',
        }
        if source_hash != EXPECTED_SOURCE_SHA256:
            case['status'] = 'FAIL_SOURCE_COPY_HASH'
            result['case_results'].append(case)
            result['overall'] = 'FAIL'
            write_json(RESULTS, result)
            continue

        argv = [
            '/usr/bin/timeout', '--signal=TERM', '--kill-after=1s', '60s',
            str(GCC15), f'-B{GCC15_BUILD / "gcc"}/', *opts,
            '-fdump-rtl-ext_dce', '-S', 'small.c', '-o', 'small.s',
        ]
        compile_result, out, err = run(argv, cwd, env)
        (cwd / 'compile.stdout').write_bytes(out)
        (cwd / 'compile.stderr').write_bytes(err)
        (cwd / 'compile.status').write_text(
            'TIMEOUT\n' if compile_result['timed_out'] else
            ('NOT_STARTED\n' if compile_result['exit_status'] is None else f"{compile_result['exit_status']}\n"),
            encoding='utf-8')
        case['compile'] = compile_result

        assembly = cwd / 'small.s'
        if assembly.is_file():
            case['assembly'] = {
                'path': str(assembly), 'bytes': assembly.stat().st_size,
                'sha256': sha256_file(assembly),
            }
        for dump in sorted(cwd.glob('*ext_dce*')):
            if dump.is_file():
                case['rtl_dump_files'].append({
                    'path': str(dump), 'bytes': dump.stat().st_size,
                    'sha256': sha256_file(dump),
                })

        if (compile_result['exit_status'] == 0 and not compile_result['timed_out']
                and case['assembly'] and case['rtl_dump_files']
                and all(x['bytes'] > 0 for x in case['rtl_dump_files'])):
            case['status'] = 'PASS'
        elif compile_result['exit_status'] == 0 and not compile_result['timed_out']:
            case['status'] = 'FAIL_MISSING_ASSEMBLY_OR_EXT_DCE_DUMP'
        else:
            case['status'] = 'FAIL_COMPILE'
        result['case_results'].append(case)
        result['overall'] = (
            'PASS' if len(result['case_results']) == len(CASES)
            and all(c['status'] == 'PASS' for c in result['case_results'])
            else ('FAIL' if any(c['status'].startswith('FAIL') for c in result['case_results'])
                  else 'INCOMPLETE'))
        result['last_case_completed_utc'] = utc_now()
        write_json(RESULTS, result)

    result['run_completed_utc'] = utc_now()
    result['overall'] = (
        'PASS' if len(result['case_results']) == len(CASES)
        and all(c['status'] == 'PASS' for c in result['case_results'])
        else ('FAIL' if any(c['status'].startswith('FAIL') for c in result['case_results'])
              else 'INCOMPLETE'))
    write_json(RESULTS, result)
    print(json.dumps({
        'overall': result['overall'],
        'consumer_execution': result['consumer_execution'],
        'cases': result['case_results'],
    }, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
