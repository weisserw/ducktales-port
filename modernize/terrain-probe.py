#!/usr/bin/env python3
"""Compile a direct-owner terrain probe against a freshly built native library.

Covers all original terrain banks, mutations, column/vertical geometry and
secondary aliases. Game scheduling/publication remains covered by source-backed
native integration contracts.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--evidence', required=True)
p.add_argument('--compare')
p.add_argument('--library')
p.add_argument('--modernized', action='store_true')
a = p.parse_args()
out = Path(a.evidence).resolve()
out.mkdir(parents=True, exist_ok=True)
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
deps = ROOT/'port/target/debug/deps'
library = Path(a.library).resolve() if a.library else ROOT/'port/target/debug/libducktales_port.rlib'
externs = {'ducktales_port': library}
for name in ['romatlas_port', 'serde_json']:
    externs[name] = max(deps.glob('lib'+name+'-*.rlib'), key=lambda path: path.stat().st_mtime_ns)
source = ROOT/'modernize/terrain-probe.rs'
cmd = ['rustc', '+1.90.0', '--edition=2024', str(source), '-L', 'dependency='+str(deps), '-o', str(out/'probe')]
if a.modernized:
    cmd.extend(['--cfg', 'modernized'])
for name, path in externs.items():
    cmd.extend(['--extern', name+'='+str(path)])
subprocess.run(cmd, cwd=ROOT, check=True)
subprocess.run([str(out/'probe'), str(ROOT/'input/game.rom'), str(out)], cwd=ROOT, check=True)
result = {'probe_source_sha256': digest(source), 'driver_sha256': digest(Path(__file__)), 'library_sha256': digest(library), 'binary_sha256': digest(out/'probe'), 'rom_sha256': digest(ROOT/'input/game.rom'), 'compile_command': cmd, 'observations_sha256': digest(out/'observations.json'), 'captures': {path.name: digest(path) for path in sorted(out.glob('*.ppm'))}, 'scope': 'Direct world/secondary alias fixture, source terrain CHR and fixed review palette. Not a natural route, full Game scheduler or original emulator comparison.'}
if a.compare:
    prior = json.loads((Path(a.compare)/'result.json').read_text())
    assert result['observations_sha256'] == prior['observations_sha256'], 'terrain events or display writes differ'
    assert result['captures'] == prior['captures'], 'terrain page pixels differ'
    result['comparison'] = 'passed: all terrain changes, collision/attribute/queue state, logical writes and sampled pixels identical'
(out/'result.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps({'ok': True, 'pages': len(result['captures']), 'comparison': result.get('comparison')}))
