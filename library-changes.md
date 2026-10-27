# Library changes

Stage4 made no changes to /root/romatlas. The port uses the existing shared Game,
MemoryAssets, desktop/browser settings, debug JSON-RPC, replay/checkpoint, NES
pixel helpers and audio/transport interfaces. No shared-library commit or
publication was made. review/toolkit-before/ and unrelated existing toolkit work
remain preserved. Game-specific ownership, source queues, aliases, content formats
and semantic journal remain local to port/src/.


## Conditional QA tooling (2026-10-23)

Uncommitted toolkit edits add shared CommandAdapter-based emulator.observe and
emulator.run_until, native port.run_until, a persistent ScenarioSession client,
regression tests and scenario-authoring guidance. Pre-edit tracked toolkit files
are retained in review/toolkit-conditional-before/ (captured from initially clean
HEAD); newly added files have no predecessor. No game implementation or accepted
QA records were modified. Native bookmark restore remains replay-based.

Local benchmark: debug/tooling/conditional_execution_benchmark.py reuses the
natural-input K087 predecessor snapshot and compares exact single-operation
client polling against server-side conditional execution. Both stop after10359
operations with identical state/observations. debug/artifacts/tooling-conditional-002/result.json
records10.137s/20719requests versus0.702s/1request (14.43x in this experiment).
Initial trial001 exhausted the existing8MiB transport logging limit; its logs are
retained. Trial002 uses the existing lossless bounded codec, not a raised log cap.
This measures synchronization, not full-game fidelity or whole-QA acceleration.

DuckTales debug/scenarios/g_routes.py now uses capability-detected server-side
conditional owner completion, retaining the original polling helper as fallback.
The same composition/task/delay/DC28 contract and32768operation bound are retained;
per-call observations are saved. The benchmark validates this exact predicate at
the previously missed K087 boundary. Accepted QA records are untouched. Existing
reference-executable provenance checks may require recipe regeneration after the
toolkit rebuild; identities are not bypassed.

Follow-up integration trial003 also executes the installed DuckTales helper:
10359operations and identical state/observations/player result,1RPC,1.112s.
The separate server trial took0.867s versus12.828s manual polling (14.79x).
The original g_routes.py is retained in review/tooling-before/g_routes.py.
Checks passed so far: Rust1.90 locked release build, agent/port tests, complete
workspace tests in release mode, targeted all-target desktop Clippy, Python
scenario client/session/API/boundary/portability checks and QA installer tests.

Final validation also passes full workspace all-target Clippy, desktop runtime
tests, port preparation/build/accounting regressions and formatting/whitespace.
Logs/commands are retained in debug/artifacts/tooling-validation-001/.
A separately built native executable at /root/romatlas/target/release/ducktales-port
passes conditional title/land stepping and two identical branches after native
checkpoint restore (12RPCs; title tick36, land tick70), recorded in
 debug/artifacts/tooling-native-conditional-001/result.json. The active QA debug
executable was not replaced. No full fidelity pass or QA acceptance is claimed.

## Minimal native QA integration harness (2026-10-24)

Uncommitted toolkit additions `scripts/qa_harness.py` and
`scripts/test-qa-harness.py` provide manifest-driven single-test/suite execution,
explicit documented expectation selection, isolated evidence, setup-failure versus
game-mismatch results, phase timing/request counts and named assertions. The new
section in `docs/port-debugging.md` documents the adapter contract and rerun command.
Prior documentation is retained in `review/tooling-harness-before/`; new files
have no predecessor. All pre-existing conditional-tooling edits are preserved.

DuckTales showcase lives in `debug/tooling/integration.json` and
`integration_adapter.py`; its local `test_integration_adapter.py` checks owner
boundary failure classification and preservation of original assertions.
`debug/scenarios/j_climb.py` only adds an optional already-validated prepared
rope state so the runner can time setup separately; its ordinary QA entry point
still runs the full original setup. The predecessor is retained in the review
directory. No gameplay implementation changed. J's source assertions are executed
unchanged with successful per-line execution counters; exhaustion of a bounded
owner search is a setup failure. No expectations are learned from observations.

The old G009 checkpoint differs in native executable, reference executable and
native source identity. Each tooling run regenerates its paired gap checkpoint
through retained `g_routes.natural_gap`, imports with original identity guards,
checks full canonical/source-yield/player/terrain preconditions, and executes the
natural rope route before J. Accepted checkpoints and QA records are untouched.
Original-fidelity is selected explicitly; any future alternative needs a documented
manifest profile and deliberately authored adapter assertions.

Checks passed: harness synthetic regression (3 tests including suite continuation,
profile rejection, fresh evidence, failure classification, metrics and cleanup),
scenario client, QA installer (7 tests), portability, boundaries, persistent
session, API (50 methods/37 rejected cases), compact client storage/stderr,
local DuckTales adapter checks, Python compilation, Rust formatting and whitespace.
See `debug/artifacts/tooling-harness-checks-001/result.json` for commands.
No Rust implementation changed in this task. Existing debug game and release
toolkit executables are reused and hashed, with no build/replacement, GUI launch,
publication or commit. Starting free disk was 214 MiB; no unrelated files were cleaned.

First fresh J run: `debug/artifacts/tooling-harness-j-001/`, passed; setup 277.284s,
execution 42.319s, 1,863 native / 16,161 reference requests; 40 assertion sites/groups and
8,126 successful executions. Second fresh run and reproducibility comparison are
recorded separately when complete. These are tooling-validation results, not a
resumed QA workflow or another accepted pass.

Second fresh J run (`debug/artifacts/tooling-harness-j-002/`, suite entry point)
passed: setup 270.183s, execution 44.906s, 1,863 native / 16,161 reference
requests, the same 40 assertion sites/groups and 8,126 successful executions.
`debug/artifacts/tooling-harness-repro-001/result.json` verifies identical inputs,
climb movement/trajectory, all vertical phase facts, final native/reference
states, selected native/reference captures, request counts and assertion records.
The QA/plan/acceptance file hashes match the validation baseline. Both runs retain
exact scenario/adapter/harness source snapshots and explicit invocation records.
Run 001 predates diagnostic-only runner/adapter refinements (CLI validation,
failure-location reporting, missing-checkpoint regeneration and exhausted-owner
classification); neither run changes any successful game assertion or setup recipe.
Run 002 exercises the final runner/adapter, including the suite entry point.

Measured remaining bottlenecks: setup takes about 86% of total wall time; the
showcase regenerates the stale historical gap each time rather than promoting a
cache. Native imports replay from reset. J's retained vertical-owner helper still
polls in 64-operation batches, while ordinary gameplay uses server-side conditions.
Both runs made 8,404 reference memory requests and 6,131 reference step requests.
The existing debug native executable was deliberately reused; no optimized-build
performance claim is made. These are possible future tooling improvements, not
reasons to alter source expectations or QA acceptance. No game failures occurred.


## Cached J setup and conditional vertical observation (2026-10-24)

Uncommitted toolkit change adds explicitly advertised eq/ne operators to shared
emulator.run_until; missing fields remain errors. Adapter boundary remains
CommandAdapter-only. Added inequality/transient/missing-field regression coverage.
Pre-edit sources retained in review/tooling-cache-before/.

DuckTales integration adapter now content-addresses the validated natural rope
entry using build/ROM/assets/native/frozen source, expectation and recipe hashes.
Files are content-checked, imports preserve identity guards, and restored full
native/source semantic preconditions are checked before use. Cold setup stages and
validates the paired checkpoint before atomic publication; warm runs execute the
same source-backed gameplay assertions afresh. Cache manifests retain origin and
payload hashes. Cache data stays local under debug/cache/.

J's vertical observer now delegates its caller/composition/fresh-IRQ conditions to
the server, with the same32768operation bound and an advertised-capability fallback.
No game rules, expected values or accepted QA records were changed. Initial cold
launcher failed to import the sibling cache module; module search path corrected
before game execution. The startup diagnostic is retained in
 debug/tooling/cache-j-cold-001.log. Execution evidence/results follow below.

Cold run tooling-cache-j-cold-001 passed:306.517s setup +49.991s execution,
356.540s total; validated entry published. Warm single-test run001 passed in
60.732s (11.989setup/48.729execution); warm suite run002 passed in58.651s
(11.986setup/46.645execution). Each warm run makes327native/1348reference
requests. Vertical execution reference requests fell12290→1334, but its wall time
is roughly unchanged; measured end-to-end improvement comes from cached setup.
Relative to prior315.140s run this is approximately5.2–5.4x faster.

Comparison tooling-cache-comparison-001 verifies identical execution trajectories,
all vertical phase facts, final native state and selected source/native images
against the prior successful harness run, for cold and both warm runs. Accepted
QA/plan/acceptance hashes remain unchanged. Cold setup and restore use only retained
natural input recipes and identity-guarded APIs, with no state writes. Native
restore remains replay-based (about12s setup); cold setup remains several minutes.

Checks pass: locked release build; agent and full workspace release tests; all-target
workspace release Clippy; session/API/boundary/QA installer/portability/harness/
scenario-client regressions; local cache invalidation/corruption and vertical
fresh-IRQ tests; formatting/whitespace. Logs are in tooling-cache-validation-001.
Cache snapshots are excluded by debug/cache/ in .gitignore. No gameplay
implementation, native QA executable, accepted records, publication or commit changed.


## QA readiness checkpoint (2026-10-24)

Toolkit QA prompts now require the common harness, explicit expectations and named
assertion groups, bounded source-informed route discovery, origin recipes and
identity-checked cached setups. Empty adapters cannot pass. ScenarioSession records
successful branches, excludes abandoned inputs on restore and replays from reset
with native build/ROM/asset identity checks. Indeterminate/partial operations block
recipe export until a reset/restore.

New DuckTales title-menu authoring probe exercises Right, rewind, Left, recipe
export and identical-state replay. Both local and portable probes pass. Owned
fixture variants demonstrate independent one/two-pixel movement contracts and an
unchanged pause invariant: intentional divergence fails original, passes its new
profile, while a pause bug still fails. DuckTales gameplay remains unchanged.

Reviewed all32 plan entries without deleting distinct coverage or changing accepted
records; debug/tooling/coverage-review.json separates prerequisites from success
conditions and bounds representative checks. K need not use the expensive Moon
approach when another source-legitimate horizontal boundary is available. Installed
QA guide/goal updated; predecessors retained in review/qa-readiness-before/.

ROM-free example source under toolkit examples/qa/ducktales versions the J adapter,
cache/helper code, authoring probe and scope review because this game directory is
not a Git repository. Original ROM/source, snapshots/captures and acceptance records
remain local. The portable copy resolves ROMATLAS_GAME_WORKSPACE explicitly; full J
validation still uses the existing game-owned helper copies.

Final J tooling rerun passed in63.90s with identical movement, vertical phase facts,
final native state and selected native/reference images against cached run002.
QA/plan/acceptance hashes remain unchanged. Locked release build,177workspace tests,
workspace all-target release Clippy, formatting, API/session/boundary/portability,
QA installer, harness, authoring and variant regressions pass. Validation record:
 debug/artifacts/tooling-qa-readiness-001/result.json. No full QA or modernization
workflow was started. Toolkit QA commit is recorded after creation below.

QA tooling/workflow checkpoint committed in ROMAtlas as7443dc7. Paused before any
modernization workflow work, which remains reserved for a separate commit.


## Optional modernization beta preparation (2026-10-24)

Installed modernize/goal.md and empty candidate/change/status records only. No
candidate inspection, code change, baseline capture, pilot or full QA was run.
The beta remains outside the five-stage pipeline; new port preparation installs
it without invoking it. Toolkit port_modernize.py checks completed full two-pass
QA records, rejects scoped/partial/stale records, and can later record exclusive
baseline hashes only after that gate. Administrative checks do not prove fidelity;
the instructions also require provenance review and a reproducible faithful backup.

DuckTales gate correctly reports awaiting_full_qa. Its candidate list is empty,
pilot_started is false and no baseline.json exists. QA audit also currently rejects
legacy plan aspect token "sound" (schema expects "audio"); reconcile that as part of
QA resumption rather than changing acceptance records during beta preparation.
No QA/plan/acceptance files or gameplay source were changed here.

Beta validation passes:5modernization installer/gate/baseline tests,7QA-record tests,
full port preparation/archive fidelity/locked headless+desktop/browser/override/
accounting regressions, boundary checks, Python compilation, Rust formatting and
whitespace. Evidence: debug/artifacts/tooling-modernize-beta-001/result.json.
Neither these synthetic toolkit tests nor check() executed game QA or a pilot.

Modernization beta preparation committed separately as f32acbf, after QA commit
7443dc7. Pilot remains unstarted until completed full QA and explicit invocation.


## QA console handoff preparation (2026-10-24)

Prepared debug/qa-handoff.json for resuming pass1/K then completing full pass2.
Reconciled the legacy plan aspect spelling sound→audio (audit schema) and advanced
its revision/hash to v5, preserving all accepted pass/check result objects,
coverage, expectations, setup and IDs. Prior plan/QA/generator/review files and the
metadata-only rationale are retained in debug/history/qa/readiness-audio-schema-20261024/.
Updated the plan generator to retain the corrected spelling, and reconciled the
coverage-review hash. Administrative audit now correctly stops at incomplete QA,
rather than the aspect-schema error. No gameplay or reference source changed.

Removed only667MiB of disposable native Rust incremental cache with no DuckTales
build/QA process active. Binaries, snapshots, original/frozen source, saves and cited
evidence are intact. Modernization is explicitly excluded from the QA invocation,
including after successful completion. No game execution or QA pass was launched.

## 2026-10-24: lightweight QA navigation and waiting discipline

QA remains paused at pass1 L, with A–K retained and pass2 unstarted. No game
code, expectations, accepted records, source identity or binaries changed.
Toolkit QA goal template and debugging guide now distinguish cheap navigation
from detailed acceptance assertions and require waiting for bounded scripts
instead of repeated polling/speculative work. Added ROM-free DuckTales ordinary
waypoint helper to toolkit examples. Changes left uncommitted.

Local navigation.py and l_live.py provide 1–8 tick ordinary batches, endpoint
player/HP checks and deliberate bookmark world/progress checks; detailed actor
comparisons remain opt-in and boundary owner handlers remain for single steps.
Unknown mode/transition/death boundaries require restore and precise handlers.
Documented resume instructions in debug/tooling/navigation-handoff.md, linked
from QA handoff and goal. Corrected stale invocation position K to L.

Retained L006 route: 411 ticks, five input spans, 53 batches; 53.71 seconds
total (11.80 restore,41.89 execution), versus historical ~281 seconds.
Native171/reference352 requests versus1660/6639. Full native endpoint equal
and independent source endpoint checks passed. Evidence:
debug/artifacts/tooling-navigation-batch-001/result.json. Interactive batch8,
bookmark restore and precise single-step smoke passed, terminaltick704.
Seven QA record tests, four harness tests, scenario client/recipe tests, QA
variant checks, Python compilation, adapter boundaries and git diff check passed.
No full QA run or modernization started; one-route speedup is not a whole-suite
performance promise.

## 2026-10-24: user-authorized player-facing QA replacement

Replaced continuous differential QA with native gameplay checklist testing and
selected agent presentation judgment. Original emulator is an adjunct for a named
uncertainty/defect; a separate once-initial original-fidelity sanity suite is
explicitly configurable/disableable without disabling gameplay tests. Harness
requires opt-in for comparison kind and reports excluded checks, not fake passes.
Future feature changes retain independently authored gameplay regressions.

Updated toolkit goal template, policy installer, port workflow/common/development
and QA/porting guides, harness and tests. Added native_gameplay.py (no emulator
imports) and native-only ROM-free DuckTales example. Installed policy locally,
archived/revised all32 plan expectations to player-visible rules, reconciled hash
and preserved A–K pass objects/coverage exactly. Retained old diagnostic scripts
and manifests disabled by default. Initial sanity record points to already accepted
reviewed title/selection/entry/vertical-scroll presentation; no new run claimed.
Full L remains incomplete; no pass restart, game changes, commits or modernization.

Native L-choice-reject behavioral test and separate four-capture visual review
passed: readable paced choice pages and YES-A/NO-B, B rejection preserves money/
progress, normal HUD/gameplay restored. 78.73s total,62.54s native replay restore,
16.15s execution;36native requests,0reference. Initial001 failed an observer page
completion assumption;002 corrected it without game changes. New subcase result
is not full L acceptance. Native replay remains slow; no snapshot claim.

Validation:6harness tests,7record tests,scenario/recipe guards,explicit variant
regressions,5modernization gate tests,Python compilation,boundaries,diff check
and full test-port.py including locked headless/desktop/browser builds passed.
The first broad build check was interrupted (143); supervised rerun completed0.
Evidence: debug/artifacts/tooling-player-facing-policy-001/result.json.
Resume via debug/tooling/player-facing-handoff.md: finish only L then stop for
assessment; use native gameplay adapter, not l_live/paired owner synchronization.
All toolkit/local changes left uncommitted.

## 2026-10-24: actual fast native branch snapshots and disk cleanup

Added optional serve_with_snapshots/serve_io_with_snapshots to MIT runtime; legacy
serve stays compatible. Up to16 process-local named save/load/delete clones preserve
game plus driver clock/input edges/audio hash+capture/drop counters/frame/journal/
health. Reset clears bank; durable checkpoints remain guarded replay. Future port
scaffolds opt in with isolated Clone; other existing game workspaces untouched.
ScenarioSession detects advertised snapshots and falls back on older servers.
NativeGameplay bookmark/rewind and persistent native_author.py expose branch retry
without repeated process launches or large observations. Abandoned inputs/forward
bookmarks are discarded; no arbitrary state writes or original emulator coupling.

DuckTales Clone ownership added across canonical state/continuations. Mutable event
journal is FORKED, not shallow-shared, and command observers rebound to its fork;
immutable content stays Arc-shared. No gameplay rule/tick changes. Original binary
preserved under debug/history/fast-snapshots-before-20261024. Old checkpoints were
not edited/bypassed: re-executed natural4923-tick journal, verified FULL canonical
state/frame/audio equality, then exported new native choice-front identity.
Native adapter/handoff now select that regenerated setup. A–K evidence unaffected;
full L remains incomplete.

Measured7 restores at current L: mean0.01288s(range0.00779–0.02753);save0.09855s.
Mid-dialogue branch replay deterministic; full state/input/audio/frame/journal
restoration verified. One initial natural setup regeneration took131.34s while
compilation ran concurrently; not a snapshot restore cost. Actual persistent CLI
smoke also advanced/rewound/exported with no reference. Named bookmarks require
saving before an experiment; this is not an automatic every-frame timeline.

Cleanup losslessly compressed1359 uncited diagnostic payloads from947123425bytes
to63445338bytes, freeing883678087bytes (~843MiB). Every compressed stream decoded
and SHA-256 matched before removal; one restore round-trip verified too. Inventory
debug/history/diagnostic-archives.jsonl and archive_diagnostics.py restore paths.
Explicit accepted JSON/Markdown/script references preserved; all QA cited paths
remain accessible. Kept setup/checkpoint/preconditions/provenance/recipes, saves,
ROM/assets/reference and historical baselines. Removed108657297bytes regenerable
wasm target cache AFTER successful wasm check; published web-dist preserved.
Folder now roughly3.3GiB versus4.2GiB before, filesystem roughly3.2GiB free.

Checks:70shared runtime desktop tests,1421DuckTales tests, strict runtime/game
Clippy, wasm library check, full test-port.py locked headless/desktop/browser
build/regressions, fast/fallback scenario tests,6harness/7record tests, variants,
5modernization gate tests, format/compile/boundary/diff checks passed. No full QA
or modernization run and no commits. Completion evidence:
debug/artifacts/tooling-fast-snapshots-001/completion.json.
Resume instructions: debug/tooling/fast-snapshot-handoff.md. Keep native session
alive, import current setup ONCE, save a branch bookmark, try/rewind immediately.

## 20261024T170826Z: explicitly authorized fresh QA reset

Archived all prior active QA records/artifacts/cache/scenarios/tooling and prior
history at debug/history/superseded-qa-20261024T170826Z, with archive-index.json path mappings and
original control hashes. Bulk trees moved without duplication. Source, fixes,
contracts, implementation check records, ROM/assets, saves and published build
retained. Optional current-build native choice setup copied as input data only.
Fresh empty plan/results/issues/sanity/manifests installed. Initial sanity now
requires newly designed tests and fresh execution/review; no historical credit.
Root goal/progress and QA handoff reset to implementation complete, QA not started.
New plan is unconstrained by old32 IDs or L scope. Two gameplay passes remain.
No agents/tests/QA/modernization launched; changes uncommitted.

## 2026-10-25: modernization target selection prompt

Updated /root/romatlas/scripts/modernize-template/goal.md and its documentation
in /root/romatlas/docs/porting.md, plus this workspace's modernize/goal.md, at
user request. Prioritize conversion to native game state, rules and actions;
helper extraction/deduplication alone no longer qualifies. Require before/after
models, removed machinery, retained boundary encodings and concrete investigation
of deeper candidates. Preserve observable fidelity, bounded scope and existing
baseline/iteration records. Workspace-specific baseline instructions preserved.
Checks: python3 scripts/test-port-modernize.py (6 tests passed) and toolkit
git diff --check passed. Documentation/prompt edits only; no game execution,
modernization pass or commits performed.

## 2026-10-25: browser controllers without standard mappings

Updated ROMAtlas's shared `scripts/port-web/shell.js` and added
`browser-gamepads.mjs`. All connected Gamepad API devices remain selectable;
standard button names and first-stick defaults are retained, additional controls
use buttonN/axisN, and analog trigger aliases apply only to standard mappings.
Settings capture polls all connected controllers regardless of saved/draft player
assignments and ignores initially held controls until release. Keyboard input and
independent player routing remain intact. `scripts/port_web.py` includes the new
module in generated bundles and the development server's explicit route list.

Added `scripts/test-port-gamepads.mjs` and integrated it into the browser suite;
extended the Chromium synthetic-game test to bind/use an unmapped controller's
extra button and axis, with all players initially unassigned and an idle -1 axis.
Updated docs/porting.md. Pre-edit files are retained in
review/toolkit-gamepads-before/. Shared changes remain uncommitted; no runtime
Rust or gameplay rules changed. DuckTales's Pages builder now accepts both the
older pinned toolkit shell and this updated shell, retaining its local adapter
for reproducible builds against the pinned remote revision.

Checks: Node controller/actual-shell regressions pass; Python compilation,
JavaScript syntax and both repository whitespace checks pass. DuckTales static
Wasm build and local controller regression pass with the updated toolkit.

Full `tools/web-tests/bin/python scripts/test-port-web.py` passes in headless
Chromium: delivered scaffold/Wasm builds, module serving, keyboard/settings,
standard multiplayer input, raw button/axis capture and gameplay, audio startup,
battery persistence/quota handling, and future-workspace server invocation.
The first attempt timed out in the existing immediate-post-resume keyboard check;
the rerun passes after allowing an animation frame interval before the test key
press. No desktop window was launched. Node-only tests are part of this suite.

## 2026-10-25: browser page defaults and Reset

Updated shared scripts/port-web/index.html and shell.js: center the game display
with responsive sizing, show the requested native-Rust/WebAssembly explanation
under the title, add Reset immediately after Pause, and continue gameplay across
browser focus/visibility changes. Those boundaries still release held inputs,
flush battery saves, discard queued audio and reset timing. Pause and Settings
retain their explicit pause behavior. Reset flushes saves and reloads the page,
returning to the initial Start prompt; settings and original battery saves remain.
Updated docs/porting.md and scripts/port-template/BUILDING.md accordingly.
Pre-edit files retained in review/toolkit-browser-page-before/; prior controller
edits preserved. All toolkit edits remain uncommitted.

DuckTales now versions web/index.html and web/shell.js and copies them into the
static bundle, so the CI-pinned older toolkit also produces the updated page
without source rewriting. Its controller adapter remains the same. No Rust,
gameplay or accepted QA records changed in this task.

Checks passed: full toolkit test-port-web.py in headless Chromium (delivered
Wasm/scaffold, centered display, exact paragraph/button order, continued updates
with document focus/visibility false, Reset/restart, existing keyboard/controller/
settings/audio/save/server regressions). DuckTales static build, local Node
controller tests, and test-browser-page.py pass; the latter also checks narrow
viewport centering and manual Pause. JavaScript syntax, Python compilation and
both repository whitespace checks pass. No desktop graphics, commit or deployment.

## 2026-10-26: persistent snapshot implementation — integration pending

DuckTales's serializer now covers mutable canonical game owners, suspended tasks,
prepared video and the audio driver/synthesizer. Immutable resources are reattached
by the active authored-level identity. Diagnostic journals are reset and command
observers rebound. Browser storage version2 contains state plus a checksum, not
an input journal. Old replay slots are rejected and remain untouched until Save
State explicitly overwrites them. No duration/call-count limit remains.

The proposed opt-in shared-runtime support is retained as
compat/romatlas-snapshots.patch (161 lines): serialization of private APU,
clock/input state and driver capture/restore accessors. It has NOT been applied
to /root/romatlas; the user's earlier DuckTales-only scope prompted a pending
permission question. All shared work/tests used an isolated source copy at
/tmp/romatlas-snapshot-runtime and port copy at /tmp/ducktales-snapshot-test.
Normal checkout integration still requires that permission and enabling the
runtime snapshots feature, plus applying the patch in the pinned CI checkout.
No commit/publication or existing toolkit changes were made in this task.

Validation in the isolated build: 1468 locked Rust1.90 release library tests pass,
including persistent game-state/frame/bit-exact-audio continuation checks and
browser fractional-frame/sample-clock/input restoration. Real Wasm snapshot:
56717 bytes,113.5ms restore on this server;100 subsequent updates match frames,
audio and serialized state exactly. Browser slot persistence/overwrite/corruption/
quota and layout/shortcut/fullscreen regressions pass. Node tests verify zero
replayed updates and no journal growth beyond300000calls. Initial full-suite
attempt missed the isolated examples path; linking the unchanged local examples
fixed that fixture setup and the suite passes. Checks establish implementation
behavior, not a new whole-game fidelity pass.

## 2026-10-26: persistent snapshots — authorized integration complete

After explicit user approval, applied the runtime snapshot support to
/root/romatlas. The opt-in `snapshots` feature also enables JSON float-round-trip
support; other ports' default features and browser UI remain unchanged. Driver
restore validates ROM/sample-rate identity and compatible clock configuration
before mutation. Added shared regression tests for driver restore atomicity and
bit-exact APU sample continuation. Pre-edit runtime files are preserved in
review/toolkit-snapshots-before/; unrelated toolkit and game changes remain intact.
All shared changes remain uncommitted.

DuckTales enables the feature in port/Cargo.toml. Its Pages workflow applies the
formatted runtime patch (including the shared tests) to the pinned ROMAtlas
checkout, installs Node22 for verification, and checks actual Wasm snapshot
continuation before upload. Applying the patch to an isolated copy of the pinned
revision reproduces the local runtime sources byte-for-byte. Updated porting/build
documentation; the earlier pending-integration record above remains historical.

Normal-checkout checks pass:1468 locked Rust1.90 release library tests;72 shared
runtime tests with desktop+snapshots and70 with desktop alone; strict desktop
all-target Clippy for both the port and runtime; strict Wasm library Clippy;
formatting, Python compilation and whitespace. Desktop code was compiled only,
with no window opened. Full toolkit test-port-web.py passes in headless Chromium,
confirming future-workspace scaffold/Wasm, controls, settings, audio, original
battery saves, server and page behavior remain working without snapshot opt-in.

The actual `python3 run.py web --host 127.0.0.1 --port 0` server passes browser
save/restore persistence, overwrite, corruption/incompatibility and quota checks,
including the no-Web-Crypto fallback. Normal Wasm snapshots are56717bytes;
measured direct restore67.3ms, then30.2ms in a separate check, with100 subsequent
updates matching frame/audio/state exactly. Layout/shortcuts/fullscreen-resume,
Node snapshot (zero replay and no300000-call growth), SHA-256 and gamepad checks
pass. Static Pages build passes. The temporary test server was stopped; no user
server was stopped, and no commit, push or deployment was performed. These are
implementation checks, not a new whole-game fidelity acceptance pass.

## 2026-10-26: publish runtime snapshot support and remove CI patching

At the user's explicit request, committed and pushed only the snapshot runtime
files, focused regression tests and snapshot documentation to ROMAtlas main:
4f89589581fc7ea0173ce2c4d17fcbac1cd788f9
(Add opt-in serialization for persistent native snapshots). Staged only the
snapshot documentation section; earlier browser documentation/template/controller
changes remain uncommitted and preserved. No other existing changes were included.
This user authorization supersedes the earlier uncommitted-only snapshot handoff.

The DuckTales workflow now checks out that published revision directly. Removed
the CI patch-application step and patch-specific cache dependency. Updated build
documentation; retained the superseded patch in
review/toolkit-snapshots-before/superseded-ci-snapshot.patch for history. DuckTales
changes remain uncommitted/unpublished.

Verified origin/main matches the new commit, the published runtime source exactly
matches the already-tested local runtime, and workflow YAML selects it with no
patch step. Whitespace checks pass. Source behavior is unchanged from the prior
validated snapshot integration, so its Rust/Wasm/browser results still apply.

## 2026-10-26: native widescreen display aspect

Local, uncommitted ROMAtlas changes in `crates/romatlas-port/src/lib.rs`,
`src/desktop.rs` and `tests/runtime.rs` add optional
`Game::display_aspect_ratio()` and `viewport_with_aspect()`. Desktop presentation
can now fit a 720×240 game frame to 16:9 while retaining vertical integer scaling.
Ports that return None retain the existing viewport behavior. Tests cover full
16:9 fitting, integer height steps, small windows and existing default fitting.

Read the toolkit's AGENTS.md, project.md and development guide before editing.
The toolkit working tree was clean at the start of these edits. All modifications
remain uncommitted; no publication, push or deployment was performed. Transfer
this runtime source with the game for desktop builds. The game implements the
new trait method only with its desktop feature, so the published browser runtime
pin and existing browser workflow need no runtime patch or revision change.

Passed with Rust 1.90.0 and Cargo.lock: toolkit formatting, whitespace checks,
`cargo test --locked -p romatlas-port --features desktop,snapshots` (73 tests),
and clippy for romatlas-port/all targets/desktop/snapshots with warnings denied.
The game also passes desktop-feature clippy, compiling desktop code without
opening a window. These are implementation/runtime checks; prior QA records and
original implementation ledgers remain unchanged.

## 2026-10-26: desktop parity and future-port display defaults

Backported player-facing browser capabilities into the shared SDL frontend in
/root/romatlas for Linux/macOS/Windows: F1 resume, F2 pause, F3 reset, F4 save
state, F5 restore state, F6 settings (Guide still opens settings). Added optional
Game full-state/reset hooks, bounded checksummed native state files with
executable/ROM/override/audio compatibility checks, atomic writes, and preserved
sample-clock phase. Restore pauses and rearms live inputs/timing; it never
replays inputs. Battery persistence retains its separate format and hooks.
DuckTales implements the native hooks using its existing serializer and a cold
reset built from the same immutable content. Browser serializers remain compatible
with the published Pages runtime pin; no workflow pin or publication changed.

Desktop controller settings now expose Automatic/None/device choices per player,
reserve explicitly assigned devices before automatic routing, and preserve legacy
primary-controller preferences. Unmapped SDL joysticks are selectable and expose
buttonN/axisN bindings, first-stick aliases and D-pad hats. Capture polls assigned
and unassigned devices and waits for initially held controls to return to neutral.

The desktop menu and future web template have no integer scaling control or
behavior. A legacy serialized integer_scale flag is accepted for compatibility
but omitted from newly serialized runtime settings. Window/display height drives
width from the game aspect; desktop uses free scaling. New templates default to
config.rs DISPLAY_ASPECT=(4,3), while DuckTales retains its own 16:9 config and
widescreen rules. No widescreen gameplay was copied into the templates.

Future templates enable snapshot support, expose the same browser toolbar and
F1-F6 commands, and provide Game snapshot/reset hooks plus driver validation before
game restore. The scaffold's empty snapshot covers its stateless demo only; updated
workflow instructions require full versioned mutable game/audio serialization
when gameplay is implemented. Unsupported existing games retain default hooks.
Generic browser state identity includes Wasm, ROM, overrides and sample rate;
state files remain separate from battery saves and frontend-specific.

Updated template generation, module packaging/routes, building/porting docs and
focused regressions. Pre-edit desktop/settings sources are preserved in
review/desktop-backport-before/. All shared-library changes remain uncommitted;
no commit, push, publication, desktop window or other-platform execution occurred.
Implementation verification is separate from game-fidelity QA and ledger accounting.

Checks completed so far: 1485 existing DuckTales locked Rust 1.90 release library
tests (the final-source run including the new desktop regression is recorded
below); shared desktop/snapshot tests; generated-port preparation, archive preservation,
locked headless/desktop/browser builds, overrides and rejection regressions;
Chromium future-template 4:3 layout, absent width/integer controls, F1-F6 and real
snapshot save/restore plus input/audio/settings/battery regressions; shared Node
raw-controller and no-replay/checksum/SHA-256 fallback tests. An isolated source
copy compiles current DuckTales Wasm against published Pages pin
4f89589581fc7ea0173ce2c4d17fcbac1cd788f9. Final broader check results follow below.

Final verification: the final source passes 1486 locked Rust 1.90 library tests
(including game::tests::desktop_state_hooks_restore_real_game_audio_and_reset_to_startup)
using a temporary target with test debug info disabled to fit this server; its
79 shared runtime tests pass with desktop+snapshots, including game-restore failure
atomicity. DuckTales's locked Linux desktop release binary builds, advertises the
new shortcuts and executes five headless frames at 341x240. Strict all-target
desktop Clippy passes for the game and runtime; strict all-target workspace Clippy
and the toolkit locked release build pass. Toolkit portability passes. Formatting,
Python/JS syntax and whitespace checks pass in the affected repositories.

The generated browser test passes in Chromium, including real full-state restore,
4:3 dimensions, absent scaling/width controls, shortcuts, raw/standard controllers,
audio, settings and original battery/error handling. The added browser regression
initially used an input press shorter than a simulation tick; changing the fixture
to wait for Running and hold input until observed resolved it. No gameplay fix was
needed. Shared Node tests cover direct snapshots without replay after 300000 calls,
corruption/incompatibility rejection and SHA-256 fallback parity.

The broad cargo test --locked --workspace attempt was interrupted during compilation
when disk space approached exhaustion; it has no passing result. This is distinct
from the passing focused runtime and final-source game suites. Removed only compiler
incremental caches created by these broad checks; preserved sources, saves, existing
reference/QA records, deployment backups and superseded evidence. No macOS/Windows
build, SDL window or physical-controller execution was attempted on this Linux
headless server. No commit, push or deployment was performed.

## 2026-10-26: reduce local build disk usage

The broad checks above left about 2.4 GiB of toolkit debug build products and
547 MiB of game debug products. With no Cargo processes running, cleared only
dev-profile artifacts using cargo +1.90.0 clean --profile dev in both projects.
Release binaries, source, original/reference data, saves, QA records and logs
were preserved. Added repository-local .cargo/config.toml in both repositories:
dev/test debug symbols disabled and incremental caches disabled. Release profiles
are unchanged. These defaults trade debugging detail and incremental rebuild
speed for smaller build artifacts; they do not change game rules.

Checks: locked Rust 1.90 metadata accepts both configurations; the toolkit CLI
and game release executables remain present; whitespace checks pass. No new
compilation, commit or publication was performed for this configuration change.

## 2026-10-26: authorized commit and push

At the user's explicit request, committed and pushed the desktop/template parity
changes and low-disk Cargo configuration to ROMAtlas main as
96eb99851c72fd838802d76281e93f368833983d. This authorization supersedes the
uncommitted-only handoff above. DuckTales's corresponding source, documentation,
pre-edit backups and local build configuration are included in its accompanying
commit. The existing Pages runtime pin remains unchanged and was checked for
Wasm compatibility. No additional build or whole-game QA pass was run for this
commit-only step; the implementation verification results above still apply.

## 2026-10-27: restore incremental builds and prefer release mode

After the user expanded the disk, enabled incremental compilation for dev, test
and release profiles in both repository-local Cargo configurations, keeping debug
symbols disabled in all three. No panic, overflow or other safety settings changed.
Updated ROMAtlas port launchers (desktop/headless/debug) and check commands to
select release mode explicitly; Wasm builds already selected release mode. New
ports receive the same Cargo configuration from a dedicated template. Updated
build guidance and repository instructions to retain the user's preference.
These changes remain uncommitted; prior publication records remain historical.

Checks: Rust 1.90 locked metadata accepts both repository configurations; parsed
all three matching profile configurations; inspected generated launcher commands
and exercised check-command construction with a mocked subprocess, confirming
release mode for each build/test/Clippy command. Python compilation and whitespace
checks pass. No game rules changed, no compilation or desktop window was started,
and no cache deletion, commit or push was performed in this step.
