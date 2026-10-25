# Optional modernization beta

Preparation installs this workflow only. It neither starts an agent nor authorizes
a pilot, candidate investigation or gameplay/code changes. This is not a sixth
stage of the fixed port pipeline and is not required for port completion.
Start it only after a separate user request to modernize this port. Full-game QA
and the former two-pass QA sweep are not prerequisites for this workspace.
Do not run full-game QA automatically before starting modernization.

## Entry and faithful baseline

Use the workspace (parent of modernize/) as the working directory.
Read workspace.json for the toolkit path, DEBUGGING.md, architecture.md,
behaviors.md, differences.md, uncertainties.json and the source/test contracts.
Paths in this paragraph are workspace-relative; this directory is modernize/.
Apply prompts/coding-standards.md (or established game standards), repository and
platform boundaries. Original ROM/source/assets and captures remain local.

Review available smoke/comparison evidence and its source/build/ROM/assets
provenance. Record missing coverage honestly; retained results do not certify the
current implementation. Do not fabricate completed QA records to proceed.
Run applicable implementation checks/audit only after modernization is invoked.
Record their actual commands/results and any unavailable platform/device checks.

From the workspace run `python3 modernize/baseline.py` to freeze native source/build
configuration hashes and available smoke/QA record hashes. Use this workspace-local
helper instead of the toolkit's port_modernize.py check/baseline commands, whose
legacy full-game QA gate does not apply here. Keep the
faithful implementation reproducible in local version control or an explicit local
source snapshot, with ROM/asset identities and pinned dependencies. The script
records hashes only; it does not make a source backup or certify semantic truth.
Preserve this baseline and original-fidelity expectations. Do not overwrite it
with the modernized implementation. After capture, set status.json to
baseline_recorded with its baseline path and keep pilot_started false until the
explicitly authorized candidate begins. Existing QA records continue to describe the
faithful baseline; they do not automatically certify a changed build.

## Select one bounded improvement

Populate candidates.json only when this workflow is explicitly invoked after the
baseline capture. For each candidate record its ID, files/subsystem, current machinery, proposed
game-level abstraction, concrete clarity/flexibility benefit, source references,
observable rules to preserve, callers/state owners/update ordering, test IDs and
coverage gaps, coupling/risks and why the scope is bounded. Inspect the code before
ranking candidates; no subsystem is presumed safe from its name alone.

The objective is idiomatic native game code: express game rules, state and actions
directly instead of preserving the original machine's representation and execution
machinery. Fidelity constrains observable behavior; it does not require retaining
register-like fields, packed display values, upload commands, source table encodings
or continuation machinery internally. Preserve any of these only where their
observable consequences or an external boundary require them.

Prioritize bounded changes to representation, state ownership or control flow that
remove hardware-shaped machinery from gameplay. Examples to investigate include:

- Represent background-region changes directly, replacing upload-buffer construction
  and chunk bookkeeping where they have no observable effect.
- Store health, money and other game quantities in meaningful native types, with
  packed digits, tile codes and other display encoding at the presentation boundary.
- Describe scenes and actor behavior with native structures and explicit actions;
  decode ROM table formats at the asset boundary rather than throughout gameplay.
- Replace source-shaped continuations and implicit phase flags with explicit game
  states and transitions while retaining observable waits and update order.

These are investigation directions, not prescribed rewrites. Prefer the smallest
coherent subsystem that can complete such a conversion, including its affected
callers and state owner. A bounded candidate may cross files; it need not be a tiny
patch. Do not invent a generic engine, add abstraction layers without removing old
machinery, or introduce an emulated RAM/CPU layer.

Helper extraction, deduplication, renaming, moving code and cosmetic Rust idioms are
supporting work, not sufficient modernization targets on their own. A shared helper
that leaves the same hardware-shaped model in every caller does not meet this goal.
Do not rank candidates mainly by ease of proving equivalence or existing test
coverage. Investigate promising deeper conversions and fill their contract/coverage
gaps within a bounded effort. Coupling in collision, movement, scheduling or sound
calls for investigation; defer a candidate when a specific unresolved dependency
prevents a sound conversion, recording that dependency and the next useful probe.
Do not repeatedly substitute easy helper extractions for that investigation.

For each ranked candidate, explain the before/after game model, which low-level
state or machinery disappears, what remains at asset/runtime boundaries and why,
and how gameplay code becomes easier to understand or change. Identify the strongest
plausible conversion considered and give concrete reasons if selecting another.
If no substantive candidate is currently supportable, report the investigation and
blockers as incomplete rather than counting routine cleanup as modernization.

A direct drawing call can replace upload machinery only when layering, coordinates,
intentional flashing/palette/raster effects and relevant timing still agree.
Preserve wrapping/signed arithmetic, ordering, shared-buffer effects, intentional
waits, audio interactions and historical quirks where observable behavior depends
on them. Establish which details matter from evidence rather than assuming every
hardware operation is either essential or disposable.

Choose one candidate within the user's scope. Establish targeted original-fidelity
integration tests and useful unit regressions first; independently verify any
missing expectations against source and the faithful reference. Scenario discovery
uses persistent branching, recorded recipes and validated setup caches. Missing
observations/setup are tooling gaps, not permission to guess game rules.
Record the starting source/build identity and a reversible checkpoint for edits.
Do not mix unrelated candidates or intentional gameplay enhancements into the patch.

## Implement, compare and retain or revert

Replace one bounded piece with the proposed abstraction. Keep public semantic debug
observations stable where practical so tests survive internal layout changes.
Update tests that assert obsolete implementation detail only with source-backed
reasons; preserve their behavioral coverage and original expectations. Never change
an expected value merely to make the new code pass. Intentional divergence requires
a separate explicit user request and documented variant contract, outside this
fidelity-preserving beta iteration; retain unaffected regressions in either case.

Run targeted fresh integration scenarios against the new build, compare relevant
original-game and faithful-baseline observations, and run affected unit/build/static
checks. Regenerate stale checkpoints from retained recipes; never bypass identity
guards. Select broader regressions from actual cross-system dependencies. Changes
to shared timing/state ownership or uncertain impact require the full relevant
integration suite; do not run two whole QA passes automatically after each edit.
A missing durable test remains a coverage gap, not evidence of equivalence.
Inspect relevant graphics/audio as well as structured assertions where affected.

If behavior differs, investigate source, test and implementation rather than
assuming any one is correct. Retain only a demonstrated improvement with passing
required checks. Otherwise revise or revert only the candidate's own edits,
preserving unrelated work, tests/evidence and the explanation. Stop incomplete
when no sound equivalence claim is available; do not broaden the rewrite to escape
a difficult result. Existing source uncertainty remains explicit.

Update changes.json and status.json with baseline/candidate/source/build identities,
actual edits, before/after game model, removed machinery and remaining boundary
encodings, test IDs/commands/results/evidence,
source-grounded contract corrections, remaining uncertainty and retained/reverted
outcome. Update architecture/behavior links and user documentation where affected.
Describe routine refactoring honestly as supporting cleanup; passing regressions
alone does not demonstrate modernization. Keep scope distinct from runtime
performance optimization or feature development.

Default to one candidate, then report and stop; further iterations need requested
scope. Do not mark the whole game modernized or invent a percentage-complete goal.
No automatic commits, publication, deployments or migration of other games are
part of this workflow. A user-requested modernization commit must remain separate
from QA tooling and faithful-port fixes.
