# B11 independent whole-area source trace (stage3 handoff; implementation pending)

Frozen game.asm SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Matching local ROM97d472f7b2c1088448d1644155b2650a546bad887b3cba571e16d7986824c169.
Bank6 CPU8000 maps physical18010; fixedC000 maps1C010. Instructions, tables,
callers and aliases are independent evidence; planning identities stay unchanged.
Controlling contract/workflow/common/standards and existing canonical owner
interfaces/traces were read. All declarations remain visibly unimplemented.

Actual MOS-authorized manufacturer document opened: Synertek SY6500/MCS6500
Programming Manual6500-50 August1976, https://syncopate.us/books/Synertek6502ProgrammingManual.html,
2.2.1/2.2.2,4.2.1,7.4..7.9,8.1/8.2,10.1/10.2/10.7/10.8. ADC consumes carry;
CMP/CPY equality sets carry, SBC uses inverse borrow, DEC/INC preserve carry and
wrap, AND/LDA/STA preserve carry. Native semantic arithmetic inputs are local
increment facts, never CPU execution or an emulated RAM/register map.

## Corrected entry identity, important superseded inference

Named PresetPath/Land/FireDelay labels each bind a preceding RTS. They are not
actual table entry addresses. Independent matching-ROM reads of split table
A585/A58A prove dispatch [A357,A386,A3EC,A495,A4C7]. A3EC/physical1A3FC starts BD;
A495/1A4A5 starts A0; A4C7/1A4D7 starts BD. RTS bytes are immediately BEFORE at
A3EB/A494/A4C6. Therefore phases2/3/4 execute complete motion/land/fire bodies.
The early working inference of inert phases is explicitly superseded by this
byte/table check. Direct branches to those labels return, while indirect table
dispatch enters after RTS. In particular ChooseHeight BCS returns atA3EB; its
successful height change falls through the same return. Preset terminal branches
return atA494; Land miss returns atA4C6. Do not fix labels by changing behavior.

## Canonical state and call boundaries

B10 ActorPool is sole16-slot store. Existing local_aux2 corresponds06D0;
sequence_cursor names06E0; boss_protection names06F0. Ordinary spawn clears06F0
(not06D0), requiring B10 reconciliation. B06 owns Player; B01 RandomState;
B04 Camera/WorldCache; B03 split controls/animation commit; B13 shared bossHP and
impact latch; B08 reaction/secondary/treasure pickup; B18 audio queue. Borrowed
Context fields require immediate writeback. B17 consumes full shared hit result
but owns its own phases. Terrain initialization writes only collision cache
cells, no tile upload or changed-counter increment. B10 live ascending dispatch
means reserved children in later slots update this frame, earlier ones next frame.

Motion/integration/player contact calls commit before following local conditions.
World fixtures inject those facts, never duplicate collision algorithms. Reaction
is B08-owned visibility8932 followed by actual35 sound/secondary/same-call launch;
its B18 queue producer must be sampled at that call (88acceptedC1,<88occupiedC0,
FFoccupied-dropC1), never a Context copy. Animation advances later in B03.

## Shared hit F6D7..F751 physical1F6E7..1F762

All five first instructions remain original metadata1019..1023 ownedB10, but
semantically the FULL helper belongsB11. CLC; if06F0 zero DECsharedHP wraps0->FF.
Resultzero calls treasure and returns defeattrue, no flash initialization. Other
HP initializes06F0=78 and ORrender02; marker remains. Nonzero06F0 decrements once
per marked helper invocation, not global time; zero expiry clears marker and
render02 only. All surviving/protected paths carryclear. Callers only invoke for
nonzero marker, so unmarked intervals do not decrement here. Five land boss
prefixes continue full local logic on nondefeat; on defeat delegate B08 reaction.
ATTACK's LDA5E before8932 is overwritten by visibility's LDArender, so does NOT
rewrite boss animation. IMPACT restores splitcontrol0/splitlowC0 before reaction.
B17FA44 calls same helper; animation84 final defeat bypasses allocation and
returns carryset, then B17 alone changes animation2A/handler3B/stateA0/profile0.

Normal treasure F702 saves originalslot, scans15..1 for inactive SIGN, fullpool
uses1 even if occupied; player0 never selected. Writes selected YB8, Xlow80,
XhighcameraHIGH ignoring cameraLOW; hit/tick/frame/profile/counter/protection0;
stateB1, renderB0, animation1B, handler14, counter3C (overwrites earlier0).
Fractions, speeds, localstate,06D0/06E0, motion/direction/identity/spawnY/restart
remain stale. Returns originalslot, defeattrue. If originalboss is1 with fullpool,
it is overwritten BEFORE caller reaction, so reaction sees1B/profile0/stateB1.
No audio itself; B08 reaction owns35, B08 collection/B14 owns later reward.

## ATTACK26 A335..A585, PROJECTILE27 A5BE..A5C9

Prefix marker tests fullhit; state low nibble dispatches5entries above. Opening
writes cache05EF(index6F)=44 everycall, tick0, INClocalstate, reveals only on
postincrement3C, including bytewrapFF->0 nonmatch. Reveal state increments,
animation2A, tick/frame0, vertical0380. Height2A waits frame!=0 AND tick==5;
then5E, tick/frame0, RNGword1=word0+word2+1 (CMP5 C1), low2 selects4heights,
returns. Otheranimation integratesnegativeY; Y>=target returns via precedingRTS;
Y<target has CMP C0, word2=word1+word3, low3 selects8targets, parity selects
2initialdelays/directions; clears06D0/06E0, INCphase and returns.

PresetactualA3EC: decrement delay if nonzero.06D0 hold0 loads one of14presets,
normalizes negative horizontal16bit to magnitude, increments cursor modulo14,
sets hold3 then decrements (so loads every3calls). Y += signedvertical with
fraction carry (opposite ordinary integrator). Directionbit1 chooses negative1B
or positive18 wrapper with count2. Expireddelay and lowbyte absolutetargetdistance
<4 advancesphase and vertical0380. Distance ignores worldpage; negative magnitude
uses SBC C0,EOR,ADC1. Hold0 plus motionbit0 reverses direction XOR3; state advance
does not skip final wall check. Following RTS prevents samecall Land.

LandactualA495 positiveY1E/count2; nofloor returns atA4C6. Floor sets2A,
tick/frame0, INCstate, FaceActorTowardPlayer(full16bit), renderbit40 from table
indexdirectionbit0, then returns. FireactualA4C7: reveal waits frame!=0/tick5;
sets5D,tick/frame/counter0,delay0F,returns. Otheranimation with nonzero delay
DECdelay,tick0,returns even when expires. delay0/counter0/frame2 unconditionally
OVERWRITES slots10,9,8 (no allocation): state/renderA0,5F,tick/frame0,Yboss,
Xlow=parentlow+offset modulo256, SAMEparentpage ignoringcarry, directioncopy,
parallel3vertical/3horizontalwords, handler27/profile1D; preserves every unlisted
field including marker/fractions/local/protection/identity. Audio1C once,delay1E,
INCcounter. counter!=0 branch clears lowstate then jumpsA36B (phaseINC/animation/frame/
velocity opening-tail initialization, not complete openingcache/counter test).
Projectile27 contact BEFORE fallingYintegrator; carryclear clearsstateonly,
retainsrender; carryset directionbit0 selectspositiveX else negativeX, no gravity.

## IMPACT2E A5DB..A6C9 physical1A5EB..1A6D9

Phase0 clears counter, YAC, delay1E, horizontal0100,direction1,cache05AF=44,
INCstate then SAMECALL ordinary horizontal movement and delayDEC to1D. Delay>0
skips vertical; directionbit0 right61/left62 count0, then DECdelay; expiry sets
vertical06B7 and fullXfaces player, then prior motionbit0 reverses. If delay0,
statebit1 clear Rise64/count0; nonnegativevelocity continueshorizontal, negative
sets nextphase,vertical0 then horizontal. Falling counter0 Fall63/count0;
no floor continueshorizontal, floor audio1E, INCimpact, setcounter1E and DEC to1D
samecall. During nonzerocounter display only: phase=(compositiontick>>2)&1,
control=phase, low=table[phase]. No X movement during shake. counter1 expires to0,
DECstate, RNGword0=word0+word3+incomingincrement (INC/DEC preserve prior carry),
low3 picks all8delays; splitC0/control0/impact0; returns nohorizontal. Entry after
marked-hit helper has incrementfalse; normal B10 slots2..15 CPX16 establishes
false; slot1 caller residue is explicit integration obligation, not global RNG.

## PERIMETER28 A6D3..A8AC physical1A6E3..1A8BC

Iteration013 decodes the complete20-node perimeter program at asset loading.
Each PerimeterStep owns a named horizontal/vertical heading, correction choice
and ContactGate (Neither/Horizontal/Vertical/Both or an unreachable source mask).
Gameplay no longer reads packed direction records or interprets collision-mask
bytes. Mirroring is a heading operation; actor direction/motion projections still
feed the unchanged shared helpers. Current runtime motion flags remain separately
observable state and are not replaced by an assumption that fixtures/callbacks
always match the current catalog node.

The wave program is likewise decoded into Spawn{x,at_tick}/AwaitLastChild
instructions instead of parallel X/time arrays. The source checks zero-X before
reading its parallel timer byte, so the unused endpoint-time value is omitted
rather than retained for an obsolete layout assertion. Spawn timestamps remain
exact byte equalities, with same-call equal-time child emission and one tick
increment only on mismatch. The last-child any-nonzero-state gate, camera shake,
partial child initialization and reset/bounce order are unchanged.

Prior source-backed contracts retain their expected gameplay values. Catalog
metadata tests compare decoded used path bits and gates, and independently
source-derived Spawn/Wait instructions. Every source mask and both axis results
were checked before conversion; all20 nodes/both orientations, paired wave
spawns, time wrap, endpoints and last-child behavior retain integration coverage.
Plain motion-preset indices remain asset identities; this conversion removes the
packed control/parallel movement-program machinery rather than renaming those IDs.


Phase0 animation2A waits nonzeroframe/tick5 then animation62,tick/frame0,
profileOR80,returns. Otheranimation clearsvelocity/localstate/motion,YB0,Xlow80
preservingpage/fractions,cache05E7=55,INCstate then Fall31/count2 SAMECALL.
Opening motion0 falling: nofloorreturn; floor sound22,vertical02D4,motionINC,
localstateINC. Thirdpostincrement setsphase2,RNGword1=word0+word1+1(CMP3 C1),
direction=(lowbit)+1 saved as orientation,2A,tick/frame/counter/06D0zero,
bothspeeds0398; directA890 decodes node0 WITHOUT increment. Otherbounce Rise52/
count2; negativespeed clearsvertical,DECmotion,returns. phase2 reveal gate then
63,tick/frame0,profileAND7F and SAMECALL contact + vertical31ifmotionbit2 else
52 + horizontal34ifdirectionbit0 else37, eachcount2. Verticalbit0 shifted gives2,
horizontalbit0 gives1; masknonzero AND filters, mask0 demands combinedzero.
Equality advances; zero-mask applies signedYcorrection selectedmotionbit3,
integeronly wrap; nextcursor modulo20,06D0INC on wrap, facing pathlow2 XOR3 for
orientation!=1, motion=path&0C. Inequality + circuitcountnonzero + absoluteLOW
X-80<3 resetslowstate0/reveal; no true16bitdistance or auto reset on equality.

## WAVE2A A8D6..AAA8 physical1A8E6..1AAB8

Phase0 clearslocalstate/counter/tick/frame, horizontal01CC,direction2,
cache05A0=11,vertical06EE,INCstate then ordinary logic SAMECALL. Counter>0
freezestick,DECcounter, incrementsframe onlyonexpiry,returns withoutmovement.
Rising localstate0 requires frame2; freezestick,Rise52/count2; negativevelocity
setsvertical0/localstate1 and RETURNS (nohorizontal); nonnegative thenhorizontal.
Falling localstate!=0 holdsframe2tick0, Fall31/count2; nofloor horizontal;
floorbit0 A=1 compared toframe, unequal storesframe1 but BEQafterSTA does NOT
branch because Z is fromCMP and remainsfalse. Thus it reaches tick/soundcheck:
framealready1+tick0 OR changedframe+tick0 both queue1E. Tick!=7 RETURNS without
horizontal; tick7 setscounter1E,vertical06EE,localstate/frame0 thenhorizontal.
Only wallbit0 WITHcounter!=0 reverses,INCstate,65,tick/frame0,audio23.

Wave phase2 animation!=64 waits nonzeroframe/tick7; selects64,frame/localstate0,
word1=word0+word1+1(CMP7 C1), lowbit pickssequence0or7,06D0=8. SAMECALL emission
clears tick, cameraLOW=shake[(compositiontick>>2)&1], pagepreserved. At each
cursor, X0 sentinel waits. Otherwise localtick==spawntime directly overwrites
slot06D0 (starts8), Xlowtable/page4,Y38,state/renderA0,animation47,tick/frame0,
profile10,handler2B,vertical0180; allunlistedstale. INCcursor/nextslot; nonzero
cursor LOOPS SAMECALL, enabling equal-time paired children. On first mismatch
INClocaltick once andreturn. Originalsequences each6children slots8..13, NOT8.
At sentinel, source06F0+15+nextslot aliases0700+(nextslot-1): reads **state of last
spawned child**, not protection or allchildren. Any nonzero (even nonactive)
state waits. Lastchildzero DECphase, clearlocalstate/counter/06D0/cameraLOW,
jumpA90D resetsvertical06EE then phase1 logic. Earlierchildren may remainactive.
Wavechild2B positiveY2D/count0; floorclearsstate ANDrender, then playercontact
STILLruns afterdeactivation. No gravity or animation advancement locally.

## CENTER29 AAE1..AC37 physical1AAF1..1AC47

Phase0 clearsvertical, Xlow80,cache05AF44,INCstate,06D0=0F,animation67;
SAMECALL delayDEC gives0E. While delay>0, lowX CMPplayerlow chooses facingflags
(index0 if>=else1), preservesother renderbits,returns. Expirycall stillreturns;
nextcall selection. Ifstatebit1 clear INCstate, clearslocalstate/counter/tick/
frame,66. word3=word0+word1+incomingincrement; direction=(lowbit)+1;
word1=word2+NEWword3+firstsumcarry (not carryfromINC or direction), lowbit selects
speed03C0/01CC and localstate0/1. Jumpindex1 then INClocalstate to2,vertical06EE,
68. SAMECALL motion. localstateeven!=0 Rise5E/count2; negativevelocity DECphase
andclearvertical thenhorizontal. localstateodd Fall5B/count2; nofloorhorizontal;
floor+counter0 relaunch0548,INCphase/counter,XORdirection3,RETURNS. Subsequent
landing counter!=0 immediately goesidle. Otherhorizontal55/58/count2; wall
XORdirection3,INCcounter,RETURN. Groundphase0 +counter!=0 + absoluteLOWX-80<4
goesidle; distanceboundary4 waits. Idle setsXlow80,DECcounter preservingwrap,
DECstate,67,tick/frame0,delay1E. Does NOT clearlocalstate/speeds; nextselection
clearslocalstate afterdelay. Lowbyte distance aliasesdifferentpages deliberately.

## Original route/template and allocation consumers

Independent parallel arrays B050/B059,B062/B06B,B074/B07D,B086/B08F,B098/B0A1
and121templates BE16 were read, not inferred from appearance. Landselectors0..4
join respectively (row,identity,X,Y,template DECIMAL index,handler HEX):
0=(1,11,0428,AC,80,2E),1=(3,27,0528,B0,81,26),
2=(2,55,0530,90,82,28),3=(2,32,04D8,B0,83,2A),
4=(5,159,0430,B4,84,29). Their templatebytes are respectively
A0/60/2E/9F,E0/5D/26/96,E0/62/28/95,A0/64/2A/95,A0/66/29/9E.
These prove allfive natural source landroutes; species labels are unnecessary.
Children27/2B are directspawn-only and do not require ordinary templates.
The67handlerplane entries give allseven exactfamilytargets, contracted through
B10 catalog join. Selectors5..8 empty and9ending remainB10/B17 separation.

B10 ordinary AF61 clears06F0 protection, localcounter and marker but retains06D0/
06E0. PriorB10 test mistakenly treated its existinglocal_aux2(06D0) asclear;
correctedinplace with evidence and preservedpriorchecklogs. SpecialB105 clears
localstate/counter/marker/tick/frame but retains ALLthreeauxiliaryplanes and
spawnY; strengthenedsamecase asserts that distinction. No blindly global rename.
BossHP5 is assigned atAFF5 only on negative LateStageMode path fromordinaryspawn;
alllate-routeHP initialization/reset andmodegate are B10/B13 callerobligations.
No invented per-family independent HP or unconditional spawn reset is introduced.

## Caller arithmetic inputs and finite data boundaries

Every banked8000 sourcecall was reviewed incontext: bank0audioCxxx,bank5render
atC9FB,D02x,D5xx,D6A9,D7xx,DB4x,DBC5,DDxx,DECB,DF16,F85x are notB10dispatch.
Gameplay DB6C alone selectsbank6 then8000; preceding UpdatePlayerProtection
includes selected B06action. BankselectFFDE usesSTA/TAX/STA/RTS preservingcarry.
Firstslot1 has no comparison in8000beforehandler and inherits player's final
arithmetic residue; slots2..15 CPX10 clearcarry. B11 contracts exercise both
possible localincrementinputs, plus hit-helper overrideCLC. B06/B13 must supply
the owner-backed final arithmetic increment for slot1 atcomposition; this is a
narrow domain input to Impact/Center RNG, not a globalCPUflags store. No implicit
feedback into B01's globaladvancealgorithm. Withincenter secondADC explicitly
consumes FIRSTADCoverflow despite intervening INCdirection (carrypreserved).

Attackrecipe: A585..A58E10splitphasebytes,A58F4heights,A5932delays,A5952directions,
A5978targets,A5A0..AD14presets,A5AE2offsets,A5B02facing,A5B2/5/8/B3eachspeedplane.
A59F'sextra50has no tracedconsumer; preservedgenericdata, no invented ninthtarget.
ImpactA6C9..D08delays/A6D12splitbytes. PerimeterA8AC20program/A8C020masks/A8D42
signedcorrections. WaveAAA814X/AAB614times/AAC42choices/AAC62shake. CenterAC372
facing/AC392lows/AC3B2highs. Everyrequiredbyte read bounded throughROM-backed
Catalog; no extracted payload is embedded. Testfixtures are authored independent
parameters. Everyoriginal14motionselector resolvesB10preset0..31; B10owns paired
wordloader, B11 signnormalization. No sourceinstruction execution/secondoracle.

## Coverage and readiness

Contracts call actual native pending APIs, using independently supplied motion/
contact facts and exact state/output expectations. Originalfinitecatalog/route
contracts access matching localROM; boundarycases use syntheticstate, not assets.
Allsevenselectors,allfiveattackphases,localstates/timers/animationevents,
normal/alternatecontact/distance/sign/wrap/fullpool/directreservedoverwrite,
RNGincoming/chainedcompareincrements,sharedsurvive/expiry/defeat/final84 and
same-slotfallbackordering covered. Source/allconsumer index is
planning/contracts/B11-consumers.json. Nonewcohesivearea discovered; B17retains
itsphaseowner, B08reaction andB14reward are borroweddependencies. Noemulator
questionneeded afterindependentROMentryverification; supersededRTSinference
was ananalysiscorrection, not anunavoidableassumption or sourcechange.

Final native compile/discovery/area/B10-focused/infrastructure outcomes and all
exact test names are planning/contracts/B11.md and checks/contracts/B11/results.json.
B11-first-slot-random-increment remains pending in uncertainties.json; bothinput
contracts are not proof of exact producer integration. No fidelity/QA claim.

## Stage4 dispatch and synthetic-boundary corrections

Immutable split dispatch planes1803C/1807F resolve selector28=A6D3(Perimeter),
29=AAE1(Center),2E=A5DB(Impact). The original B11 named constants incorrectly
rotated these three meanings despite correct routine source addresses; B10
native metadata was already correct. Native typed constants now preserve actual
numeric identities. The original finite-route tuples retain bytes2E/28/29 for
templates80/82/84 and now name Impact/Perimeter/Center respectively. Numeric
ledger identities, template bytes and all phase behavior expectations persist.
Superseded contracts/trace are checks/implementation/contract-corrections/B11-before.rs
and B11-trace-before.md. The earlier route interpretation above is superseded.

The boundary fixture now injects post_velocity only for vertical motion, as
F237..F26E horizontal integrators do not write Y velocity. Its original injection
on every horizontal continuation contradicted two apex-zero assertions; those
assertions and test identities are retained. No implementation velocity masking
is added to accommodate the old fixture.

### Stage4 canonical cache borrow refinement

The Game adapter cannot retain another whole-cache borrow while BossContext
already lends the canonical WorldCache. World.motion therefore receives that
same borrowed cache explicitly at each actual call. The mock boundary signature
was updated mechanically; all original expectations and test identities remain.
Prior fixture: checks/implementation/contract-corrections/B11-before-cache-borrow.rs.
This is an immediate owner-borrow refinement, not copied cache state.
