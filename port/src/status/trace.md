# B14 independent status source trace

Frozen source `reference/source/game.asm` SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Physical offsets include iNES16. Bank5 CPU8000 -> physical14010; bank6 ->18010;
fixed C000 ->1C010. Contracts are adjacent, native-discovered and intentionally
unimplemented. No original payload or second arithmetic implementation is stored.

Read entire controlling contract prompt/workflow/common/coding standards,
architecture/behaviors/debug API and existing B01..05/B10 declarations/contracts/
traces. Independently scanned source operands, labels, all45 primary/related
planning regions and source data; index in planning/contracts/B14-consumers.json.
Planning statements were navigation only. No research supplement exists.

## CPU basis

Independently opened actual MOS-authorized manufacturer reprint:
[Synertek SY6500/MCS6500 Programming Manual August1976](https://syncopate.us/books/Synertek6502ProgrammingManual.html),
sections2.2.1/2.2.2,4.2.1,7.4..7.9,8.11/8.12. Byte ADC includes incoming carry;
SBC includes inverse carry as borrow. CMP replaces carry, not accumulator.
PHP/PLP preserve/restore the borrow across blank comparison. Loads/stores and
index decrement preserve carry; index comparisons replace it. INC/DEC wrap8bits.
The NES uses binary arithmetic; these routines explicitly correct each digit
with subtract/add ten, never packed-BCD operations. Every arithmetic entry here
initializes carry; arithmetic loops retain carry only across instructions proven
to preserve it. Native code must express decimal/display semantics, no CPU model.

## Native money and exact source boundaries

Iteration011 replaces ordinary reward, total and transfer-source code arrays with
native quantities (`RewardMoney`/`TotalMoney`). An explicit blank-layout mask
preserves display distinctions that source payment, transfer and ending gates
observe. Original nondecimal data has a separate lossless compatibility case;
it is not assigned an invented numeric meaning. Template import and immutable
render/debug exports are the only ordinary display-code boundaries.

Reward producers name Pickup/LargePickup/ExtraPickup/Treasure/Timer credits with
native denominations200/1000/5000/100000/100. Ordinary credit, costs, theft and
transfer use native arithmetic. Paid dialogue costs30000 and health restore
costs300000; preserving lower digits is equivalent to the source upper-pair
funding check on decimal amounts. Source normalization still determines the
explicit `RewardPayment` result consumed by subsequent RNG. Transfer owns a
separate working quantity, batches at most ten100-unit transfers, and preserves
its original waits/sounds. Saturation preserves the original lower two codes.

Source reward0324..0329 has six display positions, total0334..033A seven. Their
lower positions, blanks and unusual codes remain observable through ending gates,
script11, queue normalization and payment returns. The isolated compatibility
adapter retains the original byte-wrap/carry/borrow/blank quirks for nondecimal
input; native numeric paths do not invoke those routines. Source-contract fixture
writes reimport display codes and assertions project the same old expected codes.
A test-only denomination-index adapter retains old source probes without exposing
that interface to gameplay. All20000 ordinary prefix/blank-format credit cases,
paid blank layouts, raw-code roundtrips and existing source branch/integration
contracts exercise the conversion. No expected values were changed.

## Rewards, theft and paid operations

Bank6 8B5A..8BE6 physical18B6A..18BF7: ActorApplyReward subtracts23 from animation,
queues sound then jumps through7 split pointers8BFE/8C05; sounds8C0C. Targets:
23 ->8B71 add2 at digit3;24 ->8B82 recover one;25 ->8B9C restore all;
26 ->8B9F clear protection/set special1,duration01E0;27 ->8BB0 life;
28 ->8BBB add1 at digit2;29 ->8BE7 reward dialogue. Sound selectors are
15,15,15,15,17,15,37. Animation61 enters8B77 (sound15/add5 at digit2).
Treasure8C13 sound37/add1 at digit0 then progression/player-state work (B08).
B08 owns collection trigger, handler selection, treasure/progression and sound
ordering; B14 owns these transactions, B06 owns special/protection timer stepping.
No ordinary indexed table domain beyond these7 is invented.

AddRewardDigit8BBF initializes C0. Add at selected index, store; result24 exits
unchanged. Above24 subtract24 using CMP carry1. Compare10; below exits. Otherwise
SBC10 yields low digit and C1 for the next more significant ADC with operand0.
DEY does not erase C. Passing index0 with carry saturates exactly reward0..3 to9;
trailing codes4..5 survive. Every add exits by INC dirty, including add0 and
saturation; FF wraps0. SaturateRewardDigits F83F..F84A and SaturateTotalDigits
F84B..F856 only fill4/5 upper codes, return A9 and do not independently dirty.

Bank6 AD6B..ADB8 physical1AD7B..1ADC9: theft increments the **same actor's**
local state first, including FF->0. Starting Y2 subtract1 through indexes2,1,0.
PHP/blankCMP/PLP prevents blank normalization from destroying borrow. Negative
result byte FF plus ten with C0 becomes9; explicit CLC propagates borrow. Final
borrow clears all6 codes to24. Without borrow, scan from0: skip existing24,
replace zero24, stop at first nonzero nonblank or after6. Thus000100->000000
blanks all6, but000102 stops at lower2 and leaves its following zero. Always INC
dirty. ADxx collision/cooldown/contact triggers stay B07-owned, same-frame effects
must be applied before the next actor slot and before composition.

DialoguePaidActorReward EE22..EE8A physical1EE32..1EE9B: B pressed exits with no
cost/effect. Two upper reward digits subtract3 transactionally into scratch.
Final borrow exits without modifying status, dirty or actor. On success commit
both, replace leading zeros until nonzero or6 codes, increment dirty. B15 then
changes saved actor handler0F, flags&E0, x/y80, profile|80. These actor effects
are caller-owned and occur only after successful payment, not on underflow.
Lower4 digits do not fund insufficient upper-two payment.

ApplyOverlayValueCost E2FE..E353 physical1E30E..1E364: B06 standing-only SELECT
edge invokes this, not a new player menu. Scan overlay Y by4 to first negative.
Read flag two bytes before that record; zero means no payment. Upper2 **total**
digits subtract3 with scratch/borrow, failure preserves all state. Success commits
both, blanks leading zeros across7 total codes, calls ClearStatusOverlayFlags,
INC UI latch AB. It does not dirty reward. Caller comment claiming 'apply option
rule' is false: callee clears health flags, with no title-option comparison.

## Overlay health/lives and source defaults

Iteration010 replaces sprite-shaped canonical health with native per-segment
availability and Filled/Depleted state. `capacity()` is the available prefix,
and `exhausted()` preserves the first-flag death gate. Damage, recovery and paid
restoration no longer read coordinates, tile codes or sentinels. Appearance is
owned separately and imported/projected only at template/render/debug boundaries.
Nonzero depletion attribute codes remain lossless in the boundary state. Sparse
flags and locked-state restoration are preserved rather than compressed into a
single count. Extension explicitly reveals the chosen bonus slot and updates the
presentation terminator; its capacity change uses native availability.

Existing fixture assertions now use immutable overlay projections. Fixture writes
use a test-only reimport adapter so no mutable sprite layout is exposed to game
rules. No expected values were changed. Source contracts cover every capacity and
depletion pattern, all negative sentinels, raw attribute/geometry roundtrips,
player death, object recovery/extension and transactional payment. The historical
iteration005 prefix-helper change is superseded by this representation conversion.

CopyTitleOverlay D60A..D61F copies13 stageUI bytes and21 overlay bytes. The5
four-byte health records hold Y,tile,depleted attribute,X and a terminal byte.
Initial first3 records Y20, fourth YFF; fifth is behind the sentinel. All have
attribute0. Damage EB5A..EB81: option0 increments alternating latch and skips odd
events; every other option damages every event without changing latch. Scan
until first negativeY or nonzero depleted flag; set preceding record flag1.
Repeated normal damage thus marks third, second, first; B06 then sees first
flag0342 nonzero and enters death trajectory. The death wrapper guards further
ordinary damage; a raw exhausted-health damage call is not a new health API rule.

Recover-one8B82 clears alternating first, scans until negativeY or first nonzero
flag; at a flag clears just that flag. A full/hidden list returns unchanged except
alternation reset. Restore F812..F821 clears all5 flag bytes and alternating,
preserving geometry, tile, terminator and unlocked capacity. Animation2B8B38..8B59
(stage1 also writes AE via B08) chooses fourth if Ynegative, otherwise fifth if
Ynegative; full capacity returns. Sets chosenY20 and nextYFF, then restores all.
No health number independent of these flags may diverge from them.

Life8BB0..8BBA compares **equal9**, not >=9; other bytes increment wrapping.
Reload DC49 selects gameover at lives0 before decrement. Life-loss path DCA5
otherwise decrements lives once and restores health; ordinary checkpoint reload
retains lives. B13 owns reload decision and B16 owns gameover continuation.
Gameover DEB9..DEF6 copies33 bytes (8 sprite records+FF) and performs F0
composition/yield boundaries under DMA guard, audio9 then reset; not merely a
240ms delay. Those scene/reset callers remain B16/B13-owned.

F822..F83E title option2 only remaps animation25->28 and26->24. All other animation
bytes and options0/1 unchanged. Tests cover all byte inputs without inventing
option descriptions. Title/B12 maps source option selectors to the semantic rule.

## Countdown and timer conversion

Modernization iteration009 replaces canonical display digits with `Countdown`:
ordinary time is a bounded seconds quantity and expiration is an explicit state.
Ordinary ticking subtracts one second, conversion consumes ten-second groups,
and only rendering/observation encodes the three digits. Initial template, stage
pair and game-over writes are explicit display-code imports. Upper-pair entry
replacement retains the prior units; reload replaces all three codes as before.
Divider cadence, caller gates, sound submission and cooperative waits remain
with their existing owners. Existing assertions now inspect `display_codes()`;
none of their expected outputs changed. All1000 decimal values are source-backed
for decrement/warning/delayed expiration and conversion, with a composed warning/
player-death regression and the existing exact audio/finish-wait integration.

Nonnumeric original overlays and raw source selectors retain an isolated
DisplayCodes compatibility state; they are not coerced into a seconds quantity.
Boundary round trips and arithmetic are compared against the faithful algorithm.
Source inspection identified an existing limitation: shared `subtract_codes`
normalizes24 (BLANK) to zero, while E2B5/EF8D directly subtract raw codes. For
example synthetic[00,00,24] ticks to999 in the faithful build but raw source SBC
would give[00,00,23]. This iteration preserves the faithful result rather than
mixing a source-fidelity repair into the representation change. Natural reachability
of this input at a timer gate is unestablished; raw-code compatibility tests are
not original-fidelity certification. A separate follow-up should trace actual
raw preset/overlay gate reachability before deciding whether a repair is needed.

E28C..E2FD physical1E29C..1E30E: states10/0D return before any timer/warning.
Hundreds tile1E forces state0D and clears late route, preserving divider. Otherwise
DEC divider (0->FF), returning through warning unless new zero. At zero restore
60, subtract1 from digits units/tens/hundreds using stride4. Initial000 decrements
to999; **result**000 becomes[1E,19,24]. Death transition is next gate call, not
immediate same-call state change. Warnings test resulting hundreds0,tens<3,
composition tick&0F==0, queue36 even on non-decrement calls. Expired representation
has nonzero hundreds and does not warn. Stage initialization copies29 timer
bytes and stage pairs (five physical pairs:50,50,60,50,60), divider60. Reload
restarts digits only when current stage audio==0A, and clears units. B13 owns
stage/reset/audio wrapper conditions and dynamic named-stage defaults.

Conversion EF62..EFE1 physical1EF72..1EFF2: nonnegative current stage audio waits
WaitTicks X0, which B01 proves is256 yielded ticks, then sends stageFF before
examining composition parity. Negative audio skips that wait/command. Even parity
returns untouched. Odd parity freezes player animationTick0, clears units tile,
then tests only hundreds|tens. Nonzero subtracts1 through **two upper timer**
digits, adds1 at reward digit3 through upper4 with encoded carry/saturation,
queues15, INC dirty and returns. Timer001..009 therefore gives zero reward;010
converts once,100 ten times. After upper pair reaches0 the **next odd call**
waits60 before INC scene completion. Explicit Entry/AfterAudioWait/Running/
AfterFinishWait represent the cooperative continuation, not a blocking native loop.
B06 protection timer and special16bit duration loops E354..E386 are caller-owned:
positive protection decrements, negative/zero retained; positive special ensures
stage audio8, decrements16bits, at zero clears mode/restores stage command.
Source01E0 initialization above must reach that same B06 state, no duplicate timer.
Final departure F857..F8F1 and race goal FDD5..FDF4 write divider0 every frame;
this makes next ordinary tick FF, not a permanent independent freeze switch.
B13 player/dialogue gates decide whether countdown called at all.

## Queues and displayed-value transfer

DE6C..DEA2 physical1DE7C..1DEB3: busy primary queue returns before mutation or dirty
clear. Scan6 reward codes for first<24; then only subsequent exact24 becomes0.
Leading24 preserved. If none<24 CPY6 leaves Z1, DE7E BEQ jumps to template copy;
this misleading 'countdown join' label is status fallthrough, no timer entry.
Copy12 bytes (destination2043,count7,marker2C,six codes,zero,FF), requestFF and
clear dirty. DE A3..DEB8 copies13-byte totalUI template destination206C,count8,
marker2C,seven codes,zero,FF only if idle, clearing AB. Queue bytes are prepared,
not immediately visible pixels; B01 boundary/B03 commit owns final publication.

Transfer D8D9..D9BC physical1D8E9..1D9CC sets destinations220A and226B; FFC7..FFDD
physical1FFD7..1FFEE chooses206C and2043 then same source-check entry. If source
last code0329==24 return immediately, before template copy. Otherwise copy total
count8+9payload bytes at0400, reward count7+8payload bytes at040C; appendFF.
**Only queued source0410..0415 is decremented**, persistent reward0324..0329
survives; total0334..033A updates after every increment. Subtract1 from source
upper4, blank leading zero run across6. Add1 to total upper5, with blank/carry
handling; force lower2 total0 in both queue/persistent copies. Carry overflow
fills persistent and queued upper5 with9 using returned A9. Completion checks
queued source last code0415==24 after each step, before batch count decrement.
Therefore partial batches AND exactly tenth-step exhaustion submit without
sound/wait. Otherwise ten steps request primaryFF, sound15, wait2 scheduler ticks,
then next batch. Saturation does not stop draining source. State/render inspection
cannot advance this continuation or consume its events. WaitForAcknowledgeA D9BD
checks **pressed** A, yields until edge, and uses shared upload-request return;
B12/B16 own invocation/input continuation.

## Stream/message formats and all consumers

Modernization iteration 001 consolidates only the shared marker/raw-codes/zero
payload in `number_payload`. B580 row framing, B81C message counts/submission
values, and DE6C/DEA3 normalization/busy policies remain at their existing callers.
`numeric_producers_preserve_every_raw_code_and_their_distinct_framing` checks all
256 raw code values against these source-derived producer contracts.

Bank5 B580..B61B physical17590..1762C. Update always expands stream4 (5 clearing
rows descending20A1..2021), then player0F forces9; others use status selector.
PointersB622/B62C have10 entries:0/1/4/9 are5-row streams;2/3 point to runtime
reward/total count fields;5/6/7/8 are counted14-byte fragments (8 is blank fallback).
Destination high0 ends. Each row creates fixed count1D; literal00..7F copies1.
80..9F repeats next byte (low7bits)+1 times. A0..FE takes low nibble as shared
fragment selector. Original shared selectors2/3 use runtime counts7/8 (8/9 bytes,
including marker2C and trailing0). Selectors5/6 test A3 masks2/4,7 tests A1 maskFF;
false chooses8. Save outer cursor/index around insert; counted fragments copy
count+1, not count. FF is row terminator, requestFF, yield1 before next destination.
Original streams have30 expanded tiles per row, FF after them; bounded pointers/
cursors do not wrap. Trailing3 zero bytes175? stream tail are preserved generic
source data, not extra rows. No copied source glyph payload is embedded in tests.

Gameplay DB51 toggles selector and composes before updating these streams; a
nonzero selector gates regular simulation. Dialogue entry ED46/ED82 composes then
updates streams; B15 owns bank/continuation restoration and suppression work.
B03 sprite overlay gates exclude status-selector/dialogue/player0F conditions;
status rendering cannot become an independent always-visible HUD.

Messages B800..B86B physical17810..1787C: completion actor latch nonzero is direct
mode; otherwise sceneComplete0 gives mode0; sceneComplete!=0 gives1, specialRoom
nonzero gives2. Modes1..4 clear only actor state slots0,2,3,4 (not slot1 or render
bytes). Offsets B86C are0,19,2C,3C,52; all5 records tested. Each record has destination
high/low,payload selector. Negative high ends and requests upload. Selector0 becomes
stage+2. Split payload pointersB9C1/B9D8 have23 entries; runtime selectors10/11
are total/reward, other21 have independent bounded counted data. First payload
0 is count0, so copies1 byte; counts through12 mean lengths through13. Queue
stride is payloadCount+4, no yielding within message build. Tests retain every
mode's metadata and all23 counts, not just representative text.

WriteScreenScript selector11 appends exactly0334..033B (seven total codes+trailing
zero) via0329+X for X0B..12; B03 already contracts script decoding/current PPU
address, B14 supplies this exact total tail. Ending E1D5 reads total first code
and E1DD final code, F8xx departure reads reward third code7; B16/B17 own gates.
Runtime composite selectors0..3 alias04C0 in B03. No direct04C0 writer or status
writer was found; only generic indirect FillCpuRange and reset clearing are
present. Prior B03 'B14/B16 author frames' claim is superseded, not a basis for
inventing digit graphics. B16 must finish natural frame-consumer reachability;
B03 alias contracts remain intact. This is an explicit integration uncertainty.

## Limits and ownership

B08 collection and B07 theft triggers, B06 damage/timer invocation, B12 input and
selection, B13 initialization/reload/gates, B15 paid actor aftermath, B16 scenes,
B17 ending/value selection, B18 queued audio timing are caller-owned. Tests here
assert the shared source transaction, immediate mutations and requested effects;
later workers must call these interfaces at source order rather than defer all
mutations. No new cohesive area was found. Native catalog parsing may reject
truncated external formats; original routines lack such validation and valid
original programs are bounded. No claim is made about arbitrary corrupt ROM
execution. All source behavior is pending implementation; no QA or final audit.

## B12 acknowledgment producer reconciliation

B12 source worker independently followed direct B81C and D8D9..D9BB return facts.
Direct message builder differs from B800 SelectStatusMessage: it does not perform
B800 actor clearing. New pending `write_message` returns `MessageOutput { bytes,
submission_value }`; the final negative destination requests upload without
incrementing X. For each ordinary record X advances count+4. Message3 counts
12,11,8,11,11,11,11 produce67hex; message4 counts8,10,10,2,8,7,11,5 produce5Dhex.
B12 acknowledges after message4 and must retain5D, not substitute FF/true.

Incomplete completion path Wait60 leaves X0. Transfer's early blank final-source
D8F4 preserves that caller fact, represented by `TransferOutcome.submission_value
=None`. Any processed rows finish D962 total loop with XFF (or saturation's D998
loop alsoFF); D99E/D9BB return withFF, represented Some(FF). This is not total
queue byte length. Strengthened existing blank/partial transfer contracts assert
both results; prior evidence remains preserved. B01 yield saves producer X and
restores it AFTER controller sampling, so acknowledgment retries retain the fact.
No register map, second implementation or live game logic was added. Details/
checks and43 B12 source contracts are linked in planning/contracts/B12.md.
