#!/usr/bin/env python3
"""Fresh-reset HUD integration recipe; compare semantic state and rendered output.

Route retained from debug/scenarios/sanity.py title/amazon. No stale checkpoints.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0, '/root/romatlas/scripts')
from native_debug_client import PortClient
ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('--binary', required=True)
p.add_argument('--evidence', required=True)
p.add_argument('--compare')
a = p.parse_args()
evidence = Path(a.evidence).resolve()
observations = {}
with PortClient([str(Path(a.binary).resolve()), '--rom', str(ROOT/'input/game.rom'), '--debug-stdio'], seconds=120, max_requests=1500, evidence=evidence) as port:
    identity = port.request('api.describe')
    def state(): return port.request('port.state')['game']
    def step(ticks):
        result = port.request('port.advance', {'ticks': ticks, 'timeout_ms': 300000})
        assert result['complete'] and result['completed'] == ticks
    def wait(predicate, bound):
        for _ in range(bound):
            if predicate(state()): return
            step(1)
        raise AssertionError('semantic setup bound exceeded')
    def edge(actions):
        port.request('port.input', {'actions': []}); step(1)
        port.request('port.input', {'actions': actions}); step(1)
        port.request('port.input', {'actions': []})
    def capture(name):
        s = state()
        frame = port.request('port.framebuffer')
        observations[name] = {'state': s, 'frame_sha256': frame['sha256']}
        port.frame(name+'.ppm')
    wait(lambda s: s['transition']['menu'] == 'Title(Input)', 120)
    capture('title')
    edge(['start'])
    wait(lambda s: s['transition']['menu'] == 'LandInput', 180)
    capture('selection')
    edge(['a'])
    wait(lambda s: s['mode'] == 'gameplay' and s['transition']['foreground'] == 'Yield', 350)
    step(16)
    assert state()['progression']['stage'] == 0
    capture('amazon-entry')
    before = state()['world']['camera']['x']
    port.request('port.input', {'actions': ['right', 'a']}); step(28)
    port.request('port.input', {'actions': ['right', 'down', 'b']}); step(80)
    assert state()['world']['camera']['x'] > before
    capture('amazon-traverse')
    port.request('port.input', {'actions': []}); step(90)
    capture('amazon-countdown')
result = {'identity': identity, 'script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'observations': observations}
if a.compare:
    baseline = json.loads(Path(a.compare).read_text())
    assert baseline['observations'] == observations, 'baseline semantic/frame observations differ'
    result['baseline_comparison'] = 'passed: all five complete semantic states and frame hashes identical'
(evidence/'result.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps({'ok': True, 'landmarks': list(observations), 'comparison': result.get('baseline_comparison')}))
