# B09 independent platform/carry trace (stage3; implementation pending)

Source `reference/source/game.asm` SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
`reference/reading.asm` SHA256
201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Physical offsets include16-byte header. Bank6 CPU8000 maps18010; fixed bank7
C000 maps1C010. Source instructions and local ROM independently reread, not
inferred from planning. Stage3 adds declarations/executable expectations only.

Actual manufacturer reference independently opened: MOS-authorized
[Synertek SY6500/MCS6500 manual6500-50 August1976](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
sections2.2.1/2.2.2,4.2.1,7,10.3/10.7/10.8. Byte ADC carries and SBC borrows
through each word/page; LDA/STA/PLA preserve carry. CMP sets carry on unsigned
>=, AND leaves carry, ROL feeds old carry into bit0 and old bit7 into carry.
INC/DEC wrap bytes; BPL tests sign bit rather than an arithmetic underflow flag.
These facts inform native quantities/policies, never a CPU/RAM simulator.

## Dispatch, shared owners and all entry ranges

Independently decoded67split pointers802C/806F at physical1803C..180C2.
B09 nine targets:0C90EA,0D916C,0E9225,0F9093,1D9C46,219EB3,24A024,
38F9F8,39FE69. Physical respectively190FA,1917C,19235,190A3,19C56,19EC3,
1A034,1FA08,1FE79. LauncherADC1 is selector34 owned B07 (not a tenth B09
selector); B07 launcher contracts independently match reviewedADC1..AE13.

Source regions55719075..19225 include B15 progression9065..9092, then all
B09 vertical/walk/flight;59819225..19235 contains14flight selectors and2FFsuffix.
55819235..192FD allridecode;599192FD..19323 contains36packed steps+2durations.
56919C56..19D96 allpushcode; push offsets19D96..19D9A adjacent B07region data
still required B09 consumer.57319EC3..1A075 includes B09carry9EB3..9F3C and
externalA024..A064, interleaved B07jumping/leftwalker (reviewed, B07owned).
12551FA08..1FA4F all lift.12601FC0F..1FEE6 has B15/B17 bodies before
B09FE69..FED5;12901FEE6..1FEF0 phase/stepbytes plus2FFsuffix. No original
ledger identity changed; mixed regions do not transfer adjacent family ownership.

DB6C calls B06player update before B10ascending live1..15 slots. Player jump
escape therefore changes current action before carrier runs; do not recopy a
snapshot of A/B or9. DBE4 later clears camera displacement and changed latches, then
filters queued player action (B13). B03 advances animation after family decisions;
render/inspection never advances it. ActorContact canonical pool owns physical/
animation/localfields, Player owns action/queue/control only. World callbacks
borrow B05/B08/B10 plus their dependencies and COMMIT changes before return.
Effects are observations/requests, never a deferred replacement for shared writes.
B13 completion is a borrowed byte, not a retained second progression store.

Contact F43E..F497 is caller permission, not pure geometry: render must negative,
profile&7F selects extents (strict Y, inclusive X), animation03 adjusts Y by8
because CMP equality carry survives ADC07. Profile&70 nonzero plus specialmode
increments actorhit and DENIES permission; otherwise protection can deny or
immediately set action6. Profile&70==0 returns permission without action gate.
Family eligibility and overwrite rules below apply AFTER those immediate effects.
B05 wrappers preserve terrain/contact flags, removal/release and snap effects;
B09 cannot replace them with overlap booleans or a second collision cache.
B10actor overlap80C1..8149 chooses highest visible slot, changes source animation
and potentially targethit; push policies inspect actual returned handler afterward.
DistanceF7CC/F7DA is byteY magnitude/lowbyte magnitude of16bitX subtraction;
Xhigh only selects negation, not a true distance saturation. Far pages can alias.
FaceF6BD uses full16bit equality to direction2, strictly left to1.

## Vertical9093..90E9

State&0Fzero loads positive0180 then SUBTRACTS encodedY viaF286. If postY>=68
return immediately: no proximity or reversal. PostY<68 increments FULLstate
and joins descending tail without a second integration. Nonzero phase loads0180
and ADDS encodedY viaF270. Tail uses lowXdistance<1D and byteYdistance<18;
if both, player fraction ADC00 with CLC leaves fraction unchanged and integer
ADC02 adds exactly2, wrapping byte. No action, protection, visibility or contact
profile gate. PostactorY>=A8 decrements fullstate AFTER carry. Exact thresholds
and fraction/page/byte wrap are explicit contracts; no clamp or inferred reload.

## TriggeredWalking90EA..916B

Phase0 subtracts actor16bitX from player, rejects borrow or lowdifference<10.
Highdistance need not zero:100 difference rejects,110 activates. Activation
clears statebit10, zeroesYvelocity, speed0180,direction1, fullstate++. Samecall
contact(permission discarded but immediate effects retained), Fall31/count2.
Only finalflags&1 enables horizontal. Direction&1 positive34/count2 else negative
37/count2. Posthorizontalflags&1 XORdirection03. Zeroanimationtick sound27 only
on grounded horizontal path. Airborne skip sound even tick0. Wrapper removal
and later branches remain source ordered (no invented lifetime early-return).

## PresetFlight916C..9214 and data9215..9224

Nonnegative render OR nonzerohit sets special cooldown3C BEFORE B08reaction8932.
B08owns offscreen clear versus visible launched reaction, sound35/freshB18producer
CMP88/RNG/secondary/fullpool, and samecall launched motion. No duplicate reaction.
Visible/nohit phase0 incrementsfullstate; stores preceding AND-result0 to local
countdown/cursor, faces fullX. Directionbit2clear ORrender40; opposite does NOT
clear preexistingflip. Countdown0 loads paired B10preset selected by14flight
entries, normalizes negative horizontal word with two-byte negation (8000 stays
8000), cursor++ resetting only when==0E; countdown5. Samecall DEC ->4, so each
preset is used for five updates. Nonzero countdown only decrements/moves.
Inline Y ADDS signed8.8 with fraction carry, byte wrap allowed and no Ylifetime.
Direction&2 chooses F234positive orF252negative unsignedmagnitudeX. Actual final
carry/borrow reports full24bit overflow/underflow, not ordinary pagecrossing.
Outside calls F179clear/render/state/identity; matching occupiedspecial becomes
cooldown3C, otherlatch retained. AcceptedX zeroanimationtick sound2B, then contact.

## Rideable9225..92EC and packed36program92ED..9312

Phase0 zeroescountdown/cursor AND ownXYfractions, fullstate++. Animationtick0
requests26 before loading/moving/gatingplayer. Countdown0 fetches packedstep,
low5 selects pairedpreset. PHA/JSR/PLA/ROL/ROL/AND1 yields ORIGINALbit7 for
[0F,05]duration, independent incomingcarry or preset flags. Cursor increments,
resets at24 (36entries). Samecall DECduration ->14/4. Bit5/6 do not affect selector
or duration. Nonzero countdown decrements, no preset reload.
Inline Y adds encoded signed8.8; X adds fraction/lowword/high with signextension
of speedintegerbit7. This differs from F234unsigned magnitude and never removes
on wrap. Playeraction6/D skips ALLcontact/copy after own motion. Current9 skips
contact/UP and copies actorintegerX/Y immediately without fraction/velocity/action
or animation changes. Others contact; permission and heldUP required. Successful
attachment sets9,playeranimation4,frame/tick0 then integercopy; failedUP still
retains contact's immediate writes. Current0 after B06waitA escape can reacquire
if source gates permit; no stale9 unconditional copy. Existing9 from another
ride actor can be overwritten by later live slot, source has no owner latch.

## Push9C46..9D85 and signedoffset9D86..9D89

Rendernonnegative sets cooldown3C/reaction; visiblehit first setsanimation58 then
same reaction. Normal always Fall4A/count2; nofinalsolid returns. Phase0 writes
animationtick0 TWICE (does not zero frame), distance<50 initializesYvelocity0,
localstate/counter0,speed0100,facesplayer,fullstate++; RETURNS this update.
Nonzero phase statebit2set -> animationevent; otherwise directionbit1 selects
positive44/count2 ornegative47/count2 before contact/brake.

Localcounter0: nohorizontal solid returns. Solid calls B10actoroverlap. Nooverlap
increments SPEEDINTEGER (wrap preservingfraction) and localcounter. Overlap first
setsanimation58, then returned handler08 stores targetslot inlocalstate, fullstate++,
animation++=>59,localcounter++,frame/tick0. Direction&2 indexes signed{-2,+2}
word offsets, full16bit carry/borrow with Xfraction unchanged. Otherhandler also
speedinteger++/counter++ after overlap sideeffects. Target identities/flags are
not snapshotted or synthesized.

Localcounter nonzero: localstate nonzero returns (includingFFsentinel). Zero toggles
statebit40 and subtracts0008 fromunsigned8.8. Decision is postINTEGERsign BPL,
not borrow:0008->0000 stays,0000->FFF8 triggers,8008->8000 also triggers.
Onnegative setslocalstateFF,speedfractionFF,integer02 =>02FF, XORdirection03.
The existing comment 'speed two' is imprecise. No anotherbrake whileFF.

Animationevent checks frame5 FIRST regardlesslocalstate. Frame5 decrements full
state andanimation,zeroesframe/tick, setsONLYspeedinteger02 (fraction retained),
localcounter++. Otherwise localstate0 returns; nonzero andframe3 copiesdirection
into PlayerPoseDirection, uses targetslot and JSR869D; restores source slot,
clearslocalstate even if targetgate denied. Nonzero animationtick does not block
frame3 repeat; clearedtarget prevents second call. Frame3 targetactivation is
IMMEDIATE before later live targetslot; B08changed counter controls later request.

869D actual entry begins LDA TerrainChangedThisFrame: bypasses overlap/swingframe
ANDanimation3F specialgate. Nonzerochanged returns. Zero increments changed,
ifstatebit10 clears it then ReplaceTerrainActorGate, then86B5 loadslaunch tuple,
setsdirection from PlayerPoseDirection, state local viaanimation-1A, motion0,
handler++. B08launch_terrain interface86B5 intentionally bypasses this gate and
CANNOT stand in for869D. B09World.activate_terrain preserves distinct boundary.

## Carry9EB3..9F3C

Phase0 callscontact; deniedreturns unchanged except sharedcontacteffects. No own
state6/D exclusion! Success overrides currentaction withA,actoranimation3E,
player8B,allboth frame/tick0,fullstate++,Yvelocity0,speed0180. Falls samecall.
Everyactivecall localstateB, Fall31/count2. Finalsolid1 invokesnegative38/count0
thenlocalstateA regardlessposthorizontalbits. Only CURRENTplayerA/B permits
writing actionlocalstate and playerY=actorY-0B (bytewrap), actorintegerXcopy.
Player fractions/velocities retained. Tick0 sound33 only insidecarriedcopy.
After B06player jumpescape setsAirborne before actor dispatch, carrier still
moves but must not recopy player or sound33. Hiddencarry source state0 can become
inactive nextframe; family cannot invent persistent rider ownership.

## ExternalA024..A064

Phase0 onlylowXdistance<4 loads0180/fullstate++; then samecallpositive35/count0.
Postflags&1 nonzero invokesPOSITIVEWRAPPERAGAIN WITHOUTRELOADING A/Y. A equals1
from AND; ordinaryclass1 no-special path Y0, class3response can leave another
Y. Typed MotionFacts.continuation_remaining is actualreturned caller quantity,
not oldsamplecount and not CPUregisterstate. Secondstart1/countreturned required;
then handler0 regardlesssecondbits. This subtle call also remains meaningful
for added terrain, not an excluded original-only shortcut. B05adapter must
returnthe actualpostcall residual count, including specialresponse clobber.

After optionalsecondcall computepostmotion lowXdistance<15 ANDYdistance<15;
INCexternal_motion byte (FF->00). Noaction/protection/visibility/hit gate or
assignment-to1. B06consumes latch at E555 only when its ordinary horizontal branch reaches it;
DBE4 clears CameraDisplacement, NOT PlayerExternalMotion. The trigger survives
that tail and may wait through actions which bypass E551. Preserve this field
distinction and do not invent a per-frame boost queue. Handler24positivewrapper skips handler8scan;
negativewrapper exemption differs (B05owned).

## LiftF9F8..FA3E

Phase0 callscontact first, rejectsdenial then checksCURRENTactionD (contact
could have changed it to6). Success fullstate++,playeranimation4/frame0,
copyintegerX only,actionE. Activephase neverrechecksaction orX andcanoverride
playerY afterjump/reaction. Alwaysplayertick0; actorY-=2 withbytewrap thenplayerY
copy, cmp10 BCSreturn. StrictpostY<10 incrementsSceneCompletionSignal EACHcall
includingFFwrap. AtY0/1 subtractionwrapFE/FF doesnotcomplete. NoYfraction,
velocity,actoranimation orotheractionreset. B13observescompletion immediately
afteractorloop, B06Hold onlyfreezes playeranimationtick at its earlier boundary.

## FourPhaseFE69..FED5 and dataFED6..FEDF

Phase0 localphase/count0,speedfraction0/integer1,fullstate++. Count0 usesphase&3,
loads directions[2,1,1,2],count30,fullphase++ wrap. Direction&1 selectspositiveX
else negativeX; returncarryignored (doesnotremove). Samecallcount--->2F.
Eachchoice48calls; count1->0 stilloldchoice, nextcallselectsnew. Profile8A
installed AFTERownX beforecontact. Permission causesplayerintegerX add signed
step indexed(direction&2)/2: +1 or-1 withcarry/borrow through16bits, fraction
untouched. Add to CURRENTplayerX aftercontact sideeffects. Regardlesspermission
restoreprofile84, not previousprofile. Noaction/protection/visibilityowncheck;
B05owns contactdenial. Two platforms can carry sameplayer in liveorder; no
per-frame deduplication. Data fourdirections, two signedword steps and2FFsuffix.
Catalogrecipes decodeboundedrequiredbytes, do not execute suffix/instructions.

## Adjacent consumers and implementation obligations

B07selector34ADC1..AE13 launch reviewed: current6/D skipsall;phase0 freezes own
tick0,contact;permission setsC,speed0400,velocity0728,motion0,ownphase++,tick7,
sound28. Usedphase onlyfreezes tick80 whenframe|tick0. B06C phase owns rise/fall
order/page7 andhorizontalobstruction continuation. Existing B07/B06contracts
own executable launch assertions, no duplicate B09launcher or player state.
B07contactpose85D9 and B16departureF869/B17goalFD9E similarly immediateexternal
writers but retaintheircohesiveowners. B06A/B/9/E own inputescape/hold/terrainhit;
B09ownsonlyeligibility/copy/detach/callerpolicy. B10slotorder/allocation/suppression
remaincanonical; B09has no directallocation/suppressioncall. Only sharedB08reaction
allocates; B08new869Dentry contract dependency must be reconciled rather than
mistakenly delegate86B5. No newsystemorcohesivearea discovered.

SyntheticWorld fixtures return specified boundaryfacts and record order; inline
signedintegration/phase/copy/branch arithmetic is exercised in pendingB09step.
No CPUinterpreter, RAMmap, secondgameoracle, embeddedcommercialpayload, runtime
emulator/QA/devicework or implementationclaimed. Nativecatalogshortinputrejection
is source-informed loader safetyconsistent with B02/B10; valid matchingROM all
programentries/pairedoffsets checked. Invalid localstates outside supportedsource
producerdomain are not invented gameplay; wrap cases test actual byteoperations.

Stage4 fixture correction: flight_initializes_counter_faces_player_and_sets_positive_visual_only
now supplies the ordinary negative-X carry=true explicitly for its direction2
case. Its prior defaultfalse simulated a borrow/removal at91FF/F252 and
contradicted its expected READY state. Assertions/test identity unchanged; the
prior file is checks/implementation/contract-corrections/B09-before.rs.

Stage4 composed boundary receives canonical SpecialControl during movement;
shared reaction observations retain already-submitted sound indices. Direct
sounds submit at author point through the optional native World hook. A true hook
result means attempted (including dropped), never merely accepted. Prior fixture
signatures are preserved in B09-before-special-audio-borrow; assertions unchanged.
