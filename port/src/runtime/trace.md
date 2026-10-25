# B01 source trace and executable contract derivation

Worker `/root/behavior_contracts/b01_runtime`, stage 3 only. Read the entire
`prompts/contract.md`, common, standards, workflow, architecture and B01 inventory;
independently reread maintained instructions and symbols. No behavior implementation,
CPU/RAM simulation, source changes, emulator experiment, QA or accounting audit.
Tests are in `contracts.rs`; module declarations are intentionally unimplemented.

Frozen evidence: `reference/source/game.asm` SHA256
`c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc`;
`reference/reading.asm` SHA256
`201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5`.
Physical offsets include the 16-byte iNES header. Fixed-bank CPU C000..FFFF
maps physical 1C010..20010; switch-bank CPU8000..BFFF needs bank context.

## CPU basis, not an emulated implementation

Consulted `/root/romatlas/docs/development.md` platform-reference table and
`docs/nes.md`, `docs/nes-native.md`. Independent native helpers do not execute
instructions and do not own scheduler/game rules. Actual MOS/Synertek manufacturer
programming manual (1976 authorized reprint), sections4.2.1,4.2.2.1,10.3,10.4,
appendix B, consulted at
https://syncopate.us/books/Synertek6502ProgrammingManual.html . ROR carries old
bit0 to the next rotate; INX/DEY do not overwrite carry. CMP sets carry on unsigned
>=; BIT takes overflow from operand bit6; DEC/DEX wrap at byte width. CLD is
explicit at reset; no decimal interpretation enters this area's arithmetic.
NESdev wiki fetches were denied403; no claim that those pages were inspected.

## Reset and reset/title boundary

`ResetInitialize` CPU FEE0..FF3E-ish, physical1FEF0..1FF4E within the broader
1FEF0..1FF9D region; lines22959..23006. SEI/CLD; PPUCTRL=PPUMASK=0; stack=FF.
Y=4 polls vblank high then low four times. These hardware warm-up polls are not
four interactive game frames. Zero page clear starts atY0 then wraps FF..01,
clearing every byte. **Correction to source comment:** indirect RAM clear writes
pages1 THROUGH7, not1..6: increment page; write256 bytes; compare written page
with7; stop only when equal. Source temporary pointer page byte ends7; it is not
native persistent gameplay state. No battery save restore occurs.

HideOamSprites C267 is called with GameplayDisplayMode now0, so all64 Y entries
become F8 (not the gameplay variant preserving sprite0). FillPpuArea C209/C225,
physical1C219..1C24F, called with A20 then24, X24, Y0. Both write four256-byte
pages (one nametable) plus64 zero attributes at base+3C0. The attribute address
uses CMP carry followed by ADC2: 20+2+1=23, 24+2+1=27. B03 owns the actual
presentation buffers and should preserve both requests and their order.

Reset queues entry D4E6 (`InitializeTitle`) in slot0 with state8; other slots,
input, interrupt byte and four random words are clear. Sets control shadow90
AFTER installation. Scheduler immediately dispatches fresh entry without input
sampling. InitializeTitle sets stackBF, writes shadow90 to PPUCTRL then first
YieldOneFrame. First NMI therefore commits/wakes/audio before its resumed entry
samples both controllers and then writes RandomState=88. **Reset seed is zero;
88 is title's after-first-yield seed**, other words remain zero. B12 owns the
seed write/title continuation and fades. Title frames do NOT call global
AdvanceRandomState; occasional title animation mixes words with its own carry.
Native reset reports ordered presentation requests; no PPU warm-up busy-wait or
CPU stack is retained.

Current bank logical field clears0. Physical mapper may retain its previous bank
on original soft reset until NMI selects audio/restores0. Nothing in fixed-bank
reset/title before that boundary consumes old switch data. The native context
uses logical0 at reset; source bus transient is not emulated. B02/B12 must keep
banked data accesses contextual.

## Four-slot scheduler and continuations

CPU FF3F..FFC6 across physical1FF4F..1FFD7, lines23009..23154.
Slots are source four-byte records but native enum continuations replace stack
pointer/return address. State0 empty,1 sleeping,2 running,4 resumable,8 new entry.
Only these are written. Poll treats >=4 runnable; exact8 chooses new entry,
other reachable runnable4 chooses saved-stack resume. Each scan starts slot0,
then1,2,3. `SchedulerTaskIndex = (Y-1) XOR3` yields0..3 for that order. Dispatch
marks2 before execution, so it cannot run again unless yielded/reinstalled.
Interrupt gate resets0 on scan and NMI setsFF; if interrupted after candidate
selection, scan restarts. Native atomic boundary processing preserves this priority
without reproducing partial CPU instruction interleavings or busy-poll loops.

SchedulerInstallTask shifts caller A twice to select slot, writes entry and
state8, replacing prior state/delay/continuation. Only reset calls it in the
maintained source; test all four slots as interface/boundary fixtures, not a claim
that the game starts four tasks. SchedulerFinishTaskAfterWait's name/comment are
misleading: FF8D only calls the offset helper and clears state, NO wait.
SchedulerFinishAndReschedule gets CURRENT index, clears it and jumps polling;
the former gets caller A index and returns. Native finish(slot) is shared state
operation; callers decide whether to dispatch immediately. No symbolic callers
exist for either finish entry in the maintained source.

YieldOneFrame writes delay1, preserves caller X/Y via stack, marks current slot1,
saves continuation and jumps poll. On NMI, ONLY state1 decrements delay; zero
result writes4. Starting delay0 wraps255 and wakes after256 interrupts; starting
255 wakes after255. Ready/running/entry/empty records do not decrement. Source
Yield entry always sets1; zero/255 direct sleeping fixtures cover NMI byte
boundary without claiming a normal writer of those delays.

Resume restores its stack. ONLY slot0 calls ReadControllers, and only for state4
resumption; new state8 jump skips it. Inputs remain unchanged while sleeping or
while any other slot runs. Task-owned native continuation payload replaces saved
registers; returning continuation identity and order is the observable contract,
not an artificial register object.

WaitSchedulerTicks FFA5 repeats YieldOneFrame then DEX/BNE. Count1 means one,
FF means255, 00 means256. Every iteration resumes task0 and samples controllers;
it is not a single256-frame input freeze. `FrameWait` holds native remaining
iterations and `resumed` is called after scheduler dispatch, never on render or
audio sample generation. `wait_count_one_255_and_zero...` verifies exact boundary,
continuation dispatch and per-frame alternating edges together.

All symbolic wait callers independently checked: completion palette halves4/4
(lines2284/2289 B16), selection60 (13614/13629 B12), value transfer2 (14084 B14),
ending120 and60 (15853/16346 B17), dialogue glyph3 (19346 B15), timer conversion
256 (19519 B14, X00) and conversion final60 (19598 B14). Fades use their own
four repeated Yield calls, not WaitSchedulerTicks, and retain scratch values
across nested waits (B03). Large deliberate finale delays B4/F0 are local
continuations owned B17, not automatically this helper.

## Controller acquisition, both lanes and independent conflict filtering

ReadControllers C1A7..C208, physical1C1B7..1C218, lines10692..10754. Controller
strobe1 then0. Eight serial iterations LSR/ROL lane0 and another LSR/ROL lane1
for each port. Eight rotations discard previous temporary contents. OR the two
complete lane bytes; no prior carry leaks into acquisition. Standard button
order A,B,Select,Start,Up,Down,Left,Right gives masks80,40,20,10,08,04,02,01.
Native PortSample represents the logical eight-bit result of each lane; ordinary
frontend supplies standard only. The source's second lane is preserved without
implementing electrical reads.

For port2 then1: new=T; pressed=(T XOR previousHeld) AND T = T AND NOT previousHeld;
held=T. Then X3 down to0 filters four contiguous bytes, in source order
held2,held1,pressed2,pressed1. If bits0C BOTH or03 BOTH are set, maskF0 removes
ALL directions, including a nonconflicting perpendicular direction. Action
buttons survive. Each output filters INDEPENDENTLY AFTER computing edges.
E.g. previousUp, nextUp+Down+A => heldA, pressedDown+A (the new edge has no
opposite pair). ReleasingDown => Up edge again because previous filtered held
wasA. The sixteen-direction truth table is manually enumerated in tests.

Game writes matter: B06 forced downward room transition overwrites held=04
(line17393); forced jump merges B into A (17611..17617). B15 saves pressed across
presentation yield and restores it at EDAB (18880), so following outcome reads
acknowledgment, not latest sample. Runtime ports are explicit state accessible
through owner integration; no hidden raw-history shadow may supersede the last
source-observable held byte. B06 owns action/contact consequences; B15 owns the
save/restore operation. Other readers: B07 rideable UP; B04 vertical transition
and camera horizontal input; B12 title/land select/A acknowledgement; B13 START
pause; B06 standing/airborne/climbing/pogo/held player; B15 message acceptance.
Controller2Held UP08 at E705 gates downward-room transition (B06); it must not
be discarded merely because the game normally uses one player. No other separate
port2 action consumer was found.

## Random algorithm and cross-area writers

AdvanceRandomState C2E6..C307 within physical1C2F6..1C368, lines10946..10963.
Feedback is XOR of bit1 of INITIAL word0 and word1, then CLC/BEQ/SEC maps zero to
carry0 and nonzero to1. Four RORs in increasing word order propagate old bit0 to
next word bit7. INX and DEY preserve carry. **Correction to comment:** feedback
is computed ONCE; it is not recalculated for adjacent words on each iteration.
Zero remains zero. Seed88 produces44,22,91,48/80,24/40,12/20,89/10,44/88 in the
first two words over eight calls; tests include carry propagation to words2/3,
both feedback branches and XOR cancellation. No uniform distribution claim.

Only symbolic call of this routine is FinishGameplayFrame DC23 (line14449),
after secondary upload handoff and immediately before yield. Normal, paused and
ending gameplay composition routes reaching it get one call, not one per NMI.
Nested fades/dialogue waits may run many interrupts without it. RNG is neither
advanced on input sampling nor NMI/audio/render. B13 owns path-specific call
counts in the composed tick; runtime tests forbid interrupt mutation.

Additional source accumulator operations are part of their callers, NOT replaced
with a convenient generic random-next function. Full writer inventory:
- B07 actor pose84A5: word2=word0+word1+incoming carry; animation frame CMP0C
  equality leaves carry1 on the shown fallthrough. Choice bit0.
- B07 secondary spawn89DA: word1=word0+word1+incoming carry; mask03. Incoming
  flags depend on callers; worker must trace them rather than assume CLC.
- B07 paired actor9619 reads word0+word2+carry, mask01, no random-word write.
- B11 attack heightA39B: word1=word0+word2+carry; later destinationA3B6
  word2=word1+word3+carry (CMP position's less-than branch leaves carry0).
- B11 impactA65C: word0=word0+word3+carry, mask07; collision/audio helpers may
  affect incoming carry and must be traced by that worker.
- B11 perimeterA775 and waveA9E8: word1=word0+word1+carry, mask01.
- B11 centerAB31: word3=word0+word1+carry; direction INC preserves resulting
  carry into word1=word2+word3+carry. This two-add chain must stay coupled.
- B12 titleD5F3 writes word0=word0+word1+carry; CompositionTick CMPB4 equality
  supplies carry1; LDA/STA do not erase it.
- B16 departureF9E0 writes word1=word0+word1+carry; mask07 chooses delay.
- B17 teleportFA8C writes word0=word0+word1+carry; mask0F chooses X.
All arithmetic wraps8 bits; incoming/outgoing carry is caller semantics, not
persistent CPU state in Runtime. Cross-area workers retain obligation for their
own caller/callee effects and source-derived choice assertions.

## Interrupt and bank boundaries

NmiService C000..C13D, physical1C010..1C14E, lines10400..10596; nominal inventory
region stopsC0E1, but split/audio tail IS included here via shared callees. CPU
registers/status saved at entry and restored on exit; temporary engine flags do
not leak into interrupted foreground carry. InterruptVectors physical2000A..20010
store NMI C000, resetFEE0, IRQ C000 little endian. IRQ therefore uses identical
service when delivered; reset SEI means this is NOT a second natural tick source.

Exact request order:
1. If GameplayDisplayMode nonzero, set sprite0 Y=SplitSpriteY,tile21,attr20,X08.
   This precedes the display-suspend branch, so happens EVEN while suspended.
2. If DisplaySuspendDepth nonzero, skip ALL ordinary OAM/queue/palette/UI work
   and split wait/scroll, but continue at step3. For unsuspended: control shadow
   AND78 to hardware, reset PPU latch, mask0; if OamDmaGuard=0 set OAMADDR0 and
   DMA page02; primary then secondary pending queues; palette if pending.
   Upload subroutines clear their request flags BEFORE consuming buffers. Palette
   clears pending before copying32 colors, including address/latch cleanup.
   Then install UI scrollX/Y, mask shadow, and control. Gameplay plus playerY>=44
   and StageIndex!=09 sets shadow OR18; else shadow. Y43/44 boundary and ending09
   bypass are covered. These are presentation requests; B03 validates actual
   queue payload/latches/contents, so no duplicate presentation state is created.
3. Increment InterruptTick wrapping8; set scheduler interrupt gateFF; wake four
   sleeping slots in ascending order. Suspended rendering does not freeze tasks.
4. If unsuspended gameplay: wait for sprite0-hit clear then set using BIT/V.
   Native uses deterministic raster split, never a host hang. Optional alternate
   SplitTransitionControl installs saved PPU address (read status latch, high,
   low). Optional RaceIntroLatch selects sprite-only mask10; then install
   camera/playfield scroll, control90 OR(CameraXHi AND1). Non-gameplay skips
   ALL these split effects. B03 owns pixels and split-address data.
5. Temporarily bank3 WITHOUT changing CurrentPrgBank16K; drain command queue:
   read at AudioQueueRead; sentinel88 stops; clear consumed slot to88, increment
   cursor AND07 BEFORE dispatch8003, repeat. Once empty, call8000 exactly once.
   B18 owns queue data/driver and must verify enqueue-during-drain behavior;
   B01's ordered requests guarantee driver call is after drain regardless of
   display/foreground pause. Restore saved foreground bank through normal bank
   selection, restore interrupted registers/status, return. Host sample calls do
   not create interpreter/game frames.

SelectPrgBank16K FFDE..FFE4 physical1FFEE..1FFF5 stores logical bank THEN alternate
FFE0 copies A toX and writes bank through MapperBusConflictValues[X] atFFE5.
Seven safe bytes0..6 (physical1FFF5..1FFFC) correspond valid switch catalog values;
fixed bank7 is not requested. NMI uses alternate entry specifically to preserve
foreground field. Native BankContext exposes select, temporary_audio and restore;
all seven identities including already-audio case are tested. Native assets use
resolved catalogs; no bus-conflict mapper or source instruction execution exists.

## Frame boundary policy and residual integration

The native atomic tick policy is interrupt/display+wake+audio for previously
queued effects followed by runnable foreground continuations in ascending order,
with slot0 input acquisition on resume. Fresh title entry precedes the first
resumption. Presentation shows committed buffers; fresh foreground requests
wait for the next interrupt unless source explicitly performs a synchronous
build while display suspended. This removes CPU timing/lag, as authorized, while
retaining intentional waits. Composition root/B03/B12/B13 must integrate this
policy and expose semantic pending/committed observations before QA. Until their
contracts exist, it is a source-informed integration assumption tracked in
uncertainties.json, not evidence that the currently infrastructure-only Game
already has the right frames. No debug mutation endpoint is added.

## Additional B01-ledger dispatch metadata and source data

`region-1294`, DispatchInlineWordTable C2C8..C2E5 physical1C2D8..1C2F6,
lines10920..10939: saves callerX/Y, doubles selectorA (8-bit), incrementsY to1+
2*A modulo256, pops JSR return address as pointer, reads low at offsetY and high
at next wrappingY, restoresX/Y and tail-jumps through resolved pointer. ASL's
carry (old selector bit7) survives to target; loads/INY do not change it. Pointer
indirect-index addition itself carries across pages, independently of CPU carry.
Selector127 would read low offset255/high offset0, so generic host array indexing
is not automatically equivalent for unbounded selectors. Both symbolic helper
name AND numeric C2C8 searches found **zero callers** in maintained source;
no live consumer of that wrapping edge is established. Do not build a CPU stack
or a second assembly dispatcher for this static helper. If a reachable consumer
is discovered later, that area's typed dispatch tests must prove selector bounds,
target/effects and incoming carry. No source bytes/accounting were removed or
marked implemented/excluded. Header, vector words and safe bank bytes are data
metadata consumed by reset/asset identification; startup matching-ROM validation
remains covered by the two green infrastructure tests, with B02 responsible for
its loader/data-validation contracts. Shared broad regions also contain
FlushPrimaryPpuQueue and FillPpuArea; their reset/order effects are covered here,
full payload behavior is B03 rather than duplicate runtime buffers.

Stage4 wait-return integration: FilterOppositeDirections ends at X0 (primary
pressed edges), with CMP0C on its vertical pair, else CMP03 on horizontal pair.
Thus the resumed foreground wait's final comparison is true iff raw newly pressed
primary merged edges contain either opposite pair. Filtered held/pressed values
cannot recover it. SchedulerResumeStack restores via PLA/TAY/PLA/TAX; WaitTicks
DEX/BNE preserves it. Controllers.SampleReturn captures that named sampling
outcome before filtering; cold reset is None until an actual sample. Regression
also distinguishes a held conflict from a newly pressed conflict. This is a
source-specific input return fact, not retained generic carry/CPU scratch state.
