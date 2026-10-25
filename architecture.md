# Native port architecture

The shared Game now runs the full native source systems. All18 owners and their
cross-owner continuations are implemented; strict native/Wasm gates and1411 tests
passed. Completed source accounting and the required audit are tracked in progress.md. Independent runtime fidelity QA remains the
separate stage5. Historical design/implementation checkpoints are preserved in
checks/implementation/history/architecture-before-final-stage4.md.

## Single canonical runtime

Services also owns the semantic CollisionOutcome returned across actual player
physics and ascending actor callbacks. Real probes replace it; a direct movement
that removes an actor without probing preserves the preceding outcome. Player
vertical aggregate callbacks retain recursive tail-call ordering. The read-only
summary exposes collision_return.bits, and replay reconstructs this owner.

Port owns validated raw Assets and one Machine. Machine owns Core (actors, player,
progression, status, camera/world caches, spawn/suppression controls), Services
(runtime scheduler/controllers/RNG/bank, Video, command ring/driver/synthesizer,
dialogue and immutable Content), plus entered Menu/Foreground/PlayerCall/Tasks
continuations. Frontends share Game controls, tick, render, audio and no-battery
save defaults. Mode is derived; no duplicated retained scene state drives rules.

One host tick runs one interrupt before foreground resume. Internal owner/player
requests release borrows and complete in the same host turn until a real source
wait. Actual waits yield task0; sprite-zero/menu busy loops run interrupts/audio
without sampling or scheduler resume. NextFrame loops immediately within the
resumed host turn. Reset title entry reaches its first source wait before tick1.
Game identity,256×240 dimensions, rational NES NTSC cadence, palette and settings
defaults live in config.rs. No CPU, processor flags, stack or RAM emulator exists.

## Source owners and immediate effects

Runtime, assets, presentation, world, collision, player, actor lifecycle/families,
selection, progression, status, dialogue, scenes, finale and audio retain adjacent
source traces/contracts. Actor dispatch visits live slots ascending; allocation
searches the source pool descending. Immediate collision/recovery/terrain/sound
callbacks commit before later live-slot work. Sound author hooks report attempted
submission even on drop, preventing end-of-family replay or reordered RNG facts.
SourceAE extension and FB late-mode are distinct fields; spawn-window direction,
scroll direction and previous scroll retain separate actual source ownership.

PlayerCall and DialogueCall carry complete invocation outcomes across text,
conversion and vertical waits. First-slot RNG uses the actual final named source
operation: source controller filter return, comparisons, snap/integrator/gravity,
payment scan or actual queue Submission. No global/default carry is invented.
Accepted vertical transition discards the ordinary player return and enters DBE4
AfterSpawn, bypassing actors/countdown while retaining its real tail effects.

## Display and audio aliases

Video owns logical CHR, nametables, palette, committed OAM/raster and prepared
uploads. Render is read-only. Primary owns64 prefix bytes; writes past0440 borrow
secondary's192-byte prefix, then the one128-byte world attribute cache. Secondary
composites and NMI/audio read those same owners. Byte-Y queue progression is
separate from absolute field offsets; attribute cache stores and final-only FF
terminators preserve source ordering under aliases. Dialogue temporarily moves
one canonical panel, restoring it before every serviceable owner/host boundary.

Terrain expansion updates the canonical16-byte staging owner; ordinary bottom
rows retain the eight unwritten bytes, externally visible to audio. Source audio
channels/preset/gaps/staging/primary/secondary supply low-F reset reads through
borrowed projections; there is no second memory map. Selector provenance starts
ResetZero and inherits actual audio producers across frames. CommandQueue retains
its eight sentinel slots and exact producer comparison/drop/drain behavior.
Audio callbacks alone advance synthesis sample clocks; source frame interpretation
prepares register writes, and Game duplicates mono output to stereo once.

## Immutable resources and extensions

Content loads validated original catalogs once from the matching ROM with named
local recipe replacements. Full transient original image is used only at startup
for instruction-as-data consumers, then dropped. Rules use decoded immutable
catalogs, source selector domains and native typed data. Unnamed data is retained
through established consumers, not assumed junk from appearance.

Authored registry levels have distinct named identity, predecessor and original
rule profile. Graph validation rejects forks/cycles/missing parents at startup.
Machine retains immutable original base Content; each authored view is constructed
from that base, sharing registry metadata and patching actual room/terrain/spawn/
CHR/audio payload consumers. Runtime state remains one owner. usize room payload
identities use a native presence projection at source byte-FF tests. Entry/main-map
reload uses authored checkpoint, while special layouts remain original. Source
completion work finishes before next view; global progress effects remain, and
per-entry scene/completion controls clear. Leaving a view stops its encoded audio
program before base restoration. EXTENSIONS.md specifies all payload formats and
original-only manifests; original content is never embedded/distributed.

## Observation, replay and validation

Schema2 observes canonical owners/calls/named level identities. One bounded512-event
nondraining journal records actual sound attempts/dequeues, per-slot changes,
owner boundaries, modes and source resets. Observer snapshots are transient and do
not drive simulation; source queue equality excludes only the journal handle.
Replay/checkpoint transport validates ROM/build/override identity transactionally.
Read/render calls do not mutate ticks, input, queues or synthesis clocks.

The bounded maintainability pass removed unused adapters, clarified source alias/
return/call ordering, replaced stale module headers, formatted code and mechanically
rewrote fixture loops/initializers without changing assertions. Explicit wide
borrow-boundary signatures carry local clippy annotations explaining canonical
ownership; no crate-wide warning suppression exists. differences.md and all8
uncertainty assessments distinguish implemented source-informed assumptions from
confirmed behavior. Stage5 performs the two independent fresh whole-game passes.
