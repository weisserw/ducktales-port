# B12 independent title/options/land-selection source trace

Stage3 pending contracts only. Frozen maintained game.asm SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Matching local ROM97d472f7b2c1088448d1644155b2650a546bad887b3cba571e16d7986824c169.
Fixed CPU C000 maps physical1C010; bank5 CPU8000 maps14010; bank4 maps10010.
Physical ranges include the header. Full controlling contract, common/workflow/
coding requirements, architecture, behavior inventory/debug observations and
relevant runtime/assets/presentation/world/actors/player/status interfaces and
traces consulted. Independent unit/caller/state index is B12-consumers.json.
Indices locate evidence; source instructions/data consumers derive expectations.

Primary CPU evidence opened: MOS-authorized manufacturer reprint, SY6500/MCS6500
Programming Manual6500-50 August1976,
https://syncopate.us/books/Synertek6502ProgrammingManual.html,
sections2.2.1/2.2.2,4.1.2/4.2.1,7.4..7.9,8.1/8.2/8.5/8.6,10.1/10.2.
8-bit ADC includes carry; CMP/CPX equality sets carry; LDA/STA/AND preserve carry;
INC/DEC wrap and change N/Z without carry. LSR shifts bit0 into carry and zeros
bit7. Eight-bit index wraps independently of carry. These are NES binary byte
rules, not decimal arithmetic. No CPU/RAM map, interpreter or oracle is authored.
Toolkit docs/development.md NES table and existing NES/native guidance supply
hardware context. Source-label entry assumptions are checked against actual ROM
operands/bytes, rather than assuming a prose comment denotes an entry address.

## Entry and exact cooperative title continuation

ResetInitialize FEE3..FF25 installs entry D4E6 into task0 (actual instruction
bytes A2 BF at physical1D4F6). Do not enter D4D6: that preceding16-byte helper
snaps actor Y. InitializeTitle source label starts D4E6, in region1D4E6..1D544;
region starts ten bytes before the actual title label. This is an important
region/entry distinction. Scheduler new-entry state8 jumps without controller
sampling; only resume of task0 samples (B01 FF6D). Source title resets its
foreground stack then writes control shadow directly to hardware and yields.
Seed88 is NOT reset initialization or fresh-entry state. On first resume only
RandomState word0 becomes88; words1..3 remain their current values.

First resume: CopyTitleOverlay copies StageUiTemplate13 bytes DAE0..DAEC and OverlayTemplate21 bytes DAED..DB01, backwards,
including FF encountered inside either extent. QueueStageAudioCommand(7) updates
current-stage command even if B18 ring rejects enqueue, then FadeOutPalette.
B03 fade takes16 actual yielded boundaries, four each at offsets10/20/30/40.
Only after return: gameplay mode0 AND player protection0, yield. After that yield,
disable rendering; clear UiScrollX/Y and CameraX/Hi; fill nametable2000 through
23FF with tile24 and attributes0; hide OAM; ClearNonplayerActors (B10's actual
preservation/clearing semantics, not an invented blanket actor reset); bank5
screen script2; CHR list6. Temporary switch-bank effects are loaders' responsibility.

PrepareTitleActors D534: LoadStagePalette offset60 (background16 and shared
sprite16 into both base/active); overwrite sprite palette16=11 in both. Initialize
slots1 then0 from two triples D621..D626, state/render80, animation/Y/lowX,
frame/tick0 and worldXHi0. Fractions, handler, profile, locals, spawn identity,
restart and all other fields are not reset here. Then palette26=2C,30=20,31=2C
in both shadows. Enable rendering; yield; FadeIn (16 yields at30/20/10/00,
including final wait). Copy TitleOverlayTemplate13 bytes DB02..DB0E, including
terminator; yield; only now write TitleOption1. An old option survives until this
last continuation. No source input clearing occurs in PrepareTitleActors despite
its maintained comment saying 'reset controller edges'. Source only samples on
scheduler resumes; held and pressed remain B01's independently filtered values.

Native title_resume returns immediate ordered owner work and its continuation.
An InitializeTitleActors operation must call the dedicated actor helper at that
point, after clearing/loading and before palette-tail/enable, not mutate the pool
while constructing outgoing-fade work. Land batches use the same invariant.
Palette edits are requests to canonical B03 base/active state; overlay copies
prepare bytes for B03, with no retained second palette/OAM/queue.

## Three options, every edge byte and idle animation

D5A3 reads ControllerPressed, not held. START10 first tail-calls InitializeLandSelect
without menu sound, option update or title composition. Otherwise mask23 admits
SELECT20/LEFT02/RIGHT01; B40, A80, UP08, DOWN04 alone do nothing. AND02 chooses
interleaved table index0 or2, not contiguous0/1. D62C/D62D bytes [01,FD,FF,03]
mean right/select +1 with correction-3, left-1 with correction+3. CLC precedes
both additions. Compare unsigned3: FF underflow enters correction ->02. Thus
increment targets[1,2,0], decrement[2,0,1]. LEFT wins if both movement bits reach
the consumer (normally B01 filters opposite edges). Marker lowX from D629
[20,50,90] overwrites only slot1 lowX, retaining high byte; enqueue menu2F.
Both movement and idle call title_animation, compose, yield and repeat. Held
without edge never repeats; a valid edge still acts when held is independently
conflicting. All256 edge values for all3 options are executable contracts.

Player-facing labels ARE independently proven, superseding planning's unresolved
labels: script2 pointer BB96 (split BAFA/BB06) contains record PPU2225 spelling
EASY, NORMAL, DIFFICULT with blank24. Alphabet0A..23 except shared O/zero00 was
checked against CHR list6 background pattern table1000; local bitplane inspection
of0/A/E/N/S/Y confirms actual glyphs. No extracted glyph payload is embedded.
Source option0 -> EASY -> B14 alternating damage;1 -> NORMAL -> ordinary damage;
2 -> DIFFICULT -> alternate reward remapping25->28 and26->24. B14 owns damage/
reward behavior, B12 the selector/labels/join. Other option consumers E0DD scene
script8+option and F7A7 four-color group belong to B16/B03 respectively. All three
encodings remain stable; no invented difficulty name or added UI.

D5D7: if slot0 frame !=0, do no preliminary tick/random change. If frame0 but tick
!=1 (full byte, not low7), also no change. If frame0/tick1, reset tick0 BEFORE
checking CompositionTick. Only exact B4 passes. CPX B4 supplies C=1, and STA/LDA
preserve it: new random0=(random0+random1+1) modulo256. Store lowbit as frame0/1;
zero composition clock BEFORE B03 ComposeObjects increments its clock. Words1..3
unchanged by this local update; B01 random feedback algorithm is separate.
All failed gates still HideSprites/Compose/Yield. Comparison is pre-composition,
not host tick count. All byte tick/frame/clock alternate branches and carry/wrap
examples are contracted. No attract/demo countdown or implicit stage transition
exists in this loop. Idle repeats until explicit START; waits are scheduler/
fade/composition boundaries, never render/audio callback clocks.

## Land initialization and original catalog identity

InitializeLandSelect D630: FadeOut first. Then mode0; disable; hide; clear nonplayer
actors BEFORE installing temporary stage5 (preservation therefore sees old stage/
room). Push StageIndex, set5, CHR list5, LoadStagePresentation and
BuildWholeTerrainRing. LoadStagePresentation queues stage command5, initializes
terrain bank/room/camera/player X and palette. Restore saved StageIndex only after
ring construction. Initialize eight actors slots7..0 from D8AF..D8C6, state80,
render90, copied animation/Y/lowX, tick/frame0, highX0. Duplicate STA animationTick
has no separate semantic effect. Fractions/handler/local/restart etc remain.
Enable rendering; bank5 SelectStatusMessage B800; yield; FadeIn; clear source7F
layer gate0. Only then branch on completion actor and scene signal. Native
LandPhase preserves both yields/fades and Save/Use5/Restore scope, with ordered
InitializeLandActors. Source stage5 is a temporary layout and not a sixth
ordinary menu entry, nor ending layout9. No source input/header/dimension change.

Land identities independently proven by status selectors2..6: B800->B81C zero
payload selector substitutes StageIndex+2. Split payload pointers B9C1/B9D8 give
B8EB 'THE AMAZON',B8F7 'TRANSYLVANIA',B904 'AFRICAN MINES',B912 'THE HIMALAYAS',
B920 'THE MOON'. Animation6F..73 name/progress mappings and source masks1/2/4/8/10
agree. Native finite Land enum maps to original StageId0..4; level resolution uses
B02 original_selector joins in dynamic named catalog, never collection index or
fixed total level count. Added entries may precede/interleave originals without
changing original controls. This authorizes no extra player content-selection UI.

## Completion routes, transfers and acknowledgment side effect

D69D tests CompletionActorLatch first; any nonzero goes directly D6C1 Wait60,
then bank4 BE00 completion actor; bank5 B81C message4; WaitForAcknowledgeA;
StageIndex1; INC LateStageRoute modulo256; JMP D79B. This bypasses scene signal,
composition, value transfer, progress test and message3. B16 owns BE00 motion,
sequence73/74 and16 palette flash cycles with4 ticks per half-cycle, then clears
CompletionActorLatch; B12 owns the call/order. It does not introduce a new area.

If completion actor0 and SceneCompletionSignal0, go straight FindAvailableLand,
even when progress lowfive are allset. There is no completion-bit early check.
If scene signal nonzero: bank5 ComposeObjects once, Wait60, B14
TransferDisplayedValueToTotal, then progress&1F compare1F. Incomplete: acknowledge,
message0, yield, find available. Allcomplete: message3, fallthrough Wait60
(second distinct pause), BE00, message4, acknowledge, stage1/late++/D79B.
Upper progress bits ignored for this five-complete test. B14 owns blank-source/
ten-unit batching and yield count; B12 must await its actual return before reading
progress. Snapshot planning is valid ONLY under the source invariant below.

State stability: StageProgressBits A0 has one named mutating writer outside reset,
Bank06_CollectStageTreasure8C13..8C40 ORing its animation-derived mask. Reset's
broad clearing is an external restart, not a returning selection operation.
Whole-source symbolic and numeric A0 references checked; no alternate direct
writer found. Only scheduler install call is reset; no auxiliary foreground task
runs gameplay actor dispatch while selection waits. NMI C000 commits video,
wakes tasks, drains/runs audio, never dispatches treasure/update handlers.
B03 ComposeObjects mutates projection/frame/state but does not run actor handler
logic. WaitSchedulerTicks only yield/decrement; B14 transfer copies/decrements
queued display and adds total, never progression. Thus progress and source branch
latches cannot acquire gameplay changes between selection entry and transfer
return. completion_plan may describe the invariant-stable plan but implementation
must still execute calls in source order, never start transfer/actor/message early.
A future owner permitting asynchronous progression would invalidate this snapshot
and must use a reborrowed continuation. Debug checkpoints replay, not arbitrary
concurrent writes; no new memory-edit API is assumed.

WaitForAcknowledgeA D9C0 reads pressed A80 only. No edge -> YieldOneFrame, retry;
held A alone does not acknowledge. On edge it jumps D9BB, shared transfer exit,
which writes retained X into PrimaryUploadPending then returns. It is NOT a pure
boolean check or unconditional pendingFF. Native retained_submission_value is the producer's semantic pending-submission
fact, not universally a queue length. Direct B81C producer X starts0 and each
record advances by count+4 (count-minus-one data length plus3 header). Independent
matching-ROM message3 seven records counts[12,11,8,11,11,11,11] yield103=67hex;
message4 eight counts[8,10,10,2,8,7,11,5] yield93=5Dhex, and finalFF sits at that
cursor without incrementing it. B14 MessageOutput exposes this producer fact.
Incomplete route Wait60 decrements X to0. Transfer's blank-source D8F4 RTS
leaves it0. Once any row conversion runs, total-add D962 loop ends DEX0->FF;
saturation's second D998 loop also endsFF; D99E and D9BB preserveFF. Thus B14
TransferOutcome.submission_value=None means caller0 untouched on blank return,
Some(FF) means actual prepared rows; this is distinct from the upload Vec length.
On no-edge wait FFAE TXA/PHA saves producer X before SchedulerSlotOffset clobbers
it. FF6D ReadControllers does clobber X, but then PLA/TAY and PLA/TAX restore
the saved X before returning into acknowledgement. NMI saves/restores CPU registers.
Therefore all repeated no-edge waits preserve exactly the producer fact, including
0 and5D, not just nonzero. Contracts additionally preserve arbitrary synthetic
0/1/7F/FF facts. No CPU register or second queue is introduced.
Messages/transfer must retain this semantic return fact across waits. B03 owns pending
commit at interrupt; observation must distinguish preparation from committed video.

## Initial scan and complete land navigation

FindAvailableLand D6EF startsY0, AND mask D8C7+Y, enters PositionLandSelection on
zero. Otherwise INY/BNE loops; all256 blocked falls through withY0. Original
mask table has EIGHT bytes1,2,4,8,10,20,40,80; no five-entry bound instruction.
0..1E incomplete masks select earliest unfinished0..4. With1F and no completion
signals, source selects5 (mask20 clear), then reads CursorY+5 from first actor
triple43 and NameY+5 from NameX0=80 and NameX+5 from next instruction opcodeA9.
This is not a supported sixth ordinary land or an invented title screen.
Mask3F can select6;7F selects7. FF reads beyond the eight masks, into name data
and then instructions as data: first zero at Y21 decimal (D8DC operand00).
The hypothetical256-allblocked fallthrough is not reached with matching original
bytes. Native first_available_selector contracts caller domain with at least one
clear bit among original eight masks; FF is deliberately unsupported by this
bounded parameter catalog. It is not claimed equivalent for arbitrary corrupt
progression, and no source five-choice cap/error behavior is invented. Valid
ordinary interactive callers require a clear lowfive bit; B13 must prove natural
entry reachability or preserve explicitly observed alternate path before stage4
completion. This retained uncertainty is recorded separately.

Position D6FB writes stageY; slot0 cursorY [70,78,80,88,90]; slot2/4 animation
6F+Y; slot3 nameY[28,20,30,28,30], lowX[80,60,88,68,A0]; sets state80 in0/2/4/3,
render90 ONLY in3. Initial positioning does NOT reset animation2/4 clocks or
request menu sound; goes straight INPUT before any fresh composition/yield.

INPUT D71x reads pressed&0C. Nonzero stores direction and takes precedence over
A. LSR/AND02: UP08 ->index0 ->stepFF; DOWN04 ->index2 ->step01. Both08|04
also index2 (B01 normally filters, but game-written edge field is a consumer input).
Candidate unsigned compare5, CLC then wrap correction+5 forup or-5 fordown:
0up FF+05=04,4down05+FB=00. Store candidate stage before progress mask test.
If completed, reload saved direction and repeat immediately: no yield, sound,
composition or resampling among skipped candidates. One unfinished land can cycle
back to itself and still be accepted with sound. All0..1E masks, all5 starting
lands and both directions use independent explicit cyclic-order fixtures.

Accepted candidate D755 updates cursor/name/animations as above, but now resets
slot2/4 frame/tick0, queues2F once, HideSprites/Compose/Yield. It does not reactivate
four slots again or reassign slot3 render. No movement: A edge ->confirm, else
HideSprites/Compose/Yield. B/START/SELECT/LEFT/RIGHT have no back or confirm path.
Held directions/A do not move/confirm. No source terminal route or completion
condition is bypassed by a native added menu.

## Confirmation and B13 handoff

A edge (only after no directional edge) enters D79B. Matching ROM physical1D7AB
starts20 82 C2: actual JSR FadeOutPalette, rather than a neighboring label or RTS.
Late completion JMP D79B therefore enters the SAME complete confirmation body.
FadeOut; mode0; yield; disable; hide; explicit state slot1=0; fill first nametable;
LoadStagePresentation for selected original stage; ClearNonplayerActors;
BuildInitialTerrainColumns; screen script0; selected CHR list; fallthrough
BuildSelectedStageColumns -> BuildInitialTerrainColumns AGAIN; then
InitializeStageRuntime. Both calls are source-visible and separated by UI/CHR:
no deduplication. B04 owns map builds; B02/03 decode/commit; B13 owns runtime
initialization/background polling/spawn/timers/player/reset and shared run mode.
Selection returns typed StageRequest with named level, original selector and late
route increment; both shared Game frontends consume the same request. No renderer,
audio callback, host clock or browser-only menu rule enters this contract.

## Contract scope and limits

Every assigned region/unit is indexed; helpers and caller joins above close
normal/boundary/alternate branches, immutable data and observable effect order.
Tests execute actual pending domain APIs, never a fake CPU or duplicate rules.
Existing B01/B02/B03/B04/B10/B14 contracts own mechanisms already independently
covered; B12 contracts add enclosing selection ordering and canonical joins.
No gameplay implementation, broad emulator run, final audit, GUI, QA, toolkit
change, check-in or publication occurred. Compile/discovery/failing-area/green
infrastructure evidence and exact names/hashes live in checks/contracts/B12.

Initial eager enter_lands API and title actor mutation while constructing setup
were superseded before final checks by explicit ordered initialization operations
and LandPhase continuations. Early compile remains preserved as historical build
evidence, not a semantic validation of that discarded ordering.

B14 reconciliation adds only pending owner result declarations and strengthens
its existing blank/partial transfer contracts. Direct write_message enters B81C
without B800 actor clearing. B12 source-derived message3/4 lengths/cursors contract
that owner join; prior B14 evidence remains preserved. This corrects early wording
that called every retained submission fact a queue cursor.
