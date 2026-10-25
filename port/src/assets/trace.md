# B02 whole-area source trace and contract derivations

Stage3 worker `/root/behavior_contracts/b02_assets`; entire prompts/contract.md,
workflow/common/coding standards, architecture, B02 inventory, debug API, source,
B01 trace and handoff read. No game behavior is implemented. Native executable
contracts are contracts.rs; formats.rs/catalog.rs are visibly pending interfaces.
Frozen game.asm SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm SHA256 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Physical offsets include the16-byte header. CPU addresses below retain mapper context.

## CPU and hardware basis

Consulted toolkit docs/development.md platform table, docs/nes.md and
nes-native.md. Consulted original manufacturer-authorized 1976 MOS/Synertek manual
at https://syncopate.us/books/Synertek6502ProgrammingManual.html (publication6500-50),
sections2.2.1/2.2.1.0,4.1.2/4.2.1,6.5,7.4..7.7,10.1/10.2/10.7/10.8.
Eight-bit INC/DEC wrap and change N/Z, not carry; BPL tests bit7 clear, so decrement
zero->FF ends a count-minus-one loop. INY wraps FF->00; (zp),Y uses a 16-bit base
plus unsigned Y. CMP establishes unsigned >= in carry; ADC adds incoming carry.
LSR/ASL shift zero in; four LSR extract the descriptor bank independently of entry
carry. Indexed pointer tables do not perform runtime bounds checks.
NESdev 2A03/CPU and PPU programmer-reference indexed documentation was consulted:
https://www.nesdev.org/wiki/2A03 ,
https://www.nesdev.org/wiki/PPU_programmer_reference ,
https://www.nesdev.org/wiki/Instruction_reference . Direct wiki opens returned403;
search returned their relevant instruction/register text. The NES CPU lacks decimal
arithmetic. PPUCTRL bit2 selects PPUDATA stride1/32; reading PPUSTATUS resets the
PPUADDR write latch. No CPU emulator, RAM-address map, instruction-byte oracle or
second game implementation is introduced.

## CHR descriptors, every record and endpoint

UploadChrLists game.asm10966..11014, CPU C301..C357 within physical1C2F6..1C368
(region220, shared with B01 RNG). Fixed pointer table CPU C358..C36D,
physical1C368..1C37E (region3); descriptor records CPU C36E..C42B,
physical1C37E..1C43C (region4). Physical/header context is essential: source
symbol CHR_UPLOAD_POINTER_LOW=C358 does not mean physical C358.

All11 originals are independently decoded into contracts.rs CHR_EXPECTED; every
record compares bank, physical start, page count, destination, negative terminal
byte and physical endpoint. Complete endpoint inventory (selector:CPU origin ->
physical terminator) is 0:C36E->1C38A;1:C37B->1C397;2:C388->1C3A4;
3:C395->1C3B1;4:C3A2->1C3BE;5:C3AF->1C3DA;6:C3CB->1C3FF;
7:C3F0->1C418;8:C3F0->1C418;9:C409->1C434;10:C425->1C43B.
Every original terminal is FF. Lists7/8 deliberately alias; copies preserve order
and repeated writes. No selector deduplication removes original identities.

Entry writes PpuCtrlShadow AND FE to hardware only. **Correction to a possible
inference:** FE clears nametable bit0, not PPUDATA increment bit2. It neither
changes the shadow nor guarantees stride1 for an arbitrary caller. Tests preserve
both stride branches; normal source callers prepare rendering state. B03 must own
the hardware-visible address/stride semantics. Every new record reads PPUSTATUS
before high/low PPUADDR writes, so inherited address latch phase cannot redirect it.

The negative-tag check precedes all remaining field reads, so any80..FF terminates
and need not have two following bytes. Positive tag upper nibble is mapper bank;
lower nibble pages-minus-one. Source low byte is explicitly zero, source high is
the second field, destination high third field and destination low zero. Mapping
is header + bank*4000 + (CPU-8000), never just CPU address. SelectPrgBank16K saves
foreground bank and clobbers X; saved list Y survives via Scratch07, not X.
Copy page: Y00..FF reads256 bytes, INY->00 exits inner loop, INC source high carries
across pages, DEC count/BPL copies another while0..7F. Count0 copies1 page;
count15 copies16; lists5/6 exercise16 and many one-page records. This page carry
is not CPU carry flag propagation. The source high increment is full8-bit; originals
all stay within8000..BFFF. Destination advances by PPU's selected stride, not an
explicit assembler destination-high increment in this routine. After each record,
restore saved Y, INY skips destination byte; nonzero Y loops. All original list
spans are shorter256 bytes, so the Y00 early exit never cuts a valid source list.

Tests independently exercise all count nibble values and negative endpoints,
all usable banks0..6, missing fields/terminator, invalid selectors and bounds,
source page below8000, unsupported bank7 and crossing BFFF. Original code has no
file validation. Native rejection for malformed extension descriptors is an explicit
source-informed safety policy (uncertainty B02-extension-format-validation), not
claimed original error behavior. No source descriptor crosses the switch window.

prepare_chr returns ordered copies and final selected-bank request, including no
selection for an empty list. Runtime/B01 owns the bank context, B03 the display;
this format helper owns neither duplicate mutable state. Foreground bank selected
by a graphics copy is not automatically restored at subroutine return: callers
explicitly switch before invoking the next banked routine; NMI temporarily selects
audio then restores B01's current foreground context. This matches B01 declarations.

## Background selection and complete32-list table

Poll/Choose/Begin/Queue/Complete CPU C42C..C507, physical1C43C..1C518,
regions221..227. Configuration CPU C508..C71C physical1C518..1C72D region1261:
stage offsets C508[9], room selectors C511[41], palette selectors C53A[41],
list low C563[32], list high C583[32], records C5A3..C6D8 and palette quads
C6D9..C71C (17*4 bytes). No unclassified hole in this533-byte configuration.
All32 lists compare every record and FF endpoint in BACKGROUND_EXPECTED.
CPU origin -> physical endpoint, selectors in order:
0 C5A3->1C5B3;1 C5A4->1C5BD;2 C5AE->1C5CA;3 C5BB->1C5D4;
4 C5C5->1C5D8;5 C5C9->1C5E8;6 C5D9->1C5F5;7 C5E6->1C605;
8 C5F6->1C60C;9 C5FD->1C616;10 C607->1C61D;11 C60E->1C62D;
12 C61E->1C640;13 C631->1C64D;14 C63E->1C65A;15 C64B->1C667;
16 C658->1C66E;17 C65F->1C675;18 C666->1C685;19 C676->1C68F;
20 C680->1C69C;21 C68D->1C6A3;22 C694->1C6B0;23 C6A1->1C6BA;
24 C6AB->1C6C7;25 C6B8->1C6D1;26 C6C2->1C6E1;27 C6D2->1C6E8;
28..31 alias12..15 respectively. List0 is immediately FF; do not manufacture a
chunk. Full tests also run every original span through each64-byte poll, comparing
source bytes loaded locally without embedding commercial payloads.

State negative returns immediately: no palette re-copy, bank selection, queue or
counter mutation. State0 chooses stage-offset plus current room row using CLC/ADC
and then unsigned Y; positive untagged selector goes directly to begin. Tagged
selectors mask7F; stage4 threshold camera high3; all other stages threshold4.
Only actual tagged original rows are stage2,row2=87 and stage4,row1=91.
Boundary equality advances to adjacent list (7->8 or17->18). All41 source rows and
camera0,2,3,4,FF are covered, including untagged rows that must never advance.
Valid original row counts by offsets are7,8,6,6,6,7,1,1,1. Stage9 has an ending
layout but no background-stage configuration entry; B16/B17 select other temporary
contexts before using these routines. Native invalid-stage/row errors deliberately
prevent adjacent-table reads. Palette choice always uses the original stage/row,
not the camera-selected variant. Quad index16 on stage0,row1 reaches the last
quad C719..C71C; tagged stage2,row2 palette5 and stage4,row1 palette7 remain fixed.
The exact four bytes are local source-derived expectations in the tests.

## Background continuation, every state branch

A new list increments state0->1, sets descriptor pointer, clears descriptor cursor,
source low and destination low. State AND02 false: increment1->2, parse record and
advance descriptor cursor by3, clear source cursor. Negative tag writes the actual
negative tag into state then copies four palette-tail bytes to both TerrainPaletteTail
and011C (base/active destinations), without creating a chunk. No palette upload
flag is set here: B03 and caller commit orchestration own later visible palette.
State2 AND02 true resumes current record, without repeating list selection or
source reset. Bank is selected for every chunk (interrupt may have restored a
foreground context in between).

QueueBackgroundChunk reads exactly64 bytes; initial X0 and CPX40/BNE bound it.
Y is the source-page cursor, advancing00->40->80->C0->00. Primary queue high/low
is current destination, count3F denotes64 bytes; payload64 bytes; Primary queue
terminator is written at SecondaryPpuQueuePayload (FF) which aliases the byte
immediately after this primary record. This is **not** an independent second
payload request. B03 must preserve this storage-layout effect using a logical
terminated primary command, not duplicate queue state. Destination low ADC40
starts CLC; high ADC00 carries only from low sum (STA/LDA preserve carry).
Source cursor zero after fourth chunk increments source high and decrements
pages-left. Underflow FF decrements state2->1; record advancement waits until
next poll. Therefore a one-page record requires4 calls, and its negative endpoint
requires a fifth. Multi-record next call parses and copies first chunk immediately;
there is no empty poll between records. Tests cover source/destination page carry,
count16 path through generic descriptor tests/all originals, next-record cursor
reset and bank change. No subtraction carry from DEC is applied to destination.

At every chunk source writes FF terminator then LDY DisplaySuspendDepth. Zero
falls through RequestBackgroundChunkUpload, storing FF to PrimaryUploadPending;
nonzero calls FlushPrimaryPpuQueue immediately, whose first operations clear the
pending flag and consume the primary command. Flush changes X/Y/A; callers rely
on completion and state, not their previous scratch values. Any nonzero depth,
including128/255, is immediate, not signed. Rendering-active polling must be
scheduled so next chunk cannot overwrite a pending one before NMI. B02 returns a
commit request, never advances interrupt time. This remains part of existing
B01-frame-boundary-integration and B03/B13 responsibilities. Source foreground
loops while rendering suspended may flush many chunks in one native tick; treating
every poll as a mandatory yield changes source timing.

## Callers and cross-call effects

All symbolic UploadChrLists calls independently reread:
InitializeTitle D4xx line13367 selector6 after background clear/screen script;
InitializeLandSelect D630 line13538 selector5 with temporarily saved stage;
LandSelect confirmation D7xx line13805 current StageIndex after terrain/screen;
EnterSpecialRoom DE/DF line14885 selector6+(request low nibble), after layout
loader/B06 bank AE22. That callee must not silently destroy X before CHR (B10/B16
contracts own its exact return). Scene presentation E0F4 line15704 uses
SceneChrListChoices[TerrainRoomCursor]. Those six original scene values are read
from original table by B16; B02 only promises bounded11-selector decode.

All PollBackgroundUpload call families/readers/writers reread:
AnimateVerticalRoomTransition C976 resets upload state but its phase handlers
handle terrain rows; that reset alone is not proof of a poll/yield.
EnterLateStageHorizontalTransition CFxx/D0xx lines12555/12576 sets temporary stage5,
selects late row, applies camera/player signed offset before each poll, yields only
while camera low nonzero, restores saved stage/row afterward.
InitializeStageRuntime D7D4 lines13824..13827 resets0 and loops until negative
while rendering suspended; then switches bank6 before spawn initialization.
Reload DCBx lines14535..14538 resets0 and loops, restores stage/row and switches6
before spawn cursors. Reload87/85 branches choose temporary context separately.
BuildFinalRaceTerrain E0xx lines14970/15045/15148/15166 sets stage5,row6 for
background calls, restores stage1 between calls, loops columns with source bank
changes; completion later restores row2. Existing B01 requests do not duplicate
these scene continuations. Build room scene TerrainBank is selected from six scene
values at E08A; ending path explicitly bank1 at E0B6, then later CHR list selection.
A source state write at C9xx or race/room entry cannot be replaced with selecting a
new background list every frame: positive state must resume old descriptor.

## Every graphics/generic/uniform data consumer

planning/contracts/B02-consumers.json is a complete intersection inventory, with
original ledger IDs/extents untouched; it includes every primary/related B02
region and additionally mixed bank3 payload, room fragments/uniform intervals and
bank4 tail. Its spans derive from independently read descriptor fields and source
consumer arithmetic, not tile-like appearance. It is navigation/evidence metadata,
not an executable oracle. No original bytes are copied into it.

All graphics banks0..2 physical10..C010 are covered by CHR/background and/or
terrain consumers. Every generic AssetData interval has such a consumer; there is
no unsupported 'padding' exclusion. Representative meanings must not erase overlaps:
- AssetData_00810/C10/1010/1910/2110/2510/2810/2A10/3010/3610 are direct background
  descriptor payloads, interleaved with labeled CHR; CHR and background consume
  different subsets. Source list spans can cross named incbin boundaries.
- AssetData_03F10 is bank0 BF00 packed terrain attributes **and** background list22
  or24 payload. AssetData_03610 includes terrain block tail and plane bytes.
- AssetData_07010 contains CHR source bank1 B000..B2FF (background23), followed by
  bank1 terrain B300..BFFF. AssetData_0B110 is bank2 B100 single CHR page in
  background18. AssetData_0B310 is full bank2 B300..BFFF terrain set.
- Bank3 mixed AudioStreamsAndUnresolvedBitmapData_0CAC0 extends to F110. Background
  records consume A900..B0FF, physical E910..F110: list11/22 atA9,20/24/25 atAD,
  25 atAA..AC,20/27 atAE..AF/B0. This is proved by descriptors, not bitmap inference.
  CHR lists5/6/10 then consume bank3 B100/B200 physicalF110..F310 (ChrTiles_0F110).
  Audio interpretation/reachability CAC0..E910 and possible shared use remain B18;
  no claim that graphics-looking bytes cannot be read by audio. Original unknown
  two-byte regionC522 remains B18 pending with its ID unchanged.
- ResolveTerrainBlockPointer CDB4 reads room low byte plus one bitmap bit and
  two ASL/ROL produces index9bit*4. Add highB3 gives B300..BAFF (512 records4 bytes).
  StoreExpandedTerrain CD7E reads BB00/BC00/BD00/BE00 and packed BF00, indexed0..255;
  changed-terrain paths D068/F697 use same planes/attributes. StageTerrainBanks
  DA10[9] contains1,2,2,3,3,1,0,3,3, proving banks0..3 are required, including all
  generic fragments above. B04 owns actual block/collision/plane decoding; B02
  terrain_payload supplies their complete named bytes without decoding them twice.
- ResolveRoomPayloadPointer CE2C computes8000+room*72; read64 low block indices and
  eight high-bit bitmap bytes. Finite source room lists bound room0..199, hence
  physical10010..13850 including all uniform fragments. Uniform bytes inside that
  interval are room data, not discarded alignment. B04 must test its9-bit decode.
- Bank4 BF00 physical13F10..14010 (ledger region505, provisional labelBoundedData_13F10,
  source Bank09_Data_13F10Bytes) is explicitly background list22 second record.
  **Source comment correction:** 'no executable references found' misses this
  indirect descriptor consumer. It must load even though appearance was uniform/
  bitmap-like. The earlier uniform gap13EBB..13F10 is outside room and upload spans;
  it remains B16-owned bounded gap review, not a B02 asset exclusion.

Named loader tests resolve every original graphics span through Assets source,
compare to matching local ROM, honor modified named generic slices across fragment
boundaries, reject missing fragment without fallback to instructions, and verify
all four complete terrain source sets. These are substantive pending contracts for
loader behavior, distinct from existing raw-recipe infrastructure validation.

## Named catalog and extensions

Existing Assets::load validates matching ROM SHA and all151 raw recipes using
MemoryAssets::collection(source-assets.json). Existing infrastructure tests cover
same-length replacements, nonempty extensions, identity and path/order/size rules.
The new immutable catalog is declared separately: LevelDefinition named IDs,
optional original selector, named rooms/terrain/spawns and graphics/audio references.
Payload decoders remain B04/B10/B15/B16/B18 owners. Generic source bytes are not
mistaken for decoded resources. Source selectors0..9 have layouts; source spawn
and stage terrain config have only0..8; ending9 must not index those tables.
Source selectors5..8 empty spawn-list pointers are legal lists, distinct from the
absence of selector9. original_catalog's ending resource join must use B16's
source scene context rather than inventing StageTerrainBanks[9].

Tests preserve ten source selector identities, optional ending spawn separation,
27 additional named definitions alongside originals, missing name lookup,
no fixed runtime count10, immutable validated joins, duplicate names/selectors/
resource IDs, invalid names/selectors, missing or wrong-kind resource IDs across
all five kinds, missing graphics/audio, empty payload rejection and actual File
origins. Author-authored extension manifest format is explicitly version1 with
resources{id,kind}, levels{id,rooms,terrain,spawns?,graphics[],audio[]} and absent
original_selector for added content. Proposed filename levels.json. Loading file
manifest is pending and must be wired through same desktop/browser MemoryAssets
path in stage4, not directory scans or a new content-selection UI. Manifest
syntax/validation and source-window rejection are native extension design, not
recovered original file format. Residual assumption recorded in uncertainties.json.
Original selector9's exact composition remains B16/B17 work; no alternate game
rules or fixed-source-count cap is added. No user-save format exists (battery bit
clear); debug replay remains separate and consumes immutable asset identity.

## B16 explicit ending resource join

B02 SceneResourceContext/SceneAssets now binds B17's layout9 separately from
B16's per-scene terrain [2,2,2,1,1,1] and CHR[9,9,9,9,10,10]. No stage9 spawn or
initialization record is read. One additional B02 contract exercises allsix
original scene bindings, distinct layout9 identity, actual complete block/property
slices and CHR descriptor expectations. Special scenes1/2 instead call bank6
AE22 with X7/8 preserved from layout setup, then CHR7/8; ordinary stage unchanged.
All remain pending source-backed declarations; B16 handoff supplies actual checks.
