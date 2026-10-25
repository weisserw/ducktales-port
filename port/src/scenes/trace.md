# B16 independent source trace and completed stage3 contracts

Stage3 only; worker /root/behavior_contracts/b16_scenes, sole editor. Entire
prompts/contract.md, workflow/common/coding, architecture/behaviors/debug API and
prior B01..B15 source/interface handoffs consulted. Original game.asm SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Physical offsets include iNES header. CPU8000 in bank5 maps14010, bank4 maps10010;
fixedC000 maps1C010. Tables independently inspected in matching input/game.rom.
MOS-authorized Synertek6500 programming manual publication6500-50,
https://syncopate.us/books/Synertek6502ProgrammingManual.html consulted for
ADC/SBC, CMP carry, DEC/INC carry preservation, LDA preserving C, branch signs,
indexed addressing and JSR/RTS. Arithmetic wraps at source widths. No CPU image,
gameplay implementation, emulator/GUI/QA/final audit or asset embedding.

## Scene initialization and actual six handlers

B380..B3CB physical17390..173DC; six offsets B3CC [00,04,08,0C,13,29]
index the triples starting B3D2. Animation below10 ends and writes ONLY slot0
handler, then ComposeObjectFlags immediately. FF skips the entire actor slot,
retaining fields, but advances slot and triple cursor. Other triples write
animation,Y,lowX; highX,tick,frame=0; state80/render90. CPY19hex compares absolute
list-byte cursor, not actor slot; only scene4 third record has renderD0. Other
fields stay untouched. Scene0/1/2 each one record, scene3 skips slot0 then writes
slot1, scene4 seven records, scene5 one. Terminal selectors0,1,2,4,4,4.
Source B382 entry used by palette transition loads Y from explicit X=3, bypassing
TerrainRoomCursor read: it MUST NOT overwrite current scene cursor.

B400..B50C physical17410..1751D: split pointers at B50D/B513 decode EXACTLY
[B412,B46A,B48F,B4B5,B507,B507]. Last two point at LDA5/STAhandler/RTS,
NOT B4EB palette transition. Existing prose attaching labels to returns is not
an alternate dispatch. SceneHandler typed domain is separate from gameplay67.

Bounce0 phase lowF0 sets local_state0B,velocity0400,INCstate; same invocation
integrates subtractiveY THEN gravity. Bit1 clear is ascent; negative postgravity
velocity clamps0,INCstate,DECremaining. Zero remaining joins handler3 reset:
handler3,state80,tick80,local_state0 (frame unchanged). Descent gravity THEN Y;
Y>=B4 reloads0400 and DECstate. Eleven apex crossings, not eleven landings.
Walk1 initial local_state0,speed0100,INCstate. Xlow>=90 moves left even if high
page nonzero; below90 DECstatecounter0->FF and waits256 decrement visits before
common handler3. At threshold90 it still moves, no countdown. Delay2 initial
sets counter0,INCstate and returns WITHOUT decrement. Later decrements0->FF;
only last1->0 joins handler3. Move3 lowphase0 decrements counter first (0->FF),
expiry sets both speeds0300,INCstate; Xlow>=48 moves left and returns, else
Y>=78 subtracts velocity and returns, else falls directly into B4EB countdown.
B4EB DECcounter; expiry reloads scene3 via B382 (including immediate composition),
loads palette3,sets uploadFF,ORcontrol1,handler5. This can yield only through its
composition/presentation dependencies; preserve the enclosing continuation.
Both dispatched4/5 simply assign handler5 and return, never decrement state.

## Completion actor and deliberate palette waits

Bank4 BE00..BEAA physical13E10..13EBB. Slot8, gravity40, animation6F initially;
per-animation restart writes state/renderA0,Y C0,X0080,tick/frame/counter0,
horizontal0200,vertical077D; X/Y fractions other than speed fraction retained.
Ascent integrateY then gravity; negative postgravity clamps velocity0, counter++.
Descent gravity then integrateY. Even animation moves positiveX, odd negativeX.
Hide OAM/bank5 compose/YieldOneFrame/restorebank4; inspect render sign only AFTER
resumption; active repeats motion, hidden increments animation and fully restarts.
Runs6F,70,71,72,73 (five animations, comment's four is wrong). On next74 clear
ONLY slot8state, then sixteen flashes: palette sprite0=16/upload16/wait4,
then0F/upload0F/wait4. After128scheduler ticks clear CompletionActorLatch5E,
NOT SceneCompletionSignal5D or impactA7. Palette requests retain source byte even
though existing B03 observation is bool; canonical active color commits at NMI.

## Special-room initialization and caller domains

FixedDF27..DFCE physical1DF37..1DFDF. DBBE calls only negative SpecialRoomRequest;
B15 message0A produces81, linkeddeparture produces82. Mask lowF happens AFTER
fadeout, disable and ClearNonplayerActors (whose stage1/row<2 preservation must
not be replaced by blind clear). Reset display,specialspawn,late,split,spawn
upper/lower/start,momentumcontrol/direction0; scan direction1. Terrain bank3;
layout selector=request+6 ->7/8. LoadStageSpawnTables bank6 AE22 skips its LDX StageIndex entry; preserved X7/8
from LoadTerrainLayoutPointers selects spawn tables, and remains7/8 for
UploadChrLists. This is NOT BuildInitialTerrainColumns; checkpoint entry
DD52 accepts request+10 ->11/12 and owns cache/camera/player fields. Stage palette
X70 selects authored background palette7; no StageTerrainBanks[7/8] inference.
DFCE is preceding RTS60; actual offset entries for modes1/2 at DFCF/DFD0 are0/0D.
Animation negative terminates six-byte records. Mode1 writes slot0 anim04 X00F0
Y80 handler0/profile0 and slot2 anim5A same position handler36. Mode2 writes slot0
anim00 X03D0 YA0 handler0/profile0 and slot2 anim6C X0380 Y10 handler37. Increment
slot by2, list by6. state/renderA0,tick/frame0; fields not named remain retained.
Player actions from DFEA+mode:1=Hold0E,2=Standing0. Queue stageaudio0B;enable;
yield; only after resume display40 then FadeIn; caller must await fade completion.
Selector0's offset60 is not silently treated as a third authored mode. Keep any
additional natural producer/reachability uncertainty explicit, never read absent
stage config9 or invent arbitrary selector validation for original behavior.

## BuildSceneScreen content requested by B17

FixedE054..E11A physical1E064..1E12B,103 primary ledger regions mostly one
instruction each; split classification does not split native owners. Fadeout;
clear controlbit0/display/cursors. Except scene3, ClearNonplayerActors,clear slots0
and1state,hide OAM; scene3 preserves them. Yield BEFORE disable/fill2400 tile24
attribute0. Terrain bank from [2,2,2,1,1,1];LoadScenePalettes original selector;
save scene. Resolve64columns with scratch04=1/05=0 each, increasing cursor0..3F
without masking stored value. If scene<3, replace roomcursor3,ring0,bank1,
room_in_row1 and build64more. This effective scene3 selects script list; original
scene restored only after scripts. Scene3/4/5 build64only. Lists offsets
[7,7,7,0,4,6], encoded list [0B,83,88,00,06,00,07,00]. Thus original0..3 all
write scripts0B,(3+variation),(8+titleoption); scene4 writes6;scene5 writes7.
Positive script uses byte directly; negative uses lowF; nibble8 adds titleoption;
others add variation; zero SUM falls through to add titleoption (not list end).
The zero LIST BYTE terminates without WriteScreenScript. Scratch10 increments
and wrap0 also exits. B03 owns actual script content/writes, no second script
interpreter/queue. Restore original scene, initialize actors (compose),CHR list
[9,9,9,9,10,10],enable,Yield. AFTERresume scenes4/5 splitY70/90,display40,
camera high1 preserving low; others preserve these fields. FadeIn then return
bank5 context. No scene sequencing/scroll/terminal advancement here (B17).
Scene palette sprite endpointDF for scene3 uses low7F=5F and optional-title flag;
it is NOT a code-byte read. B03 owns backwards16color copy/universalcolor sync
and option palette override; B16 must pass unmaskedDF. Layout9 loaded by B17
E1C4..E1C8; scene terrain bank is explicit, no StageTerrainBanks/initial/spawn9.

## Linked departure, timed spawner and RNG

FixedF869..F9EF physical1F879..1FA00; gameplay handlers36/37. Freeze playertick0
and countdowndivider0 on every linked update. Init lowF0 clears verticalspeed,
horizontal fraction;speed0100;INCstate. Bit1 enters gravity/subtractiveY and
copies Y to player; final INTEGRATOR carry (not gravity) set -> request82;
clear -> no request. Horizontal path subtracts fractions to obtain borrow but
DOES NOT STORE Xfraction; then full16bit integerX mirrored to player. Reward
code at0326 (third reward digit) exactly7 selects braking; Xlow<B0 subtract0004
speed, saturating only by source exactzero. Zero INCstate enters vertical next
tick; unsigned underflow is retained if arbitrary speed<4. Other digit codes
Xlow<10 increments scene signal wrapping; highX ignored. No new life counter.

Timed spawner first tests PLAYER render sign; hidden tail-calls common B06 life
loss before init/countdown/motion. Visible init horizontal0100,vertical0200,
fractions0,counter0,delay3C,INCstate. Immediately delay-- to3B then return;
each remaining nonzero decrements and returns including1->0. Following call:
phase2 travels;3 exits upward; all other lowphases positiveY until Y>=50, then
INCstate/departuresignal80. Travel cooldown0 calls reward allocation, then ALWAYS
DECcooldown (failure0->FF), IntegrateNegativeX. Only highX0/lowX<70 startsphase3
and overwrites slot3 state/renderA0,anim5A,X0030,Y80,handler38,profile/tick/frame0;
other slot3 fields retained. Phase3 subtractiveY then Y<10 clears only ownstate.
Reward allocation scans15..9 descending active sign; does NOT inspect8. No free
slot -> no RNG/cooldown change inside allocator. Success writes state/renderA0,
anim28,handler13,tick/frame/profile0,copies parent position integers/Y,velocity0400;
fractions other than velocityfraction and other locals retained. RNG ADC is
RandomState0+RandomState1+carry WITHOUT CLC. Crucially preceding all successful
instructions preserve carry; carry comes caller: travel CMP #02 equality leaves
C=1 for natural phase2 calls (AND/LDA/JSR do not clear it), including first
phase2 after initialization countdown has returned. Native interface conveys
semantic incrementOne, not global CPU flags; direct allocator accepts explicit
source-derived increment for independent callers. Save wrapped sum to word1;
low3select cooldown[30,30,30,60,60,60,120,120]; after return travel decrements it.

## Runtime composite alias: writer facts and explicit remaining uncertainty

This B16 audit supersedes earlier 'status-authored frame' attribution. Independent
all144 animation-pointer/record reads find alias consumers ONLY animation0A ->
frame0, animation8C ->frame0,animation8D ->frame1. No animation references frame2/3,
but all four table pointers still alias04C0 and must remain supported. All121
spawn templates were independently inspected: none selects0A/8C/8D. All six scene
lists/special records/completion5poses avoid those animation IDs. This is NOT a
proof of global unreachability: computed animation writers and final/adjacent
source paths must still be reconciled with natural placements by B17/stage4.
The full124 direct animation-writer index, all144 metadata entries/all121 template
animation IDs and956 relevant state references are in B16-consumers.json. B03
original aliases and source-zero decoding contracts remain required, never disabled.

Actual writer identity matters: reset FEFC..FF11 clears pages1..7, including04C0.
There is no direct04C0 STA operand. However secondary queue starts0440;
04C0 aliases encoded SECONDARY OFFSET80hex (128 decimal). F574..F648/F69C build
changed-terrain records at SecondaryQueueCursor, fields+0..+14, then advance cursor
by0E. Their indexed absolute writes add unsigned X to base+field WITHOUT wrapping
that effective address within an artificial256byte queue. Only the stored cursor
addition wraps. First record at cursor00, tenth at7E: its offset+2 count1 writes
alias header04C0; payload follows. Eleventh at8C overwrites tenth sentinel at04CC.
Earlier working message's 'sixth/seventh' inference confused decimal80 with hex80;
corrected BEFORE contracts: executable cases use TEN/ELEVEN records. No source
change or evidence destruction. Source fields are two count1 tile rows, count0
attribute,FF sentinel. The B04 result's five actual VideoWrites independently
encode [21,46,01,0B,0C,21,66,01,0D,0E,23,D1,00,3F,FF] for the synthetic mutation
fixture already derived by B04; B03 PreparedSecondary owns exactly one backing
encoded buffer and runtime_composite selectors0..3 borrow the identical tail.
Source-zero initialization and immutable pointer-identical projection are
contracted; no second frame/status buffer, CPU RAM map or fake author added.
A high source cursor can also overlap B04 attribute-cache storage; the final
cursor-wrap fixture retains every queue byte through index266 (base+FC+14),
not modulo-wrapping each field. Stage4 must reconcile any naturally reachable
cache overlap through the canonical B04 cache, rather than silently assume a
small secondary buffer. Full placement/frame co-occurrence of ten mutations and
alias-consuming animations is not proved. `B14-runtime-frame-author` remains
pending with these concrete writer/consumer facts and required integration.

Generic regions13EBB..13F10 and17529..17590 have no found direct/table consumers
in the bounded complete scene/adjacent table review; preserve original bytes,
without claiming padding or omitting future consumer obligations. Region505
13F10..14010 IS background-list22 second-record payload (B02), despite original
comment. B02 named loader/override contracts remain authoritative; no exclusion.
B02 canonical SceneResourceContext/SceneAssets join now explicitly borrows layout9
and selects terrain bank2 for scenes0..2,bank1 for3..5;CHR9 for0..3,10 for4..5;
spawn_configuration None. The additional B02 owner contract exercises all6 joins
with distinct layout fixtures and actual localROM terrain/CHR expectations,
not a nonexistent StageTerrainBanks[9] access. Existing B10 source5..8 empty
spawn-adapter contract covers actual special AE22 input7/8, stage selector kept
unchanged; B16 explicitly preserves X7/8 until CHR list request.

## Owner boundaries and contract limits

B01 owns scheduler completion/interrupt sampling; B03 canonical display queues,
secondary/runtime shared backing, composition/fades/palette commit; B04 actual
terrain/cache/camera integration; B05 motion returns postcallee facts; B06 common
life-loss work; B10 live ActorPool and authored empty7/8spawn contexts; B12 caller
completion route; B13 canonical requests/signals/checkpoints/mode and await points;
B14 canonical digits/timer/health/lives; B15 actual81 producer; B17 sequence/race/
scroll/terminal progression; B18 ordered sound. Scenes own continuations and
handler policies only. SpecialControls is a borrowed canonical view (not copied
across fades), ContentContext and DepartureContext borrow owner values. Pending
operations return Suspended while the owner is Running; later effects cannot
be recorded as already completed in one eager log. Tests explicitly stop and
inspect fades/yields/composition boundaries. B10 canonical-pool clear helper's actual ordinary
slots15..2 and special stage1,row<2 rule remain delegated to existing B05 helper contracts.

All authored behavior methods remain explicit unimplemented stubs. Runnable
contracts demonstrate readiness for stage4, never fidelity or passing gameplay.
No added cohesive area found. No implementation accounting/ledger disposition,
reference source, QA record, toolkit files, check-in or publication changed.

B16 final branch review adds explicit hidden-player immediate EBB4 effects (not
merely a queued LifeLoss enum): common B06 commit_life_loss applies playeraction0,
queuedNone,slot0state0,healthalternation0 immediately, returns StageId+87 then
LifeLossVariant+priorcarry modulo256 for B13 reload. No life decrement here and
no falling physics/cooldown/motion before the entry. New86th-case boundary
proves FF+87 then+variant1 yields88, not87. B15/B06 existing expectations remain
authoritative; the shared pending entry is declared in player/mod.rs rather than
a new independent scene life-loss policy.

Final native evidence: checks/contracts/B16/results.json;86B16cases,1154total,
warningfreecompile0/discovery0,area101all86explicitpending,infra2/2pass. Prior
84case evidence remains superseded, not deleted. Wholehandoff planning/contracts/B16.md.

Stage4 interface correction: update_scene_actor now borrows Catalog. The source
B382 interior actor-list read on AwaitRebuild cannot be implemented from the old
argument list without embedding original animation/placement bytes. Existing
contract call sites now pass their independently authored catalog; expectations
and identities are unchanged. Prior fixture retained in checks/implementation/
contract-corrections/B16-before-catalog-borrow.rs.

Stage4 flip correction: list4 offset13hex (19decimal), then cursors13/16/19hex.
Immutable initializer CPY operand19hex flips the third record, x5D, not first
x44. The former trace/fixture confused decimal19 with hexadecimal19. Existing
absolute-cursor test identity retained; native decoder unchanged. Prior fixture
is checks/implementation/contract-corrections/B16-before-flip.rs.

## Modernization iteration014: canonical scene actor behavior

Scene actor Bounce/WalkIn/Delay/MoveToMark/Settle/Settled now retain a native
SceneActorState in the canonical ActorControl payload. Each moving behavior owns
its named bounce/pause/wait counter and explicit entry or bounce progress. The
gameplay dispatcher matches native SceneHandler identities and never masks raw
actor progress or dispatches integer handler literals. There is no side session
or retained source-byte twin. Initial scene placement and the actual scene owner
select this context explicitly because handler0..5 also name ordinary enemies.
Legacy-family entry switches context losslessly through the same actor owner.

Original B412..B507 instructions were reread. Native entry preserves same-call
initialization, ascent Y-before-gravity, descent gravity-before-Y, low-byte X
thresholds, X-before-Y movement, byte-count zero wraps and exact80/tick80 move
entry reset with frame/other-local preservation. Settle4/5 still only chooses5.
Existing HandlerContinuation retains rebuild -> composition -> palette/control
completion boundaries. Counter exhaustion never adds an extra yield.

Source projections retain arbitrary initialized progress values, unusual bounce
progress, flags and the untouched other-local byte for shared partial writes and
debug compatibility. Normal scene updates retain native variants without repeated
source decoding. All previous expectations are unchanged; added source-backed
contracts cover every initialized progress code and zero/one/wrapped counters,
all state-byte roundtrips across scene/family handoff and noncanonical bounce
ordering. No original-emulator complete-scene route or physical platform run is
claimed by these focused contracts and the retained baseline route.
