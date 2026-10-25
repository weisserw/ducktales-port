# B05 shared motion, sampling, terrain responses and contact trace

Stage3 worker `/root/behavior_contracts/b05_collision`; contracts only, behavior
visibly pending in mod.rs. Executable cases are contracts.rs. Source game.asm
SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc and
reading.asm SHA256 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Fixed CPU C000..FFFF adds10010 for physical offsets; bank4 BD80 maps13D90.
Primary inventory75 regions is navigation, not semantic proof. Source bytes and
ledger identities remain frozen. No CPU execution or second oracle is authored.

## CPU and ownership basis

Read entire prompts/contract.md, workflow/common/standards/architecture/debug API,
B01..B04 adjacent traces and handoffs, toolkit docs/development.md platform table,
NES and nes-native documentation. Independently consulted MOS-authorized Synertek
MCS6500 programming manual6500-50 (August1976),
https://syncopate.us/books/Synertek6502ProgrammingManual.html , sections2.2.1/2.2.2,
3,4.2.1,8.11/8.12,10.1/10.7/10.8 and appendixB. ADC includes incoming C; SBC
subtracts inverted borrow; compares set C for unsigned>=; loads/stores/AND/ORA/
INC/DEC/push/pull A preserve C. Eight-bit overflow wraps; PHP/PLP restore all
arithmetic flags. Source NES binary arithmetic applies, not65816 widths.
Native functions use semantic wrap/borrow outputs rather than emulated registers.

B04 alone owns WorldCache128 collision/attribute cells and descriptor decoder.
B05 borrows cache and requests TerrainProbe{room_row,world_page,block_coordinate,
quadrant}; it adds no competing cache. B10 owns stable slots/allocation/spawn
suppression. ActorContact is a borrowed view of canonical actor fields for these
pending signatures, not an independently ticking actor store. PlayerContact
carries immediate and queued state separately. B06/B13 must commit each at the
source time. B03 owns render flags/animation observation, B18 audio request order.
B04 already contracts burst video replication and descriptor particle origins;
B05 contracts the resulting shared four-particle allocation/motion initialization.

## Source bounded data and caller domains

D3CE..D432 are101 signed Y offsets; D433..D497 are101 signed X offsets; D498/499
are page parity bits00/40. F498..F4B7 heights andF4B8..F4D7 widths each32 entries.
D1E2/D1F2 are16 split response pointers, decoded targets independently reread:
0..3 D199 pose;4..7 D16B mark;8 D1B1 momentum;9 D176 damage;10/11 D1BF exit;
12/13 D176 damage;14 D1D1 mark+momentum;15 D162 solid. All are fixed bank.
Response enum expresses these meanings, not raw CPU addresses.

F313 horizontal preset and F321 vertical preset double caller A and read F330/331.
**Correction to maintained comment restricting selectors to0:** WalkingBounce
82AF and ProjectileThrower831F enter with A=0; JumpingRewardStealer ACCF loads
A=1, stores speed integer, INC state (preserves A), then calls setter. Thus reachable
selector1 reads F332/F333, also bytes of the following LDA immediate instruction.
Named ROM consumer recipe physical1F340, length4 includes both supported words.
No instruction is executed or hardcoded as a replacement constant. Vertical setter
has no direct symbolic/numeric caller found; same bounded setter API is retained,
without inventing more selectors. B07 independently traces stealer behavior.

Bank4 BDC7/BDCB/BDCF contain4 Y-offset/X-offset/animation-tick entries, consumed
index3..0. Suffix BDD3..BDFF physical13DE3..13E10 has no found consumer in these
bounded4-element arrays; preserve as raw B02 source record, no invented padding
meaning. D110..D11F burst descriptor finite selectors0/6 belongs B04's existing
changed_burst contracts (including paired writes and particle requests).

## Sampling and every terrain response

D350 signed offsets are sign-extended separately. X16-bit addition wraps first;
result page bit7 rejects to class0. There is no check of layout width or positive
page count; page parity aliases cache every two pages. Y signed-add carry changes
room-row byte; if changed, source returns offset-sign parity (negative underflow
class1; positive overflow class0). This remains true at row0/FF. Fraction is never
sampled. Valid cache index=(page&1)*64+(y>>5)*8+(x_low>>5); quadrant=(y&10?2:0)|
(x_low&10?1:0). Full coordinates retain low nibbles for snap. Descriptor block
coordinate is cache_index&3F. B04 decoder resolves room-row stride/ninth-bit.

D120 clears render bit04, resets aggregate, visits start ascending/count descending
inclusive (count is number-minus-one), stores raw class under remaining index.
Ordinary classes OR aggregate; actor slots nonzero stop if accumulator A bit0 is
set. Player always completes. Class3 reads descriptor via CDE2; that helper saves
X/current bank/aggregate, decodes room/block/quadrant, restores aggregate and bank,
then X. SelectPrgBank16K clobbers X to bank, so the restore is necessary. Native
callback must preserve actor identity/context by construction. Descriptor high
nibble dispatch is independent of class3's raw result; do not OR3 automatically.

Solid sets aggregate bit1. Mark ORs render04 but leaves aggregate untouched; its
continuation tests render A bit0 for nonplayer early termination. Damage always
solid; only player, no protection/special mode, remaining1 can queue6. Animation6
additionally requires current start0A. Pose normally solid, but exact player/
start7/animation6 queues7 without solid. Momentum normally solid; player remaining1
increments byte latch. On FF->0 the BNE is not taken: falls through low-screen
exit and skips solid for that sample. Remaining0 next sample can still set solid.
MarkMomentum marks first then joins player momentum or actor solid; CPX/BEQ proves
no fallthrough into pointer bytes. Exit queues8 at player Y>=D0, never solid;
nonplayer sees dispatch A=D1 (high pointer load), so exits after one sample despite
aggregate0. Player response writes queue, not immediate PlayerState. B13 DBC5
commits queued reactions after composition/spawn, filtering6 by protection.

## Alignment and integration

D49A positiveX subtracts probeX&15 from full16-bit integer X with borrow; negativeX
D4B2 adds16-(probeX&15), including full16 when nibble0. PositiveY D4D6 subtracts
probeY nibble; negativeY D4C8 adds16-nibble; Y wraps8-bit. Fractions unchanged.
F234/F252 integrate unsigned horizontal magnitude8.8 into X integer16+fraction8.
Low-byte carry/borrow propagates to page only when needed. Returned carry is full
world overflow for positive; full no-borrow for negative, not mere page crossing.
F270 add/F286 subtract encoded vertical velocity into Y integer8+fraction8.
F2A1 falling is identical encoded subtraction; signed positive rises, negative
falls, but its raw unsigned borrow must not be confused with physical escape.
F2B8 gravity subtracts unsigned fractional step from velocity16 with wrapping.
MoveAndProbeTerrain F298 PHP/PLP preserves integration's carry across all probes.

F0F8 positiveX removes on full overflow; F13B negativeX removes on borrow.
Otherwise terrain snap precedes facing update (positive sets40, negative clears40),
then handler8 actor scan. Player/handler9 skip; positive additionally skips24,
negative does not. Positive correction subtracts penetration; negative adds it
with C clear from contact. Boundary release clears render and state, sets spawn
identity FF, changes matching special-slot latch(80|slot) to3C; other latch stays.
These are same-frame B10 effects, not deferred removal. Fractions stay advanced.
F192 positiveY removes on add carry; F1A8 negativeY removes on subtraction borrow;
accepted paths terrain snap only, no gravity/actor scan. F1BE falling applies
gravity BEFORE integration/probe, removes when integration has NO borrow, snaps
solid terrain and clears velocity, else profile!=0/handler!=9 tests actor landing,
subtracts penetration and clears velocity. F1F9 rising integrates/probes BEFORE
gravity and ignores saved movement boundary. Terrain uses negativeY snap then
velocityFF00. Actor uses Y+penetration+1 because SEC precedes ADC; also velocity
FF00. Profile0/handler9 bypass only actor scan; they do not bypass terrain.

## Contact predicates and transition helpers

Further executable contact/lifetime cases and exact player-scan flag derivations
are recorded below as authored; pending declarations above implement none.

D202 horizontal player scan visits15..1, requires render80/profile80, ignores
state80 and render02. Candidate profile&7F selects extents. Height comparison is
strict; width adds1 after successful height CMP (C0), so horizontal equality is
contact with penetration0. World-X subtraction uses full16-bit borrow to choose
whether to negate the LOW BYTE; high difference is never rejected. Two-page
separation can alias, but lowFF from an unsigned full positive difference must
not be mistaken for signed-1. Preserve this oddity instead of a Euclidean distance.

D22D and D2EC pose adjustment source comment says seven. **Correction:** equality
CMP animation3 leaves C1; PLA preserves it; ADC7 actually adds8. It wraps a byte;
248+8 becomes0. Horizontal/vertical player scans apply this only when profile&70
nonzero. Generic actor-to-player F456 applies it to ALL profiles. Width/height
operands must not be casually renamed full sizes or combined with player extents.

Horizontal protection allows only handler08 or15(hex), then penetration=width-
distance, aggregate bit1. Flagged profile&70 immediately writes PlayerState6
unless special mode, which increments target hit marker. No queued write occurs.
First success returns carry clear with matched X; exhaustion X0 returns carry set.
B06 E5A7/E5CA clamps penetration12 before directional displacement, updates actual
spawn direction then terrain; E913 swing uses returned actor handler exceptions
08/15/42. These clamps/swing policies remain B06's own caller contracts.

D291 vertical scan requires render80, render02 clear, profile80, ignores activity.
Horizontal extent strict; vertical accepts height equality (height+1). It scans
ALL qualifying slots, counts contacts, overwrites last penetration each time,
returns carry clear via CPX count if count>0. Final matched contact is lowest slot,
not nearest penetration. Protected player rejects only flagged profile&70;
ordinary platforms still collide. First eligible descending(velocity integer
negative)/animation6/motion04 contact sets contact flag1 and increments actor hit.
Only one such pogo path runs per scan (local latch). If increment FF->0, its BNE
fails and that SAME contact falls through profile reaction: special mode can
increment marker again to1, or ordinary mode writes immediate6. Later flagged
contacts retain immediate damage/special-hit semantics even after first pogo.
No-contact must remain distinct from penetration0; report any_contact is source
carry-clear outcome and bits are caller-mergeable delta, not another shared RAM.
B06 E970 landing subtracts penetration; E9F5 rising adds penetration+1 via SEC/ADC.
Both merge subsequent terrain contact; EA3B saves/restores render flags around
probe, so a temporary terrain mark there does not persist as an object flag.

F332/F3B8 solid actor scans require source visible; target !=source, target active
and visible, handler08, use TARGET profile&7F but do not require profile80.
Only slots15..1, never player0. Horizontal strictheight/inclusivewidth; vertical
strictwidth/inclusiveheight. First success returns matched slot/penetration,
restores caller X; exhaustion returns no contact. Horizontal clears contacted-slot
scratch before source visibility test; API Option naturally expresses no slot.
Vertical source has no matched-slot scratch write; semantic Contact carries the
identity already known during native scan, not a fabricated CPU side effect.
Neither helper reacts to player or mutates target hit. Wrapper ORs contact bit.

F43E generic actor/player test needs actor render80 only; no state/profile80 gate.
Strict height, inclusive width; same low-byte/full-borrow world-X quirk. Unflagged
profiles return carry-clear permission regardless protection/special mode.
Flagged+ordinary mode checks protection; if unprotected writes PlayerState6 and
returns permission. Flagged+special mode increments hit then falls through SEC:
**returns rejection despite geometric contact and mutation**, even protected.
B07/B08/B09/B11/B15/B16/B17 callers must distinguish hit-side effect from carry-
clear continuation (collection, theft, attach/launch, dialogue, goal, carry).

## Clear/lifetime and particle branch derivations

F2C8 ordinary clear starts X14 and writes array+1,X, then DEX/BNE:15..2 inclusive,
NOT1..14 (maintained comment inaccurate). Slots0/1 survive; only render/state/
spawn identity clear, all other fields remain. Stage1,row<2 bypasses ordinary
clear; X1 then0 accesses slots15/14. OR state10 preserves them; timer high0 skips
arithmetic. Otherwise low-=3C; on borrow the immediately following SBC0F sees
C0 and subtracts another10(hex), then explicitly CLC so high-=1. Low0/high2 becomes
B4/1, low3B/high2 becomesEF/1, low3C/high2 becomes0/2. DEC X loops twice. These
are literal source encodings, not a generic decimal countdown. B10 owns canonical
actor storage and B13/B04/B12/B16/B17 call transition clear. B10 must preserve slot1
and immediate identity effects; no synthetic permanent terrain mutation history.

Bank4 BD80 receives origin(Y,X) and allocates via F7BE highest state-sign-clear
slot15..1 (state7F is free). Parameter index3..0 preserved across allocator. Set
stateA0/renderA8/animation2A/frame0/profile0/handler0; leave fractions, velocity,
spawn identity, hit marker and other fields untouched. For EACH actor CLC then
originY+offsetY, STA Y, LDA originX then ADC offsetX **retains Y overflow carry**.
Player page is copied directly; X overflow does not carry into it. YF8+10 =>08
with carry; XF8+10+C =>09, page unchanged. Copy per-index tick. Full pool stops
batch, never tries player0; sound16 always queued once even zero allocations.
B04 changed_burst requests this origin after its primary PPU rows; source maps
bank4 then tailcalls, and does not restore earlier bank at this helper. Native
asset context eliminates mapper state while keeping caller resource obligations.

## Caller and area reconciliation

The complete companion `planning/contracts/B05-consumers.json` inventories every
one of75 primary/related regions, every assigned unit, all direct symbolic/numeric
callers and relevant shared-field source readers/writers. Independent caller
windows were reread: bank4 completion bounce; bank5 scene bounce/walk; bank6
all motion/terrain/overlap consumers; fixed player E5xx..ECxx; dialogue/escort
FCxx..FExx; departure/liftF8xx..FAxx. Callers often use scratch aggregate bit1
AFTER wrappers; generic player test does not clear that aggregate. Domain report
separates geometric contact, permission, penetration and motion/lifetime so these
cannot be conflated. Banks4/5 scenes and B06/B08 arcs deliberately choose gravity
before or after integration; primitive APIs leave that ordering caller-owned.

Additional B10 region523 helper80C1 is a different actor-to-actor scan with bank6
extents814A/816A, source profile, target handler exceptions08/09/25, animation2A/
2B preserving1B. It remains assigned B10 for independent whole-area tracing and
contracts, not silently replaced with fixed-bank handler8 scans. B01/B03 own
bank/foreground and atomic video boundaries; B04 burst video already has contracts.
B06 must trace unusual E8C2 ObjectScreenY,x caller and separate vertical room
eligibility, plus source pose/probe merge. No new cohesive behavior area found.
No unresolved B05 rule is delegated as an exclusion. B04-discarded-bottom-staging
and B01 boundary integration uncertainties remain with their prior owners.

## B09 caller continuation reconciliation

A04C second positive wrapper call does not reloadA/Y. AND1 leavesA1; accepted
F0F8handler24path skips actor scan atF11F and preserves returnedY. Ordinaryclass1
D12x leavesYcount0; class3Solid response leaves descriptorhighnibble5 (for synthetic
50), then D13Ereturn and SnapActorPositiveX preserveY. MovementReport now declares
repeat_positive_count:Option<u8> for this source-backed continuation, Some0/Some5
in direct actual-owner contract external_positive_wrapper_returns_actual_repeat_count.
B09MotionFacts.continuation_remaining maps it; no CPUregister or RAMmap is added.
Old69cases retained, this newcase makes70. Stage4 must implement return andadapter;
currentcode is explicitlyunimplemented. Exactreconciledchecks in B09results.
