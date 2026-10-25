# B10 independent lifecycle and spawn trace

Reference game.asm SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Source paths reference/source/{game.asm,symbols.inc}; physical offsets include the
16-byte header. Switch-bank6 CPU8000 maps physical18010; fixedC000 maps1C010.
All declarations are pending; tests never use implementation as an oracle.

Read entire prompts/contract.md, workflow/common/coding-standards, architecture,
behaviors/debug API and prior runtime/assets/presentation/world/collision traces,
contracts and interfaces. Planning map/survey/static consumer index locate source;
the independent per-region and full operand cross-reference is
planning/contracts/B10-consumers.json. Source units retain original identities.

CPU reference independently opened: actual manufacturer-authorized MOS reprint,
[Synertek/MOS SY6500/MCS6500 Programming Manual August1976](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
sections2.2.1/2.2.2,4.2.1,7.4–7.9,8.1/8.2/8.5/8.6 and10.1/10.2.
NES NMOS6502 has byte accumulator/index arithmetic. CMP uses unsigned subtraction
carry; SBC subtracts an extra one on clear carry. ASL's final carry is bit6 of
the original selector after two shifts; LDA/TAY/STA preserve that carry. INC/DEC
wrap bytes. RTS adds one to the pulled address; PLA changes N/Z but preserves C.
No CPU execution, generic RAM map or native mapper is introduced.

## Dispatch and identities

UpdateActorHandlers bank6 CPU8000..802B physical18010..1803C lines3305..3326
seeds GravityStep40 and ActorUpdateSlot1. Active is **state sign**, independent
of render. It reads each slot's current handler on arrival, pushes8022 and jumps
through802C/806F split planes. RTS resumes8023; continuation increments the saved
slot, not handler X, until16. Slot0 is updated by B06. Earlier handler mutations
of later state/selector and gravity are visible immediately; lower allocated
slots wait next frame. Handler00 is true RTS at80B2. Selectors00..42 are the entire
67-entry domain, with aliases30/31. They resolve to native family entries, not
CPU addresses in simulation state. Full label/owner table is executable in
original_handler_table... and retained in B10-consumers.json.

Source-defined handler owners: 00 B10;01..07,10..12,18..1C,1E..20,22..23,25,30..34
B07;08..0B,13..17,2C..2D,42 B08;0C..0F,1D,21,24,38..39 B09;26..2B,2E B11;
2F,35,3D..3F,41 B15;36..37 B16;3A..3C,40 B17. Scene handler table B1751D is a
separate selector domain (B16); it is not a second gameplay actor dispatch table.
All handlers remain meaningful even if no ordinary stage template selects them:
child spawning, transformations and scene/final initialization assign selectors.
Every ObjectHandler read/write and dispatch boundary is indexed by source label
and line in the consumer artifact. Families own their caller-specific fields,
allocation fallback, reactions and sound; B10 owns allocation/dispatch primitives.

Both free scans: bank6 80B3..80C0 physical180C3 (Y result preserves source X),
fixedF7BE..F7CB physical1F7CE..1F7DC (X result preserves Y). Visit15 down1; any
nonnegative state qualifies, even renderFF/profile enabled. Full pool returns C1
with chosen index register0; never allocate player. Success C0. They initialize
nothing. Caller-specific policies observed: projectile frame-gate83xx retains
source X and local countdown on failure (B07); secondary89DA saves/restores
sourceX and incoming ADC carry influences RNG (B08/B01); four particlesBD80 stop
batch but always sound16 (B05); boss treasureF702 falls back slot1 (B11); escort
FDxx saves/restores sourceX and retains its committed approach phase on failure
(B15). No blanket initialize/reset policy may replace these differences.

## Suppression

Bank5 B760..B79E physical17770..177AF lines3067..3117 clears32 bytes, then five
stage offsetsB79F choose count/address recordsB7A4, masksB7BC,destinationsB7D4.
The stored count is remaining **minus one**: INX before first test and DEC/BPL
executes count+1. Supported source stages0..4 have1,4,4,3,7 tests; selectors5+
are scene contexts and must not index this five-entry table. Consumers read
source flag bytesA0,A1,A3,A6,A8,AE. OnlyA0 has established StageProgressBits name;
other native fields honestly preserve source identity rather than inventing item
meaning. Every matching nonzero mask ORs a bit; repeated source tests can target
different identities. Destinations include156/157, proving full32-byte domain.
Exact finite records and semantic field mapping appear in executable contract.

FixedF7F0..F809 physical1F800..1F81A reads spawn identity, uses low3 for mask at
F80A and remaining5 for suppression byte. No FF check: identityFF sets byte31
bit7. Native optional identity None encodesFF at this specific source adapter.
Suppress does not remove/deactivate, invalidate identity, change latch or clear
other bits. B08 recovering animation29's8A6B clears its own bit; B15 progression
actors/final marker suppress before local exits; only stage suppression rebuild
clears whole bitmap. Reload does not implicitly rebuild persistence. Build occurs
from InitializeStageRuntime D8xx with bank5 selected; caller stage0..4 retained.

## Main catalog, immutable formats and templates

LoadStageSpawnTables AE20..AE54 physical1AE30..1AE65 lines9845..9870 reads nine
entries from each low/high planeB050/B059,B062/B06B,B074/B07D,B086/B08F,B098/B0A1.
These are Xhigh,Xlow,rowstart,Y,template-index respectively. Source levels0..4
have rows7,8,6,6,6; parallel arrays lengths181,156,126,119,182 including per-row
negative high sentinels. Exact start pointers are documented in planning survey
and independently checked by local ROM-consuming contracts. Each row starts at
its encoded identity; identities remain stage-global raw indices, not renumbered
per row. Decode only records before negative high; sentinel bytes do not become
actors. Full table extents and every record across all rows are tested.
Selectors5..8 point all five planes to singleB0AA=FF: empty definitions, with
special lists empty. No entry9 exists. Layout selectors are ten entries; finale
must use ending-only geometry without indexing main/suppression stage adapters.

121 templates atBE16..BFF9 physical1BE26..1C00A, four fields flags/local,animation,
handler,profile. Selector double-ASL forms low byte offset and carry adds one to
BE high, so indices64..120 cross pages correctly. Signed animation bytes retain
source meaning until ordinary persistence processing. Loaders retain names and
original selectors; extension level count is dynamic and no ninth-level ceiling
is imposed. File-backed definitions share native records/templates, validate
reference bounds and duplicate names transactionally, and join B02 LevelDefinition
spawns by AssetId. Empty/generic suffixes are preserved without fictitious padding
meaning. B0A[A] sentinel has a consumer; AE55..AE5F/AE7D..AE9F uniform portions
and finalBFFA..BFFF zero suffix have no traced behavior consumer. Data overlapping
pointer/table boundaries is not thrown away based on an imprecise region label.

## Reset, both cursor directions and allocation

AE60..AE7C physical1AE70..1AE8D lines9881..9912 starts at rowstart. It stops when
high is negative, equal to camera high **or unsigned less than camera high**;
otherwise increment Y and retry. This is the actual BCC rule, not a generic
ascending lower_bound. Both cursors receive Y, room_start stays original index,
direction1. Wrap-to-zero exits scan by falling through reset. Synthetic fixtures
exercise greater, equal,less and sentinel branches.

AEA0 first calls FindSpecialSpawn; then forms integer interval[camera,camera+255]
with low-byte ADCFF and high carry, wrapping16bits. Direction low2=0 returns;
bit0 chooses forward even when both bits set; otherwise backward. Higher bits
ignored for ordinary direction. Source player horizontal dispatchE593/E5C2,
actual corrected movementE5F0,carriedEBE0,externalEC53 set direction; B06 owns
those choosing policies. World automatic-camera writes are B04.

Forward AEFE..AF28: inspect upper; spawn while entryX<=inclusive high; increment
upper regardless suppression/duplicate/full slot. Stop on outside entry. Then
advance lower while entryX<=camera; equality is consumed. This differs from
candidate's lower-edge rejection(<camera). Exhaustion/wrap uses byte-index BNE,
not an invented retry queue. Backward AEC0: if lower==roomstart, skip candidates;
otherwise inspect lower-1 and stop if X<camera, attempt inclusive equality, then
dec lower. Retreat AEDC decrements upper for previous entriesX>=inclusive high;
equality is removed. Never retreat past roomstart. These asymmetries and exact
camera page carry are executable cases. Arrays are bounded in native decoder;
source sentinel safety follows immutable source data, not arbitrary CPU reads.

Candidate AF2E..AF60 physical1AF3E..1AF71 tests suppression before low bound,
then scans identities15..1. On **first** matching identity, active sign suppresses;
inactive match proceeds immediately without searching lower matching active
slots. No identity match also proceeds. FindFreeActorSlot independently selects
highest inactive; it need not replace the inactive matching slot. Suppression,
below-camera,active duplicate and no-free return without initialization. Cursor
still advances in enclosing scan. Y remains candidate identity through free scan.

Initialize AF61..AFF4 copies identity/worldX/Y and spawnY; clears hit,localcounter,
aux2 and animation frame/tick. It splits low nibble of template flags into local
state; high nibble into both active/render flags. It copies animation,handler,
profile and preserves all unrelated fractions,velocity,direction,motion fields.
If split control!=0 skip persistent preprocessing. Otherwise bit10 state enables
negative animation normalization (AND7F,localcounterFF). Same persistent flag
with handler08/0A calls ReplaceTerrainForActor and RETURNS, bypassing late and
stage-one suppression. That call mutates B04 logical cache and B03 upload in
same frame (B08 owns its choice). It cannot be modeled as a delayed pure event.

If LateStageMode sign set, seed shared bossHP5 **before** checking route/progress.
Nonzero late route or all lowfive progress bits==1F select handler3A,profile0,
state|10 and animation84; render remains its original template high nibble.
Failed progress still leavesHP5. Nonnegative late value skips seeding/override.
Next stage1 plus progressbit2 deactivates handlers14/15,or08 with animation1B;
state alone clears. Render and identity persist until later composition. This
stage gate follows override, so handler3A is not suppressed. All exact combinations
and bypasses are contracted rather than inferred species names.

## Special page spawn and full-pool branches

B0AB..B163 physical1B0BB..1B174 lines10256..10386. Negative late mode returns
before cooldown. Negative special control means occupied latch and returns;
positive1..7F decrements then returns (1 reaches0 without spawning that call).
Zero ready scans. Stage offsetsB164 nine entries index room offsetsB16D (34 bytes);
then page recordsB18F (38),templateB1B5 (38),YB1DB (38), FF terminates. Page sign
requests pair,low7 compare camera-edge high only. First match wins. Original
stage/row restrictions match catalog rows,scene contexts choose final empty list.

Index direction&2 chooses edge offset **00F8** or0008 from interleaved
B201/B202/B203/B204. Forward is plus248, not signed minus8; page carry matters.
Direction3 takes backward edge while ordinary scan uses forward priority.
Single calls fixed free scan preserving recordY; failure returns without writes.
Template selected with ASL carry exactly like ordinary, but copies **whole first
byte** to state/render, clears localstate/counter/hit/frame/tick, does NOT reset
aux2 or spawnY, and does not apply ordinary persistence/progression exceptions.
It copies edgeX/recordY and stores spawnFF (nativeNone),special latch80|slot.

Pair preserves record cursor inScratch07, calls first CreateSpecialSpawn,
**does not test its carry**, writes first selected ObjectScreenY=C0, restores
recordY,toggles edge low byte XORF0 (high unchanged), then falls into second
allocation. Two free:15 then14,final latch14. One free: first actor succeeds,
second fails,first latch retained. None free: allocator leavesX0,so caller writes
**playerY=C0** despite no actor spawned,then second allocation fails. This is
source-observable failure behavior, not an assertion fixture bug. Contracts cover
all outcomes and preserve unmodified nonplayer slots. No extra error policy
sanitizes this source branch.

## Removal, composition and cross-area ordering

FixedF179..F191 physical1F189..1F1A2: bounds removal loads0,clears render, falls
through release. DirectF17E entry stores callerA to state (not guaranteed zero),
sets spawnFF, compares specialcontrol with80|slot and sets3C only on exact match.
It does not clear velocities/local phases/profile or suppression. B05 movement
wrappers already contract same-frame removal; no duplicate actor arrays. B03
ComposeObjects also clears inactive render/identity and may deactivate offpage
actors, before ordinary AEA0 spawn scan (DBC5 caller lines14371..14390). Thus
inactive stale identities become available same frame; composition removal does
not automatically invoke F17E's matching latch cleanup. B03 owns that distinction.

ClearNonplayerActors F2C8..F312 is B05-owned: ordinary clears slots15..2, preserving
0/1,despite misleading source comment1..14. Stage1,row<2 marks only14/15 bit10
and adjusts their encoded local timers (subtract3C then extra10 on borrow,
propagate clear carry); no ordinary slot clearing in that path. Existing69 B05
contracts remain authoritative and borrow this canonical pool. Initial runtime
D88x explicitly prepares special slot1,state/render/spawnFF; it is not a normal
free actor. B13 owns initialization/reloads: clears control and late mode, restores
stage/row after temporary backgrounds before AE60; callsAEA0 after fade/audio.
Vertical transitionC9FB finishes sixty source frames then AE60/AEA0 while split
control still set; only afterward clears split control and requests palette.
Special-roomDF27 selects actual initialization context before these calls;
endingE1xx bypasses ordinary spawns (B16/B17). No absent stage9 entry is invented.

## Contact, motion and distances

Bank6 80C1..8149 physical180D1..1815A is **not** B05 fixed solid contact scan.
Source render must be negative; no state-active or profile-enable test. Mask its
profile7F and use its own extent heights814A,widths816A (32 each). Scan targets15
down1 skipping source and nonnegative render; target state/profile ignored.
Y separation is unsigned byte absolute,strictly<height. Full worldX subtraction
uses unsigned16bit borrow to choose low-byte magnitude; high distance is never
rejected,allowing page aliases. X equality at width succeeds. C0 success returns
source X and saved target Scratch11; C1 failure restores sourceX. Native Option
contact carries target identity without scratch aliases. Targets08,09,25 do not
increment hit and select source animation2A; others INC byte marker (wrap) and
select2B. Animation1B is preserved with old clocks; otherwise reset frame/tick0.
B08 callers876x/877x/879x/87Dx/881x inspect carry and saved target,may transfer
terrain state or remove source immediately; B07 A065 contact caller owns reaction.
Extent selector supported0..31 is proven through source template/caller profiles;
source high-bit mask does not justify128-entry invented data. B05 extents atF498
are different data and must remain separate.

818A..81A4 physical1819A..181B5 doubles preset index0..31 and copies signed16bit
vertical table81A5 and unsigned magnitude81E5, each32 words.
No sign normalization here; handlers may do that afterward. All32 words,including
page/negative values,are source-decoded and per-selector copy contracts. Family
callers select finite table presets; no actor-selector aliasing.

FixedF6BD..F6D6 physical1F6CD..1F6E7 facing stores1 then unsigned16bit actor-player
subtraction,no signed interpretation; borrow leaves1,else2 including equality.
F7CC Ydistance is unsigned8bit absolute (0 vsFF=255,not shortest wrap). F7DA X
distance uses full unsigned borrow for sign but returns only low magnitude,with
same page aliases as bank6 overlap. Contracts cover page crossing,equals,zero,
FF and distant alias behavior. FixedF6D7..F6DF five initial boss-hit instructions
are B10 planning-owned metadata fragments but semantically B11 shared boss-hit
entry: clear carry,zero source06F0 (`boss_protection`, NOT existing06D0
`local_aux2`) gatesHPdecrement,zeroHPbranches treasure. B11 independently
corrected ordinary-spawn test:06F0 clears,06D0/06E0 remain stale; prior
ambiguous aux2 wording/assertion is superseded without source/ledger rewrite. B11 must
contract full helper including timer/treasure/final branch; do not duplicate or
implement these fragments separately merely because ledger primary saysB10.

## Contract limits and handoff

All expectations derive from above instructions or direct local ROM format reads.
No implementation,CPU emulator,GUI,QA,toolkit edit,ledger disposition,source edit,
check-in or publication. Native error validation for authored definitions reuses
B02's documented transactional extension policy; this is distinct from source
execution quirks. No new cohesive behavior area found. Unknown flagA1/A3/A6/A8/AE
semantics stay explicit source identities and require later family/progression
names if established; observable mask/destination behavior is fully specified.

## B15 reconciliation (supersedes full-entry-only interpretation)

Dialogue ED49 actualJSR B76A begins AFTER B760..B76932byte clear. New pending
add_suppression applies identical decoded rules to existing canonical bitmap;
rebuild_suppression retains its full-clear entry contract. Two new executable
actors::contracts::dialogue_additive_* cases use all32preexisting bytes and high
identityFF addition plus disabled rules. Source trace and exact results
planning/contracts/B15.md, checks/contracts/B15/results.json. Prior58cases/evidence
remain; currentB10count60. No behavior implemented. SourceA1/A3/A6/A8 projections
map canonical B13 fields established in B15; lateAE stays source-provisional.
