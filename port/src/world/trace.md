# B04 room, stream, camera and terrain-edit source trace

Contracts: [contracts.rs](contracts.rs). Interfaces: [mod.rs](mod.rs), deliberately
unimplemented until stage4. Reference: `reference/source/game.asm` SHA256
`c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc`, and
`reference/reading.asm` SHA256
`201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5`.
Source labels below are CPU addresses; physical fixed-bank addresses add `$10010`.
Switch-window bank N physical mapping is `$10+N*$4000+(cpu-$8000)`.
No original room, graphics, or extracted payload is embedded in these contracts.

## Source CPU, flag and context audit

Read toolkit `docs/development.md` platform table, `docs/nes.md` and
`docs/nes-native.md`. Independently consulted manufacturer documentation:
[Synertek/MOS MCS6500 Programming Manual, August1976](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
sections2.2.1 ADC,2.2.2 SBC,3 condition codes,4.2.1 compare,10.1..10.4 shifts/rotates,
8 stack/subroutines (8.11 PHP/8.12 PLP), and appendixB instruction flag tables. This is the
manufacturer-authorized MOS reprint. Secondary instruction-page lookup was only
a navigation aid; source CPU rules were rechecked in the manual.
NESdev CPU/Status_flags/CPU_ALL fetches returned403; this is recorded rather than
claiming those fetches succeeded. This is the NES NMOS6502 family, not65816.
Eight-bit A/X/Y wrap; CMP/CPX/CPY set carry on unsigned >=; BMI/BPL inspect bit7,
not mathematical sign after arbitrary overflowing subtraction. ADC consumes carry;
SBC consumes inverted borrow. AND/ORA/LDA/STA/transfers do not reset carry;
ASL/LSR replace it; rotates consume/replace it. INC/DEC do not alter carry.
PHP/PLP explicitly preserve the proposal's low-byte carry across the first
high-byte addition. All actual B04 arithmetic is binary.

`SelectPrgBank16K` `$FFDE` stores bank, TAX, indexed mapper write, RTS: X is
**clobbered to the bank**, A preserved, N/Z changed by TAX, C preserved. This
matters in terrain expansion, querying, and actor register restoration. Native
world types do not retain a bank register or CPU scratch array. Original-bank
selection is resolved to immutable decoded resource references; original callers
requiring bank6/4 remain caller resource contexts. Physical addresses must never
be used as CPU addresses or vice versa.

## Formats and all finite source lists

`LoadTerrainLayoutPointers` `$C7B0` reads rows `$CE57`, columns `$CE61`, split
pointers `$CF7F/$CF89`. All10 layout selectors are required, with dimensions
7x8,8x8,6x8,6x8,6x8,1x1,1x1,1x1,1x4,1x6. Room-list starts are
CE6B,CEA3,CEE3,CF13,CF43,CF73,CF73,CF74,CF75,CF79. First five lists have
16,24,9,10,11 FF entries respectively; FF means no room, not room255.
Selectors5/6 alias room0,7 uses195,8 uses196..199,9 uses114,115,116,43,17,19.
Selector9 is an ending layout and does not imply stage configuration9: StageTerrainBanks
and spawn configurations have9 entries. Scene callers supply their own context.
B02 owns raw complete named assets; B10 independently owns the empty spawn lists
for5..8. Dynamic added layouts use `Layout` vectors, not fixed ten-slot runtime
storage. The original-format adapter has exactly10 source layouts.

`ResolveRoomPayloadPointer` `$CE2C` multiplies selector by72 through a selector*8
partial plus selector*64; its rotated high bytes and addition carry are required.
Finite non-FF list domain reaches every original room index0..199; full logical
room span is physical10010..13850 (200*72). Each record contains64 low bytes,
then8 ninth-bit bytes. The maintained incbin/uniform fragments are **not** room
boundaries. `every_room...` compares all200 complete records, including every
uniform slice within that consumer span. B04 ledger region462 extends beyond
13850 and regions499/463 are after it; these do not become room201 or another
room format merely because the planner assigned their containing records toB04.
They remain preserved raw source records under B02's catalog; the explicit
consumer audit below tracks that unreferenced suffix without asserting a game
mechanic for it.

`ExpandTerrainBlock` `$CCBD`: ring index r has row=r&7, block column=(r>>3)&7.
Packed coordinate is row*8+column (transpose). `ResolveTerrainBlockPointer`
`$CDB4` uses bitmap byte64+row and mask80>>column. Initial high scratch is
bitmap hit?1:0; two ASL/ROL pairs add the low-byte's upper bits, so descriptor
index is low+256*ninthbit; pointer=B300+4*index, spanning B300..BAFF. All512
four-selector descriptors, all four256-byte tile planesBB00,BC00,BD00,BE00 and
property planeBF00 are required in banks0..3, not only bank3's named record.
`all_four_terrain_banks...` checks every byte through each decoded resource.
`ninth_bit...` varies every cell in both descriptor halves independently.

## Expansion, cache coordinates and substitutions

`PrepareTerrainExpansion` `$CCD8`, `DecodeTerrainQuadrant` `$CCE7`,
`StoreExpandedTerrain` `$CD7E`: visit q3,2,1,0. Quadrant staging offsets are
0,2,8,10. Each selector's TL/TR/BL/BR populate a4x4 row-major tile block. Property
low2 bits are palette, high-nibble low2 bits are collision. Shift by2*q and OR
independently. Synthetic selectors1..4 have distinct four-plane tiles and
properties10,21,32,03; expected paletteE4/collision39. Masking ignores descriptor
bits2/3/6/7. Cache index=(room_in_row&1)*64+transposedcoordinate. Triple ROR and
AND40 extract source parity independent of incoming C; native cache index must
not depend on an emulated carry flag.

At coordinate>=38 and q>=2 the source jumps directly to StoreExpandedTerrain,
without loading selector or staging offset. X at the first jump is the terrain
bank from SelectPrgBank16K; Scratch03 is residue. Those writes are overwritten
or belong to the discarded lower two tile rows. Observable valid top8 tiles
come from q1/q0 and packed high nibbles stay0. Native `ExpandedBlock` promises
only the visible first8 tiles at this boundary. This deliberately does not give
undefined discarded staging bytes invented values. Residual unusual caller
scratch side effects are tracked in uncertainties.json; normal stream/vertical
call paths establish bounded staging before the discarded writes.

Stage3 `$CD36`: only when split transition control0, controlA2 nonzero,
roomcursor28, coordinate21 or29. Each matching quadrant clears **the whole**
accumulated palette/collision bytes and substitutes selector88. Thus final
properties0, not only cleared bits for one quadrant. Wrong stage/room/coordinate,
zero control and split transitions preserve ordinary behavior.

Stage4 `$CD50/$CD54`: split must0, A1 negative, roomcursor27 and coordinate>=20.
For r&38=18, odd q usesD6; for r&38=20, even q usesD7. Other half unchanged.
Descriptor palette/collision were accumulated **before** substitution and remain
original. Tests cover both halves, positive7F/negative80/FF controls, boundary,
wrong room/column and split bypass. B13 owns the progression-control writers;
B04 takes explicit semantic source-control values until those are named there.

## Initial loading, reversible streaming and queue shaping

`BuildInitialTerrainColumns` `$C71D` starts roomcursor=cameraroom. Selector x0
if room-in-row0 or preceding FF; otherwise x1, then check `roomcursor-2 ==
LayoutColumnCount` literally (not a guessed left/right-bound test), else next
FF keepsx1 and valid next givesx2. ArraysC786,C789,C78C give counts5F,5F,7F;
starts00,20,20; roomadjust00,FF,FF. DEC/BPL means count+1 visits=96/96/128.
Adjusted room-in-row becomes roomcursor&7. Each visit builds one column, increments
ring modulo64 and increments roomcursor on wrap; room-in-row is then roomcursor&7.
Final direction1/ring1F regardless final visited ring. Missing columns still
consume visits; no truncated representative load. `BuildWholeTerrainRing`
`$C78F` is distinct:64 positions of same current room, no extra room increment.
LoadStagePresentation `$D9CA` establishes room-in-row before this caller.
Land-select `$D64A`, stage choice `$D7A6`/`$D7D1`, checkpoint `$DD4E..DDC0`
and scenes/finale call these primitives. B13 owns checkpoint selector/record
interpretation; its loop preserves its inclusive count and supplied cursors.

`MeasureCameraMotion` `$C7C5` subtracts low bytes only. Negative-bit result is
negated by EORFF/CLC/ADC1. Therefore page crossingFF->00 is1;00->FF is1;80
remains80 under negation. Forward work=(previous&3+absdelta)>>2; backward uses
(previous^FF)&3. ADC is eight-bit before the two shifts. No-work returns before
queue mutation. No 16-bit absolute-distance rewrite is equivalent.

`PrepareTerrainStream` `$C7F7` zeros queue cursor. Same direction advances ring
by+1/-1; an unmasked step with high bits set carries room by+1/-1. Reversal
preserves ring and adds room+2/-2 (direction table indices1/3), then builds.
Room-in-row during streaming is a full wrapping byte, not forcibly&7.
`CheckTerrainColumnBounds` `$C960` rejects FF or room-in-row>=columncount;
carry from CMP is caller contract. `ResolveRoomPayloadPointer` runs before the
bounds helper but rejected columns produce no expansion/cache/upload. Rejected
work decrements remaining; previously built queue is still submitted/terminated,
otherwise no upload. Normal work queues four inclusive-count3 records (four tiles
per row), plus one inclusive-count0 attribute record, then negativeFF terminator.

`BuildTerrainTileRows` `$C852` computes destination from r&7 high table
20,20,21,21,22,22,23,23 and low00,80 alternating; add horizontal(r>>1)&1C;
roomparity OR4 selects second nametable. Row addresses advance20. Expanded
collision is committed before early display gates. For stages<5/even room:
row0 queues nothing, row1 discards first two tile rows and palette low nibble.
For transposedcoordinate>=38 all stages discard last two rows and palette high
nibble. Short queue has8 tiles+1 attribute versus17 writes normally.
Attribute address=23C0|coord|(parity<<10); logical attribute cache stores the
masked palette used here. `ColumnResult.collision` and `.attribute` are separate optional writes: the early
row0 branch changes collision but leaves the old attribute cache untouched.

Active display marks primary pending; suspended depth!=0 flushes immediately.
`UploadCommit` is shared B02/B03 vocabulary; no World-owned PPU queue is created.
B01 interrupt consumes prepared presentation on the following atomic boundary.
B03 owns actual queue/VRAM/raster state. No synchronous render call advances World.

## Camera and exact asymmetric clamps

`UpdateCameraMode` `$EFE2` tests SpecialRoomRequest bit1. Ordinary path returns
when scroll direction&3=0; otherwise starts direction1 and computes two separate
SEC/SBC: playerlow-cameralow-80, keeping wrapped byte only. Zero returns; bit7
selects backward. `ComputeCameraPosition` `$F0C4`: proposed low=cameralow+delta;
PHP preserves carry while page+step and room+step both consume same original
carry. Direction step0 forward,FF backward. Do not increment room from the
already changed page or use high-byte player difference.

Positive check `$F067`: negative late mode, proposedpage+1==columncount,
proposedroom+1 wrapping0, or next FF clamps. Clamp sets camera low0, limits player
low toF0 iff playerlow>F0, then **commits proposed page/room**. Negative check
`$F094`: negative late, proposedpage negative, or proposedroom FF clamps low0 and
player low to10 iff<=10, but **does not commit proposed page/room**. Tests use
inputs deriving actual signed low-byte branches, including page carry/borrow.

Automatic `$EFE8`: departure negative and camera nonzero decrements16-bit camera
by1. Held left/right considered only at screen-relative low-byte margins10/F0;
right bit takes priority if both held. Left below10 adds+1.5 to24-bit player
position (fraction80,integer01,high00). Right at/aboveF0 adds-1.5 (80,FE,FF).
Otherwise player unchanged. Nonzero camera sets negative streaming direction;
zero camera returns. This route does not apply the normal room-bound proposal.
B13/B16 own the automatic-mode/departure triggers. PreviousCameraX is recorded
in `$DBE9` only after B03 composition and B10 spawn-window processing; per-frame
changed/displacement flags clear at that same source boundary, not at rendering.

## Mutable terrain, recovery and dialogue bursts

`ReplaceTerrainForActor` `$F4D8` saves actor slot and callerY, strips animation
bit7, SEC/SBC1A, AND3, **ADC13 with SBC carry intact**. Animations>=1A therefore
use base14, not13. Stage*4 then adds that selector. `ReplaceTerrainFromActorState`
`$F551` uses stage*4+local, marks TerrainChangedThisFrame by INC (not boolean
assignment), appends14-byte record, restores bank6 and actor slot/callerY.
Animation wrapper appends14 without itself incrementing changed. B08 controls
once-per-frame gates and flag-bit10 before invoking; World cannot silently treat
all mutations as deferred until the next frame.

`ReplaceTerrainActorGate` `$F506`: negative local counter first scans slots15..1
for nonnegative state; no player slot0. Full pool still proceeds to terrain
replacement. Recovery initialization `$F51C` sets stateA1/handler13, copies
screenY/worldX+page, resets animation tick/frame, collision profile and vertical
velocity; local-state animation tableF67B gives24,28,25,26 for ordinary0..3.
B10 owns the store/allocation mechanism and B08 owns subsequent recovery behavior;
`RecoveryActor` contracts define the source one-shot effect without a second actor
store. No-free, positive-counter, highest-free and every source animation branch
are covered.

`BuildChangedTerrainRecord` `$F574`: use spawnY (not falling actor screenY), worldX
and page. Block index=((Y>>2)&38)|((X>>5)&7)|((page&1)<<6), quadrant=(Y>>3)&2
OR(X>>4)&1. Destination high=F67F[Y>>5]|pageparity4; low=F687[Y>>5]|((X>>3)&1E)
OR quadrant offsets00,00,40,40; next row OR20. Two tile-pair records(count1) plus
one attribute(count0) total14 encoded bytes; terminator atnextcursor. Tile planes
and descriptor come from stage terrain bank via ChangedTerrainSelectorsF653.
`MergeChangedTerrainAttributes` `$F606` extracts collision high2/palette low2,
shifts2*q and combines clear masksFC,F3,CF,3F with respective caches. Cacheindex<10
masks the **outgoing attribute only** toF0; the full logical attribute cache remains
updated. Both room parities and all four quadrants are independently asserted.
No permanent global modified-map history is invented: actor spawn suppression
and current streamed caches carry source persistence, and rebuilding must follow
source lifecycle. B08/B10 own removal/suppression, B13 reset/reload.

`DialogueRevealActor` `$EDCF` clears collision indices21/29 only (addresses05A1,
05A9), not attribute cache, then starts burst0. Actor initialization is B15.
Rightward transition actor `$FD6E..FD9D` invokes burst6 at page7/X>=70 (B15 owns
trigger). Burst `$D068..D10F` descriptor0/6 has repetition3/6, destination2204/
260E, two selector seed entries88,88 orD6,D7, zero terminator. Seed is two
four-tile rows; each repeat adds40 including low-byte carry to high, resulting
4 or7 rowpairs (32/56 writes), marks primary pending then requests bank4 four
particles with origins(88,28)/(88,78). B03 uploads; B10 owns actual particle slots.

## Sixteen vertical phases and ordering

`AnimateVerticalRoomTransition` `$C976` derives direction from held&0C&04:
X0=Up, X4=Down. Save ring/room-in-row, set splitcontrol0C, ring7/0, counter0,
background state0, clear nonplayer actors, hide sprites, **yield before motion**.
`VerticalTransitionTick` `$C99D`: every tick playerY += FF/01. On even counters,
unless the special reset counter, split +=FFE0/0020 and playerY +=08/F8.
Special counters12/46 set splitlowA0/00 and splithi=(oldhi&24)|03/00;
no8-pixel compensation on that tick. Odd counters leave split unchanged.
Dispatch index=(counter&7)*2 +10/00; last ASL guarantees carry0 before ADC.
Sixteen actual little-endian words atCA2A:
Down: BuildRoom,NextRoom,PreviousRoom,PollBackground,CopyStaged,Poll,Poll,Poll.
Up: NextRoom,PreviousRoom,BoundaryOrCopy,Poll,Poll,Poll,BuildRoom,Poll.
The C9FA synthetic return lets phase RTS resumeC9FB; native explicit continuation
replaces stack trick, not a general CPU interpreter. Poll is B02/C42C and may
advance chunk/palette/background completion; it is not a pure no-op phase.

Every phase then hides/composes bank5 actors and yields, increments counter,
and repeats until3C. Restore ring and room-in-row; reset bank6 spawn windowAE60,
updateAEA0, clear splitcontrol, set palette pending1. Exactly60 motion/composition
frames plus entry yield. Starting player80/split24C0 ends player2C Up orD4 Down,
split24C0 both. B01/B03 must preserve the foreground/next-interrupt distinction;
B13/B06 owns legality and target room commit before entry. In particular player
transition `$E878..E8E2` requires camera low0/high odd, legal FF-free adjacent
row and row bounds; its ASL/ADC and source `ObjectScreenY,x` unusual indexing
must be traced by B06 rather than hidden in a World API with invented input.

`BuildVerticalTransitionRows` `$CAA0` resolves CameraRoomIndex, visits8 horizontal
blocks by ring+=8, sets room-in-row1, expands4x4 tiles, writes current upper/lower
pairs and retains opposite pair in staging. Up first takes lower8 tiles, Down
upper8. Attribute old masks0F/F0 and newF0/0F merge into upload; cache gets full
new descriptor. Build destination is second nametable,32-byte rows(count1F)
and8-byte attribute(count7), FF terminator, pendingFF. Ring wraps to original&7.
`VerticalPhaseCopyStagedRows` `$CA4E` copies32-byte staged planes and8 cached
attributes, updates ring by-1/+1 modulo8, then uses **old row** and opposite
direction upload parameter40/00. `VerticalPhaseWrapBoundary` `$CA8B` at row7
flips direction, builds row, decrements row, restores original direction; other
rows copy. Both branches are executable contracts.

`VerticalPhaseNextRoom` `$CB5A`: reject last camera page or next FF. Previous
`$CB6E`: reject camera page0 or previous FF. Previous selects ring|20 and tile
half10; next half0. `BuildVerticalNeighborColumns` `$CB8E` visits4 blocks by+8,
room-in-row0; row0 is skipped four times and returns **without upload**. Other
rows copy four16-byte tile planes plus4 attributes. All output addresses use
first nametable and selected horizontal half. Row1 discards first two planes,
row7 discards last two; attribute high/low nibble masks respectively. Counts0F
and03 are inclusive. Ordinary68 writes, edge36 writes. Cursor finishes&7.
The methods in mod.rs return semantic writes rather than a duplicate queue.

QA-001 (stage5): source D9CA writes TerrainRoomInRow from initial camera low3;
C710 tests retained TerrainRoomInRow before choosing its work count. First Amazon
initial build leaves it1, so the second source call includes preceding room39
with128 visits; substituting camera page0 chooses96 and loses far-cache data.
Services now owns that exact retained field across calls. C78F whole build leaves
ring40hex and room=camera, retaining in-row. Its sole source caller is land setup
immediately after D9CA, establishing in-row==camera.room&7 for whole-ring payload
visits. Two focused source-backed integration contracts and fresh original/native
full-cache comparisons close this caller defect; original trace identities remain.

Stage5 QA003 source caller correction: F4D8 animation helper is used by B044
spawn and performs no recovery/count increment. F506 active gate optionally
allocates recovery before F551 local-state replacement and count increment.
Both computed indices go through immutable ChangedTerrainSelectors F653 at F600
before F69C reads tile/property planes. Content decodes that fixed byte-indexed
extent once; callers retain the single native cache and secondary queue owners.
Source initial Moon spawn attr46/47 are0; preserved native event472 erroneously
wrote40/54hex. Actual spawn and active-gate regressions reproduced both defects.
