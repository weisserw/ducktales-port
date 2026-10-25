# B06 independent player trace (stage3; implementation pending)

Maintained source `reference/source/game.asm` SHA256
`c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc`;
`reference/reading.asm` SHA256
`201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5`.
Physical offsets below include16-byte iNES header. All player code E354..ECFF
and dispatch/data are fixed mapper bank7: physical1E364..1ED15. EF62 selector10
is B14 timer conversion (physical1EF72 onward); ED05 selector0F is B15 dialogue.
The maintained .org CPU address must not be confused with physical +10 offset.

Independently opened actual MOS-authorized manufacturer programming manual:
[Synertek SY6500/MCS6500 publication6500-50, August1976](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
sections2.2.1/2.2.2,4.2.1,7 and appendixB. ADC includes carry, SBC includes
inverted borrow; CMP preserves A but sets carry on unsigned >=. LDA/STA do not
clear carry. X/Y are bytes. Signed velocity decisions test integer byte bit7;
threshold subtraction is unsigned16-bit with chained borrow. Source uses binary
arithmetic, no decimal-mode inference. Instruction semantics are not a CPU model
in native code. Toolkit docs/development.md NES references and nes-native.md
supply context/helper scope; game policies stay here.

## Native contract boundaries

ActorContact is the sole mutable player actor, index0 in the canonical B10 pool.
B06 adds its animation_restart field, matching B03 borrowed visual.restart;
B03 must project that field instead of retaining another physical actor store.
Player holds only action, queued action, controls and owned timers. B01 Controllers
are writable after sample; no resampling in player policy. B14 Health is borrowed,
not mirrored. Effects return typed caller obligations, sound requests and transient
scroll/spawn/collision outputs; caller commits them immediately in source order.
ContactWorld is an injected B05 boundary for gravity/integration/scan/probe, not
an oracle. Synthetic fixtures set only boundary outcomes and record call order;
asserted state/branch/merge/correction is real pending B06 code. Production adapter
must borrow B05, WorldCache and canonical actor pool. It may apply immediate
reaction writes during scan/probe. Queued reactions remain independently pending.
Policy methods cover native subroutine boundaries; rising/falling wrappers must
call shared primitives, not recreate CPU instructions. APIs are declarations only.

## Source dispatch and caller order

E387 loads selector into X, split pointers lowE396/highE3A7, no clamp. Bounded
source domain is0..10 from producers. All17 targets independently decoded from
matching local ROM:0 E3B8 standing;1 E63B airborne;2 E7EA forced jump;3 E806 climb;
4 E8EB swing;5 EAB9 protected;6 EB00 damage;7 EB82 terrain pose;8 EBA7 fall out;
9 EBC9 waitA;A/B EBD9 carried(shared target but stateA jump forbidden);
C EC2D launch;D EC9B death;E ECFF hold;F ED05 dialogue;10 EF62 convert timer.
No invented out-of-domain behavior; loader rejects invalid pointer extents.

DB6C calls protection/special/action before bank6 actor dispatcher and countdown.
Dialogue cursor gates actors/countdown, not player call. Camera follows player,
deferred reload/status/UI/special room then composition and spawn follow. DBE4
clears displacement/terrain-changed and queued reaction after composition/spawn:
nonzero queue clears even if rejected; only state6 is blocked by nonzero protection.
B05 scans write immediate state6 (D280/D341/F48C) or actor hit marker during scan,
not queued state. Terrain D16B..D1CE queues6/7/8. This distinction is essential:
standing may become damage during vertical scan and skip standing controls;
state4 still protects swing animation during vertical work. B13 owns frame tail
application; it must not use a single indiscriminate deferred-reaction queue.

## Timers E354..E387

Every invoked player update sets GravityStep33. Protection0 and80..FF retained;
01..7F decrement before action dispatch. Special0/80..FF skip audio and timer.
Positive mode ensures stage command08 only if current command differs, then
subtracts1 from16-bit duration with borrow. Initial0 wrapsFFFF;1 expires0,
clears mode and queues current StageIndex even if08 was queued earlier same call.
Protection/special timers belong to player caller, B14 timed reward initializes
special01E0 and clears protection; unrelated title clear must not be generalized.

## Standing E3B8..E482 and jumping

Standing first executes falling physics. Nonzero state afterward skips all
standing controls and continues horizontal. Otherwise SELECT edge calls B14
ApplyOverlayValueCost before B/A/idle checks. Ready animation07 + B edge chooses
animation09/state4, zeroes frame/tick, clears momentum direction, resets speed0180
and horizontal. B precedes A when both newly pressed. A edge copies configured
jump velocity, state1 and enters entire airborne routine same update. Without
movement/down(held&07), animation08 landing is retained; otherwise idle00,
frame/tick0/delay10. UP alone is idle here. DOWN dominates horizontal, selects
crouch03/frame80/restart1/tick0/delay10 only when selector differs; existing03
returns unchanged without horizontal. Walk invokes horizontal first. Clear solid
bit resets delay10; solid and non07 decrements byte delay, at0 captures held&03,
sets07/frame0/tick0; nonzero chooses01 unless already01. Existing07 returns
without delay decrement. A delay0 wrapsFF. Airborne entry always delay10, rising
physics, then the same07+B edge but returns immediately without horizontal.

## Horizontal E482..E639

Animation02/06 bypass momentum, clear remembered direction and use0180. Otherwise
nonzero momentum control/direction enters remembered movement. Clear control
on entry. No remembered direction/no held horizontal returns without speed change;
new direction captures held&03 then accelerates0010. Same direction accelerates.
Cap applies only when post-add integer==2 AND fraction>=80; integer3 is not
normalized! Release subtracts0006, reverse subtracts0010. Floor only when integer
==1 AND fraction<80: set fraction80 and clear direction. Release at floor returns
without movement. Reversal calls remembered-direction movement (possibly none)
then sets facing from held RIGHT bit using table00/40; it does NOT replace remembered
direction with opposite held input. The isolated CapturePlayerMomentumDirection
entry has no independent symbolic caller; preserve fallthrough reachability rather
than inventing a new frame behavior.

Ordinary external latch E551: consume latch once; held horizontal adds0180 to
24-bit worldX+fraction BEFORE ordinary chosen movement, including when LEFT held.
With no held horizontal it branches directly E593 positive motion without
initializing common movement scratch (actual caller state must be preserved);
its direct scan path remains real. ScrollDirection uses held/momentum bits;
negative bit takes priority. Integration sets face bit40 for positive, clears it
negative. B05 horizontal actor contact carryclear gives penetration; cap at12,
subtract for positive/add negative using16-bit world X, never change fraction.
Compare old-new16-bit: zero retains initial spawn direction; nonzero no-borrow
(old>=new) sets1, borrow sets2. Thus direction names cannot simply mean final
velocity. Terrain samples0..2 for spawn bit2clear,3..5 otherwise; preserve prior
actor bits, OR terrain bits. Solid snaps positive via SnapActorPositiveX or
negative via AlignObjectWorldXForward. Shared B05 query side effects stay immediate.

## Vertical E970..EAB3

Falling: reset collision/contact flags, motion04. Skip gravity only when velocity
integer==F4 (fraction irrelevant); otherwise B05 wrapping gravity33. Save oldY,
IntegrateFallingY (subtract signed8.8 encoded velocity) then vertical actor scan.
Actor hit subtracts penetration from byte Y without cap. Terrain wrapper samples
actual old-newY direction: equality uses motion flag, old>=new chooses08 upper,
old<new chooses04 lower (byte unsigned wrap observed). Sample starts table[09,06]
indexed(direction>>2)&1, count2 (three samples). Save render, collision/contact
flags before probe. Solid lower snaps positiveY; upper forward. OR saved/new
contact and collision flags and restore prior render exactly (terrain mark lost).
Any final solid clears velocity ONLY, retains Y fraction. Otherwise standing
state0 becomes1, other actions retained, then airborne animation policy.

Rising entry with integer bit7set goes straight falling. States>=5 bypass hold
cutoff. For lower states choose normal threshold042D for state&2==0 or pogo05D2
for state&2!=0. Velocity<threshold and !heldA clears velocity AND Y fraction,
then immediately executes falling physics in same update. Equality continues
rising. Otherwise motion08/reset flags/saveY, IntegrateFallingY BEFORE gravity,
scan actor; contact SEC ADC penetration adds penetration+1 (wrap), not plain
penetration. Probe terrain; solid also clears velocity/fraction then FALLS in
same call. Clear terrain invokes gravity once afterward and animation policy.
State2 always pogo; state4 preserves animation even if DOWN+B. Otherwise DOWN+B
chooses06; already06 retained, newly06 tick80/frame0. Existing07 preserved only
when held&03 equals remembered pose exactly; otherwise02/frame0/tick0. Other
animations unconditionally reset to02 even when already02.

## Landing E661..E719

Only motion bit04 and final collision bit1 enable landing. Animation07 or state4
sets standing without landing sound/animation. Actor-contact bit1 plus heldB plus
(state2 OR heldDOWN) permits pogo; when additional a7 control nonzero, require
Y<A0. Pogo sound11, velocity05A8, animation06/tick0/restart2/frame81/state2.
Otherwise sound10, animation08/tick0/frame80/restart1/state0. Horizontal runs
then if animation07 and horizontal solid bit clear replace02/frame0/tick0.
Upward probe runs next. Downward room only if motion04, port2 UP clear, Y>=E0.
At final row (next==rowcount) return. Camera low>=80 increments high and camera
room byte, then low0; previous scroll bit0 selects column20/1F; writes held
input DOWN before room_transition. It does not modify pressed input.

## Climbing E740..E878

UP held probes samples0C..0F/count3. Solid bit returns. Test offsets[00,00,08,08]
with X=3..0: require ((worldX+offset)&0F)<8 AND result[X]==2. Highest qualifies.
Indices2/3 round UP to NEXT16pixel X cell even if aligned; increment page on
wrap and scroll1. Indices0/1 round down, scroll2. For all indices y=X&2;
compare result[y] to result[y+1] (PlayerActorContactFlags alias is result1,
not a separate gameplay boolean). Unequal and evenX snaps Y to row|08;
unequal oddX snaps (Y&F0)-10|08 wrapping. Equal keeps Y. Set04/frame0/tick0/state3.
Forced jump state2 ORs heldB shifted into heldA; pressed stays unchanged. If
resulting heldA clear set1, then airborne handler. Climbing delay10 every call;
no UP/DOWN: pressedA leaves state0/velocity0, otherwise freeze tick bit80.
Direction present clears tick bit80; resulting tick0 sound12. Copy horizontal
speed into Y velocity. Exactly UP(08) rises samples14..17; any other direction
(including both08|04 if game writes it) descends samples10..13. Result3 must
be class2 else state0/velocity0. At upperY<39 or lowerY>=E0 calls room transition.

## Room transition E878..E8E2, overlapping tables E8E3..E8EA

Late route nonzero and playerY>=C0 yields message0C/stateF before camera checks.
Otherwise camera low must0 and high odd. Direction X=(held>>2)&2: down0/up2.
Stride=(columns XOR table[X])+table[X+1] via byte ADC; following ADC cameraRoom
preserves that ADC carry (normally0 for positive widths). Independently stride+
terrainRoom uses CLC. FF next room rejects before row bounds. New row byte uses
row+table[E8E8+X] (down+1/upFF); reject >=rows. Camera/terrain indices commit only
on success. No callee occurs between TAX and indexed store E8C2: X still0 down,
2 up. table[E8E7+0]=E6 goes to slot0 Y; table[E8E7+2]=32 goes to SLOT2 Y! Upward
player Y stays unchanged. Source comments claiming unconditional playerY change
are superseded by these instructions and true X lifetime.

Commit discards enclosing helper return, clears special spawn control and player
render low2bits, calls B04 sixty-tick vertical animation then DBE4 gameplay tail,
not the remaining player caller. Native result Committed is nonlocal continuation,
not a regular return followed by movement. B04 must consume updated row/cursors
and preserve slot2 write. No World-input field invents a replacement playerY.

## Swing E8EB..E970

Rising physics first. Animation41 processes only tick0/frame0: decrement byte
local_state, state0/delay10 only at0, initial0 wrapsFF. Otherwise at frame6,
held & remembered pose nonzero chooses07/frame0/tick0/state0 and returns;
zero chooses state0/delay10 then continues impact gate (frame still6 so no hit).
Impact only tick0/frame2: sound13, actor scan; contacted handlers08/15/42 return.
No contact is blocked too:41/frame0/tick0/local5/sound18. Other contacted handler
also blocked, with scan's immediate state/hit-marker side effects retained.

## Protected/damage/pose EAB9..EBA7

State5 protection>=8 holds (including negative);1..7 falling physics then05 and
opposite-facing horizontal movement, finally restore original facing. At0 state0,
speed0180, B14 damage rule and inspect first depleted flag afterward; if exhausted
stateD, clear actor phase low nibble (preserve high), immediately death trajectory.
State6 initial protection0 emits14, velocity0169/frame0/tick0/speed fraction0,
protection46; B14 damage rule once then same exhaustion check. Nonzero skips init.
Select rising/falling by velocity sign, then if protection==3C set speed fraction80
and state0 (integer is NOT reset); set05 and same recoil path. Global timer
predecrement happens only in pre_dispatch, not inside these helpers.
Pose7 sets2C and resets frame/tick/Y+4 only on animation change; byte Y wraps.
A edge follows normal jump install+entire airborne update, not heldA.

## Fall/wait/carry/launch/death/hold EBA7..ED05

Fall8 falling first; Y<D0 horizontal; else reload=(stage+87 carry)+variant with
chained ADC carry, clear action, queue, actor flags and health alternating latch.
Do not decrement lives here: B13 reload owns it. Wait9 pressedA releases state0;
tick always0. CarriedA/B set scroll/spawn2 and stateA0; heldDOWN hides(state0),
skips terrain. Only stateB, noDOWN, pressed EXACT80 jumps (A+B does not).
Visible carry probes3..5; solid Y+8,state6 then damage initialization immediately.

LaunchC motion!=0 sets actorE0, gravity then integrateY. motion0 andY<18 skips
vertical work; otherwise integrateY THEN gravity. Always positive X wrapper
samples0..2; scroll/spawn1. After wrapper motion0 & worldX high==7 increments
motion and clears velocity. Solid obstruction action0/speed0180, then animation02
and frame/tick0 ALWAYS. Full X boundary removal remains B05 wrapper policy.

DeathD phase low nibble0: motion/frame0,animation05,direction1,velocity0400,
speed0080, inc actor flags (preserve high). Upward phase integrateY then gravity;
when postgravity velocity integer negative: increment motion, velocity0, return
WITHOUT X drift this frame. Other upward frames drift positiveX. Falling phase
gravity then integrateY; Y>=E0 common reload and NO X drift; otherwise drift.
HeldE only clears tick, no gravity/contact/position/action effects. DialogueF and
conversion10 retain dispatcher/timer ownership but delegate their resumable
bodies to B15/B14; neither a blanket gameplay freeze nor duplicate text/status code.

## External writers and consumers

Bank6 contact85D9 attaches state5/protection44/speed0300, position from actor,
zero Y velocity/frame/tick and animation-dependent Y/pose tables. Actor85FD-release
observes protection8 after player predecrement, then0; this is caller-specific.
Rideable927B gates state6/D; overlap+UP enters9, copies X/Y while state9, no fractions.
Carrying9EB3 entersA, actor3E/player8B; ground selectsA versus airborneB, copies
actorX andY-0B only while A/B. Player A jump escapes before later actor loop,
so actor must check current action before recopy. LaunchADB6 gates6/D and sets
C/speed0400/velocity0728/motion0/sound28. External triggerA02D increments one-shot
latch while proximity predicates hold; ordinary movement consumes it once.
LiftF9F8 gatesD, entersE, copies X, moves actor+playerY-2 and completion below10.
B08 treasure8C03 choosesF/message0D or10 conversion; B15 dialogue contact choosesF;
B16 escortFC9B and B17 goalFDB2 chooseE/F and may write immediate actor/player
positions. These families own eligibility/tables; B06 methods preserve all
source-backed immediate writes and never recopy stale player state after slots.
Other readers: B14 countdown action1..?, B13 pause/reload, B03 protection flashing,
B05 contact gates. Complete symbolic consumer/assigned-region index is emitted
under planning/contracts/B06-consumers.json, not treated as semantic coverage alone.

No new cohesive area was discovered. Tentative snow-artwork meaning stays
provisional TerrainPose, not an invented artwork/level-specific requirement.

## Whole-call integration and evidence limits

Player::step is the complete E354 entry, not only dispatch-table lookup. Policy
subroutine contracts stop at explicit native continuations (for example carried
contact chooses DamageHealth; protected depletion chooses DeathTrajectory).
Those continuations must execute immediately before the full call returns.
Complete-call tests independently require protection46/health/sound14 on carried
upper hit, death phase initialization and Y/gravity/X order on exhausted protected
expiry, immediate scan reaction before SELECT/A controls, held-B mirror before
rise cutoff, committed-room nonlocal exit and F/10 pre-timer ownership. Terrain
queue test requires pending7 to survive player update while action remains0;
B13 applies it only after composition/spawn. Effects.work may record ordered
observations and subroutine continuation plans, but cannot be drained later in
place of required same-frame mutations. No test installs an alternate game or
uses a future implementation as expectation.

ContactWorld.probe result classes are the four reversed-result slots, so the
climb final class is classes[3]. It must not be conflated with actor contact
bits; fixtures keep these separate. B05 PlayerContact is a borrowed projection
of Player for shared helper invocation: map action/queue independently and merge
back before the next B06 branch. RoomContext is an ephemeral borrowed world view,
not another world store. B03 animation restart projection is required. Production
adapter wiring is pending stage4, visibly unimplemented. These are required
implementation obligations, not excluded functionality or QA acceptance claims.

Initial compile errors from unsupported Rom.bytes and an unnamed fixture pool
parameter are retained with their exact logs. Corrected fixtures validate the
local raw bytes with the existing Rom API and name the borrowed pool; there was
no toolkit workaround. Retained125-case results are superseded only by the final
same-source128-case compile/discovery/run, not rewritten as passing behavior.

## B16 shared immediate EBB4 caller

B16 hidden-player timed spawner jumps directly EBB4, bypassing falling physics.
Player::commit_life_loss now declares this shared entry for B06/B15/B16. Actual
canonical action/queued/activity/alternation clears and B13 reload request must
commit before return, not just produce a deferred enum. Stage+87 then ADCvariant
retains first addition carry; no LivesCounter decrement at this entry. B16 adds
two direct integration contracts; existing B06/B15 common-loss contracts retained.

Stage 4 correction: the superseded trace and UP_SLOT_Y=01 assertion misread
the overlapping transition data. Immutable ROM bytes at physical 1E8F3 are
00 00 FF 01 E6 01 32 FF; E8BF is BD E7 E8, with X=2 on upward entry.
Thus slot2 receives 32. The original contract identity is retained; the prior
file is preserved in checks/implementation/contract-corrections/B06-before.rs.

### Stage4 actual swing contact identity

ResolvePlayerSwingImpact E92C calls PlayerHorizontalOverlapScan; on carry-clear
it reads ObjectHandler,X from that exact returned slot. A preexisting hit marker
in another actor cannot replace X. CollisionFacts now retains the scan's actual
actor slot and the full step uses it. The prior fixture is preserved at
checks/implementation/contract-corrections/B06-before-contact-slot.rs; original
expectations/identities remain, plus a stale-hit-marker regression. The Game
adapter separately retains general overlap bits and the actual descending-pogo
actor-contact latch; ordinary contact does not invent that latch.

Stage5 QA002: source StartPlayerDamageArc EB35 calls falling physics before
comparing protection3C, restoring control/speed fraction, and writing animation05.
The policy now queues a typed DamageContinuation after the real vertical work;
that continuation reads current render for the horizontal tail. A complete step
regression first reproduced Airborne instead of Standing before repair. Natural
Moon discovery002 then matched both games through64movement ticks, including
that exact damaged-to-standing boundary; saved original foreground return DC28.

QA007 source EAD9 protected near-expiry: PlayerVerticalPhysics may write the pogo/air animation. Only after that callback returns does EADC select05, preserving its frame/tick, and horizontal recoil follows with inherited facing. ProtectedContinuation now sequences this tail beside DamageContinuation; the damage-only release boundary remains exclusive to state06. Actual Amazon722 exposed the eager setter; complete protected regression checks Down+B/ordinary airborne callback and both facing directions.
