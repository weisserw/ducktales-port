"""DuckTales showcase adapter. No accepted QA records or game state writes."""
import ast
import json
from pathlib import Path
import sys
import types

SCENARIOS = Path(__file__).resolve().parents[1]/'scenarios'
sys.path.insert(0, str(SCENARIOS))
import common
import g_routes
from scenario_client import SetupFailure


def instrument_j(run):
    """Keep every original assert and its lazy diagnostic; count successful executions."""
    path = SCENARIOS/'j_climb.py'
    source = path.read_text()
    records = {}
    def passed(line, text):
        key = (run.stage, line)
        if key not in records:
            records[key] = dict(name=f'j_climb.py:{line}', expression=text,
                                phase=run.stage, status='passed', executions=0)
            run.assertions.append(records[key])
        records[key]['executions'] += 1
    class Assertions(ast.NodeTransformer):
        def visit_Assert(self, node):
            record = ast.Expr(ast.Call(func=ast.Name(id='_assert_passed', ctx=ast.Load()),
                args=[ast.Constant(node.lineno), ast.Constant(ast.get_source_segment(source, node.test))], keywords=[]))
            return [node, ast.copy_location(record, node)]
    module = types.ModuleType('j_climb_integration')
    module.__file__ = str(path)
    module._assert_passed = passed
    tree = ast.fix_missing_locations(Assertions().visit(ast.parse(source, filename=str(path))))
    exec(compile(tree, str(path), 'exec'), module.__dict__)
    # Bounded owner searches establish observation boundaries, not game oracles.
    def boundary(operation):
        def call(*args, **kwargs):
            try:
                return operation(*args, **kwargs)
            except AssertionError as error:
                detail = error.args[0] if error.args else ''
                reason = detail.get('reason', '') if isinstance(detail, dict) else str(detail)
                if 'inaccessible' in reason:
                    raise SetupFailure(str(error)) from error
                raise
        return call
    module.finish_source_frame = boundary(module.finish_source_frame)
    module.finish_vertical_owner = boundary(module.finish_vertical_owner)
    return module


def run(run, entry):
    if run.expectation != 'original-fidelity':
        raise SetupFailure('No authored adapter for this expectation')
    pair = common.Pair.__new__(common.Pair)
    # Register cleanup before construction so partial transport startup also closes.
    run.stack.callback(lambda: hasattr(pair, 'stack') and pair.stack.close())
    with run.phase('setup'):
        try:
            common.Pair.__init__(pair, run.evidence, reference_request_limit=100000,
                                 compressed_reference=True)
        finally:
            for name in ('native', 'reference'):
                if hasattr(pair, name):
                    run.clients[name] = getattr(pair, name)
        run.stack.callback(lambda: common.save(run.evidence/'inputs.json', pair.steps))
        def retain_failure(exc_type, error, tb):
            if exc_type is not None:
                common.save(run.evidence/'observations-partial.json', getattr(pair, 'observations', []))
                try:
                    pair.capture('failure')
                except Exception as capture_error:
                    common.save(run.evidence/'failure-capture-error.json', dict(error=str(capture_error)))
            return False
        run.stack.push(retain_failure)
        j = instrument_j(run)
        provenance = common.identity()
        common.save(run.evidence/'provenance.json', dict(provenance=provenance,
            adapter_sha256=common.sha(__file__), harness_sha256=common.sha('/root/romatlas/scripts/qa_harness.py'),
            source_contract_sha256=common.sha(SCENARIOS/'j-source-contract.md'),
            route_sha256=common.sha(SCENARIOS/'routes/climb-vertical.json')))
        for path in [SCENARIOS/'j_climb.py', SCENARIOS/'g_routes.py', SCENARIOS/'common.py',
                     SCENARIOS/'j-source-contract.md', SCENARIOS/'routes/climb-vertical.json', Path(__file__), Path('/root/romatlas/scripts/qa_harness.py')]:
            (run.evidence/path.name).write_bytes(path.read_bytes())
        keys = ['rom','executable','reference_executable','source','frozen_source','overrides']
        try:
            old = json.loads((j.SETUP/'gap-checkpoint-preconditions.json').read_text())
            stale = [key for key in keys if provenance[key] != old['provenance'][key]]
        except FileNotFoundError:
            stale = ['missing_checkpoint_metadata']
        for name in ('gap-checkpoint.json', 'gap-reference-snapshot.json'):
            if not (j.SETUP/name).is_file():
                stale.append(f'missing:{name}')
        if stale:
            # Regenerate via the retained G natural input recipe, never edit identities.
            points = []
            game = run.check('regenerate natural gap recipe', lambda: g_routes.natural_gap(pair, points))
            common.save(run.evidence/'gap-checkpoint.json', pair.native.request('port.checkpoint.export'))
            pair.reference.request('emulator.snapshot.export', dict(path=str((run.evidence/'gap-reference-snapshot.json').resolve())))
            common.save(run.evidence/'gap-checkpoint-preconditions.json', dict(native=game,
                source_yield=pair.reference_yield_return(), provenance=common.identity()))
            j.SETUP = run.evidence
        common.save(run.evidence/'checkpoint-selection.json', dict(stale_identity_fields=stale,
            selected=str(j.SETUP), regenerated=bool(stale)))
        points = []
        game = run.check('identity-checked gap restore and natural rope entry', lambda: j.natural_rope(pair, points))
    with run.phase('execution'):
        return run.check('source-backed climb and sixty-phase transition',
                         lambda: j.climb_vertical(pair, prepared=(game, points)))
