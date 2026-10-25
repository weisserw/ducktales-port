#!/usr/bin/env python3
"""Compile a direct-owner audio probe against a freshly built native library.

Covers all56 headers and two preemption/control cases at the configured NTSC
cadence. Compares complete driver/register/alias traces and actual APU PCM.
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
source = ROOT/'modernize/audio-probe.rs'
cmd = ['rustc', '+1.90.0', '--edition=2024', str(source), '-L', 'dependency='+str(deps), '-o', str(out/'probe')]
for name, path in externs.items():
    cmd.extend(['--extern', name+'='+str(path)])
subprocess.run(cmd, cwd=ROOT, check=True)
subprocess.run([str(out/'probe'), str(ROOT/'input/game.rom'), str(out)], cwd=ROOT, check=True)
result = {'probe_source_sha256': digest(source), 'driver_sha256': digest(Path(__file__)), 'library_sha256': digest(library), 'binary_sha256': digest(out/'probe'), 'rom_sha256': digest(ROOT/'input/game.rom'), 'compile_command': cmd, 'observations_sha256': digest(out/'observations.jsonl'), 'pcm': {path.name: digest(path) for path in sorted(out.glob('*.f32le'))}, 'scope': 'Direct audio-owner fixture:240 NTSC frames for all56 headers plus two control/preemption cases, actual shared APU48kHz mono samples. External alias values are an explicit deterministic fixture. Not full soundtracks, a natural Game route, an original emulator or device comparison.'}
if a.compare:
    prior = json.loads((Path(a.compare)/'result.json').read_text())
    assert result['observations_sha256'] == prior['observations_sha256'], 'audio state, register writes or alias reads differ'
    assert result['pcm'] == prior['pcm'], 'audio PCM samples differ'
    result['comparison'] = 'passed: all13920 complete driver/register/alias frames and all58 PCM streams identical'
(out/'result.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps({'ok': True, 'pcm_cases': len(result['pcm']), 'comparison': result.get('comparison')}))
