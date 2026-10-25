# Local replacements and authored levels

Supply `--assets DIR` to desktop/headless/debug, or select the matching asset
collection in the browser. Both paths deliver the same MemoryAssets to Game.
No rule changes or content-selection menu are needed.

Start with examples/original-only: its source-assets.json explicitly loads
levels.json as an entry (`extension/levels`), whose empty resources/levels arrays
retain every original level. levels.json is game metadata, not a second toolkit
collection. Every payload file must also be an entry in source-assets.json; file
keys are paths relative to that manifest. Missing files, duplicate identities,
wrong resource kinds, malformed payloads and invalid predecessor graphs fail
startup transactionally. Names use ASCII letters/digits and `-_./`, at most128
characters. Original selector identities cannot be replaced by added definitions.

A levels.json version1 contains `resources:[{id,kind}]` and `levels` entries:

```json
{"id":"my/level","rooms":"my/rooms","terrain":"my/terrain",
 "spawns":"my/spawns","graphics":["my/graphics"],"audio":["my/audio"]}
```

Rooms and terrain are required. Spawns may be null/absent to retain the selected
source-profile spawns; graphics/audio arrays may be empty. One stage song is
supported per level. Resource kinds are rooms, terrain, spawns, graphics, audio.
Each payload is JSON with integer version1 and the matching format below.

| Format | Payload fields and rules |
|---|---|
| ducktales.rooms | after: predecessor named level; rule_stage: original shared rule profile0..4; rows/columns; cells: room-payload indices or null; rooms: low[64] byte descriptors plus ninth_bits[8]. Multirow maps use8 columns; a single row may use1..8. Grid coordinates fit one source byte (at most256 cells), independent of registry level count. entry_room defaults0, player_x/player_y default128, countdown defaults[5,0] decimal digits. Entry must be present. |
| ducktales.terrain | palette[16] NES colors0..63; tiles:[{id,corners[4],palette:0..3,collision:0..3}]; blocks:[{id:0..511,quadrants[4]}]. Unspecified descriptors are zero. The same source terrain expansion/collision/reaction rules consume these values. |
| ducktales.spawns | templates: either {original_template:0..120}, or {handler,animation,flags_and_local?,profile?}; rows: ordered [{x,y,template}] per room row. Handler/animation and masked profile must fit shared catalogs. Template refers to this payload's template list. Source template references decode from the user's ROM, never from distributed extracted bytes. Encoded records plus row terminators fit256 identities; worldX is below32768. |
| ducktales.graphics | copies:[{destination,bytes}]; tile-aligned, nonempty16-byte multiples within CHR0000..1FFF. Copies append after the source-profile upload, preserving UI/fonts unless explicitly replaced. |
| ducktales.audio | periods[32] integers0..2047; voices[4] note arrays, each note {key:0..31,ticks:1..255,volume:0..15,duty?:0..3}; loop defaults true. Key0 rests. At most1024 notes per song. Compiler produces safe shared primary-driver commands; ticks are actual driver frames at normal tempo. Original duration tables remain intact. |

Each original playable level and added level can have at most one successor.
Missing predecessors, cycles and forks are rejected; there is no fixed number of
added levels. Natural completion runs the original land-return transfer/message/
acknowledgement work, then enters the successor automatically. Chain end returns
to ordinary source selection. Entry/reload uses the authored checkpoint and main
map; source special-room layouts remain original. Source rule-stage and named
level identity remain separate in semantic observation.

Source progress bits already authored by item collection remain intact, including
through chained levels; their source-profile conditions can influence later
actors. Completion_actor/scene_signal clear per authored entry so stale completion
cannot advance a chain. Late-route effects performed by source completion remain.
Use an appropriate rule profile for designed actors and progression. Changing
profiles does not create independent copies of the original global progression.

Views are constructed from the immutable original catalogs on each entry. Runtime
actors, world caches, command ring and display storage remain single canonical
owners. Leaving/entering authored views stops the old encoded audio program and
silences its voice controls; queued source commands and synthesis sample clocks
remain. This extension transition is documented native behavior, not an original
cartridge feature.

examples/authored-level supplies authored air/floor/pattern/music payloads and a
source boss-template reference after Amazon. It demonstrates file delivery and
shared rules; balance and full traversal are not independently QA-certified.
Edit these authored files to design a level. Do not redistribute ROM-derived
replacement art/music/maps. Original-only loading, all five actual consumers,
predecessor rejection, note timing, base restoration and checkpoint joins have
focused implementation tests; they do not replace stage5 gameplay QA.
