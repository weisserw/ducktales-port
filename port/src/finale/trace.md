# B17 independent source trace (stage3, pending implementation)

Primary source reference/source/game.asm SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reference/reading.asm SHA256
201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Every physical offset includes16 header bytes. All direct B17 code is fixed
bank7: CPU C000..FFFF corresponds physical1C010..2000F. Bank6 actor
pointer planes are CPU802C/806F (physical1803C/1807F); handler3A resolves
FA3F,3B FBC9,3C FB68,40 FD9E. These are actual table targets, not adjacent
coarse region labels. Stage3 contracts call visibly pending native production
entrypoints. No game logic, ROM payload, second oracle or CPU/RAM map is added.

Read controlling contract prompt, workflow/common/coding standards, architecture,
behaviors/debug API; reviewed canonical adjacent actors/bosses, player, dialogue,
progression, status, world, presentation, assets and scenes declarations/traces.
Independent region/unit/operand intersections are retained in
planning/contracts/B17-consumers.json, including all238 primary regions and the
shared1260 span whose other handlers remain B09/B15 owned. Planning is navigation,
never evidence replacing instructions/consumers. No research/current.json exists.

## Arithmetic and flags

Manufacturer MOS Technology MCS6500 Programming Manual January1976, chapters2/3
[arithmetic](https://lbaeza.neocities.org/mcs6500/6500_ch02) and
[flags](https://lbaeza.neocities.org/mcs6500/6500_ch03), consulted independently.
ADC includes incoming carry; SBC subtracts inverse carry, CMP changes carry,
loads/stores/AND/INC/DEC do not change carry. Byte INC/DEC wrap. NES game routines
here use binary arithmetic, not decimal mode. Conditional branch tests use the
most recent producer, including LDA/STA rather than stale arithmetic zero.
Native names represent results, not a stored emulated status register.

## Final handler3A FA3F..FB53 (physical1FA4F..1FB64)

B10 AFxx spawn override installs shared HP5, handler3A, animation84, collision0,
persistent state bit10 when late mode is negative and route/progress permits.
Five hits are not a separate B17 HP store. On each hit_marker!=0 call canonical
B11 F6D7: CLC then protection test; protection0 DEC shared HP (0 wrapsFF), HP0
jumps F702. Animation84 CMP bypasses ALL ordinary allocation/slot1 fallback and
returns carry1. Surviving hit sets protection78 and render flash02; repeated
marked calls decrement protection, clear hit marker/flash only on reaching0.
Unmarked calls never decrement this protection. All nondefeat hit returns retain
carry0. B17 defeat replaces animation with2A, clears animation tick/frame/profile,
sets stateA0/handler3B and returns before any opening/terrain/RNG work. Hit marker,
local fields, position, render and every other actor survive this B17 transaction.

Ordinary handler always clears animation tick before low-nibble phase check.
Zero phase sets screenY74, delay78hex, collision cache05EF=44 (native
WorldCache.collision[6F], NOT a sprite/display field), INC state; immediately
falls through delay and DEC to77hex. Nonzero delay decrements; only1->0 selects
teleport. No hidden per-frame timer is supplied externally. At delay0 on entry,
go directly to attack-window processing. Teleport reads RandomState+RandomState1
with **no CLC**. Without hit this inherits B10/B13 caller arithmetic fact; marked
nondefeat shared-hit CLC overrides it. Sum wraps8bits, writes only random word0,
AND0F selects all16 X positions from actual FB54 (physical1FB64), not FB64 CPU.
Position low byte changes; page/fractions survive. Set flash counter3C, render|02,
state&EF. F6BD compares full16-bit actor X-player X (SEC/SBC twice): direction1
if actor left,2 otherwise including equality. AND1 indexes two facing masks at
FB66/physical1FB76; render&BF|mask. Delay path returns, so flash isn't decremented
on teleport tick. Catalog bounds:16 choices at1FB64, offsets1FB74..75, masks1FB76..77.

Flash counter nonzero decrements. Only1->0 opens: render&F0 clears all low bits,
frame1/profile9B; unconditional overwrite slot14 screenY58, tick/frame/hit0,
state/renderA0, animation85, handler11, profile90. Copy boss page/direction; choose
FB64 CPU offsets10/F0 using direction&1. CLC/ADC writes ONLY low X, discarding
carry into page (03F8+10 ->0308,0308+F0->03F8). Fractions, speeds, local fields,
identity and protection in overwritten slot survive. Set boss Aux0=F0 then return;
no exposure decrement on attack-spawn tick. B07 owns dispatched handler11 behavior
and live slot ordering remains B10; slot14 may execute later in the same frame.
When flash0, compare Aux0 with3C BEFORE decrement: equality render|02/frame0,
collision remains9B. DEC Aux0, nonzero returns;1->0 profile0,state|10,delay3C.
Aux0=0 entry wrapsFF without reset. Post-reset next call begins delay decrement.

## Defeat handler3B FBC9..FBFE (physical1FBD9..1FC0F)

Low nibble0 waits for frame!=0 AND animation tick exactly5 (not >=5 or masked).
Then stateB1, delayB4, immediately falls through and decrements toB3.
In nonzero phase no further animation test: delay!=0 DEC only; old1->0 does
NOT enter dialogue until following call. Old0 removes actor state, player action0F,
INC canonical RaceIntroLatch with byte wrap, dialogue message0B. Ending latch
and unrelated physical fields survive. B03 advances animation after actor logic;
B13 skips later spawn/reaction/late/secondary work after composition sees race
latch. Player/actor/countdown work at next race-intro entry is not globally frozen;
B15 owns actual dialogue continuation.

## Race terrain DFED..E053 (physical1DFFD..1E064)

B15 ED05 message0B calls this BEFORE clearing/rendering dialogue panel; only final
message outcome EDC7 clears race latch. Builder calls B10 ClearNonplayerActors;
only after return: terrain column0, background state0, terrain room/camera room17,
room-in-row17&7=7, set player/camera X **page**7 preserving low bytes. Set stage5,
room row6, PollBackgroundUpload (B02/B04), then restore stage1 after return.
While cursor!=40hex, ResolveTerrainColumn with forward direction1 and work0,
INC cursor, YieldOneFrame, repeat. Exactly64 forward resolves/64 per-column yields.
At40 use stage5 for another poll, restore stage1, inspect signed upload state;
nonnegative yields and polls again, negative completes. No extra column when
polling. Set row2, select actor bank6, FindSpawnWindowStart; only after return
LateStageMode0 then UpdateSpawnWindow; only after return palette pendingFF.
The canonical palette pending type is bool, preserving its nonzero observation.
No arbitrary new stage/level is introduced. B10 stage1,row2 actual spawn records,
B04 geometry and B06 climbing remain shared owners. Race session preserves owner
awaits, including temporary stage5 throughout a suspended upload, no early spawn.

## Rival handler3C FB68..FBC8 (physical1FB78..1FBD9)

Zero low phase initializes Aux0=0, low encoded position=screenY, row byte2, INC
state, then continues same call. Canonical ActorContact.local_aux2 is source06D0
Aux0 (historic Rust name); boss_protection is source06F0 Aux2. Never conflate.
Read slot15 row, then low: if row0 and low<90 store that EXACT low into slot15
Aux0. BNE tests stored A: low1..8F freezes current climber, but low0 writes0 then
continues moving! Low90 is outside gate; nonzero leader row skips low check and
preserves old marker. Marker is not normalized bool storage. Normal movement
SEC/SBC1 writes low. With borrow (old0), A=FF,C0 then SBC0F subtracts16 ->EF;
CLC explicitly requests borrow on following row SBC0. Without borrow preserve row.
Thus encoded rows have F0 positions, not100; low0,row0 can wrap toEF,rowFF.
Read updated row against CurrentRoomRow. Equal: state&EF and screenY=low; unequal:
state|10, preserve stale screenY. Every rival reads same live leader slot; B10's
ascending order means earlier slot14 sees slot15's previous-tick position and
slot15 updates after it. No per-actor synthetic leader copies are allowed.

Independently decoded stage1 spawn planes B4E0/B57C/B6B4/B618/B6BC: row2 start2,
records2/3/4 are pages2/2/3, template86; record5 at07B0,YB0,template114;
record6 at07AC,Y94,template115; record7 negative sentinel. Template114 atBFDE
contains A0,69,3C,00; template115 atBFE2 A0,5E,3C,00. Existing distant
records fail camera interval; normal descending allocation therefore installs
leader animation69 in15 then second animation5E in14. Row0 starts0: record0
0780,Y98,template113 (A0,1B,40,00 atBFDA), record1 sentinel. Row1 is empty.
Those actual entries establish shared natural race actors; no final actor is
injected as a new stage. Completion/late-route spawn override precedes this route;
B15 builds while intro latch is nonzero and late mode clears before race spawns.

## Goal handler40 FD9E..FDF4 (physical1FDAE..1FE05)

Actual dispatch is FD9E, not the broader1260 fragment's FBFF start or informal
FDA0. Nonzero low state bypasses leader/contact and enters delay. Otherwise if
leader Aux0!=0 INC goal **low X** (page unchanged), compare new low>=B0. At
boundary remove state first; if player Death0D return silently, otherwise
message0C/playerDialogue0F and return without overlap or sounds/divider change.
This loss arbitration runs BEFORE player contact, so simultaneous lead-boundary
and overlap loses. FF wraps00 so it does not passB0. Below boundary (or no leader)
call B05 TestPlayerActorOverlap F43C: render-visible gate, signed profile flags,
strict vertical/inclusive horizontal, crouch+7/borrow and immediate reaction rules
are B05-owned. Return carry1 means no overlap. Even overlap rereads player Death
AFTER callee, then rejects if dead. Otherwise INC goal state, QueueStageAudio FF,
QueueAudio37 in that order, delayF0/playerHold0E. Same call clears countdown divider
and decrements delay toEF. Every future phased call ignores leader/death, resets
divider0, DEC delay;1->0 INC EndingSequenceLatch (wrap),0->FF no increment. B13
reads ending latch after composition before tail operations, but ending work only
begins on next frame entry. Loss message0C calls B15 EDCC -> canonical B06 EBB4
immediate life-loss transaction; it is not merely an enum change or ending reset.

## Ending E1AF..E289 (physical1E1BF..1E29C)

Caller B13 only invokes on nonzero ending latch. Nonnegative first entry OR80
preserves low bits before calling FFC7 PrepareValueTransferDestinations. This
prepares206C/2043 then enters B14 transfer: six reward codes working copy, seven
persistent total codes, actual yielded batches. It does NOT initially stop audio
(the source comment is imprecise). Wait78hex only AFTER transfer returns, then
QueueStageAudio05, stage9 and LoadTerrainLayoutPointers. Only then select variation.
Canonical B14 total0334..033A contains seven display codes, blank24 is not numeric0.
Y2 initially. If (progress&7F)==7F AND total[0]!=24, choose2, ignoring last code.
Otherwise DEY to1, test total[6]; if blank choose1, if nonblank DEY to0. Hence
all numeric zero codes with last0 choose0, all blank choose1, no wealth threshold
comparison exists. High progress bit is ignored by AND. Reward and middle five
total codes do not directly choose variants. Fresh post-transfer values matter.
Store canonical scene cursor and ending variation, BuildSceneScreen (B16 Content)
with0/1/2. Wait for actual return, INC the LIVE shared cursor; only result6 becomes0.
B16 may update shared fields while building; no cached copy written afterward.

Negative latch entry: if canonical player-slot scene handler !=5, select bank5,
JMP Bank05_UpdateSceneActor (B16). That return returns to B13 composition; do NOT
recheck newly settled handler or begin pause/split in the same owner call. B16
actual handler targets4/5 settle; handler3 internally rebuilds scene3/sets palette
but enclosing B17 sequence cursor remains its own current source field as passed.
If handler5 and display mode0, pause. If display mode!=0 clear it and split delay0
then enter scroll. Scene content has its own resumable fade/terrain/script/CHR/
composition waits and explicit B02 SceneResourceContext: layout9, terrain2/1,
CHR9/10. Never index nonexistent StageTerrainBanks[9] or spawn table9.

Split increments canonical Composition.oam[0].y BEFORE stop checks. ExactE8 stops
all cursors via TimerNoUpdate RTS; exactB0 additionally stops cursor5, not others.
ByteFF wraps0. Stop returns to B13, allowing one composition/RNG/yield tail; next
frame settled+mode0 begins pause. Otherwise YieldOneFrame then wait old spritezero
hit bit40 clear and new bit40 set; write PPUCTRL=PpuCtrlShadow|1 WITHOUT changing
shadow. Native SplitRaster requests B03's committed raster boundary; do not busy
wait hardware flags or silently remove intentional per-yield cadence.
If (Y&7)!=7 go next pixel. At every eighth pixel, for cursor!=0 or Y<B7,
INC delay then AND3F: nonzero goes BACK to Yield at SAME Y (not pixel increment).
After64 such raster/yield completions reset delay0 and advance pixel. For cursor0
at eighth pixel Y>=B7: compare current stage audio7. If already7 directly next
pixel; else queue7 then next pixel. Neither path increments delay at those late
music checkpoints. Non-eighth late pixels never request audio. This is repeated
same-pixel waiting, not a simple one-frame-per-pixel or occasional pause model.

Pause E269 explicitly issues TEN separate WaitSchedulerTicks(3C), whose source
FF A5 yields before DEX; each is60 yields, no counter decrement on suspended owner
poll. Preserve requests, not a fused Wait600, so composed observations retain
source boundaries. Then cursor0 -> permanent E286 loop. Cursor1..3 force4;
cursor4/5 survive; JMP E1E9 builds that scene then live cursor increment/wrap.
Natural sequencing0/1/2 opening -> cursor1/2/3 -> scene4/cursor5 -> splitB0 ->
scene5/cursor0 -> splitE8/music7 -> pause -> End. B16 may rebuild interior scene3
inside its actors. Terminal E286 YieldOneFrame/JMP E286 never returns to B13;
no controller read/START/A/B branch, no title restart and no ordinary composition
or DC23 RNG tail occurs while this owner is permanently suspended. B01 still
runs interrupts/audio/task wake. Terminal API deliberately takes no input.

## Canonical interfaces, alias audit and remaining integration

ActorPool, Player, Progression, Dialogue, Status, RandomState, WorldCache, Camera,
StreamCursor, Display and Composition are borrowed. EndingContext borrows the same
scene cursor/display mode exposed to B16 ContentContext, rather than caching them
across fades. Ending owns only continuation/variation/pause/split-loop counters;
RaceTerrain owns continuation only. External operations explicitly acknowledge
Running/Complete; no post-call mutations occur until Complete. Stage4 adapters
execute requested audio through B18, background through B02/B04, scenes through
B16, and direct canonical B11 hit. Tests use external contact facts only, not a
mock boss/race/ending implementation or second oracle.

Independently scanned ALL animation writes (STA plus INC/DEC) and predecessor
contexts, including computed writers, in B17-consumers.json. Final direct writes
are84 (B10 override),2A(defeat),85(slot14 attack); natural race templates69/5E/1B;
B16 all six scene lists/special records/completion6F..73 were rechecked. None of
these final-specific writers introduces aliases0A/8C/8D. Computed cross-area
writers include parity+0D enemy, table8663 player pose, restore saved local,
8FC6 transform,96C5 companion, ordinary/special template copy, initial title/land
triples,6F+selection, F67B terrain recovery, option25/26 remap and INC/DEC pose
cycles. These require their owner preconditions, not a global assertion that any
source byte value is unreachable. Existing B16 all144 animation metadata prove
only0A/8C/8D currently consume runtime frames0/1; reset source-zero plus tenth and
eleventh changed-terrain records author the canonical secondary alias tail.
B17 does not claim to have proved natural co-occurrence of alias animations and
high queue cursor/cache overlap. B14-runtime-frame-author remains pending for
stage4 full-game integration with all source writer contexts retained; no alias,
region or contract is excluded. All238 B17 primary regions plus shared1260 were
reviewed; neighboring dialogue/platform logic stays B15/B09, generic table records
are bounded by their actual indexed consumers. No new cohesive behavior area.

Stage4 finite-data/fixture corrections: immutable FB54..FB63 position table ends
E0; the following FB64 byte10 belongs to attack offsets10/F0. Endpoint assertion
now checksE0. The natural race-window test now supplies row2's absolute starting
identity2 from its decoded first record, matching AE60's room-start read; prior
window0 exited before seeing row2 at all. Expected rival identities5/6 and slots
15/14, all numeric/template/source identities and test names remain unchanged.
Superseded fixture: checks/implementation/contract-corrections/B17-before.rs.
