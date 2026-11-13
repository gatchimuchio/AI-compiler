#!/usr/bin/env python3
"""Run the four pre-registered GCC PR 116488 reporter-source cases once."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess

ROOT = Path('/tmp/aic-g2-gcc-pr116488-reporter-reproducer-2026-11-13')
SOURCE = ROOT / 'source/reporter-small.c'
RUNS = ROOT / 'runs'
EXPECTED_SOURCE_SHA256 = '77a92761fadde9449f03271f6f186f59091f1a5c697a55f523b4388ab9f405c2'
GCC15_BUILD = Path('/tmp/aic-gcc15.2-pr116488-2026-11-13/build/gcc-15.2.0')
GCC15 = GCC15_BUILD / 'gcc/xgcc'
GCC15_B = GCC15_BUILD / 'gcc'
GCC15_LD_LIBRARY_PATH = ':'.join([
    '', str(GCC15_BUILD / 'gcc'),
    str(GCC15_BUILD / './gmp/.libs'), str(GCC15_BUILD / './prev-gmp/.libs'),
    str(GCC15_BUILD / './mpfr/src/.libs'), str(GCC15_BUILD / './prev-mpfr/src/.libs'),
    str(GCC15_BUILD / './mpc/src/.libs'), str(GCC15_BUILD / './prev-mpc/src/.libs'),
    str(GCC15_BUILD / './isl/.libs'), str(GCC15_BUILD / './prev-isl/.libs'),
    '/tmp/aic-gcc15-pr116488-2026-11-13/tools/lib',
])
CASES = [
    ('debian-gcc-14.2-baseline', ['/usr/bin/gcc-14', '-O3', 'small.c'], False),
    ('debian-gcc-14.2-trigger-flag', ['/usr/bin/gcc-14', '-fno-forward-propagate', '-O3', 'small.c'], False),
    ('gcc-15.2-baseline', [str(GCC15), f'-B{GCC15_B}/', '-O3', 'small.c'], True),
    ('gcc-15.2-trigger-flag', [str(GCC15), f'-B{GCC15_B}/', '-fno-forward-propagate', '-O3', 'small.c'], True),
]


def utc_now():
    return datetime.now(timezone.utc).isoformat(timespec='microseconds').replace('+00:00', 'Z')


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def write_text(path, value):
    path.write_text(value, encoding='utf-8')


def run(argv, cwd, env, timeout, stem):
    started = utc_now()
    write_text(cwd / f'{stem}.started.utc', started + '\n')
    try:
        proc = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, timeout=timeout, check=False)
        out, err, code, timed_out = proc.stdout, proc.stderr, proc.returncode, False
    except subprocess.TimeoutExpired as exc:
        out = exc.stdout or b''; err = exc.stderr or b''; code = None; timed_out = True
    except OSError as exc:
        out = b''; err = str(exc).encode('utf-8', 'replace'); code = None; timed_out = False
    completed = utc_now()
    (cwd / f'{stem}.stdout').write_bytes(out)
    (cwd / f'{stem}.stderr').write_bytes(err)
    write_text(cwd / f'{stem}.status', 'TIMEOUT\n' if timed_out else ('NOT_STARTED\n' if code is None else f'{code}\n'))
    write_text(cwd / f'{stem}.completed.utc', completed + '\n')
    return {'argv': argv, 'cwd': str(cwd), 'started_utc': started, 'completed_utc': completed,
            'timeout_seconds': timeout, 'exit_status': code, 'timed_out': timed_out,
            'stdout_bytes': len(out), 'stdout_sha256': sha256(out),
            'stderr_bytes': len(err), 'stderr_sha256': sha256(err)}


def main():
    if RUNS.exists():
        raise SystemExit(f'refusing to reuse or overwrite existing run directory: {RUNS}')
    if not SOURCE.is_file() or sha256(SOURCE.read_bytes()) != EXPECTED_SOURCE_SHA256:
        raise SystemExit('source is missing or SHA-256 does not match preregistration')
    if not GCC15.is_file() or not Path('/usr/bin/gcc-14').is_file():
        raise SystemExit('one or more pre-registered compiler binaries are missing')
    RUNS.mkdir(parents=True)
    result = {'schema_version': 1, 'record_id': 'G2-HISTORY-2026-11-13-10',
              'batch': 'B62', 'source_sha256': EXPECTED_SOURCE_SHA256,
              'run_started_utc': utc_now(), 'case_results': [], 'overall': 'IN_PROGRESS'}
    result_path = ROOT / 'observations/results.json'
    for name, argv, gcc15 in CASES:
        cwd = RUNS / name
        cwd.mkdir()
        shutil.copyfile(SOURCE, cwd / 'small.c')
        source_hash = sha256((cwd / 'small.c').read_bytes())
        env = {'PATH': '/tmp/aic-gcc15-pr116488-2026-11-13/tools/bin:/usr/bin:/bin' if gcc15 else '/usr/bin:/bin',
               'LC_ALL': 'C', 'LANG': 'C'}
        if gcc15:
            env['LD_LIBRARY_PATH'] = GCC15_LD_LIBRARY_PATH
        case = {'case_id': name, 'compiler': 'GCC 15.2.0 xgcc build' if gcc15 else 'Debian GCC 14.2.0-19',
                'source_copy_sha256': source_hash, 'compile': None, 'execution': None,
                'expected_stdout': '0\n-1\n', 'status': 'IN_PROGRESS'}
        if source_hash != EXPECTED_SOURCE_SHA256:
            case['status'] = 'FAIL_SOURCE_COPY_HASH'
            result['case_results'].append(case)
            result['overall'] = 'FAIL'
            write_text(result_path, json.dumps(result, ensure_ascii=False, indent=2) + '\n')
            continue
        case['environment'] = env
        compile_argv = ['/usr/bin/timeout', '--signal=TERM', '--kill-after=1s', '60s', *argv]
        case['compile'] = run(compile_argv, cwd, env, 65, 'compile')
        if case['compile']['exit_status'] == 0 and not case['compile']['timed_out']:
            exec_argv = ['/usr/bin/timeout', '--signal=TERM', '--kill-after=1s', '5s', './a.out']
            case['execution'] = run(exec_argv, cwd, env, 8, 'execution')
            observed = (cwd / 'execution.stdout').read_bytes()
            if (case['execution']['exit_status'] == 0 and not case['execution']['timed_out']
                    and observed == b'0\n-1\n'):
                case['status'] = 'PASS'
            else:
                case['status'] = 'FAIL'
        else:
            case['status'] = 'NOT_RUN'
        result['case_results'].append(case)
        result['overall'] = 'PASS' if len(result['case_results']) == len(CASES) and all(c['status'] == 'PASS' for c in result['case_results']) else ('FAIL' if any(c['status'].startswith('FAIL') for c in result['case_results']) else 'INCOMPLETE')
        result['last_case_completed_utc'] = utc_now()
        write_text(result_path, json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    result['run_completed_utc'] = utc_now()
    result['overall'] = 'PASS' if len(result['case_results']) == len(CASES) and all(c['status'] == 'PASS' for c in result['case_results']) else ('FAIL' if any(c['status'].startswith('FAIL') for c in result['case_results']) else 'INCOMPLETE')
    write_text(result_path, json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'overall': result['overall'], 'cases': result['case_results']}, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
