# B03 source trace and independent executable contracts

Worker /root/behavior_contracts/b03_presentation, stage3 only. Full contract prompt,
workflow, common/coding standards, architecture, debug API, B01/B02 traces and
interfaces read. Pending declarations only; no game implementation or oracle.
Source game.asm SHA256 c29c4aa93881aeeaf643fc3172e093d8f3bb50ea50150b574df8b6cfe08f69cc;
reading.asm 201b3482124caef3bdabd4e83dab7403b6ceef172fe2df99f23c3a3071d046b5.
Physical offsets include header. Bank5 CPU8000 maps14010; fixed C000 maps1C010.
The independently generated planning/contracts/B03-consumers.json records every
assigned unit, and symbolic presentation readers/writers across the entire source.
These indices aid navigation; the following instructions derive expectations.

## CPU and flag evidence

Consulted toolkit docs/development.md platform table, nes-native.md; manufacturer
MOS-authorized Synertek6500 programming manual publication6500-50 at
https://syncopate.us/books/Synertek6502ProgrammingManual.html (sections2.2.1,
2.2.2,3.7,4.1.2,4.2.1,6.5,7.4..7.7,10.1,10.7,10.8). Eight-bit results wrap;
SBC subtracts one extra when C clear. INC does not affect carry but changes N/Z.
LDA changes N/Z but preserves carry. This matters for signed piece offset tests:
ADC produces carry, then loading offset changes N but retains carry for BCC.
BPL is a sign-bit test, not a mathematical nonnegative test on an unbounded integer.
Indexed indirect reads use base plus unsigned Y and can cross a page. The PPU has
14-bit address wrapping and control-bit2 stride1/32; source native helpers own
vertical mirroring and color/sprite pixel primitives, not source scripts/clocks.

## Composition flags, gates, all slots and projection

Bank5 CPU8000..8132, physical14010..14142 (game.asm2314..2507).
ComposeObjectFlags clears player render bits0/1; OR of protection and special mode
sets bit1 if either nonzero. Other bits survive. This happens before overlay and
all-slot traversal, including when player is inactive. Overlay requires negative
7F latch AND zero dialogue cursor/ending/status stream AND player state !=0F AND
player Y>=44. Cursor starts4, preserving sprite zero. Three byte streams in order
OverlaySprites, TitleOverlaySprites, TimerSpriteOverlay copy bytes until firstFF;
FF is a byte sentinel, not merely a sprite-Y sentinel. Each stream has its own
8-bit X counter; Y shared cursor wraps. The original authored buffers are bounded;
extension format validation must reject invalid lengths rather than fabricate
an unbounded host copy. Lists are populated by B12/B14; B03 owns composition.

CompositionTick increments BEFORE parity test. Odd traverses15..0; even0..15,
including FF->00. Inactive state (bit7 clear) clears render and stale spawn ID;
active renderFF skips without projection or animation, preserving actor fields.
State bit20 selects world projection. Low X SEC/SBC and high SBC implement full
16-bit world-camera subtraction. High result0 means visible, screen X is low
result. World state bit10 then returns before animation, but screen-positioned
actors never test state bit10. Screen-positioned uses low world X unchanged.
Offpage objects with animation5A always retain state, clear screen X and visible.
Others take absolute value of LOW difference, including80->80; magnitude<40
retains but hides, >=40 clears state/render/spawn. This deliberately also retains
world100/camera0 (low difference0) and never substitutes 16-bit absolute distance.
Retained offpage objects do not advance animations. Visible bit7 is set before
flash gate. Flash bit1 allows composition/animation only when clock bit1 set;
clock sequence0,1 blocked;2,3 admitted. Flash therefore affects logic frame clocks,
not only pixels. Hardware flicker relaxation cannot remove this protection rule.

## Counted animations, frames and piece emission

Bank5 CPU8132..823B, physical14142..1424B. Animation144 pointer selectors at
B1C7/B257 include all generic uniform fragments17278..172F5 as pointer bytes.
Sequence record [last_frame,last_tick,frame IDs...] is count-minus-one. Player
state0F or negative tick freezes counters while selecting/drawing current frame.
Otherwise compare OLD tick&7F with last_tick, after incrementing stored tick.
Mismatch draws unchanged frame (including7F->80 becoming frozen); equality resets
tick0 and compares OLD frame&7F with last_frame after stored frame increment.
Mismatch draws incremented frame. Equality: render bit8 deactivates; otherwise
post-increment frame nonnegative resets0, negative restarts restart|80. FF->00
therefore resets rather than restarting. Duration is last_tick+1 compositions,
not source comment's ambiguous 'ticks per frame'. Last index0 is one frame.

Frame index masks7F then adds2 into animation bytes. Frame table has248 entries:
**selectors0..3**, not244..247, alias mutable RAM04C0. Other244 records exactly
cover1424B..15691. Decoded content retains runtime-authored frames from B14/B16,
not a read of instruction/RAM addresses. Coordinate224 split pointers ADE1/AEC1
cover16DF1..16FB1; signed pair streams15881..16DF1 include aliases. First header
byte is last piece index; second base coordinate selector. Render bit40 adds1 to
coordinate selector (8-bit ADC), selects authored flip geometry, and XORs40 into
piece attributes. It does NOT simply negate X offsets. Render bit4 shifts to
attribute20 priority OR; palette bits remain from each piece. Opaque original
record bytes stay in local ROM, not source/distributable assets.

For each piece, signed DY added to originY with wrapping ADC: positive DY requires
no carry; negative DY requires carry. Zero is positive. A wrapped result is hidden.
Nonplayer slot rejects Y<30 unless render bit10 set. Player bypasses this UI gate.
Attributes are (piece XOR horizontalFlip) OR priority. DX uses same signed/carry
rule. Rejection writes hidden Y=F8 into CURRENT OAM entry, leaves cursor unchanged,
and still advances paired stream indexing. The next accepted piece overwrites
that same entry. Accepted increments cursor4, stopping immediately at wrap0.
Entry cursor0 skips all emission. Count is DEC/BPL, so original last index0 emits
one; legal original records remain below sign boundary. Render is read-only;
advancement occurs only at source composition boundary. Gameplay and dialogue
call HideOamSprites before composition (ED40/ComposeAndStreamGameplay); scene
initializer B380 composes immediately. Inactive/hide cull changes must be visible
to later spawn-window/actor lifetime consumers this frame (B10/B13).

## Data consumers and finite domains

Contracts read the matching local ROM only. Independent raw pointer/header fixture
reads check every144 animation, every244 ROM frame's tile/attribute pairs and both
coordinate variants, all248 selectors including four runtime aliases,224 coordinate
pointer domain and all12 scripts. These are data-format comparisons, not a second
animation/composition algorithm. Synthetic fixtures establish signed arithmetic,
flip variant selection and temporal rules independently of original assets.
Script5 points B9F0 into StatusSpriteRecordFragment_17941 area; do not restrict
scripts to17B0A..17FEE. The generic suffix172F7..1730F isFF,17310..1738F is0,
17FEE..1800F isFF. No direct/symbolic consumer of these suffix bytes was found;
keep original raw spans preserved; no padding meaning or execution is invented.
All meaningful classified pointer fragments ARE consumed, including uniform bytes.
No new cohesive behavior area discovered; existing B16 owns scene handler dispatch.

## Both queues, scripts, fills and controls

Modernization iteration006 centralizes the complete LoadStagePalette copy in
`Display::load_palette`: sixteen background plus sixteen sprite colors replace
both shadows, without requesting an upload or consuming queued writes. Title,
stage and special-scene callers retain their ordering and later palette edits.
The source-backed progression regression checks every byte value with pending
both clear/set and verifies that writes/control/suspension remain unchanged.

Modernization iteration004 shares logical write construction through private
`append_write`/`append_sequence` operations and `Stride::increment`. Source queue
byte cursors, counts and pending gates remain at flush; script terminators and
the value tail remain at script; fills still overwrite attributes last. Boundary
contracts cover both strides, 14/16-bit wrap and page-FF attribute ordering.

Fixed C129..C183 physical1C139..1C193. Each flush clears its own pending BEFORE
reading. Negative destination high80..FF terminates without low/count read.
Otherwise address high/low; count byte controls one unconditional write followed
by DEY/BPL. Normal0..7F gives1..128;80 gives129 because decrement80->7F;
81..FF gives1 because postdecrement stays negative. Cursor adds payload count+3,
8-bit wraps; zero exits. Source pending stores are flags only, not queue content.
Primary terminator aliases secondary payload in B02; do not manufacture another
request. Primary/secondary share representation but different owner buffers.
B02 prepares uploads; active display queues defer to interrupt; suspended loading
can flush synchronously. Preserve PPU control stride bit2, even AND FE leaves it.
Logical writes mask PPU address14-bit and use toolkit mirror semantics when drawn.

Screen writer bank5 BAC0..BAFA (17AC0..17B0A): destination high0 ends script,
FE ends only payload; payload00 is ordinary tile. Pointer Y carries by incrementing
base high at wrap. The BNE following helper tests INY/INC flags, not payload value:
there is no zero-data early exit. Pointer-high wrap to0 would exit, unreachable for
valid originals. Selector0..11; selector1 is empty. Selector11 at end appends eight
bytes from status0334..033B (0329+X=0B..12) to CURRENT destination; no address reset.
B16 chooses script IDs through ending/title variants and supplies status tail;
B14 owns digit generation. Catalog script length excludes dynamic eight-byte tail.
Totals per selector:89,0,315,147,158,150,87,100,16,32,31,31; first destinations
2021,none,20B2,2504,2504,2504,2204,2244,26CE,26CE,26CE,2486.

FillPpuArea C209..C24F forces hardware control shadow AND FE (retains stride).
Destination page>=20 always writes four256-byte pages, THEN64 attributes at
(page+3):C0: CMP leaves C=1 and ADC2 includes it. Thus20 ->23C0,24->27C0,
not22C0. Lower destination uses Y pages; Y0 means256 pages by DEX wrap, no attrs.
FillPagesBackward C24F..C267 (1C25F..1C277), no symbolic/numeric callers found:
X - highPage -1 with carry clear forms loop count; descending entire pages through
zero-page indirect. This unused generic memory helper is not a native RAM map or
public presentation gameplay operation. Reset uses its own inline RAM clearing.

DisableRendering increments suspension8-bit and writes mask0. EnableRendering
decrements8-bit and writes1E regardless of remaining depth (not a nesting-aware
reference-count enable). NMI helpers preserve only control11 then OR80. These
helpers are standalone state transitions; frontends do not bypass their effects.
HideOamSprites preserves sprite0 iff gameplay mode nonzero, else hides all64;
changes only Y bytes, not tile/attributes/X.

## Palettes and precise fade timing

Fixed C277..C2C8 physical1C287..1C2D8. FadeIn hides sprites (gameplay rule applies),
starts subtract30/stepF0. FadeOut starts10/step10 without hiding. Each immediate
step copies32 base colors then subtracts offset with SEC; negative8-bit result
clamps0F, preserving base. Palette pending becomesFF. Four YieldOneFrame calls
follow EACH step, including final. In offsets30,20,10,00 ->F0 stops; out offsets
10,20,30,40 ->50 stops. Total16 yielded boundaries, not12. A resumed step publishes
before its next yield; no audio/render call decrements fade waits. Tests use
explicit continuation state; B12/B13/B16/B17 own enclosing scene transitions.

LoadScenePalettes F76D..F7BE physical1F77D..1F7CE, not just tiny split ledger
fragments: background endpoint raw, sprite endpoint masked7F. Copy16 backwards
to both base/active background and sprite halves. Then synchronize palette16 to
palette0 in BOTH shadows. Raw sprite endpoint negative requests option override:
option*4 chooses four colors written active/base indices28..31. Options0..2 are
B12 finite menu domain. Routine does NOT set palette pending; its caller must.
B16 passes scene palette selectors from E11B parameters; scene transition B4E4
uses selector3 then explicitly pendingFF. NMI palette upload writes exactly32
active bytes to3F00..3F1F, clears palette pending first, then PPUADDR3F00 and0000
latch reset; do not mutate base or clear unrelated pending flags.

## B01 reconciliation: ownership and display latching

B01 owns ordered effects, interrupt tick/wake/audio and foreground continuations;
B03 applies content. NMI C000..C129: prime sprite0 even when suspended (Y from
SplitSpriteY,tile21,attr20,X08). Unsuspended: control&78, mask0, OAM DMA unless
guard, primary then secondary then palette, UI scroll X/Y, mask shadow. UI control
is shadow|18 only gameplay && playerY>=44 && stage!=9; else shadow. Wake tasks
always. Unsuspended gameplay split waits sprite-zero edge; alternate transition
first writes saved address; race intro mask10; then camera low/playfieldY scroll
and control90|(cameraHigh&1). Shadow control stays independent from temporary
hardware control. Rendering snapshot applies B01 effect list, never repeats gates.
B01 contracts already enumerate suspended/guard/ending/playerY/IRQ/audio gates;
B03 executable contracts establish actual content and all effect variants.

Boundary assumption B01-frame-boundary-integration remains explicit: native tick
commits previous foreground's prepared display at interrupt before resuming
foreground, as B01 specifies. Source OAM guard skips DMA while queues/palette may
still commit; suspension suppresses all display commits but not sprite priming,
wake or audio. B03 must retain distinct prepared/committed snapshots in stage4.
Read-only render/observe calls never publish queues or advance animations/fades.
Frame0/reset/title after-first-yield obligations remain B12 integration work.

B17/scene ScrollEndingSplit E212..E268 is source foreground continuation, NOT
ordinary NMI split: increment spriteY; E8 stops, scene5 B0 stops; yield then
sprite-hit edge installs shadow|1; only low3bits7 enters periodic64-yield delay.
Scene0 Y>=B7 requests audio7 iff different; B16/B17 own these timers/transitions
and call B03 raster content interface. Preserve deliberate waits and split mask;
never replace polling with a headless hang. World quake/camera displacements are
B04's typed inputs; B03 owns resulting raster output, not world simulation.

## Limits and handoff

Synthetic malformed-input policy beyond source finite domains remains B02's
extension-validation assumption; source provides no general bounds checks.
B01 boundary integration remains pending full B12/B13 wiring, not disproven.
No emulator/GUI/gameplay QA/audit, toolkit change, check-in or publication occurred.
Every executable operation is explicitly unimplemented; passing compilation only
proves contracts can run. Exact command outcomes and all names are in
planning/contracts/B03.md and checks/contracts/B03-results.json.

## B14 source correction of runtime-author assumption

The earlier statement that B14/B16 author runtime frames is superseded by B14
independent operand/writer tracing: no direct04C0 or status digit writer found.
Reset clears memory; generic indirect helpers are not evidence of a status
consumer. Keep selectors0..3 alias contracts. B16 must resolve natural frame
reachability; do not invent digit graphics. See ../status/trace.md and
B14-runtime-frame-author uncertainty. Original claim retained above as history.

## B15 canonical primary queue reconciliation

PreparedPrimary(encoded,pending:u8) is now the one canonical B03 primary upload
state in Display.primary. Dialogue Panel reexports this exact type and borrows it;
no persistent row copy. rows() is pending read-only derived observation. Existing
B03 bool pending assertions now inspect exact pending byte for the same truth
condition. New commit_primary must retain prepared bytes whenpending0 and commit
nonzero5D exactlyonce, clearingpending0; sourceC13x clears before upload, B01gate
is nonzero. Two new executable canonical_primary_* cases complement prior53
contracts (now55). This supersedes boolean-only pending ownership, preserving
prior logs/history and their actual outcomes. See dialogue/trace.md and
planning/contracts/B15.md for full trace/results. All behavior remains pending.

## B16 canonical runtime-alias writer reconciliation

B16 independently found04C0 = SecondaryPpuQueue0440 +80hex. B04 changed-terrain
absolute-indexed writes can author this tail starting tenth14-byte record at
cursor7E (not sixth). PreparedSecondary provides one canonical encoded buffer,
source-zero reset and immutable pointer-identical alias projection; allfour
frame selectors remain supported. The four integration contracts live in
scenes/contracts.rs, not a second B03 test count. All144 animation records use
these aliases only in0A/8C/8D; natural co-occurrence/caller reachability still
pending, no status-frame invention or source behavior exclusion. Details and
full writer index: ../scenes/trace.md, planning/contracts/B16-consumers.json and
B14-runtime-frame-author. Existing B03 source contracts remain intact.
