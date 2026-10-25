# B18 independent whole-area trace (stage3; no implementation)

Frozen maintained source `reference/source/game.asm` SHA256
c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading SHA256201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Bank3 CPU8000..BFFF = physicalC010..1000F (16-byte header included).
Queue/NMI live fixed bank7: F752..F76C physical1F762..1F77D and
C0F8..C15D physical1C108..1C16D. Planning map is navigation only. Entire
controlling contract/workflow/common/coding standards/architecture/behavior/debug
API and adjacent runtime/assets/presentation/actor families/player/progression/
dialogue/scenes/finale traces reviewed. No research/current.json exists. Metadata
index B18-consumers.json preserves all58 original regions/units and references.

Arithmetic uses the actual MOS Technology MCS6500 Programming Manual chapters
[2](https://lbaeza.neocities.org/mcs6500/6500_ch02) and
[3](https://lbaeza.neocities.org/mcs6500/6500_ch03), consulted independently.
CMP/CPX/CPY set carry for unsigned >=; ADC consumes carry, SBC subtracts inverse
carry; loads, stores, byte increments/decrements and AND/ORA preserve carry.
ROL/ROR include carry. Byte operations wrap. Indirect indexed addressing adds
its index across a page without changing processor carry. NES2A03 arithmetic
is binary. Native interfaces retain semantic producer results, never processor
flags/registers, CPU execution, or a RAM/bus map.

## Canonical queue and every reaching producer

QueueStageAudioCommand F752 writes CurrentStageAudioCommand BEFORE falling into
F754 enqueue. Enqueue preserves caller X; producer byte compared against88:
empty88 accepts/C1, ordinary occupied<88 drops/C0, occupiedFF drops/C1. Accepted
path LDA/STA/INX/TXA/AND/STA/LDX do not modify that comparison carry. Producer
advances modulo8 only on acceptance. Stage command still changes on rejection.
Submitting88 itself is legal source behavior: it advances producer but leaves a
sentinel hole, so consumer stops before later commands. A VecDeque length model
would lose this effect. Queue stores encoded commands, not an emulated RAM area.
B08 Context now borrows this ONE queue; no per-frame/per-context producer snapshot.
Every reaction call must obtain its own Submission and immediately consume its
random_increment result, including successive calls within one actor invocation.
B15 reveal06 returns the final particle sound's exact Submission provenance to
B06/B13's nondefault FirstActorIncrement; B11/B13 slot1 integration remains typed
and pending, not an implicit false/default/global flag.

All symbolic queue callers and their preceding producer instructions are indexed
in B18-consumers.json; this includes collision bursts, objects, ordinary enemies,
bosses, title/land movement, progression/reload/terminal scenes, dialogue glyphs,
finale and stage selection. Direct callers request source IDs, never direct
synthesis. FC/FD have driver entries but no direct LDA#FC/FD queue producer found;
source stream/header data must not be mistaken for such a producer.

NMI C10B selects bank3, reads ring consumer, compares88, clears each consumed slot
BEFORE incrementing index/dispatch, loops to first88, calls8000 exactly once, and
restores prior bank/registers. Display suspension and foreground pause never skip
this tail. B01 retains clock/bank ownership and atomic interrupt-before-foreground
policy. Audio frames are not calls to Game::audio or sample production.

NMI parameter provenance matters: scheduler loop C0B5 counts Y4 down to0 on EVERY
NMI path, including suspended display. Bank select changes no Y. Thus first
command FC sees0 and stores1, FD sees0 and stores0. Later commands inherit the
preceding command's final semantic parameter. Primary install finishes LDY2,
INY to3 (preset high byte), then only mask shifts/loads: subsequent FD sees3.
FC increments that parameter with byte wrapping; FD preserves it. Secondary
install/stop parameter depends on actual selected channel/preset path; it is not
always0. command() takes explicit traced parameter; interrupt() must retain its
command-local producer across the drain without inventing a persistent CPU Y.

## Dispatch/install/stop, CPU8003..823C physicalC013..C24C

FC stores parameter+1 in tempo, FD stores parameter in fade, sets level1 and
phase&1. FE marks busy and stops secondary high priority; FF marks busy and stops
primary low priority. Header selection doubles source command byte (byte ASL),
reads little-endian pointer at8A40. All56 actual headers are indexed; IDs0..15
have primary priority07 (12..15 alias0);16..55 are secondary priorities50..F0,
masks02/08/0A/0F. This is a closed NATURAL table domain, not runtime source input
validation: raw commands38..7F can index following data, and80..FB alias after
ASL. Such foreign requests have no established natural producers.

If header low nibble nonzero, compare FULL header X against current low nibble,
not header's isolated low nibble. Reject only strictly smaller. Merge current
high nibble OR full header. Busy1; clear tempo/fade. Each of four records receives
its two-byte cursor, next14 primary working bytes zeroed. Unmasked channels also
clear15 secondary/modulation bytes; masked channels preserve those. Header cursor
walk increments with full16-bit carry (1, then2 each); final preset base at bytes9,
10. Mask rotates right once/channel then shifts four at pass end to restore low
mask. Busy0 only after complete install.

Secondary header requires full priority >= current high nibble and preserves low
priority. Cursor header+2 with page carry, shared delay/repeat0. New mask header
byte&0F; selected channels clear secondary15bytes. Oldselected/newunselected
channels ResumeOrInitialize: existing primary period invalidates high cache then
restores preset; absent primary period silences hardware, and only nonnull primary
cursor then loads preset. No primary state copy. New mask rotates/restores once.

FE clears high priority, walks old mask, clears selected secondary state and
restores primary through same helper. IMPORTANT: FE has NO final four-bit discard;
its mask remains rotated into high nibble after four channels. Current low active
bits are zero; next frame rotates those bits through and discards them. Do not
mistake retained high nibble for continued secondary ownership. FF clears low
priority/tempo/fade and primary cursor pairs only, preserving secondary state;
it advances channel offsets without rotating mask. Neither is ordinary primary
opcode9 or secondary opcode6, which have different mask/cache/silence effects.

Preset restore masks note_control low5, multiplies by4 in byte arithmetic, copies
four bytes from preset base via indirect indexed addressing. AudioLoadPreset
88D9 instead takes full X from the streamed index, repeatedly adds4 modulo256,
then adds full16-bit base, advancing source with carry. Both copy only four
modulation parameter fields, not counters/cache. Named native fields correspond
to domain state rather than a second31-byte RAM record.

## Frame, level/envelope, pitch, CPU823D..8512 physicalC24D..C522

Every frame writes4001=08,4005=08,4015=0F BEFORE incrementing wrapping phase and
checking busy. Busy returns before any channel counters/fade/shared delay. Four
hardware voices pulse1,pulse2,triangle,noise correspond source ChannelsLeft4,3,2,1
and APU offsets0,4,8,12. For EACH channel increment pitch counter and level counter
before secondary; low mask bit selects secondary. StatusStreamSelector bit0
suppresses primary; otherwise nonnull primary cursor increments envelope counter
then runs primary. Unowned inactive voice silence is30 to control; triangle is0
to4008,400A,400B, in that order. Mask rotates once per channel; final four shifts
restore low mask. Global fade is processed AFTER channels, shared delay decrements
ONCE AFTER fade, only if nonzero. Delay therefore affects all selected secondary
channels in one frame, not separately decremented per channel.

Fade active iff fade&7F !=0. Advance only when phase equals that7-bit interval;
phase then &=1, level increments. Exactly reaching16 clamps15; positive mode
clears fade, negative mode retains it (reclamps on subsequent matching intervals).
Per-voice base level: low4 primary control, triangle full byte. Positive fade
caps at min(level,fade_level); negative decrements level until zero or fade count
spent. Initial zero is NOT guarded: DEC precedes BEQ, so it wrapsFF; fade5
producesFB, which survives the final upper-control OR (explicit contract). Triangle uses13*fade_level modulo256, derived from12 loop additions after
initial accumulator (not12* and not saturating). Envelope skipped for triangle
or zero interval. At interval equality counter0; packed high nibble is signed
step direction from interval bit7, added to low nibble; negative result clamps0,
then upper clamp computed level. Nonmatching interval retains/clamps packed low
level. Updated low nibble preserves step high nibble.

Masked primary still advances its signed pitch step and period accumulator but
returns before oscillator/register output, leaving secondary ownership intact.
Unmasked primary and secondary output use shared modulation. Level oscillator
interval equality clears counter, adds signed step to level; <=0 clamps1 and
reverses step, >=16 clamps15/reverses. Effective output limited by level. Nontriangle
control upper nibble comes from corresponding primary/secondary control; triangle
writes full level. Secondary uses signed step at record16/period17..18/control19;
primary uses step9/period10..11/control12. Sign extension/word carry exact.

Pitch oscillator: pitch_interval&7F zero disables; otherwise acts at exact counter
equality then clears counter. Magnitude is (pitch_shape>>5)&7 (four ROLs then mask;
carry input cannot affect retained bits). Direction pitch_phase bit7 chooses add
versus subtract to16-bit pitch_offset. Increment phase byte; compare low7 with
shape low5. At span equality retain phase bit7; if interval bit7 clear set it and
toggle phase direction; otherwise clear interval bit7 WITHOUT second toggle.
Zero span's equality occurs only after low7 wraps. Pitch writes sum base period+
offset full16-bit. Noise writes low4 plus corresponding control bit7, sets high0.
Low period always writes. High only writes when different from cache, stores
uncooked high then writes high|08 to length. This avoids unwanted retriggers;
channel-specific silencing and reset must preserve source order.

## Reset/dispatch/interpreters, CPU8514..896A physicalC524..C97B

Reset clears interval direction bit, optionally seeds modulation level from a
caller-selected control field if level_interval bit7 set (triangle full byte,
other low4); clears six bytes24..29, then high cacheFF. Byte30level not cleared
without seed. KeepPeriod wraps reset and restores cached high. Inline dispatch
pops JSR return and selects word using doubled X +1 with byte Y, reading actual
ROM targets, not coarse adjacent labels. Targets:
primary87CF,87D9,87E3,87F3,880D,8855,8872,8885,88BF,890F;
secondary85D1,85D9,85E3,85F3,860D,863E,8654.
These include instructions BEFORE misleading labels, e.g secondary opcode1 starts
JSR85D9 before label85DC; opcode6 starts LDAsecondarycursor at8654.

Secondary uses shared delay: nonzero/null heldperiod returns; nonzero/period
outputs its level and modulation. Zero decodes. High-bit bytes except lowF consume
next lowperiod and high=(command&7), reset state with control19 then SILENCE now;
held output begins later. LowF consumes a parameter but does NOT use that byte as
new period or level; KeepPeriod reset returns. Low7bit commands dispatch0..6:
0 read delay then continue;1 signed pitch step;2 uppercontrol merge preserve3F;
3 lowercontrol merge preserveC0 (triangle full replace);4 count0 unconditional
word jump, count!=current increments and jumps, equality resets count and skips
word with16-bit carry;5 read four modulation bytes;6 backs shared cursor by1,
clears high priority and CURRENT mask bit, silences if no primary period, returns
if also no primary cursor, else restores primary preset/reset. End command is
re-read independently by later selected channels, clearing each bit. Cursor reads
return byte in A/X and increment16 bits; no implicit end-of-page truncation.

Primary tempo0 means one invocation; tempoN nonzero meansN invocations (push count,
call full duration path, pop/decrement). Continuation bit in repeat_control adds
one extra duration tick on odd phase. Nonzero duration subtracts4 as word wrapping;
ONLY exact zero decodes. A value2 wrapsFFFE; no <=0 normalization. Nonzero remaining
with period0 returns; nonzero period computes level even when hidden by secondary.
Decode clears continuation bit then parses. Byte highnibble0 indexes ten command
handlers. Highnibble20 recursively parses next command then writes low3 prefix
bits into note_control high3 after inner return, preserving low5 preset. Highnibble
30 sets continuation and continues. Other note classes use (command>>5)&7 duration
class. Class table first8bytes at897B:0,0,2,4,8,16,32,64; alternate next8:
0,0,3,6,12,24,48,96. Duration scale repeated byte additions produces16-bit product;
scale0 loops256 times. Deferred note highbits nonzero decrement20, retain period,
and reset keeping cache only when unmasked. Otherwise note low5=0 rest/period0;
noise nonzero uses low5 directly as lowperiod; other voices read word from period
base+2*note with page carry. Envelope reinitializes its sign-selected endpoint if
enabled. Masked note updates period/envelope without output; unmasked resets
modulation using primary control12 then silences current voice immediately.

Primary handlers0 scale;1 signed pitch step;2/3 opposite control bit merges with
triangle lower-control full replacement;4 finite/infinite loop preserving repeat
continuation bit with full16-bit jump or skip;5 periodbase897B+((selector*2)&FF),
NOT dereference of another word table;6 alternate duration/note immediate byte;
7 envelope interval+packed step/level then optional initialization;8 preset index
OR retained high3 and immediate fullX preset load only if unmasked;9 clear cursor
and low priority, return if masked, otherwise silence. Primary cursor reads carry
at page boundary. Loop count byte increments and ORs current continuation bit;
no invented bounded/saturating loop count.

## Complete asset consumer closure, unknown bytes, remaining dependency

Static graph script checks/contracts/B18/static-consumers.py independently reads
matching local ROM and actual table words, follows EVERY56 header and all primary
channel starts (including aliases) and secondary stream starts, both finite-loop
branches/unconditional jumps, all commands and operand lengths.100 roots/9863
visited nodes; distinct5041 primary and633 secondary instruction nodes. No invalid
opcode/target. Every preset command and header preset is included; every selected
period window over-approximates all32 note indices. Audio consumed bounds physical
C997..E903 (exclusive), presetsCF03..E3C2, period windowsC997..CA41. Header/table
bytes additionally have direct loader consumers. These are navigation/consumer
metadata, no copied original payload or second playback interpreter oracle.

Unknown original ID source-unknown-00c522 retains physicalC522..C524, bytesA907,
CPU8512..8514. Previous850B tail JMP silence cannot fall through; next8514 starts
LDY14/LDAchannel field, overwriting A before use. All17 actual dispatch words,
all direct/numeric calls/branches and bounded data graph reviewed: no entry or
stream/period/preset consumer reaches those two bytes. Mask-selected indexed
reads are channel-state RAM, not ROM and not this range. This proves no CURRENT
source-backed native behavior consumer; it does not invent padding, an isolated
load entry, or semantic classification. Classification remains explicit low-impact
uncertainty; frozen source and ledger identities unchanged. Contract requires
bounded audio catalog rejection at8512/8513 with positive8AB0 consumer, not blanket
exclusion of adjacent data. B02 independently owns CHR sources physicalE910..F110
within mixedCAC0..F110. They remain valid graphics even if an audio consumer were
to overlap; consumer roles never mutually exclude. Graph maxE903 leaves13bytes
before graphics range; no purpose invented for that gap.

A separate actual dependency remains:16 reachable secondary lowF reset sites
leave AudioTemp from previous work, rather than selecting control19. When shared
modulation level_interval bit7 set, ResetEnvelopeState indexes current channel
using that previous byte; out-of-record values can reach subsequent channel or
adjacent runtime state, and that actual producer/domain must be derived, never
assumed record-bounded or emulated as a RAM map. Actual secondary modulation presets include80/83/90,
so this cannot be dismissed as dead seed logic. AudioReadSecondaryByte modifies
A/X/Y/cursor but NOT AudioTemp. Previous primary level computation, pitch write,
preset read or other channel can produce AudioTemp; caller/path provenance must
be preserved as typed seed source/value, never default19 or global CPU temp/RAM.
Bounded source-backed uncertainty B18-secondary-reset-seed records this exact
integration obligation. Explicit reset(seed value) contracts cover both low/full
triangle levels, cache preservation and nonseed branch; full frame must resolve
the actual producer before implementation completion. No claim of closed runtime
behavior or QA is made by compiling pending contracts.

## Synthesis boundary

Synthesizer owns independent MIT romatlas_port::nes::apu::Apu. Driver produces
ordered RegisterWrite schedules. Actual driver writes feed APU; contracts require
nonzero finite sound and whole versus partitioned exact samples. Apu render keeps
fractional sample phase; its clone retains phase, counters and DAC state. Native
sample requests advance only APU clocks, never queue/driver/game frames. No CPU,
bus or GPL emulator implementation and no DMC sample stream is inferred (driver
writes4015=0F, DMC disabled). Device/analog fidelity is untested; independent
helper has documented NTSC/filter/DMA limits. No toolkit edits are needed here.

### Stage4 source fixture correction and indexed reset implementation

The superseded contract file is preserved at
`checks/implementation/contract-corrections/B18-before.rs`.
`drained_primary_install_supplies_three_to_following_fade_command` installed a
fresh primary header, which clears the period-table pointer, then supplied a
synthetic first NOTE without the source's table-selection command. It consequently
attempted an absent address0002 read during the mandatory same-interrupt frame.
The fixture now explicitly issues command5 and provides its synthetic897B period
words. The original fade3/level1 assertions and test identity are unchanged.

The low-F secondary reset retains an explicit semantic predecessor producer and
selector: primary reads, inline targets, control parameters, envelope endpoints,
preset pointers, computed levels, oscillator spans and noise modes have distinct
provenance. Indexed reads project canonical typed channel fields directly,
including adjacent records and preset-base bytes; farther selectors use a borrowed
canonical display-byte resolver. No serialized channel-byte copy or CPU memory map
is introduced. Actual Game wiring must supply the display owner and initial NMI
producer provenance. Focused regressions cover adjacent-channel control, an actual
D9 inline-target selector reaching0449, and FE's retained high mask nibble being
discarded by the next frame's four logical shifts.
