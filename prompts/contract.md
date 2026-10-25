# Stage 3: source-backed behavior contracts

Read workflow.md, prompts/common.md, architecture.md, behaviors.md, debug/api.md
and the existing source tree. This stage writes executable unit tests before game
implementation. Apply the coding requirements below to all test code, fixtures,
assertions, constants, interfaces and adjacent trace documentation.

## Coordinator and worker handoffs

If assigned the stage, coordinate one worker subagent for each behavior area in
behaviors.md. Give each the area ID, relevant files/interfaces, source revision
and this entire prompt. Run workers sequentially to preserve shared file/state
ownership. Record worker status and evidence/test paths per area. Reconcile shared
interfaces, cross-area effects and duplicate tests after each handoff. New areas
receive their own workers; do not silently omit discoveries. Workers do not spawn
further workers or proceed to game implementation.

If assigned an area, trace and write its contracts, then hand off to this stage's
coordinator. Work in the logical source modules established by the skeleton; create
new files/modules when needed to keep ownership clear. Add minimal type/interface
stubs so tests compile, with unimplemented behavior visibly pending. Do not write
the gameplay implementation or a second implementation as a test oracle.

## Independent detailed source tracing

The planning inventory is a navigation aid, never a source of truth. Independently
reread original source and relevant data for the assigned area. Check comments
against instructions/consumers and record corrections with evidence. Follow
callers, callees, dispatch/data tables, shared-state readers/writers and relevant
interrupt/deferred work until their effects on the behavior are understood. Do
not treat a call as pure or assume flags/registers survive it.

Trace inputs, entry preconditions, branches, results and ordering in detail.
Account for the source CPU's operand widths, signedness, overflow/wraparound,
carry/borrow propagation, flag lifetimes, shifts/rotates, compare/branch semantics,
register/memory aliasing, bank/context changes, table bounds and persisted state
where they affect behavior. Consult the actual platform documentation; do not
apply one console's CPU assumptions to another. Preserve observable effects using
native concepts, without building an emulated CPU/RAM map into the game.

Document the trace alongside the tests, in module/test comments or a neighboring
source-tree trace file linked from the tests. Include source paths/labels and
bank/physical ranges, reference revision, calling contracts, cross-call effects,
branch derivations and each expectation's evidence. Name source-defined values;
show arithmetic/flag reasoning where it determines an expected result. Never
invent intent, copy planning claims as facts or use the future implementation's
output as the expectation. Mark unavoidable source-informed assumptions and
residual uncertainty explicitly in tests and uncertainties.json.

## Executable contracts and completion

Write unit tests for every area's observable rules and meaningful normal, boundary
and alternate branches, including cross-system ordering/effects. Cover loading/
asset formats, graphics/layout rules, sound sequencing, controls, actors, modes
and transitions as applicable. Use source-derived expected results and independent
fixtures, with synthetic inputs or local ROM access; do not embed original assets.
Test public/domain interfaces and debug observations where relevant. A trivial
assertion or test of only a mock cannot stand in for the area's behavior.

Use descriptive test names, semantic identifiers and useful comments explaining
preconditions, expected effects and source reasoning. Add declarations/stubs in
the modules that will receive implementation in stage 4. Register tests in native
cargo test so browser module gating cannot hide them. Never ignore, disable or
replace substantive tests with tautologies to make a handoff pass.

Compile the test suite (cargo test --locked --manifest-path port/Cargo.toml
--no-run), list discovered tests and run the area's tests. Record actual results,
distinguishing expected assertion/unimplemented failures from compile errors,
hangs or faulty fixtures. Infrastructure tests must remain green. Stage 3 does
not require behavior tests to pass; it requires runnable tests of the pending
implementation, source-linked traces and complete area coverage. Do not run the
final implementation audit on this intentionally unfinished game.

Update behaviors.md with trace paths, test names, cross-area effects, assumptions
and handoff results. The stage coordinator checks that every area has had its
own worker, reconciles dependencies and compiles the combined suite. Stop when
all areas have substantive, discovered contracts ready for implementation.

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
