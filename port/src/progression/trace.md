# B13 independent source trace (stage3; behavior unimplemented)

Reference game.asm SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Fixed PRG bank7 CPU C000..FFFF maps physical1C010..20010. Switch calls use explicit
bank identities; physical offsets include the iNES header. Original labels and
ledger identities remain unchanged. Declarations/tests do not implement a CPU.

Consulted actual MOS-authorized SY6500/MCS6500 programming manual, Synertek
publication6500-50 August1976, https://syncopate.us/books/Synertek6502ProgrammingManual.html
(ADC/SBC chapters2.2, compare4.2.1, index7, shifts10). CMP/CPX carry means unsigned
>=; LDA/STA/TAX/INC do not clear carry; ADC includes carry, SBC inverted borrow.
A zero countdown decremented as a byte wrapsFF. Source does not use decimal mode.

## Canonical ownership and executable boundaries

Progression owns stage/progress/route and transition latches, selection/status
selector and dialogue cursor. Player owns action/queue/protection/special/motion;
ActorPool is the sole physical actor store. B14 Status owns lives, health,
countdown and dirty/UI flags. Camera/WorldCache stay B04. Systems borrows actual
presentation/audio/runtime owners; Frame borrows queues/guard/displacement views.
No second persistent RAM image or copied owner store is authorized. Progression
field views in B10 SpawnContext/B12 CompletionInput are temporary projections.
Stage4 composition root must refresh them at each source call, not keep divergent
copies. Native bank selection preserves source data context and has no mapper CPU.

Systems.player MUST return a nondefault FirstActorIncrement, after actual B06
selected action finishes, for B10 slot1 dispatch into B11 impact/center ADC. Every
possible B06 return path must derive it from its final reaching arithmetic or
compare; this is a concrete required integration result, NOT a false assumption
that all player paths return carryclear. FFDE STA/TAX/STA/RTS and first-slot B10
setup preserve it. Slots2..15 get Zero from CPX16 false; marked-hit helper F6D7
CLC resets it. B11 MotionFacts/random boundary retains this semantic fact locally.
No global CPU flag field or RNG reseed is permitted. Existing uncertainty remains
pending until actual B06 adapter derives every reachable return path; orchestration
contracts independently require forwarding both possible explicit results.

## Initial presentation / runtime D79B..D8A5, D9CA..DA0F

B12 ordinary confirmation and positive DB93 restart both enter actual D79B:
FadeOut, clear gameplay, Yield, disable/hide, clear actor1, fill nametable,
LoadStagePresentation, ClearNonplayerActors, BuildInitialTerrainColumns,
bank5 UI script0, selected CHR, second BuildInitialTerrainColumns atD7D1.
Only then D7D4 runtime starts; no shortcut directly into asset/player setup.
D9CA saves stage across QueueStageAudioCommand (F762 enqueues and remembers source
command), indexes nine-byte planes DA10/19/22/2B. CameraRoom/terrain room is packed
initial room; low3 supplies CameraXHi/player page/inrow; X from initial playerX;
row from initial rows. Stage palette uses selector*16 as byte X; copy16 background
bytes into base AND active and shared DAC4 sprite16 into base/active sprite halves.
Nine source configuration selectors0..8 are distinct from ending layout9 and
arbitrary named added levels. Dynamic extension capacity belongs B02, not a cap
on added named levels. Original timer data DB2C has five deliberate pairs; actual
selector5..8 reads following code bytes. B13 preserves those exact reads in source
recipes rather than inventing zero timers or declaring nine authored timer pairs.
Natural availability of nonordinary selectors is a separate unresolved invariant.

D7D4 sets BackgroundUploadState0 and polls C8D9 until negative; polling yields in
B04 owner and must not spin without scheduler progress. Bank6 AE20 loads tables
then AE60 resets spawn window. Split transition0, camera low0, split PPU24C0;
player state/renderA0, Y40, animation/frame/tick0, Xfraction0; jump velocity04BB,
horizontal magnitude0180. Retain player worldX/page from preceding presentation,
Yfraction and live vertical velocity, handler/profile/locals/spawn identity.
Copy13 UI bytes, submitFF, EnableRendering, Yield. Only after yield SplitSpriteY2C,
gameplay40, FadeIn. After fade LateStageMode/SpecialSpawnControl/action/scene signal/
PreviousCameraX0; contiguous16-byte controls QueuedPlayerState.. reset includes
momentum/external/protection controls but not arbitrary unrelated fields. Native
commit must reset their actual owners, not recreate an addressed array. Copy12
status template, restore health, copy29 timer sprite bytes, replace upper2 timer
codes from selector*2, divider60. Timer units comes from copied template, not from
an invented initializer. Actor1 state/render/spawn sentinelFF, Y/X/profile0;
source7F FF. Bank5 B760 rebuilds suppression before GameplayLoopEntry.

## Foreground DB36..DC2B (physical1DB46..1DC3C)

Ending latch nonzero calls B17, then joins composition; ordinary simulation is
bypassed. Otherwise player action0F skips START and status-selector checks and
enters player update. Any other action checks START edge only. START requests15,
XORs status selector1, hides, bank5 compose then B580 status streams. Selector
nonzero goes straight DC04 late-position check; selectorzero resumes normal player
simulation in this SAME invocation. This is source toggle/pause semantics, not a
new pause overlay or unconditional yield. Both values other than0/1 remain exact
XOR bytes. HeldSTART without edge has no toggle. Action0F bypass ignores nonzero
selector even when set before dialogue began.

DB6C always invokes B06 protection/special/action first. Its writes are immediate;
then reread dialogue cursor. Nonzero cursor skips actor dispatch, countdown,
scene/boundary handling but continues reload/status/UI/special/composition.
Otherwise bank6 actor dispatch occurs, followed by B14 CountdownStateGate. Reread
scene signal after both owners: nonzero enters BeginSelectionTransition (DF08).
If no scene signal, negative LateStageMode skips camera+boundary. Other modes call
camera then boundary; reread A4 request AFTER these calls. Negative request clears
A4 and enters land select; positive clears nonplayer actors, clears A4, sets stage1
and enters full D79B confirmation. Neither path reaches old frame-tail RNG.

Deferred effects reread each owner at call site: negative StageReloadRequest calls
reload; dirty status nonzero calls B14 refresh; UI latchAB nonzero calls B14 stage
UI; negative SpecialRoomRequest calls B16 setup. Do not snapshot all conditions
before prior effects, which can change subsequent flags. AB producer is successful
E34E overlay payment; queue waits preserve latch until primary upload free.
Special room is not just a bool: negative request enters setup; setup masks lowF,
retains resulting nonzero mode for later reload-to-selection branch.

DBC5 increments OAM guard, hides, bank5 composition, decrements guard. Composition
may clear actors/advance animation/trigger owner effects, so B10 spawning must
observe postcomposition state. Recheck ending latch then race intro: either
nonzero jumps straight DC23, bypassing spawn, previous camera, displacement/cache
clear, reaction, late position AND secondary submission. RNG/yield still occur.
Otherwise bank6 spawn; previous camera=current lowX; camera displacement0;
terrain changed0. This DOES NOT clear Player.external_motion (cleared by E555).
Nonzero queued action is consumed BEFORE filtering: only Damage6 is rejected when
protection!=0; every other action overwrites immediately despite protection.
Immediate B05 contact reaction was already applied during player/actor call; do
not defer that reaction or merge it into this queue.

DC04 compares full byte row then page with nine-byte arrays. Exact match sets
LifeLossVariant5; mismatch retains previous variant (no reset). FF sentinels are
compared literally if caller supplies row/pageFF. DC19 if secondary cursor!=0
copies its byte to pending then clears cursor; zero preserves old pending. DC23
advances random ONCE, yields once then next loop. Paused selector path includes
DC04+secondary, ending/race path bypasses both. No RNG advance belongs to NMI,
composition, nested reload/fade/dialogue/scene waits unless explicitly local caller.

## Boundary CF93..D04C (physical1CFA3..1D05D)

Five rows/pages/directions atD04D/52/57 are indexed by ordinary stage0..4. Compare
row first; on match add direction-bit1 selected offset: direction2 uses +0008,
direction0 uses FFF8 (-8), with low carry to high; compare page only, not fullX.
Stage1 with progress bit2 present but lowfive!=1F requests A4=80 and returns.
Allother matching stages set scroll direction, clear actors and enter horizontal
transition. Save original stage/row; allfive bits chooses temporary row5 and sets
LateStageRoute5, otherwise temporary row=callerY (original stage selector). Then
stage5 and BackgroundUploadState0. Directionbit1 chooses camera +1/-1 per tick
and player low8.8 +0040/-0040 respectively. Player page is NOT updated. Poll
background on every tick before checking CameraX. Nonzero camera low stores
previous camera, clears displacement, hides, bank5 compose/yield. Zero restores
original row/stage, stageaudio6, LateStageModeFF and palettependingFF, returning
without that tick's extra composition/yield. No OAM guard increment in this loop.

## Reload DC3E..DD4D (physical1DC4E..1DD5D)

Clear impact latchA7 before checking SpecialRoomRequest. Any nonzero special mode
enters selection transition; selection DF08 sees nonzero and returns immediately
without180frame delay. Otherwise request>=87 unsigned queues stageaudio0A and
checks lives before decrement: zero gameover; nonzero proceed. Request<87 queues
ordinary sound2D. Grant protection1; composed wait180 if>=87 else60; FadeOut.
Clear special spawn/late and twelve contiguous player controls; split control0,
PPU lowC0; disable; checkpoint; clear nonplayer actors. Keep original selectors
saved around temporary background upload. Only85 uses temporary stage5,row0,
lateFF; >=87 resets player animation/frame/tick, decrements one life and restores
health. A request87 with lives1 respawns at lives0; NEXT loss gameover, not before.
Poll background negative, restore row/stage, set scroll+previous1, bank6 AE60,
enable, Yield, gameplay40, FadeIn. RestoreAudio: negative late alwayscommand6;
otherwise currentstageaudio0A reloads timer upper2+units0/divider60 and queues
original stage; any other currentcommand leaves timer/audio unchanged.
Bank6 spawn, player flagsA0, SECOND composed wait chosen by original request.
DD2F waits return A0 (last SBC result), so DD1C stores0 protection, action and
reloadrequest. It does NOT grant180 protection after reload. Initial protection1
is not decremented by ordinary player updates during these nested waits.
DD2F composes under guard, Yield then freezes animationtick0 on EACH resumed
iteration. Zero source count would mean256 iterations; exposed count is u16 so
native lifecycle requests60/180 preserve actual waits without CPU registers.

## Checkpoints DD4E..DDC0 (physical1DD5D..1DDD1)

Low5 request multiplied9 via three ASLs plus ADC. Last ASL carry0 for0..31,
so stride is exact byte9n modulo256; named records only0..18 are physically
bounded before DE6C code. Natural reload producers: B15 contact selectors0..4
set80|selector, playerfall/death stage+87+variant produces7..15 for ordinary0..4
with variant0/5; special room B16 explicitly calls entryDD52 with selector+10
for records16..18 (request mask path is skipped). Request85 record5 late path;
other possible callers/invalid injected values must not be given invented records.
Read9bytes: row, camera low/page, count, room cursor, ring cursor, Y,X,packed.
Terrain inrow=room&7; playerpage=packed&7; camera room=(packed&F8)|camera_page.
Each iteration resolves current column unless room entryFF, then decrements count.
Negative result stops BEFORE cursor advancement; count63→64visits,95→96,127→128.
Ring wraps&3F; onzero increment roomcursor AND inrow as bytes. Missing rooms still
consume count and advance normally. Retain terminal current cursor at last visit.

## Game over and completion selection DEB9..DF26

Queue stageaudio9, copy33bytes overlay (8fourbyte records+FF). Exactly240 guarded
hide/bank5compose/yield iterations, then ResetInitialize. Gameover does NOT freeze
player animation tick after yield (unlike DD2F). Selection with specialmode0 uses
180 guarded compose/yields but does NOT hide OAM first, then InitializeLandSelect;
nonzero specialmode skips all waits. Reset clears progress via runtime reset, not
via invented save persistence; battery headerbit clear, default save hooks valid.

## Full producer/consumer closure and remaining natural entry uncertainty

Progress bits only runtime reset0 and bank6 CollectStageTreasure8C13 OR animation-
6F mask; no NMI/queue/scene helper mutates it. Ordinary6F..73 setlowfive then
playeraction10, B14 conversion eventually waits60 then increments scene signal.
Later74/75/76 set20/40/80 and dialogue0D; those bits are not five extra menu lands.
Scene signal comes timerconversion, linkeddepartureF84F and timedspawnerF8..
(their full state machines belong B16). B12 land entry initializes actors/fade,
then completion actor nonzero bypasses transfer; else scene signal nonzero routes
completion testing; only bothzero goes raw available scan. B12 completion scene
ultimately stage1/incrementroute/fullD79B. Initial runtime resets scene signal0,
late0 and player controls but retains progress bits. Boundary A4negative producer
CFB5 requires incomplete mask; contact selector4 negative also requires incomplete;
EE19 dialogue increments A4 if sourceA3 bit2 clear (positive ordinarily, bytewrap
not silently ruled out). B13 rereads A4 after live-slot handlers and boundary.

There is NO proved global invariant that an incomplete condition when a request
is produced remains incomplete after later live slots. Multiple same-frame
progression contacts/treasures and special-room/scene routes require placement and
source actor reachability proof. Therefore B12-available-land-entry-precondition
is retained explicitly, not falsely closed. Raw no-signal masks1F/3F/7F/FF scan
selectors5/6/7/21 respectively: masks8byte table continues into name/instruction
bytes, firstzero atY21 forFF. Actual cursorY/nameY/nameX respectively:
5=(43,80,A9),6=(70,60,22),7=(46,88,8D),21=(D8,22,6B); animation=6F+Y modulo256.
These are actual consumed bytes, NOT asserted visual lands or added content.
Native raw_selection_entry contracts preserve exact fallback reads without a
five-selector cap. Stage4 must either prove natural entry precondition with actual
map/actor producers OR preserve any reachable raw path all the way through source
consumers; bounded B12 Land enum cannot silently discard it. No emulator run or
natural reachability claim made here.

B01-frame-boundary-integration remains source-informed pending implementation:
NMI commits previous prepared queues, wakes tasks, drains+advances audio then
restores bank; scheduler resumption samples input once before continuing suspended
owner. Foreground protection decrement happens only DB6C, not perNMI/nestedyield.
Frame tests require exact DC23 RNG boundary and paused/dialogue/ending branches;
physical instruction-phase equivalence to host tick is not proved by these mocks.

## Source-order corrections retained during executable-contract authoring

Frame borrows current room row and Catalog for the real DC04 comparison; neither
is inferred from collision cache or actor visuals. Canonical B13 boss_hit_points/
impact_latch supply B11 Context's borrowed run fields; temporary SpawnContext
projections cannot own them. Initial16 control bytes clear queued/momentum/external,
statusdirty,specialmode/timer,specialroom,departure,dialogueactor,late_route,
lifevariant,protection,reload and overlay gate. Reload12 stops at late_route,
retaining lifevariant/protection/request/overlay gate. Late-owner fields are
borrowed until B15/B16 establish their coherent state, not duplicated here.

Stage presentation changes camera high page but RETAINS camera low until later
D7F4 runtime reset. Checkpoint initialization likewise does not write previous
camera low; CheckpointBuild reports only the actually written camera position/
packed room, leaving previous-camera ownership alone. DD11 activates playerA0
BEFORE final composed wait. DD1C after return only clears protection/action/request;
any actor-state mutation during final composition remains. Contracts separate
activate_reload_player from finish_reload to forbid accidental reactivation after
that wait. Historical partial compile output and corrected fixture failures are
preserved under checks/contracts/B13; final checked source has no compile errors.

OwnerStep explicitly suspends player or other owner calls; FrameContinuation
retains exact awaiting point. Resuming a suspended B06/reload/composition call
must not rerun DB40 input gate, rerun earlier owners, decrement protection early,
consume queued action early or advance RNG. Composition suspension retains OAM
guard; only owner completion releases it. A suspended final Yield resumes without
another RNG call. Contracts inspect these live pending state boundaries, rather
than treating a requested Yield in an effect list as completed simulation.

Final native discovery: 76 B13 contracts,963 total (887 prior retained). All76
area cases stop at explicit pending behavior stubs; zero ignored. Infrastructure
2/2 green. Source-derived recipes and direct fixtures were independently checked;
passing Rust compilation is not original-game behavior verification. No original
emulator, QA, GUI, final implementation audit or toolkit change was performed.
