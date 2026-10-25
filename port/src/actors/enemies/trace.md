# B07 independent whole-area source trace

Frozen reference/source/game.asm SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reference/reading.asm201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Bank6 CPU8000 corresponds physical18010, fixedC000 physical1C010. Source offsets
include iNES header. Planning metadata is navigation only. Entire controlling
contract/workflow/common/coding requirements, architecture, owner interfaces,
prior traces and contracts are consulted; source instructions override comments.
No instruction execution, emulated RAM or extracted assets are introduced.

CPU reference independently opened: [actual MOS-authorized manufacturer reprint,
Synertek SY6500/MCS6500 publication6500-50, August1976](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
sections2.2.1/2.2.2,4.2.1,7.4..7.9,8.5/8.6,10.1/10.2/10.7/10.8. Byte INC/DEC wrap;
CMP supplies unsigned comparison carry. Loads/stores and logical operations
preserve carry; PLA restores A and N/Z but preserves carry. SBC subtracts extra
one on clear carry, ADC consumes carry. Native quantities preserve these source
results without storing processor flags. B01 global feedback is a distinct RNG
operation from family-local additions below. NES binary arithmetic is used.

## Inventory, shared owners and call ordering

Independently decoded split pointers1803C/1807F from matching local ROM: B07 owns
26 selectors/25 distinct targets:01..07,10..12,18..1C,1E..20,22..23,25,30..34.
30/31 aliasAC3D;00 is B10 empty handler, not B07. B09 owns1D/21/24, B08 owns08/09
and launched reaction8932..8A10. Per-region/operand index B07-consumers.json retains
all81 primary/related regions (80primary); mixed regions retain original identity.
Consumer evidence includes caller-assigned descendants and following shared data;
not every byte in a mixed region becomes B07-owned behavior.

Canonical ActorPool/ActorContact owns all physical/animation/local fields.
ObjectLocalAux0 is ActorContact.local_aux2 (source06D0), not a second field/store.
B05 World boundary borrows that pool and Player, commits movement/contact before
return. ContactFacts reports the full final probe bits and collided actor slot;
no CPU scratch map. Shared motion may remove actors immediately; family source
still executes its continuation unless the original helper abandons it. B03 owns
animation advancement during composition later than handlers; render is read-only.
B10 ascending live slots observes newly allocated later slots same frame; lower
slots wait. B14 Status::steal mutates the same actor immediately. Effects record
ordered calls/sound/palette requests, never replace immediate actor mutations.

Shared FaceActorTowardPlayerF6BD uses unsigned16-bit actor>=player -> direction2,
otherwise1 (equality2). DistanceF7CC/F7DA uses unsigned borrow for sign, then byte
absolute magnitude; world distance aliases across pages. It is not shortest wrapped
distance or signed-high test. MoveActorByFacing82F9 uses bit0 positive18/count2,
negative1B/count2. Shared velocity setterF313 reads two words via B05 named recipe:
selector0 ordinary walkers; JumpingStealer passes1 because A retains LDA#01 across
INC/store. Four physical bytes1F340..1F344 include next instruction as consumed
data; no executable-byte oracle/hardcoded preset in family code. B05 owns decode.

Reaction8932 offscreen clears state/render only; visible queues35 then secondary
RNG spawn, rewrites handler0B and launches same call. B08 owns entire body/full
allocation/random/carry semantics. Caller-specific animation and unconditional
special-spawn3C writes below belong here. Superseding B08 source correction: QueueAudioF757 CMP sentinel88 supplies C1
on empty88 AND occupied commands>=88 (includingFF stop), C0 on occupied<88; callers must not assume flags survive. Family RNG
pose/paired additions have no intervening audio call affecting their incoming C.

## 01 JumpWalker8225..82AE (physical18235..182BF)

Invisible OR visible hit -> retry3C and reaction; only visible hit writes animation0C.
First low-nibble0 clears vertical, fractionX, increments full state, speed0100,
faces player. bit1 selects rise21/count2 or fall1E/count2. Rise negative integer
DECstate then clear vertical and FALL same call. Nonnegative rise joins animation
reset unconditionally (not a falling contact test). Fall clears frame/tick only
when final bit0 absent. Then horizontal helper REPLACES final bits. If statebit1
clear, horizontal final bit0 set, and full verticalword0: INCstate, launch0400.
The trigger is horizontal result after movement, not retained floor contact.
No reverse on wall. High state bits preserved except real byte INC wrap.

## 02 BounceWalker82AF..82F8 (physical182BF..18309)

Hit writes0C then reaction, no visibility/cooldown prefix. Init clears vertical,
INCstate, preset0, face. Fall1E/count2; missing bit0 resets clocks, then horizontal
18/1B count2. Final horizontal bit0 reverses direction XOR03. No jump launch.

## 03 Thrower830E..8461 and04 Projectile8462..8490

Thrower invisible/hit sets retry3C/reaction without animation rewrite. Init resets
vertical/local_state/local_counter, INCstate,preset0,face. lowphase3 is throw mode:
if counter0 and low animation-frame nibble nonzero, highest inactive slot gets
clocks0,speedfraction0, copied parent state/render/worldX,animation17,handler04,
speed0200,Yparent-8 bytewrap,profile17. Other stale fields survive (direction,
hit,spawn identity,Yfraction/velocity/local fields). Allocation failure retains
counter0 for next-frame retry. Sound1B and INCcounter only on allocation success.
Always DEClocal_state; nonzero return; zero clears frame/tick,animation14,state&F1.
Counter remains set, preventing later throw until some source reset reinitializes.
Zero timer wrapsFF and does not finish. Frame80 has low nibble0 and cannot spawn.

Ordinary bit1 rise09/count2; negative apex DECstate,zero vertical,fall06/count2
same call. Airborne sets15 and clears clocks; supported fall sets14 without clock
reset. Horizontal positive00/count2 or negative03/count2. Only statebit1clear and
verticalword0 evaluate final HORIZONTAL bits: contact launches0400/INCstate;
otherwise counter0 and distance<51 starts animation16/frame80/tick0/restart1,
state=(state&F0)|3,timer3C. At51 reject. There is no Y-distance throw gate.
Projectile uses renderbit40 for positive24/count0 versus negative25/count0.
Then TestPlayerActorOverlap (B05 immediate Player reaction) occurs before checking
retained movement bit0. Only movement collision clears state/render/identity;
contact alone does not remove. Source profile is unflagged17, independent visibility.

## 05 RandomPose8491..84E4

Hit animation0F/reaction. Only exact frame0C changes pose. CMP equal leaves C1:
sum=(random.words[0]+words[1]+1) modulo256 stored words[2], words0/1/3 unchanged.
Pose=(sum&1)+0D; pose0E sound2A. Clears tick/frame. Every no-hit call then compares
full unsigned worldX: actor<player sets render40; >= clears40; retain other bits.
No state/motion initialization. Frame0B/0D leave RNG and clocks untouched.

## 06 ProximityRise84E5..8566

Invisible clears state only, retaining render/identity/retry. Hit11/reaction.
Init localstate0/INCstate. bit1clear sets vertical0200 and negativeY27/count0.
Without bit0 return (cooldown not decremented). Supported distance>=32 clears
cooldown; distance<32 decrements nonzero cooldown, while zero INCstate but does
not move rising branch until next call. bit1set vertical0300, IntegratePositiveY
(adds screenY), then Y>=B0 OR vertical-distance<18 -> DECstate,cooldown96.
AtB0 stop even if player distant; distance18 does not stop belowB0. Wrap8-bitY
before comparison. The source comment 'rise' does not change integration direction.

## 07 ContactPose8567..8660; tables8661..8667

Player Death0D returns before all visibility/hit/state work. Invisible clears
state only. Hit delegates reaction. Lowphase0: frame0 clears tick, distance<0C
sets frame1 except animation12 and actorY<playerY directly clears frame/tick and
returns. Frame2/tick0 sound19 before profile table indexed current frame0..2.
TestPlayerActorOverlap may set player Damage immediately; denied return. Permission
then distance>=8 sets Damage; distance<8 attaches: INCstate,actorframe2/tick80,
protection44/actionProtected5,playerY=actorY+ROM unsigned offset (wrap),worldXcopy,
speed0300,vertical0,clocks0,ROM pose. Animation12 selects table0 else1.
Lowphase!=0: protection nonzero except8 returns untouched; exactly8 sets render
from(player.render&FC)^40, sound1A,INCanimation; clears actor clocks. AnimationFF increment wraps0 and falls into the protection0 release branch: clearsflip, decrements animation backFF and decrements state, then clearsclocks.
Protection0 clears render40,DECanimation/DECstate,clears clocks. No position copy
on release. Offsets/poses/profiles are ROM data, not species assumptions.

## 10 WaitingJumper9313..942F

Init low0: INCstate,vertical0,counter0,speed0200,timer78,face. Bit1clear timer>0:
DECtimer, tick0,return (old1 expires but transition waits next call). timer0 waits
exact frame3, then clears clocks,animation2D,profile|80,INCstate and FALL THROUGH
movement that call. Nonzero counter freezes tick at frame3. Counterpositive uses
rise09/count2; negative integer saves integer into counter,zeroes vertical,return.
Counter0 or negative uses fall06/count2. No bit0 returns before horizontal/hit.
Grounded negativecounter restores42/clocks0/counter0/timer1E/DECstate,returns.
Otherwise horizontal00/03 count2, sound31 if tick0, then hit reaction (sound first).
Final horizontal bit0: XORdirection03,animation2E/clocks0/profile&7F; contacted
actor from B05 is checked for handler08 and B08 transition86F7 invoked immediately;
sound1D,INCcounter,vertical02D4. No general early hit/offscreen gate.

## 11 PresetFlight9430..9543; sequence9544..9551

Only animation85 honors hit marker. Invisible clears state/render and retry3C;
identity untouched. Low0 otheranimation waits distance<60; at60 no init/audio.
Within queues30; animation85 skips distance/sound. Init INCstate,localstate/counter/
aux0=0,vertical0300,face; directionbit1clear ORs flip40, other direction leaves
existing flip unchanged. Always B05 playercontact before selected phase.
Bit1clear checks unsigned actorY-playerY: no borrow and >=10 INCstate, then addsY
same call (does not immediately execute preset phase). Preset phase timer0 loads
sequence[counter] through B10 paired words, absolute-values negative horizontal
word using full two's complement (8000 stays8000), leaves vertical word signed.
Timer3 normally; counter>=7 AND cycleauxodd gives8. INCcounter; at0E resets0 and
INCaux. Statebit0set always DECtimer; bit0clear only enables/DECtimer when
playerY>=actorY with difference>=20. Signed presetY is ADDED directly to Yfraction
and byteY, not falling integrator. Directionbit1clear integrates positiveX else
negativeX. Early half does not countdown, so a loaded preset may be held indefinitely.

## 12 PairedController9552..96BA; data96BB..96D4

Animation31: nonzero timer DECreturn; timer0 sets32/reaction. This prefix ignores
hit/offscreen. Low0 searches active actors15down1 for animation1D/1E and exact
16-bitX (Y ignored), including self if its animation qualifies. Otherwise highest
inactive15down1. Full pool stores link0; source still clears timer/counter/fractionX,
speed0100,INCstate and returns. New companion Y+8wrap,profile80,handler08,stateA0,
clocks/hit0,animation1E; render/identity/velocity/other locals deliberately stale.
Active linked handler09 OR own hit: sound2C,animation31,clocks/profile0,timer3C.
No linked active check here; slot0/player handler may trigger it on full pool.
Timer0 chooses phase=counter before increment. Active linked animation1D/1E alone
gets animation/flip from8-entry tables. Even phase -> direction/flip table[phase/2],
timer20,animation2F; odd ->direction0,timer3C,animation30. Clocks0,INCcounter.
If counter&3==0 choose table[(random0+random2+incomingC)&1], where incomingC is
phase parity from LSR (odd1/even0), not global RNG feedback. Random words unchanged.
Always DECtimer same call, then direction0 stationary or bit1positive/negative
integration, then playercontact. No general visibility removal gate.

## 18 LedgeWalker96D5..973B;1E FastLedgeWalker9D8A..9DF4

Hit reaction without animation/cooldown. Init clears vertical;18 speed0080,
1E speed0133;direction1,INCstate. Fast additionally playercontact before probes.
Support fall count0:18 starts2D+(direction&1); fast ROM table9DF5. Missing bit0
XOR03 before horizontal; horizontal collision XOR03 AGAIN, so both restore original
direction.18 positive29/negative2B count0;fast positive19/negative1C count0. Uses
new direction after ledge. No actor visibility prefix, no animation reset.

## 19 Ambush973C..993B; tables993C..9955

Iteration012 replaces the complete family's packed phase and generic local bytes
with canonical `AmbushPhase`, `delay_remaining` and `preset_step` in ActorControl.
WaitingForPlayer/Opening/Tracking/Closing/Attacking are direct game states. The
native body no longer masks/increments a state byte or reads generic local bytes.
Source projection APIs exist only for shared legacy-family/lifecycle/collision
callers and unchanged debug observations. ActorContact's old three raw fields
are removed, so there is no retained shadow of the native owner. Generic callers
update this same owner through getters/setters; a handler boundary selects its
payload losslessly. Normal native calls retain their payload without redecoding.

Actual source9740..993B was reread for all five phases. Both motion gates can
advance during one call; motion/contact/sound/animation order stays immediate.
The vertical nibble threshold is expressed as an unsigned gap>=80 (all byte
height pairs at representative actor heights checked). Animation tick gates and
wrapping opening/rest/segment delays remain observable rules. Unsupported source
phase codes remain a bounded compatibility variant with the prior no-work policy.

All prior fixture expectations are unchanged: fixtures use projection setters and
getters for the removed fields. New regressions verify every state byte across
representative local values, native/legacy handler transitions, partial reset,
native payload continuity and the composed family boundary cases. Mechanical
migration of other actor consumers changes only how they access the one owner;
their source rules remain separate from this converted family.


Invisible OR hit -> LDA39 then reaction WITHOUT STAanimation: retained animation,
no retry write. Visible nohit calls playercontact unless animation39 BEFORE local
dispatch. Five source pointers:phase0 9768,phase1 9787,phase2 AND3 alias97F3,
phase4 98C8. RTS at97F2 is return for previous phase, not phase2 behavior.
0 distance<0C ->INCstate,timer1E,counter0,vertical0280; else unchanged.
1 timer nonzero DEC; expiry sets3C/clocks0 and return. timer0 animation3C/tick8
DECanimation to3B/clocks0; down40/count1. Contact OR byte(playerY-actorY)<28
advances state,faces,ORs facing table flag (does not clear previous flip),3B/clocks0.
Borrow is NOT checked before CMP28: wrapped above/below separation matters.
2/3 timer0 clears motionflag,loads14-sequence preset; negative horizontal absolute;
negative vertical absolute plus motionflag08. Counter wraps0 at0E,timer3.
motion0 positiveY40/count1 else negativeY42/count1; directionbit1 negative3D/count2
elsepositive3A/count2. Tick0 sound32. Phase2: playerY>=actorY and (difference&F0)>=50
INCstate and DECtimer; otherwise timer remains held. Phase3 always DECtimer.
Then distance<0C advances state AGAIN (2->3 or3->4),animation3C/clocks0,
vertical0280,sound1F. Thus source can reach4 in a single phase2 call when both
vertical and horizontal gates succeed. Phase4 animation39 DECrestduration; expiry
sets3B/clocks/counter0,state=(state&F0)|2,profile&7F. Otherwise3C/tick8->5C/clocks0,
down40/count1; bit0 landing->39/clocks0/timer78/profile|80/sound20. Zero rest timer
wrapsFF. No omitted fifth dispatch target or generic unseen-species exception.

## 1A Eruption9956..9A74; four3-entry tables9A75..9A80

Hit reaction first. Profile0, then profile91 iff oldY in01..CF; evaluated BEFORE
launch/integration. Low0 distance>=20 returns after profile update. Otherwise
motion/counter/aux0, YEF, vertical selector-localstate recipe, tick80/frame0,
INCstate; same-call arc. INCaux EVERY active update including pause/wrapping255.
Motion0 integrateFallingY THEN gravity. Negative integer ->INCmotion,zerovertical,
frame1. Selector riseevent nonzero and equal aux triggers pair[19,27]. Motion!=0
counter0: gravity THEN integrateFallingY; Y>=F0 clears Yfraction/Y and counter78.
Selector fallevent nonzero/equal aux triggers pair[21,17]. Counter>0 DEC; zero
clears state low nibble and returns BEFORE fall-event check; otherwise event may
still fire DURING pause. Each event always sound21,palette pendingFF; allocation
failure does not roll those back. Highest free child state/renderA8,YBC,worldXcopy,
animation37,clocks/profile/handler0; stale identity/velocity/local fields survive.

## 1B RepeatingJump9A81..9B0F

Invisible/hit retry3C/reaction. Init speed0100,face,state|2 (no cleared velocity).
Always playercontact. bit1clear: nonzero frame freezes tick80, rise21/count2;
negative apex zerovertical,INCstate. Bit1set fall1E/count2; bit0 landing DECstate,
clocks0,vertical04E5. Both then horizontal18/1B count2. No wall reversal, no
horizontal-triggered launch. No low-nibble increment during init beyond OR2.

## 1C CompanionPatrol9B10..9C45

Hit56/reaction before patrol; no visibility prefix. Low0 distance>=50 returns.
Near:vertical0,speed0133,timer50,state=(state&EF)|1,direction1; playercontact.
Bit1clear support fall2D+(dir&1)/count0. Tests ENTIRE bits!=0 (not bit0), then
Y+=2 bytewrap,horizontal29/2B count0. Missing support OR horizontalbit0 OR timer
DEC reaching0 triggers XORdir03,timer50,INCstate,vertical0400,animation56/frame0/
tick80,sound34,highestfree effect state/renderA8,handler/profile0,animation57,
clocks0,Yparent+4,worldXcopy. Full pool retains parent's already committed launch.
With statebit1set: parityeven rise2F/count1, negativeapex INCframe,zerovertical,
INCstate. Parityodd fall2D/count1; contact clearsclocks,55,Y+=1,DECstate twice.
Fall without contact leaves state/animation; no horizontal movement in arc phase.

## 1F TimedPatrol9DF7..9E50;20 VerticalPatrol9E51..9EB2

Hit reaction. Timed init speed0100,timer16,direction1,INCstate. Playercontact then
bit0positiveX setsflip40 else negativeX clearsflip40. DECtimer after movement;
expiry XOR03,timer20. Vertical init vertical0100,timer16,face,INCstate; sets/clears
flip only during init from directionbit0. Contact then bit0negativeY elsepositiveY;
DECtimer; expiry XOR03,timer2C but flip is NOT synchronized afterward. Zero input
timer wrapsFF; both do actual movement before countdown/reversal.

## 22 TriggeredJump9F3D..9FEB;23 LeftWalker9FEC..A023

Triggered low0 distance>=32 freezes tick0 and clearsflip40 (not no-op). Near init
vertical02D4,speed0180,direction1,INCstate. Playercontact then phase2 rise53/count0;
negativeapex zerovertical,INCstate,return. Phase3 tick0 sound27,fall31/count2;
only landing calls horizontal. Otherphase horizontal positive36 if bit0CLEAR,
negative39 if SET (opposite ordinary). Wall XOR03; compare old/newdirection:
changed ->tick0/flipclear return,unchanged INCstate. Phase3 landing horizontal
DOES NOT reset phase to1 or increment it: source comment 'landing resumes walking'
is misleading. No source hit gate. Leftwalker initvertical0,speed0180,INCstate;
allcalls contact,fall31/count2; bit0ground integrateNegativeX, tick0sound27.
Unsupported returns without sound/integration, no own hit/offscreen gate.

## 25 ApproachA065..A1E8

Invisible/hit retry3C/reaction. Initmotion0,speed01CC,face,INCstate; no vertical
reset. low3 ->specialheight path below; otherwise statebit1clear distance>=40
horizontalonly,nearINCstate,vertical01CC. Motion0 positiveY1E/count2; no contact
returns WITHOUT horizontal. Contact4D/clocks0/sound24,motion08,vertical02D4,
speedfraction0 (speedinteger preserved). Motion08:4D waits exactframe2 then
freezetick80; otheranimation skips gate; horizontal18/1B count2 by directionbit1,
save itsbits, rise21/count2. Negativeapex AND savedhorizontalbit0 ->resetapproach;
negative without horizontalbit0 motion04,vertical0. Nonnegative simplyreturn.
Motionnonzero without08 ->fall1E/count2; nofloorhorizontal. Floor ifanimation!=4E
sound24,4E/frame0/tick1; wait until frame|tick0. Save olddirection,faceplayer,
COMPARE then STORE olddirection back (source intentionally restores old direction).
Facing unchanged sets4D/frame2/tick0 then joins setupmotion08/vertical02D4/
speedfraction0. Changed INCstate,motion/clocks0,4C,verticalANDspeed01CC;return.
Low3 motion0 negativeY21/count2; bit0contact OR Y<88 INCmotion; Y88+ no advance.
Low3 motionnonzero horizontal; bit0wall negativeY21/count2; no extra gravity.
No untraced phase3 special path or preservation of the temporary face result.

## 30/31 TimedStealerAC3D..ACBB;32 JumpingStealerACBE..AD6A

Timed hit reaction only. Initcounter0,base interval fromhandler-30 atACBC (two),
store interval bothlocalstate andaux, speed0100,dir1,INCstate. Counter>0 DECreturn;
old1->0 still no movement/theft. Otherwise positiveX/flipset ornegativeX/flipclear,
B05 contact. Permission ->B14steal increments SAME localstate, edits upper3 reward
digits/normalizes6/dirty. Then DEClocalstate. Successful theft cancels this call's
countdown decrement; repeated contacts can repeatedly steal (no once-ever latch).
Expiry XOR03,counter78,localstate=base*2 modulo256. No sound; missing contact
ordinary decrement. Handler31 chooses second interval;30first. Status underflow
still increments phase and dirty, so also delays timer. Reward arithmetic B14 only.

Jumping invisible/hit retry3C/reaction. Initvertical/fractionX/localstate0,
speedinteger1,INCstate, horizontal PRESET1 (instruction bytes consumed as named
data),face. bit1rise21/count2; negativeapex DECstate,zerovertical,FALL samecall.
Fall1E/count2 unsupported clearsclocks; nonnegative rise also clearsclocks.
Then horizontal18/1B count2. Localstate0 contactpermission ->B14steal (phase latch),
nonzero skips future contact/theft. Check statebit1clear, retained HORIZONTALbit0,
verticalword0 ->INCstate,vertical0400. TestPlayerActorOverlap doesn't modify these
bits. Status::steal touches scratch00 and flags, not movement bits. No direction
reverse. The once-per-encounter summary applies only to Jumping, not Timed.

## 33 HitOnlyADB9..ADC0;34 PlayerLauncherADC1..AE13

HitOnly nohitreturns with no motion/contact/lifetime effects;hitreaction.
Launcher skips all work for playerDamage6/Death0D. Low0 freezesactor tick0,contact;
deniedreturn. Permission playerExternalLaunch0C,INCactorstate,tick7,playerspeed0400,
vertical0728,motionflags0,sound28. Immediate B05 damage is overwritten by launch
as source does. Nonzero actorphase doesn't contact again. After both, ifframe|tick0
sets tick80; otherwise retains timing. No visibility/hit prefix. Player queued
reaction unaffected (B13 later filter), protection/special timers not reset.

## Remaining integration obligations

B05 full movement/contact/motion integrators and preset recipe; B08 shared hit
reaction and handler08->09 transition; B10 live dispatch/allocation and immutable
paired presets; B03 clocks/palettecommit; B06 immediate action/protection/position;
B14 single status owner; B18 sound scheduling; B01 runtimefeedback must remain
separate. Unnamed identities remain address-derived: no visual species inference.
No emulator experiment required for these deterministic source branches.
Implementation, whole-game runtime fidelity and QA are all still pending.

Stage4 immediate audio integration: sounds are submitted at their source author
point through World::submit_sound, with retained submitted indices. This matters
for WaitingJumper's step31 before shared reaction35: the latter samples the NEW
producer slot, including a dropped/full-ring comparison. Native Game regression
checks that ordering, RNG word1 and no later replay. Fixture default keeps its
historical observation-only boundary. Special control is an explicit motion
borrow, committed before the next handler; terrain launch receives actual player
pose direction. Prior fixture signatures are preserved in B07-before-special-borrow.
