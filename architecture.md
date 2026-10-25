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

Title and land-selection screens apply their decoded `ActorPlacement` records
through one private menu initializer. Each screen supplies its rendering flags;
descending slot order and partial field updates remain source-defined. See
[selection/trace.md](port/src/selection/trace.md) and
[modernize/changes.json](modernize/changes.json) for the contract and evidence.

Runtime, assets, presentation, world, collision, player, actor lifecycle/families,
selection, progression, status, dialogue, scenes, finale and audio retain adjacent
source traces/contracts. Actor dispatch visits live slots ascending; allocation
searches the source pool descending. Immediate collision/recovery/terrain/sound
callbacks commit before later live-slot work. Sound author hooks report attempted
submission even on drop, preventing end-of-family replay or reordered RNG facts.
SourceAE extension and FB late-mode are distinct fields; spawn-window direction,
scroll direction and previous scroll retain separate actual source ownership.

Perimeter and wave movement programs decode into immutable native records at
asset loading. Named headings/contact gates replace packed path/mask arrays;
Spawn-at-tick/AwaitLastChild replace parallel wave position/time arrays. Actors
retain their existing cursor/timing ownership and shared motion projections, so
callback order, mirrors, byte waits and equal-time emissions remain unchanged.
See [bosses/trace.md](port/src/actors/bosses/trace.md) for iteration013.

ActorContact owns one ActorControl payload. The ambush enemy retains explicit
WaitingForPlayer/Opening/Tracking/Closing/Attacking phases with named delay and
preset cursor. Other families retain a compatibility payload while their shared
callbacks and debug views use lossless control projections. Handler changes select
the payload without losing partially preserved fields. The old state/local byte
fields are removed; no shadow native/source pair exists. See
[enemies/trace.md](port/src/actors/enemies/trace.md) for iteration012.

Scene actor control uses native Bounce/WalkIn/Delay/MoveToMark/Settle/Settled
variants with associated counters and progress. Scene context is selected by the
entered owner, since small handler codes overlap ordinary enemies. The canonical
ActorControl switches context through lossless shared-field projections; no side
scene-state copy exists. Source waits, motion order, exact move-entry reset and
rebuild/composition/palette continuations remain unchanged. See
[scenes/trace.md](port/src/scenes/trace.md) for iteration014.

PlayerCall and DialogueCall carry complete invocation outcomes across text,
conversion and vertical waits. First-slot RNG uses the actual final named source
operation: source controller filter return, comparisons, snap/integrator/gravity,
payment scan or actual queue Submission. No global/default carry is invented.
Accepted vertical transition discards the ordinary player return and enters DBE4
AfterSpawn, bypassing actors/countdown while retaining its real tail effects.

## Display and audio aliases

Status number producers share one private `number_payload` encoder for the marker,
raw display codes and trailing zero. Stream rows, messages and direct uploads
retain their own framing, normalization and publication order. The bounded
modernization evidence is indexed in [modernize/changes.json](modernize/changes.json);
the original source contract remains in [status/trace.md](port/src/status/trace.md).

Reward, total and transfer-source money use native quantities with explicit
blank-layout metadata. Gameplay names credit denominations and semantic payment,
theft, transfer, departure and ending operations. Original display-code imports
and exports remain at template/render/debug boundaries; nondecimal source input
uses an isolated compatibility representation. Paid-return provenance, trailing
codes, staged transfer waits and canonical immediate ownership are preserved.
See [status/trace.md](port/src/status/trace.md) for iteration011 and source rules.

Stage countdown owns native remaining seconds or explicit expiration. Numeric
ticking and ten-second reward conversion operate on that quantity; original
codes are imported at initialization and exported only at render/debug boundaries.
Game-over and nondecimal raw-selector inputs use an isolated compatibility state.
The divider, warning sound, player-death commit and audio/finish waits retain their
existing owners. See [status/trace.md](port/src/status/trace.md) for source rules
and the pre-existing unusual-glyph subtraction limitation.

Health stores native segment availability and Filled/Depleted state separately
from appearance. Damage, recovery, paid restoration and player death use native
health queries. Template import and immutable render/debug projections preserve
original geometry, nonzero attribute bytes and the distinct FF rendering marker.
Full restore still clears hidden segments; extension updates native availability
and its corresponding presentation in one owner operation. See
[status/trace.md](port/src/status/trace.md) for the contract and iteration010 evidence.

Video owns logical CHR, nametables, palette, committed OAM/raster and prepared
uploads. Render is read-only. Primary owns64 prefix bytes; writes past0440 borrow
secondary's192-byte prefix, then the one128-byte world attribute cache. Secondary
composites and NMI/audio read those same owners. Byte-Y queue progression is
separate from absolute field offsets; attribute cache stores and final-only FF
terminators preserve source ordering under aliases. Dialogue temporarily moves
one canonical panel, restoring it before every serviceable owner/host boundary.

Display queues, scripts and fills share private ordered-write operations for
address masking and stride advancement. Their parsers, publication boundaries
and queue cursor arithmetic remain separate; see [presentation/trace.md](port/src/presentation/trace.md).

Complete stage/title/special-scene palettes use `Display::load_palette` to update
both shadows. The typed background/sprite pair does not publish colors; partial
palette effects, title edits and fades remain at their existing source boundaries.

Terrain expansion updates the canonical16-byte staging owner; ordinary bottom
rows retain the eight unwritten bytes, externally visible to audio. Source audio
channels/preset/gaps/staging/primary/secondary supply low-F reset reads through
borrowed projections; there is no second memory map. Selector provenance starts
ResetZero and inherits actual audio producers across frames. CommandQueue retains
its eight sentinel slots and exact producer comparison/drop/drain behavior.
Audio callbacks alone advance synthesis sample clocks; source frame interpretation
prepares register writes, and Game duplicates mono output to stereo once.

## Immutable resources and extensions

`Room::block_at` decodes a room descriptor once for streaming and collision
consumers. Callers retain the distinction between transposed ring cursors,
row-major probes and source raw-room selection; see
[world/trace.md](port/src/world/trace.md) and [modernize/changes.json](modernize/changes.json).

Content loads validated original catalogs once from the matching ROM with named
local recipe replacements. Full transient original image is used only at startup
for instruction-as-data consumers, then dropped. Rules use decoded immutable
catalogs, source selector domains and native typed data. Unnamed data is retained
through established consumers, not assumed junk from appearance.

Background spans compile once into immutable ordered tile updates at asset loading.
Tasks owns an explicit Updating/Complete session and executes one update per poll;
source record/chunk counters and destination advancement no longer drive gameplay.
The final palette poll, live suspension policy, bank selection and Video queue alias
writes retain their original order. Debug record/chunk coordinates are projections
from plan metadata. See [assets/trace.md](port/src/assets/trace.md) and
[modernize/changes.json](modernize/changes.json) for the contracts and evidence.

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

Modernization015 moves animation state into `presentation::AnimationPlayback`,
owned canonically by ActorContact and copied into the transient ActorVisual for
composition. Running/frozen clocks and explicit frame loop origins replace packed
byte owners. Asset identities and lossless source observation projections remain
boundary concerns. See `port/src/presentation/trace.md` for overflow and ordering
contracts.

Modernization016 decodes original dialogue bytes into immutable MessageProgram
text actions at asset loading. Runtime executes Glyph/Blank/NextRow/Finish,
retaining the single shared progression cursor and B03 panel owner. Control-byte
meaning disappears from gameplay; tile IDs and encoded shared display submissions
remain at presentation boundaries. See `port/src/dialogue/trace.md` and
`modernize/dialogue-probe.py` for the source rules and bounded comparison recipe.

Modernization017 gives World native tile regions, palette runs and clipped block
columns. Game supplies these changes to Video without constructing upload headers
or byte offsets. Video/presentation retains exact source framing only at the
canonical prepared queue boundary, where cache/audio/runtime-composite aliases
make those bytes observable before commit. There is no second semantic pending
queue. See `port/src/world/trace.md`, `port/src/presentation/terrain.rs` and
`port/src/game/video/terrain.rs` for geometry and source-boundary responsibilities.

Modernization018 compiles primary and secondary audio sequences into immutable
native actions in Catalog. Playback handles typed notes, controls, branches and
modulation; source opcode/operand classification occurs at asset import. Private
mixed source data remains for genuine header/preset/period reads and overlapping
resource roles. The source boundary preserves observable resource-cursor/read
provenance, including primary-only seed selection and skipped branch operands.
File-authored overrides rebuild both programs before playback. Shared Driver,
APU, channel aliases and sample clock ownership remain unchanged; see
`port/src/audio/trace.md` for the source contract and bounded PCM comparison.
