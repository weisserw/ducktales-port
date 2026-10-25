# B08 independent source trace and contract evidence

Frozen reference/source/game.asm SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reference/reading.asm201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Bank6 CPU8000 corresponds physical18010 (header included); fixedC000 maps1C010.
Independent instructions, symbols, local ROM tables and all direct callers were
read. Planning records identify preserved ledger regions, not behavioral truth.
All signatures remain unimplemented; synthetic boundary facts do not emulate CPU.

Manufacturer documentation consulted: MOS-authorized Synertek6500-50 August1976
[SY6500/MCS6500 programming manual](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
2.2.1/2.2.2,4.2.1,7.4..7.9,8.5/8.6,10.1/10.2/10.7/10.8. Binary byte addition
consumes carry; SBC subtracts extra one on borrow; CMP/CPY set carry on equality
or greater. Loads/stores/AND/ORA/EOR/INC/DEC preserve carry; INC/DEC wrap. ADC and
SBC do not infer signedness. Native values preserve exact outcomes and ordering.

## Canonical ownership and immediate boundaries

ActorPool is B10's sole16-slot state. Player is B06, Status/Health B14, RandomState
B01, suppression B10, mutation/cache/change counter B04, queues/palettes/animation
advance B03 and audio queue B18. Context borrows these owners; progression fields
are a temporary B13 writeback view, not retained parallel state. Slot0 actor owns
player animation/coordinates. World calls commit B05 motion/player interaction,
B10 actor overlap and B04 terrain work before returning. B05 geometric contact,
permission and hit-marker side effects are distinct; player contact can change
Player.action immediately even when the caller overrides it next. Effects record
sound/palette/reward/allocation observations, not a deferred mutation queue.
B10 live ascending dispatch sees newly allocated higher slots this frame, lower
slots next frame. Composition advances animations later, never during rendering.

## Complete handlers and local dispatch

08=8668;09=8722;0A=8845;0B=897B;13=8A59;14=8C45;15=8D19;
16=8DD1;17=8E23;2C=A1E9;2D=A2BA;42=8EE6. Source split motion
88FC/8905 contains9entries:0/3/4/5/7=8735 Arc;1=876D FallThenSlide;
2=8792 ContactTransfer;6=87D0 SlidePose;8=87FF TouchPose. No ninth distinct body.

08 offscreen clears state except animations1D/1E which still interact. Swing
requires player tick0, animation09, overlap carry clear, frame3. Animation3F
jumps special reaction BEFORE changed-frame gate. Other swings return immediately
when TerrainChangedThisFrame nonzero, without hit fallback. First activation INC
changed, clears sourcebit10 and calls B04 replacement only if bit10 was set.
Replacement increments changed AGAIN and may allocate recovery before mutating
terrain even if recovery allocation fails. Hence first claimed swing with terrain
form changes counter0->2; without terrain form0->1.86B5 bypass entry skips claim/
replacement and sets direction from Player.pose_direction, lowphase=animation-1A,
retains high state, loads four-byte preset890E, clears motionflags, INChandler.
Hit fallback checks hitmarker first; bit10+changed nonzero defers; otherwise B04
replacement then clearbit10.1B/3F suppress identity and specialized reward;22
ignores reaction; ordinary88DF sets2A then88E4 clears handler/profile, render high
nibble|08, sound16. Direct callers88E4 deliberately SKIP animation store, retaining
B10 contact-selected2A/2B or preserved1B. No automatic removal/suppression here.

09 Arc: while motionflags0 Rise2F/count1; negative velocity resets vertical and
sets motionflags but no same-call Fall. Otherwise Fall2D/count1 stores full bits
separately, then directionbit0 chooses positive29/negative2B count1. ORsaved/final
bit0 removes. If both clear B10 actor overlap; contact jumps88E4, nohit returns.
FallThenSlide: Fall; CLEARbit0 sets motionflags1 and scans actor overlap without
horizontal. SETbit0+oldmotion nonzero special reaction; SETbit0+oldmotion0 probes
horizontal, wallbit0 special reaction, otherwise scans actor contact and returns.
This branch direction contradicts broad source prose about landing. ContactTransfer
uses NegativeY2F/count1 (not Fall) then horizontal; floor/wall removes before scan.
Contact handler08+animation1B invokes86B5 on TARGET immediately, with targetbit10
retained and no frame claim/mutation; then source ordinary removal88DF. Other
contact jumps88E4. SlidePose installs20 before Fall; follows FallThenSlide latch
policy but terminal removal instead of special reaction; contact88E4. TouchPose
Fall clear sets motionflag and settles; Fallset+oldmotion0 probes horizontal then
actor contact if still flag0. Contact changes source22; target08 invokes86F7,
which begins targetbit10 gate, no target hitmarker test. Without target handling,
settle tests FINAL Scratch10 bit0; set restores handler08, clear retains09.
B10 overlap changes source animation/clocks and targethit except08/09/25; fixtures
may supply those already committed results, never substitute a second detector.

0A visible only; phase0 checks changed==0 then low-byte world distance strict<4,
animation87 strict<20. Clears vertical, INCstate.87 skips terrain replacement,
render rewrite/profile. Others clearbit10, render=remaining high state, call B04
replacement and profile10 only when old actorY<playerY. Active checks player first;
contact removes, else Fall2D/count0; floorbit0 removes. Offscreen state0 only.

## Shared reaction8932..8A10 and incoming carry

Reaction offscreen clears state AND render only: no sound/RNG/handler rewrite.
Visible queues35, then secondary, tick80/frame0/profile0/statehigh, handler0B,
direction1 if unsigned16 actorX>=playerX (equal1), else2, and launched body SAME
CALL. Generic FaceActorTowardPlayer has opposite equality direction2; don't reuse
without adapting. Lowphase0 launch clears motionflags, INCwhole state, vertical
02D4 and horizontal0080. Rising IntegrateFallingY then Gravity; ignore initial Y
boundary return. Negative resulting integer sets motionflags and clears velocity.
Falling Gravity then IntegrateFallingY and Cset removes. Horizontal positive Cset
removes; negative Cclear removes. Removal clears state only, preserving render.

Secondary89DA has sole direct caller8945 after QueueAudioCommand. It updates
RandomState1=(RandomState0+oldRandomState1+incoming carry) mod256, tests low2bits0,
then handler9025 mapping nonzero, highest free slot, reset8CAC and integer source
X/Y copy only. Fractions, render, spawnidentity, direction and other stale fields
are retained unless8CAC resets them. Full pool/zero mapping/random rejection still
advance RNG; sourceX restored at8A0E even full. QueueAudioCommandF757 compares
producer-byte to sentinel88 (symbols.inc), notFF. Empty88 accepts35 and C1;
occupied<88 drops35 and C0; occupiedFF control command drops35 but C1. B08 tests
all three exact RNG results. B07 earlier sentinelFF prose is explicitly superseded;
its caller-local RNG tests derive CMP/LSR before audio and have no affected expected
value. Context's audio command is a lossless immediate B18-owned slot observation;
implementation must obtain it at THIS queue request, fresh for multiple calls.
No owned queue copy or CPU flag state is introduced.

All named/numeric direct callers of8932 were indexed, including B07 and B11
hit/visibility prefixes and unconditional retry3C writes. They select actual
handlers01..07,0D/10..12,18..1C,1E..20,22/23/25..2B,30..33, not40..42.
Direct89DA caller is only8940 reaction; B10 dispatch target40=FD9E,41=FE50,
42=8EE6 contain no reaction or secondary call (full bodies independently checked).
Their ordinary continuations do not tail-call another family before returning.
This is direct source-consumer evidence, not an emulator reachability claim.
Named source table9025..9064 has64bytes. Bytes9065..9067 are following B15 entry
instructions. They are not silently treated as67 ordinary table bytes. If future
source evidence adds a reaction caller retaining40..42, extend through a NAMED
bounded adjacent-instruction ROM data consumer exactly as B05's preset1; never
clamp/index-wrap these selectors or omit their newly evidenced behavior.

## Specialized rewards/recoveries8A11..8D18

Special first suppresses spawnidentity. Animation3F chooses stage*3+local map9016;
zero result falls back8FB8[local]. Others direct fallback. Save reward animation in
localstate; state&=A0, profile/tick/frame0, vertical0400, handler13, animation2A.
Initial13 offscreen animation29 unsuppresses exactly its identity bit; all offscreen
cases state0 only. Visible2A needs frame!=0 AND tick5, then restores local animation,
applies option2 remap25->28/26->24, resets clocks and continues SAME call. Phase0
integrates then gravity; nonnegative returns before collection. Apex INCstate,
vertical0 then same-call Fall2D/count1. Floorbit0 INCstate, localFF then contact
without decrementFF. Bit1 set countdown applies ONLY animation<29 or61. Old0
removes before contact; old78 ORrender02 then DEC; other values DEC.29/treasures
have no expiration. Contact player action6 returns, action0D removes, otherwise
actual B05 overlap may collect. These gates precede no invented protection filter.

14 offscreen state0. Phase0 requires protection==0 and overlap; suppress identity,
INCstate, counter1E then falls into countdown in same call ->1D. Phasebit1clear DEC
counter (0 wrapsFF), expiry clearsbit10 then stops vertical, saves animation,
sets2A, INCstate without resetting existing frame/tick. Phasebit1set restore2A
only when frame!=0/tick5; non2A goes directly special initialization.8CAC external
entry stores suppliedA as animation before option rule.8CAF option remap,
clocks/profile0.1B/22 handler08/profile80, clears localstate/counter/motion; late
mode local=stage+6, exact0A prepares base+active final palette pair0F/29 and pending
upload29. Other animation handler13,stateA1,vertical/local/counter0; motionflags,
render, spawnidentity and position fractions remain stale. B14 owns reward math.

## Transform and sliding8D19..8EDD

15 offscreen state0 before profile write; visible profile81. Swing tick0/09/frame3
and overlap triggers suppression, indexstage*4+local, actorY-=offset9002 modulo256.
Handler tableZERO stores local28 and jumps8A32 (preserves saved28). Nonzero mapping
clears profile/clocks/hit, state&E0; index6 directly handler17/profile10/anim21;
other indexes anim2A and INCcurrenthandler, NOT mappedhandler. Stage1 fallback
local>=2 clearsstate&E0; unless playeranim09 selects5B/profile82, unsigned byte
absoluteY>=18 increments83. SBCborrow/EORFF/ADC01 yields exact magnitude for
negative differences, no signed clamp.16 offscreen state0; reveal frame!=0/tick5,
stage*4+local, index7 CPY7 leaves carry1 so ADC17 shiftsY by18! Then three tables
supply animation/handler/profile, clocks/hit0; index4 alone sound30.

17 initializes lowphase0 vertical/fractionX/local0,speed0200,facesplayer,INCstate.
While local!=8, quarter CompositionTick&3==0 adds signed16 nudge sequence, INC local;
returns every call even reaching8. Next call Fall2D/count1 saves FULL bits in local
counter; floorbit0 and speed!=0 subtract0010 modulo16, no clamp. Statebit1 skips
horizontal/contact. Else directionbit1 chooses negative/positive horizontal count1;
positive JMP8EBB joins TestPlayerActorOverlap just like negative (source indentation
can mislead). BOTH call player contact before speed/wall tests. Stopped OR wall
bit0, AND savedfloorbit0, restores handler08/profile80, otherwise retains17.

## Swing-released collectible8EE6..8FB7

42 phase0 swing09/frame3 overlap uses profile81 but DOES NOT require tick0.
Success INCstate, anim2A/clocks/profile0,Y-=18 modulo256 then return. Failure
profile83, swing09 and unsigned(actorY-playerY)>=18 ->84; no absolute conversion.
Phase!=0 animation25/28 skips reveal initialization entirely. Others wait frame
nonzero/tick5 then anim25+option remap, clocks/local/vertical0,speed0120,face and
return. Update Fall2D/count1 then contact; collection wins before floor/latch/X.
No floor increments localstate (can wrap); floor with local0 does not increment;
floor with localnonzero stateA1 then JMP8ADB reuses saved floorbit, INCstateA2,
localFF, contacts again. Handler42 is NOT rewritten13 on this path, so subsequent
calls still use42 animation25/28 route. Horizontal directionbit0 count0 last.
Planning/source comment claiming first contact latches is superseded by branch.

## DispenserA1E9..A334

2C phase0 low-byte world distance strict<4 activates and clears local/counter.
Counter0 attempts highest15..1 inactive allocation; full returns before sound,
animation, sequence/counter change (next frame retries). Success sound25, INCsource
animation, childstate/renderA0,animation24,integerX copied,Ysource-10, clocks source
AND child0, profile0,handler2D,vertical02D4, paired direction/speed arraysA2AE.
Does not initialize childfraction/identity/local/motion beyond listed writes.
INCsequence; fourth setsFF, other3 set3C. Positive oldcounter DEC then animation
restoration, never allocates on old1 becoming0. Negative also restoration. If
sourceanim!=48 and tick7 DECanimation/clocks0. Anim48 no decrement even tick7.
2D phase0 IntegrateY then Gravity; negative integer clears vertical/INCstate then
horizontal same call. Phase!=0 integerFD (any fraction) skips gravity, then Y.
Direction0 skips X AND braking. Other directions integrate bit0-selected X;
postapex subtract0040 modulo16, exactzero direction0, underflow wraps. Both X
paths join A2F8 braking. Player overlap before Ycutoff; hitstate0 then8B82 health
recovery (not ordinary sounddispatch); render retained, evenY>=F0. No contact and
Y>=F0 state/render0; EF retained. Horizontal/Y integration boundary returns ignored.

## Collection8B21..8C44 and owner transactions

State0 occurs BEFORE every class dispatch, including unavailable extension.23..29
index sounds/pointers:23add2digit3;24recoverone;25restoreall;26clearprotection,
specialmode1/timer01E0;27gainlife (only exact9 caps, FF wraps0);28add1digit2;
29dialogue:stage4 ORflags4/message10, others OR2/message0E, player0F.61sound15/
add5digit2.2B stage1 sets extensionflag BEFORE capacity gate; tries overlay extra
record3 negativeY, else4; if neither negative returns with actor removed. Available
slotY20/following sentinelFF then restoreall. No ordinary reward sound for2B.
Treasure>=6F sound37, completion latch0, AddRewardDigit(0,1), bitmask[anim-6F]
ORstagebits. Animation<74 player10 conversion; >=74 player0F/message0D. Existing
bits preserved. B14 digit saturation/blank arithmetic/health owns transactions,
B06 timers, B13 stage/conversion and B15 resumable dialogue own consumers. Source
collect does not check protection; incoming contact permission determines pickup.
All eight supported removal-maskbits retained; no generic identity guessing.

## Consumer/data index and limits

B08-consumers.json preserves all assigned primary/related ledger identities and
units, every direct shared entry caller and state references. Catalog tests read
matching local ROM via named recipes; synthetic tables deliberately differ from
original so contracts cannot pass by baking assets. No original payload embedded.
All12 assigned targets, every9motion entry and shared reaction are covered by
substantive native-discovered cases. No game behavior implemented, QA/traversal,
GUI, toolkit edits or check-in/publication. No new emulator-required uncertainty;
source queue/caller corrections have direct instruction evidence. Stage4 must
integrate owner interfaces before interpreting passing local checks as fidelity.

## B09 independent869D caller reconciliation (supersedes nine-tuple extent only)

B09review of literal9D61JSR869D proves this entry starts at changed-counter LDA;
it skips swingframe andanimation3Fspecialgate before it. New activate_terrain
interface/tests preserve claim/replacement gate, then86B5 launch. It does not
replace existing86B5bypass. With changed0, animation3F is NOT reward/suppression:
(3F-1A)=25hex; ASL/ASL gives94, so890E+94=89A2 readsA1/F2/20/B5 asdata,
verticalF2A1/horizontalB520. Lowstate OR25 isunmasked, preservingbit20/local5.
Actual handler08template includes3F, so cannot exclude this consumer. Named
TERRAIN_LAUNCH_PHYSICAL1891E/TERRAIN_LAUNCH_ENTRIES38 extends through189B6;
ordinarymotionprogram count stays9. Intermediate tuples are preserved only as
bounded ROM-backed data, no execution of instructions or invented mechanism.
All38tuples and3Fgatebranch now contractually checked. Prior94cases retained;
B09adds5 actual-owner cases =>99. Full resumption/checks in planning/contracts/B09.md
and checks/contracts/B09/results.json; prior B08results retained/superseded.

## Stage4 synthetic boundary corrections

The original fixture injected post_velocity for horizontal motion/integration,
contradicting its apex expectations and source F237..F26E X-only writes. Restrict
that injection to vertical/gravity operations. Default negative-X integration now
returns true (ordinary SEC/SBC no-borrow), while explicit exits deque choices are
unchanged; source89CE..89D4 removes on false. This closes the original apex test's
unintended default borrow/cull. No behavioral assertion or test identity changed.
The superseded fixture is checks/implementation/contract-corrections/B08-before.rs.

### Stage4 borrowed cross-family reaction entry

Shared hit reaction, secondary recovery selection and launched motion now expose
narrow entries receiving the actual random owner, canonical audio ring,
source-derived recovery parameters and immediate movement callback. Existing
object entry points delegate to the same implementation. Enemy/platform/boss
adapters need no dummy status/progression owner and no cloned mutable state.
Fresh audio Submission provenance is consumed before secondary RNG exactly as
before; submitted sound observations remain distinguished from unsubmitted ones.
