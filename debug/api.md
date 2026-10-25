# Game observation schema 2

`port.state.result.game` is read-only game-owned JSON. Require
`schema_version=2`, `implementation=native_integration`, and the intended semantic
preconditions before interpreting a checkpoint. This identifies native execution,
not independent fidelity certification. Historical stage3 reservations are retained
in [history/schema1-stage3.md](history/schema1-stage3.md).

| Field | Canonical owner and units |
|---|---|
| ticks | Shared Machine host ticks since debug reset; source cold resets preserve this monotonic count |
| mode | title, land_selection, confirmation, gameplay, dialogue, status_panel, transition, ending or stalled |
| assets | Validated source/level counts; active_level is the named identity, independent of source stage/rule profile; pending_successor is optional |
| input | Two source lanes, held/pressed byte masks after the most recent actual task0 sample |
| scheduler | Interrupt count, task states, foreground bank, four RNG words, pre-filter raw opposite-edge return fact |
| progression | Source stage, row, completion/progress masks, AE extension and FB late-mode separately, dialogue/ending/race controls |
| player | All17 action names/selectors, queued action, protection, special duration, gravity, momentum, pose and contact latch |
| actors | Stable slots0..15, active/render/profile, source handler/family, optional spawn identity, integer/fractional positions, signed8.8 vertical velocity, unsigned8.8 horizontal magnitude, animation clocks and local fields |
| world | Camera world pixels, room/stream cursors, collision/attribute cells, suppression and typed special spawn control |
| status | Literal six/seven display codes, lives, health segments/alternation, three countdown codes/divider and dirty/UI flags; BLANK24 is distinct from zero |
| calls | Actual entered wait owner/remaining ticks, fades, transfer working source, conversion phase, pending status rows, lifecycle, horizontal/vertical cursors, background upload, ending phase/pending work and busy-loop flag |
| transition | Foreground/menu continuation, awaited owner, first-actor increment fact and split controls |
| dialogue | Canonical text/choice continuation, actor/message, waits and panel row/submission state |
| presentation | Prepared primary/secondary/palette/OAM state and committed raster/OAM state; these intentionally differ until NMI |
| audio | Actual command ring/producers/consumer/stage command, driver channels/cursors/envelopes/modulation, reset selector provenance and synthesis clocks |
| events | Bounded nondraining semantic journal described below |

Positions and raw codes preserve source encodings. Handler/local-state values are
not invented generalized states. Optional absent continuations use null, rather
than fabricated elapsed counters. Observation/rendering never samples input,
advances waits, drains commands, synthesizes audio or mutates game rules.

Events have monotonic sequence, tick, kind and payload. Capacity is512; entries
retain the newest events, next_sequence never rewinds during source reset, and
dropped counts evictions. Reading does not consume events. Consumers retain a
sequence cursor and reject a window containing lost required events. Shorten
advance windows after overflow; absence from a truncated window proves nothing.

Actual queue authors emit AudioCommand for accepted AND dropped attempts, with
command, stage flag, prior slot, comparison and producer movement. AudioDequeued
records the actual pre-advance consumer. Other bounded events are SourceReset,
ModeChanged, ActorSpawned/Removed/Replaced/Changed, ActorHandlerBoundary,
PlayerChanged, StatusChanged, ProgressionChanged, CameraChanged, TerrainChanged,
OwnerBoundary, PlayerReturn, PlayerNonlocalReturn and LevelActivated. Per-slot
changes are recorded before the next live ascending slot; broad actor dispatch
is not recorded again as a duplicate spawn/removal. Before/after snapshots are
transient observers and never drive rules.

Use the transport methods/limits in [../DEBUGGING.md](../DEBUGGING.md).
`port.reset` reconstructs ROM/overrides and starts a new tick/event history.
Checkpoints replay reset plus input/tick/audio requests, validate ROM/build/asset
identities and hashes, and import transactionally. They are not arbitrary memory
snapshots. Rebuild checkpoints from a saved input recipe after code/content changes.
There are no state writes, warps, diagnostic controls or battery files.

QA entry/setup recipes are in [entry-recipes.md](entry-recipes.md). Stage4 focused
owner fixtures are implementation evidence; the two required complete QA passes
remain separate in qa.json.

## Fast branch snapshots

api.describe now advertises port.snapshot.save/load/delete with {name: STRING}.
They clone/restore the complete native driver/game (including input edges, rational
audio clock, journal, events, rendering and audio capture) in memory. Max16 named
slots, reset clears them, no process/build transfer or arbitrary state edits. Save
before a trial and load to retry; no tick replay on load. Port uses deep clone with
a forked event journal and rebound command callback. Disk checkpoint import/export
remain guarded replay for durable setup, not fast rewind.
