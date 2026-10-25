#!/usr/bin/env python3
"""Capture this workspace's modernization baseline without a full-game QA gate."""
import json
from pathlib import Path
import sys


def main():
    work = Path(__file__).resolve().parents[1]
    metadata = json.loads((work / 'workspace.json').read_text())
    sys.path.insert(0, str(Path(metadata['toolkit']) / 'scripts'))
    import port_modernize

    source = port_modernize.source_identity(work)
    names = ['workspace.json', 'input/game.rom', 'debug/acceptance.json',
             'debug/plan.json', 'debug/qa.json', 'debug/sanity.json',
             'debug/sanity-results.json', 'debug/sanity-review.md']
    record = dict(version=1, status='baseline_recorded', source=source,
                  evidence_files={name: port_modernize.digest(work / name)
                                  for name in names if (work / name).is_file()},
                  qa_prerequisite='none', full_game_qa_assessed=False,
                  semantic_correctness_assessed=False, pilot_started=False)
    destination = work / 'modernize/baseline.json'
    with destination.open('x') as output:
        output.write(json.dumps(record, indent=2) + '\n')
    print(json.dumps(dict(ok=True, baseline=str(destination), pilot_started=False)))


if __name__ == '__main__':
    main()
