# Stage 4: implement the whole native game

Read workflow.md, prompts/common.md, architecture.md, behaviors.md, debug/api.md,
the behavior tests and their adjacent source traces. Implement the actual game
logic in the prepared files until all unit tests pass. Read the relevant original
source while coding; contracts and planning do not replace it. Preserve work;
do not reinitialize. A scaffold, first level or playable demo is not completion.

## Write readable native code from the first implementation

Implement behavior in terms of the current game's concepts, not a transcription
of assembly operations with the same unexplained numbers. Understand the relevant
source and consumers, define meaningful types and names, then use them while
writing the logic. Do not postpone naming or comments to a later cleanup pass.

Do not use bare numeric literals for values whose meaning comes from the game
or original hardware. The following examples illustrate categories, not mechanics
or names that every game must have:

| Avoid in executable game logic | Express the meaning instead |
| --- | --- |
| `rom[0xf4db]` or repeated numeric ROM ranges | A named ROM location or asset-loader definition, followed by decoded data |
| `memory[0xc120]` for game state | A named field in a native state structure |
| `flags |= 1 << 14` or `flags & 0x40` | A named flag/mask or typed flag operation |
| `state == 3` or `set_state(0x2a)` | An enum variant, or a named ID when an integer representation is needed |
| `distance < 16` or `position >= 176` | A constant identifying the range or boundary, with units/coordinate space |
| `speed -= 4` or `timer = 30` | A named rate, increment or duration in the appropriate units |

Small decimal integers need meaningful names just as hexadecimal addresses do.
Names must explain the role: `STATE_3`, `VALUE_16` and `DATA_F4DB` only repeat the
encoding and are insufficient. Reuse existing names instead of creating aliases.
Use enums for finite semantic choices and typed interfaces where practical; use
scoped named constants when numeric representation is appropriate. Keep original
encodings and numeric locations in definitions, loaders or serialization code,
not scattered at call sites. Distinguish banked CPU addresses from physical ROM
offsets. Original RAM addresses should become native state fields, not a second
memory map. Preserve exact data encodings and deterministic behavior.

Write useful comments alongside the implementation. Document major modules and
important type/interface contracts. For a condition, calculation or transition
whose purpose is not apparent from its names, explain what its inputs represent,
what rule it implements and what effect it has. Also explain relevant invariants,
update ordering, source quirks and intentional differences. Naming each constant
alone is insufficient if the combined expression still obscures the behavior.
Derive names and explanations from this game's source; distinguish facts from
assumptions. If a meaning is unresolved, use an honest provisional name and state
what is known without inventing intent. Cite source labels or addresses when
useful for traceability. Avoid comments that only paraphrase the Rust statements.

Ordinary literals whose meaning is evident, such as zero initialization, and
opaque ROM table contents need no artificial constants. Executable conditions,
call arguments and game parameters are not opaque data merely because they came
from assembly. No abstraction or comment-count quota applies. These are coding
requirements, not an additional review phase, report or gameplay traversal.

## Implement the contracts and maintain the design

Implement all gameplay, actors, title/menu logic, display, sound, scene transitions,
asset preparation and other inventoried behavior. Keep modules cohesive, ownership
explicit and interfaces small. Add files where they improve logical organization.
Use modern native algorithms while preserving source-backed rules, cross-call
side effects and ordering captured by contracts. All logic must run headlessly
and expose the semantic state/events needed by unit tests and later QA through
the debug API. Extend debug/api.md and observations as systems become real.

When a test or plan contradicts source, reread the source/callers/data, document
the evidence and correct the affected contract in place. Retain trace/history
and area IDs. Do not weaken valid expectations, skip tests or substitute stubs
just to obtain green results. Add contracts for newly discovered required areas
before implementing them; reconcile the inventory without restarting the pipeline.

After substantial features inspect affected boundaries, fixing concrete coupling,
duplication and unclear algorithms before building on them. Update architecture.md
when decisions change. Finish with one bounded maintainability pass for semantic
numeric literals, bare addresses and missing/stale explanations; do not repeatedly
redesign a coherent implementation.

## Implementation and behavior accounting

Use ledger.json to account for all archived regions and meaningful units as
implemented, merged, replaced or excluded, with a behavior explanation and native
equivalents. Add discovered units while preserving original IDs and ranges.
Implement every gameplay-relevant equivalent in the whole game; modern methods may merge routines
or remove original mechanisms. Judge obvious irrelevant junk/unreachable paths
from source without a formal reachability proof. Favor implementing borderline
or unclear behavior. Lack of observed execution alone does not justify exclusion.

Incomplete disassembly/classification is allowed. Assess unknown ranges and
semantic ambiguities by their consumers and port impact as implementation reaches
them. Do not complete the whole-ROM mapping task as a port prerequisite. Prefer
source reading, data inspection and existing documentation. Where a reasonable
source-informed implementation is possible, implement it, record the assumption,
basis and residual uncertainty, and continue. Distinguish an implementation
assumption from an established fact about the original. Do not demand proof of
every interpretation or runtime confirmation of every branch.

For uncertain data, keep original values ROM-backed when a supported consumer can
interpret them; unit-test loading/bounds where useful. Preserving code bytes is
not native behavior. Do not silently skip handlers, zero unknown fields or label
behavior irrelevant to finish accounting. Missing required implementations remain
incomplete; implemented best-effort behavior with documented assumptions does not
remain blocked merely because fidelity has not been independently confirmed.
Use prompts/reference-updates.md for supported source corrections; its map and
byte-identical rebuild requirements protect reference integrity only. Never publish
an assumption as confirmed classification. Frozen reference/ stays unchanged.

## Checks and handoff

Run python3 run.py check for formatting, clippy, unit tests and desktop/Wasm
compilation. All behavior contracts must pass and all required behavior must be
implemented; green tests alone do not excuse missing game systems. Follow
prompts/common.md for bounded source-question experiments and relevant toolkit
regressions after library changes. Do not execute an additional gameplay smoke
route or full fidelity pass here; stage 5 owns runtime QA.

Complete behaviors.md links, ledger/uncertainty assessment (including implemented
assumptions), README.md controls/running/asset replacement/added-level/game-rule
instructions, BUILDING.md build and actual check results, differences.md and
library-changes.md. Provide an original-content-only extension manifest. Document
python3 run.py web, original battery saving and untested device/platform behavior.

Set ledger completion.status=complete and run python3 run.py audit; after it
passes, stop this subprocess and hand off to QA. Record changed systems, contract
results, debug schema and reproducible QA entry recipes in progress.md. Overall
completion still requires two fresh whole-game QA passes. Do not mark missing
behavior implemented. Leave shared-library edits uncommitted for separate review.
No check-in, publication or agent launch is authorized by this implementation
prompt. Release packaging and dependency pinning remain later work.
