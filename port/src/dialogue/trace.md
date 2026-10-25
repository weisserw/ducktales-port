# B15 independent source trace (active)

Stage3 pending declarations/contracts, not implementation. Frozen game.asm SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
The matching local ROM is97d472f7b2c1088448d1644155b2650a546bad887b3cba571e16d7986824c169.
All physical offsets include16byte header. Fixed CPU C000=physical1C010;
bank4 CPU8000=10010, bank5=14010, bank6=18010. Complete contract/common/workflow/
coding requirements and architecture/behaviors/debug schema consulted. Prior
runtime/presentation/world/collision/player/actor families/lifecycle/status/selection/
progression contracts are dependencies, never a substitute for rereading source.

Independently opened actual MOS-authorized SY6500/MCS6500 Programming Manual,
Synertek publication6500-50 August1976:
https://syncopate.us/books/Synertek6502ProgrammingManual.html . Sections2.2.1/2.2.2,
4.2.1,7.4..7.9,8.5/8.6/8.11/8.12,10.1/10.7/10.8 establish binary byte ADC/SBC,
CMP unsigned carry, INC/DEC wrapping and stack preservation. PHP/PLP preserves
borrow across the blank-digit CMP in paid outcome. PLA changes N/Z but not carry;
LDA/STA/AND/ORA preserve carry. No native CPU flags/stack/RAM image is introduced.

## Exact original data and row processing

Bank4 B888/B89A split18-byte low/high pointer planes are physical13898..138BC.
Messages occupy physical138BC..13C59 and each ends AE. Actual original starts
and lengths, checked against ROM table and terminators independently:

| selector | CPU | physical | bytes | delimiter-separated content lengths |
|---|---|---|---|---|
|00|B8AC|138BC|45|16,10,8,7|
|01|B8D9|138E9|47|5,13,10,15|
|02|B908|13918|66|12,8,16,12,13|
|03|B94A|1395A|50|13,14,7,12|
|04|B97C|1398C|29|14,13|
|05|B999|139A9|44|15,11,15|
|06|B9C5|139D5|46|12,5,10,15|
|07|B9F3|13A03|61|10,13,8,11,14|
|08|BA30|13A40|54|8,9,14,14,4|
|09|BA66|13A76|56|13,15,14,10|
|0A|BA9E|13AAE|51|14,12,5,8,7|
|0B|BAD1|13AE1|100|12,14,16,15,14,14,8|
|0C|BB35|13B45|76|15,16,10,12,12,5|
|0D|BB81|13B91|23|13,8|
|0E|BB98|13BA8|42|7,8,12,11|
|0F|BBC2|13BD2|60|15,16,9,8,7|
|10|BBFE|13C0E|24|12,10|
|11|BC16|13C26|51|8,12,14,13|

Fixed EF59..EF61 physical1EF69..1EF72 three parallel arrays are destinations
low[4C,6C,8C], raw queue ends[14,28,3C], raw queue starts[00,14,28]. Each row
has3-byte header,17glyph payload and shared next-row header/last FF terminator.
Native Panel represents prepared rows/encoded submission only, never committed
video. B03 alone commits pending commands on the B01 interrupt boundary.

EEAE initializes selected row header high20 and low from table; row count starts
FF. Literal glyph writes payload then increments queue position/count/cursor.
Blank24 skips sound/wait AND skips row-limit CMP; it is not a paced glyph.
A visible glyph making count10hex (=17glyphs) enters EF08 BEFORE sound/wait.
Ordinary visible glyph queues sound2E, writes FF terminator, submits current
count (payload-length minus1), and waits exactly3 scheduler yields. This is
source cadence, not wall clock. Resumption uses stored queue position.
FF consumes text cursor, advances raw offset by3, pads with24 until raw row end,
counts padding, writes terminator/submits. Line<2 advances and renders next row
without a page-input wait; line2 returns with nonzero cursor. AE does not advance
text cursor: writes literal tile2E (period), FF afterward, count+1 with CLC (empty
row FF+1 wraps0), submits and zeroes canonical cursor. This terminal period has
no audio or glyph wait. Every original row has<=16 content glyphs; synthetic17th
visible case contracts genuine boundary behavior without claiming original usage.
Malformed overlong blank runs are outside decoded original string evidence;
no invented automatic wrap rule is authorized by the source blank branch.

EE8B scroll copies17 positions of rows1→0 and2→1 then blanks row2; existing row
headers/counts remain, pendingFF, yield BEFORE line2/queue28 setup and rendering.
Original FF-page input returns through ED66, so after scroll/render another yield
occurs before waiting for a fresh acknowledgment. One input edge never closes two
pages.

## Full dialogue continuation and cross-call effects

Actual ED05 physical1ED15 checks message0B and calls B17 BuildFinalRaceTerrain
first. Then bank2, prepare16byte panel record at PPU12D0 from CPUA8F0
(physicalA900), count0F, FF terminator and pendingFF; YieldOneFrame. Only after
resume zero cursor/queue/line. Bank5 HideOamSprites, ComposeObjectFlags (includes
composition/animation/culling and clock), Bank05_UpdateStatusStreams (may yield
per row), then JSR B76A. IMPORTANT: B76A skips BuildSpawnSuppression's32-byte
clear at B760..B769. It ORs the selected stage rules into existing suppression.
Never replace this with full rebuild. Then bank4 load actual split pointer/render.
No actor mutation/queue work can be moved before its preceding yielding owner.

ED60 checks fresh primary ControllerPressed ANDC0; held alone, port2 and direction
edges do not acknowledge. Cursor nonzero→scroll→ED66yield→wait. Cursorzero→save
FULL primary pressed byte, set playeraction0 before bank5 status update (ordinary
stream now), select bank1, restore16byte panel at12D0 from CPU82D0 physical42E0,
submit/yield. After resume restore saved full pressed byte to canonical B01 input,
leaving freshly sampled held intact, then outcome. B13's suspended B06 actionF
must not run actor/countdown/camera/scene tail before this whole owner returns.
Cursor is reread at actual source boundaries; it is not a blanket freeze flag.

## Every message outcome and ordering

EDAD compares0A,0C,06,0F,03,0B in that order. Other selectors return unchanged.
0B clears RaceIntroLatch even on B. 0C JMP EBB4 is a real nonlocal common
life-loss body: reload=(stage+87 with CLC)+(variant plus first carry), action0,
queuedaction0, slot0state0, alternatingdamage0; B does not reject this outcome.
No state or queue default is invented. Source stageFF/variant1 gives request88:
FF+87=186hex→86 C1;86+1+1=88. B06 owns the common native operation; B15 must
invoke that body at this exact continuation rather than repeat a divergent rule.

06 uses saved actor slot (not whichever X particles leave): stateA0, handler08,
animation1B, screenYB8, lowworldX30, frame/tick/hit0, localstate0B. HighworldX,
fractions, render/profile and other locals are NOT initialized. Collision cache
source05A1/05A9 (native indices21/29 relative to TerrainCollisionCache0580) clear immediately; attributes
and unrelated collision cells stay intact. Then tail-call burst descriptor0.
Burst D061..D10F constructs canonical primary records from terrain bank; particle
origin and bank4 tail-call BD80 may allocate up to4 actors and always sound16,
even full pool. B04/B05 own burst/particles, B10 allocation. Clearing cache occurs
BEFORE prepared upload and before later live slots; no deferred collision effect.
No increment of B04 changed-cell counter is present in this caller.

0F tests ControllerPressed&B40, so B wins simultaneousA+B. On accept OR the
current stage's StageCompletionMasks[D8A6+stage] into canonical sourceA6 and set
SpecialRoomRequest81. Original mask table has8 entries, not a stage-number mask.
03 ignores B: sourceA3 bit02 clear increments sourceA4 modulo256. This may change
7F→80 or FF→00; there is no saturation or proved positive-only invariant. B13
boundary and B12 raw-selection uncertainty must retain these producers.

0A also rejects B first. B14 pay(RewardDialogue) owns the upper TWO reward-digit
subtraction, blank24-aszero, borrow propagation, transactional reject/commit,
normalization and dirty increment. PHP/PLP preserves incoming borrow across CMP24;
underflow cannot partially change persistent digits. Reward03xxxx exact payment
zeroes upper2 but leaves lower4 intact, normalizing leadingzeroes until a nonzero
or all6 exhausted. Total7 digits unaffected. Only success changes savedactor:
handler0F; state AND E0 (not replacementA0); lowX80/screenY80; profile OR80.
No animation/frame/tick, coordinate fractions/highpage or unrelated actor reset.
B14 source trace/payment contracts and B15 full actor-after-success contracts
jointly enforce atomicity; no mock payment bool decides production result.

## Actual actor table targets and all six bodies

Checked actual bank6 table low802C/high806F using matching ROM (physical1803C/
1807F). Targets and first instruction bytes:
2F→9065 `20 36 F4` (JSR overlap), 35→FDF5 `A5 39`,3D→FBFF `BD 00 07`,
3E→FCA0 `BD 00 07`,3F→FD68 `BD 00 07`,41→FE50 `A5 A3`.
These are entry addresses, distinct from coarse region starts. Fixed entries
map physical1FC0F/1FCB0/1FD78/1FE05/1FE60, bank6 9065 maps19075.

2F/9065..90A4: B05 TestPlayerActorOverlap first. C1 exits (including rejected
reaction-mode contacts), not merely geometric miss. Localstate0 requires A8bit08,
then request80. Nonzero selector except4 OR80→reloadrequest, preserving whole byte.
Selector4 tests progress&1F==1F; success reload84, failure boundaryA4=80 and retains
existing reload. No actor removal, suppression or action rewrite in this body.
B05 itself can change playeraction or hitmarker before its returned permission.

35/FDF5..FE4F: early playeraction0D or06 rejects before overlap. Any state low
nibble nonzero rejects. Otherwise B05 overlap; rejected permission returns with
callee mutations intact. Animation33 adds current-stage A6mask gate; other
animations do not. Accepted: playeraction0F, INC actorstate, localstate→message,
source slot→savedactor. Message08: INCA1 modulo256, state0, suppress original
identity; ordinary message retains active actor and is not suppressed. ALL messages
OR bit[message&7] into sourceA8+(message>>3). For original0..17 that includes
three bytesA8/A9/AA, not just one flag. Original onebit table indexed0..7 is
D8A6 (same values as F80A). Writes do not check prior seen bit: used-localphase
is the immediate sameactor gate. No protection or generic special-mode gate is
added beyond the actual B05 result and explicit action tests.

3D/FBFF..FC9F: lowphase0 without hit→tick0 only. First hit INCA2 (wrap), velocity0,
localcounter0, profile80, INCstate. Continue same invocation: if statebit02 clear,
FallWithCollision samples2D/remain1. Only resultbit01 initializes speed0100,
velocity054B and INCstate; no rise/left motion yet. Any nonground result→tick0.
If statebit02 set, localcounter0→RiseWithCollision2F/remain1. Inspect actual
post-callee velocity integer sign, not pre-call value: negative increments local
counter and clears full velocity, nonnegative includingzero retains rise phase.
Nonzero counter→Fall2D/remain1; groundbit01 clears handler, action0F, ORA8bit40,
localstate→message, saves source slot, suppresses identity. Landing still reaches
MoveNegativeX2B/remain1, then tick0; it does not return immediately after dialogue.
Motion may itself apply B05 reactions, terrain work and integration. Handler0 is
B10 actual RTS; no invented actor deactivation here. Existing hitmarker is not
cleared by first-hit initialization.

3E/FCA0..FD67: phase0 measures B05 low-byte nearby magnitude first. Distance>=38
waits regardless of prerequisite; smaller requires A1nonzero (negative also
qualifies). ORA1bit80, suppress identity, action0E, copy full cameraX/page to
actor integer position while retaining fraction/Y, clear statebit10, INCstate,
speed0180, profile0, RETURN (no walk/allocation yet). Stage4 B04 stage4 room27
terrain substitutions consume negativeA1; A2 nonzero enables stage3 room28
substitutions. Their previous contracts remain source-defined.

Approach statebit02 clear: IntegratePositiveX, render OR40, then measure distance.
At>=38 return AFTER motion/flip. Close: INCstate, localstateB4, save source slot,
FindFreeActorSlot fixedF7BE scans15→1 and returns X (not bank6 Y-scan). Fullpool
still retains newphase/pause and player's hold; do not invent retry or control
recovery. Success initializes ONLY selected slot: animation7E, source integerY/
fullworldX (after integration), frame/tick0,state/renderA0,handler3F. Existing
fractions/velocity/profile/locals/identity/direction survive because allocator
clears nothing. Restore source slot regardless of success; later live slots may
execute newly initialized child sameframe, earlier slots wait until nextframe.

Statebit02 set: nonzero localstate decremented once and returns, including1→0.
Next invocation withzero: render nonnegative→state0 only. Visible→Fall sample06/
remain2; if groundbit01 clear return, else IntegrateNegativeX then renderAND BF.
No generic reaction/removal helper or animation freeze occurs here.

3F/FD68..FD9D: state lowphase0 resets speed0300; no phaseincrement. Every call
IntegratePositiveX first. Then require EXACT worldpage07 AND lowX>=70. Page08
never qualifies even if globally further right. At threshold state0, action0,
save source slot across burst descriptor06, SelectPrgBank6 and restore source
slot. The burst's bank4/particle allocation scratch effects cannot select a new
actor for caller continuation. Other fields retained. The generic source actor
dispatch increments saved iteration slot independently (B10), never this finalX.

41/FE50..FE68: uses bitwise (A3 & StageIndex), NOT StageCompletionMasks[stage].
Zero returns. Nonzero clears actorprofile BEFORE B05 overlap. Qualified contact
ORs player renderbit04 preserving others; rejected contact leaves profile0 and
callee effects intact. No player Damage/Death pre-gate or actor localphase gate.

## Cross-owner closure, flag return facts and explicit limits

Canonical Progression now owns sourceA1 escort prerequisite,A2 firsthitcount,
A3 rewardprogress,A6 acceptedstagebits,A8..AA seenmessageplanes. Existing A4 is
boundaryrequest,A7 impactlatch; completionactor is5E, not A7. Dialogue owns
message/savedactor/line/continuation/acknowledgment only; existing Progression
owns source6C cursor. B01 owns sampled/game-written input; B06 owns action/timers.
B03 owns PreparedPrimary encoded bytes and exact pending u8; Panel is a reexport,
not a retained second row/buffer store. rows() is read-only observation from bytes.
Display owns this queue, and interrupt commit consumes it. Existing frame-level
bool views in B01/B13 are call-boundary projections, never another pending owner.

All named/direct/numeric entry callers, source fields and assigned regions are
indexed in planning/contracts/B15-consumers.json; full148 primary regions include
fine-grained splits of the renderer and two adjacent genericdata ranges. The
uniform ranges13C59..13DD0 after the final AE are not read by any valid original
18message pointer and are not invented executable messages. Original decoded
text uses glyphs only under its proven row bounds; native validation must not
mistake unreferenced gap for a19th message. All ledger identities stay pending.

Adjacent producers: B08 rewarddialogue8BE5 uses stage4 comparison with A=4,
so OR4 intoA3 and message10; others OR2/message0E. Treasure8C03 clears impactA7,
sets progress bit from animation6F and chooses conversion10 or message0D.
B06 E87C late-route bottomY>=C0 and B17 goal-loss FDBF choose message0C; B17
intro0B builds terrain then clears intro only at completed message outcome.
B12/B13 choose stage/requests; B14 status stream has special player0F branch
and can yield before text. B10 suppression consumes A1/A3/A6/A8 and lateAE, not
all seenplanes. B03 overlay gating reads cursor separately from action0F; B13
rereads cursor after actual B06 return. Status-selector, scene/ending and cursor
are specific gates, not a global freeze. B18 QueueAudioCommand may drop a sound;
requested versus committed audio remains distinct, no queue copy here.

Existing B11-first-slot-random-increment/B06→B13→B10 integration question remains
explicit. B15's actual final source arithmetic provenance is now traced rather
than assuming false: ordinary messages after final CMP0B have C0 below0B and
C1 for0D/0E/10/11; race0B returns C1; boundary03 and stagechoice0F retain their
successful dispatch comparison C1. Paid0A B-reject returnsC1, underfunded returnsC0;
success normalization retainsC1 when first upperdigit nonzero or all6 zeroes
exhaust CPX6, otherwise C0 after a partial leadingzero scan. Life-loss outcome
returns final secondADC carry. Reveal06 tail-calls burst/particles/audio; final
QueueAudioCommand compares actual producer slot with88, so C1 for88 oroccupiedFF,
C0 foroccupied<88. Producer must be sampled/returned from THAT audio call before
ring advancement, not read afterward. These are algorithm result facts for the
native PlayerReturn.first_actor_increment adapter, never a global CPU flag store.
Current pending B15 methods expose outcome work but stage4 composed B06 adapter
must retain this source return fact (including after nested yields) before B10
slot1's random sum. No test currently claims the whole composed provenance closed;
existing uncertainty and its executable integration cases remain preserved.

No new behavior area discovered. No new emulator question needed: source/table/
consumer evidence resolves this area's declared policies. No game behavior,
GUI/emulator/QA/audit/toolkit/check-in/publication performed.

Stage4 canonical borrow refinement: Systems.motion receives Context.cache at
call time, as the owner context already borrows that WorldCache. This avoids a
second cache or a retained alias in the native adapter. All102 current contracts
pass unchanged expectations; B15-before-cache-borrow preserves prior signatures.

## Modernization016: decoded native message programs

Catalog owns immutable MessageProgram instructions: Glyph(tile), Blank, NextRow,
and Finish. Original split pointers and AE/FF/24 are decoded once at import;
there is no retained raw message twin or runtime control-byte interpretation.
Tile identities remain assets. Native operations retain one-to-one source cursor
positions because Progression.dialogue_cursor is shared with input/overlay/call
boundaries; no second cursor or cached rendered rows are introduced.

Independent source reread EEAE..EF58 confirms: blanks consume a position but
bypass both sound/wait and row-limit comparison; visible glyph17 finishes its
row before pacing; NextRow consumes a position then pads; Finish appends tile2E
without pacing and resets cursor0. Renderer still writes the single B03 prepared
panel and publishes at the same owner boundaries. Outcome/scroll/input/scheduler
logic is unchanged. Shared encoded panel storage remains a presentation/audio
alias boundary, not the message program representation.

Two pre-conversion source-backed regressions exercise the seventeenth blank and
all253 noncontrol tile IDs including high values. Native program tests distinguish
instructions and exclude adjacent data after the first terminator. Source
metadata assertions project codes only in cfg(test); all behavioral expectations
and original18 sound/scroll counts remain unchanged.

modernize/dialogue-probe.{rs,py} runs all18 original messages through the actual
renderer, prepared panel commit and shared tile sampler. Starting evidence records
807 events and42 complete page images. The probe uses common original CHR list0
and a fixed review palette; it is an explicit direct-owner fixture, not a natural
game route, full scheduler or original-emulator run. Full native continuation and
Game integration contracts separately retain nested waits, acknowledgment and
canonical-panel sharing coverage.
