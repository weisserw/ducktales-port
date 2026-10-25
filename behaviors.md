# Source behavior inventory

All18 source behavior areas now have native implementations and shared Game
integration. Strict desktop/Wasm gates and1411 tests pass. Final check/accounting/audit results are recorded in
progress.md and checks/implementation. Independent whole-game QA remains stage5.

The source inventory and stage3 worker evidence below remain historical evidence;
old per-area pending counts describe that stage, not current implementation.
Source-proven contract/fixture corrections retain prior versions and trace notes.
Current native implementations are linked in the table. All1302 original regions
and1537 units are accounted in ledger.json with unchanged identities/extents;
checks/implementation/ledger-accounting.json indexes area/native consolidation.
Frozen source and contracts, not planning labels alone, govern the implementation.

## Historical stage3 coverage and worker contract

Exactly 18 cohesive areas below each require one separate sequential stage-3 worker under the **entire** prompts/contract.md. Each worker rereads actual source, traces branches/carries/callers/data consumers, creates substantive compiling/discovered contracts and adjacent evidence, and records failing assertions honestly. Module paths below are design proposals, not implemented modules. Test locations are `<module>/contracts.rs` (or `<module>_contracts.rs` for a single-file owner); evidence is `planning/contracts/<ID>.md`. Workers may add declarations/interfaces required by contracts, with no gameplay implementation.

[planning/ledger-area-map.json](planning/ledger-area-map.json) assigns all **1302 original regions** and every original unit without changing IDs or pending dispositions. Zero regions are excluded. Primary physical ownership is a routing aid; `related_areas`, actual consumers and cross-area effects govern shared records. [planning/dispatch-survey.md](planning/dispatch-survey.md) enumerates finite actor/player/scene/terrain/vertical/audio target domains and stage pointer tables; [planning/source-consumers.json](planning/source-consumers.json) indexes every symbolic source operand. The source-label inventory remains [identifier-inventory.md](identifier-inventory.md). These cross-checks establish coverage assignments, not semantic truth or implementation acceptance.

Uncertainties: `source-unknown-00c522` belongs B18 with B02 consultation. Unnamed graphics/packed payloads, generic uniform ranges, title option labels, individual stage selector roles and actor/audio names remain unresolved source questions assigned below. Uniform data inside the 200-room logical interval is still consumed as room bytes; do not discard it as padding. Unused tails must gain source-backed consumer/bounds exclusions during tracing if appropriate. Battery saving is excluded as a behavior because matching header flag6 battery bit is clear, not because no save was seen. Hardware lag/flicker may be omitted by shared contract; deliberate flashing, freezes, splits and slow scene waits are required.

| Area | Owner/module proposal | Primary ledger regions | Contract worker | Implementation |
|---|---|---:|---|---|
| B01 Reset, deterministic scheduling and controls | `port/src/runtime` | 21 | complete: 21 discovered contracts; explicit pending failures | [implemented](port/src/runtime/mod.rs); shared [Game](port/src/game.rs) |
| B02 ROM asset access, CHR/background preparation and extensions | `port/src/assets` | 39 | complete: 34 discovered contracts; explicit pending failures | [implemented](port/src/assets.rs); shared [Game](port/src/game.rs) |
| B03 Sprite composition, animation, display queues and effects | `port/src/presentation` | 58 | complete: 55 discovered contracts; explicit pending failures | [implemented](port/src/presentation/mod.rs); shared [Game](port/src/game.rs) |
| B04 Room layouts, terrain decoding, scrolling and mutable world | `port/src/world` | 86 | complete: 60 discovered contracts; explicit pending failures | [implemented](port/src/world/mod.rs); shared [Game](port/src/game.rs) |
| B05 Shared collision, integration and terrain reactions | `port/src/collision` | 75 | complete: 70 discovered contracts; explicit pending failures | [implemented](port/src/collision/mod.rs); shared [Game](port/src/game.rs) |
| B06 Player action state machine and life/death motion | `port/src/player` | 91 | complete: 128 discovered contracts; explicit pending failures | [implemented](port/src/player/mod.rs); shared [Game](port/src/game.rs) |
| B07 Ordinary enemies, projectiles and reward stealing | `port/src/actors/enemies` | 80 | complete: 97 discovered contracts; explicit pending failures | [implemented](port/src/actors/enemies/mod.rs); shared [Game](port/src/game.rs) |
| B08 Interactive objects, recoveries, items and treasure collection | `port/src/actors/objects` | 78 | complete: 99 discovered contracts; explicit pending failures | [implemented](port/src/actors/objects/mod.rs); shared [Game](port/src/game.rs) |
| B09 Platforms, carrying and externally imposed movement | `port/src/actors/platforms` | 7 | complete: 81 discovered contracts; explicit pending failures | [implemented](port/src/actors/platforms/mod.rs); shared [Game](port/src/game.rs) |
| B10 Actor lifecycle, spawn tables and suppression | `port/src/actors` | 66 | complete: 60 discovered contracts; explicit pending failures | [implemented](port/src/actors/mod.rs); shared [Game](port/src/game.rs) |
| B11 Land boss encounters and shared hit/treasure rules | `port/src/actors/bosses` | 43 | complete: 81 discovered contracts; explicit pending failures | [implemented](port/src/actors/bosses/mod.rs); shared [Game](port/src/game.rs) |
| B12 Title options and land selection | `port/src/selection` | 21 | complete: 43 discovered contracts; explicit pending failures | [implemented](port/src/selection/mod.rs); shared [Game](port/src/game.rs) |
| B13 Gameplay orchestration, stage progression, reload and game over | `port/src/progression` | 45 | complete: 76 discovered contracts; explicit pending failures | [implemented](port/src/progression/mod.rs); shared [Game](port/src/game.rs) |
| B14 Status, health/lives, value arithmetic and timers | `port/src/status` | 45 | complete: 61 discovered contracts; explicit pending failures | [implemented](port/src/status/mod.rs); shared [Game](port/src/game.rs) |
| B15 Dialogue, progression contacts, choices and escorts | `port/src/dialogue` | 148 | complete: 100 discovered contracts; explicit pending failures | [implemented](port/src/dialogue/mod.rs); shared [Game](port/src/game.rs) |
| B16 Special scenes, departures and scene-actor presentation | `port/src/scenes` | 103 | complete: 86 discovered contracts; explicit pending failures | [implemented](port/src/scenes/mod.rs); shared [Game](port/src/game.rs) |
| B17 Final boss, rival race, ending variants and terminal screen | `port/src/finale` | 238 | complete: 95 discovered contracts; explicit pending failures | [implemented](port/src/finale/mod.rs); shared [Game](port/src/game.rs) |
| B18 Music/effects sequence engine and synthesis | `port/src/audio` | 58 | complete: 97 discovered contracts; explicit pending failures | [implemented](port/src/audio/mod.rs); shared [Game](port/src/game.rs) |

## B01 — Reset, deterministic scheduling and controls

**Scope and observations.** Own reset/title entry, four cooperative task slots, input held/pressed edges for both ports, directional conflict filtering, random state, interrupt tick and mapper context. NMI wakes sleeping tasks, commits display, drains audio then runs one audio frame. IRQ points at the same entry; no independent IRQ routine. Preserve zero-delay wrapping (X=0 means 256 waits), task-slot priority and every intentional wait; convert stack continuations to explicit typed tasks.

**Source entry points and data.**
- [`ResetInitialize`](reference/source/game.asm#L22959) — line 22959, physical `$1FEF0..$1FF9D`, mapper bank 7.
- [`SchedulerPoll`](reference/source/game.asm#L23009) — line 23009, physical `$1FEF0..$1FF9D`, mapper bank 7.
- [`SchedulerInstallTask`](reference/source/game.asm#L23062) — line 23062, physical `$1FEF0..$1FF9D`, mapper bank 7.
- [`YieldOneFrame`](reference/source/game.asm#L23138) — line 23138, physical `$1FFBC..$1FFD7`, mapper bank 7.
- [`WaitSchedulerTicks`](reference/source/game.asm#L23111) — line 23111, physical `$1FFB5..$1FFB8`, mapper bank 7.
- [`NmiService`](reference/source/game.asm#L10400) — line 10400, physical `$1C010..$1C0F2`, mapper bank 7.
- [`ReadControllers`](reference/source/game.asm#L10692) — line 10692, physical `$1C1A7..$1C209`, mapper bank 7.
- [`FilterOppositeDirections`](reference/source/game.asm#L10730) — line 10730, physical `$1C1A7..$1C209`, mapper bank 7.
- [`AdvanceRandomState`](reference/source/game.asm#L10946) — line 10946, physical `$1C2F6..$1C368`, mapper bank 7.
- [`SelectPrgBank16K`](reference/source/game.asm#L23222) — line 23222, physical `$1FFEE..$1FFF5`, mapper bank 7.
- [`InterruptVectors`](reference/source/game.asm#L23239) — line 23239, physical `$2000A..$20010`, mapper bank 7.

**Dependencies, state and order.** B02/B03 commit display and B18 consumes the ring at the interrupt boundary; B12/B13/B16/B17 schedule foreground work. Controller sampling belongs to resumed task zero, not arbitrary render/audio calls.

**Proposed interface and tests.** `port/src/runtime` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/runtime/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B01.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove reset seeds, pressed versus held, both opposite-pair filters, task wake/order, input sampling boundaries, RNG call count and delay 0/1/255. Trace whether interrupt and foreground edges map before/after each native tick; do not copy stack/register emulation. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Stage3 worker `/root/behavior_contracts/b01_runtime` complete: adjacent detailed source trace `port/src/runtime/trace.md`, whole-area handoff/test names `planning/contracts/B01.md`, independently scanned runtime-symbol consumer index `planning/contracts/B01-consumers.tsv`; native interfaces/stubs and21 executable contracts in `port/src/runtime/{mod,contracts}.rs`. Rust1.90.0 locked compile/list exit0;23 discovered total (21 B01+2 infrastructure). B01 run exit101:0 passed,21 explicit unimplemented failures,0 ignored; infrastructure2/2 passed. Exact logs/results `checks/contracts/B01-results.json` and sibling logs. No implementation or runtime QA claim. Corrections: reset clears RAM pages1..7; random feedback computed once before all four rotations; fresh task entry skips input sampling; held/pressed conflict filters independently follow edge calculation. Cross-area effects/obligations: B03 reset/PPU/split queues, B12 after-first-yield88 seed/title RNG, B13 interrupt-before-foreground integration and gameplay RNG call counts, B06/B15 game-written input preservation, B07/B11/B12/B16/B17 source-specific random ADC carries, B18 audio drain/frame separation. Boundary integration assumption tracked as `B01-frame-boundary-integration` in uncertainties.json. Static inline dispatcher region1294 traced with zero found callers; no CPU/RAM emulation introduced. Implementation pending.

## B02 — ROM asset access, CHR/background preparation and extensions

**Scope and observations.** Own validated immutable asset catalog and named decode recipes. Eleven CHR lists encode bank/count/source page/destination page and negative terminators; count is pages-minus-one. Background descriptors upload 64-byte chunks and select palette variants by stage/camera threshold. Survey every graphics bank generic AssetData record, mixed audio-tail bitmap and uniform gaps through consumers. Existing 151 raw recipes remain infrastructure, not decoded game assets.

**Source entry points and data.**
- [`UploadChrLists`](reference/source/game.asm#L10966) — line 10966, physical `$1C2F6..$1C368`, mapper bank 7.
- [`ChrUploadListPointers`](reference/source/game.asm#L11019) — line 11019, physical `$1C368..$1C37E`, mapper bank 7.
- [`ChrUploadLists`](reference/source/game.asm#L11024) — line 11024, physical `$1C37E..$1C43C`, mapper bank 7.
- [`PollBackgroundUpload`](reference/source/game.asm#L11033) — line 11033, physical `$1C43C..$1C440`, mapper bank 7.
- [`ChooseBackgroundListVariant`](reference/source/game.asm#L11064) — line 11064, physical `$1C451..$1C46A`, mapper bank 7.
- [`BeginBackgroundUploadRecord`](reference/source/game.asm#L11088) — line 11088, physical `$1C46A..$1C4EE`, mapper bank 7.
- [`QueueBackgroundChunk`](reference/source/game.asm#L11127) — line 11127, physical `$1C46A..$1C4EE`, mapper bank 7.
- [`StagePaletteConfiguration`](reference/source/game.asm#L11208) — line 11208, physical `$1C518..$1C72D`, mapper bank 7.
- [`ChrTiles_00010`](reference/source/game.asm#L12) — line 12, physical `$00010..$00810`, mapper bank 0.
- [`AssetData_00810`](reference/source/game.asm#L18) — line 18, physical `$00810..$00B10`, mapper bank 0.
- [`AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes`](reference/source/game.asm#L1998) — line 1998, physical `$0CAC0..$0F110`, mapper bank 3.

**Dependencies, state and order.** B03 uses decoded CHR/palettes; B04/B10/B15/B16/B18 each own their format-specific data interpretation. Provide typed collections with original source selector adapters and added named level slots; never require extension level count=10.

**Proposed interface and tests.** `port/src/assets` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/assets/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B02.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove all list endpoints, repeated origins, page carry, chunk completion and stage threshold variants. Validate full ROM-backed plus named file-backed level resolution together, including referenced graphics/audio and rejection of invalid cross-references. Do not infer tile identity solely from tile-like bytes. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b02_assets` complete; adjacent whole-area trace `port/src/assets/trace.md`,33 substantive native contracts `port/src/assets/contracts.rs`, visibly pending semantic interfaces `assets/{formats,catalog}.rs`, handoff/all test names `planning/contracts/B02.md` and exhaustive consumer intersections `B02-consumers.json`. Rust1.90 locked compile/discovery exit0;56 tests discovered (33 B02+21 B01+2 infrastructure); B02 exit101:0 passed/33 explicit unimplemented failures/0 ignored; infrastructure exit0 2/2pass. Actual results `checks/contracts/B02-results.json` and sibling logs; transient fixture type-error log retained separately, final compile clean. Entire11 CHR/32 background lists and endpoints,41 row selectors/camera/palette variants,64-byte continuation branches, generic/mixed/uniform consumers and dynamic file-backed level/catalog validation covered. Source corrections: AND FE preserves increment bit2; generic bank0..3 B300..BFFF is required terrain; bank4 BF00 tail has descriptor consumer; mixed audio bank A900..B0FF has graphics consumers. B04 owns packed decoding including200*72 uniform room bytes; B16/B17 own ending selector9 resource context without nonexistent stage entry9; B18 owns mixed audio interpretation/unknown00c522. B01/B03/B13 must integrate immediate versus interrupt upload and palette commit; existing frame uncertainty remains. New source-informed extension schema/rejection assumptions tracked `B02-extension-format-validation`. No behavior implementation/QA claim. Sole editor relinquished; implementation pending.

## B03 — Sprite composition, animation, display queues and effects

**Scope and observations.** Own logical nametables/CHR/palette/OAM presentation snapshot, counted animations, composite piece decoding, flip/priority/offscreen culling, overlay sentinel lists, alternating slot composition and player protection flashing. Queue records use destination high/low, count-minus-one, payload, negative destination terminator. Screen scripts use FE payload terminator and zero end. Preserve UI/playfield split and ending/race split masks, deliberate fades and effect timing; hardware flicker/lag need not be copied.

**Source entry points and data.**
- [`ComposeObjectFlags`](reference/source/game.asm#L2314) — line 2314, physical `$14010..$1401E`, mapper bank 5.
- [`ComposeOverlaySprites`](reference/source/game.asm#L2337) — line 2337, physical `$14026..$1413B`, mapper bank 5.
- [`ComposeObjects`](reference/source/game.asm#L2387) — line 2387, physical `$14026..$1413B`, mapper bank 5.
- [`CullAndProjectObject`](reference/source/game.asm#L2436) — line 2436, physical `$14026..$1413B`, mapper bank 5.
- [`AdvanceObjectAnimation`](reference/source/game.asm#L2508) — line 2508, physical `$14142..$14205`, mapper bank 5.
- [`PrepareCompositeSprite`](reference/source/game.asm#L2556) — line 2556, physical `$14142..$14205`, mapper bank 5.
- [`EmitSpritePieces_NextPiece`](reference/source/game.asm#L2600) — line 2600, physical `$14142..$14205`, mapper bank 5.
- [`HideRejectedSpritePiece`](reference/source/game.asm#L2669) — line 2669, physical `$14243..$14244`, mapper bank 5.
- [`CompositeSpriteFrameRecords_1424BBytes`](reference/source/game.asm#L2685) — line 2685, physical `$1424B..$15691`, mapper bank 5.
- [`CompositeSpriteFramePointers_15691Bytes`](reference/source/game.asm#L2691) — line 2691, physical `$15691..$15881`, mapper bank 5.
- [`SpritePieceCoordinateStreams_15881Bytes`](reference/source/game.asm#L2697) — line 2697, physical `$15881..$16DF1`, mapper bank 5.
- [`SpritePieceCoordinatePointers_16DF1Bytes`](reference/source/game.asm#L2703) — line 2703, physical `$16DF1..$16FB1`, mapper bank 5.
- [`CountedAnimationSequences_16FB1Bytes`](reference/source/game.asm#L2709) — line 2709, physical `$16FB1..$171D7`, mapper bank 5.
- [`WriteScreenScript`](reference/source/game.asm#L3231) — line 3231, physical `$17AC0..$17AF8`, mapper bank 5.
- [`ScreenScriptRecords_17B0ABytes`](reference/source/game.asm#L3292) — line 3292, physical `$17B0A..$17FEE`, mapper bank 5.
- [`FlushPrimaryPpuQueue`](reference/source/game.asm#L10597) — line 10597, physical `$1C108..$1C16E`, mapper bank 7.
- [`FlushSecondaryPpuQueue`](reference/source/game.asm#L10626) — line 10626, physical `$1C16E..$1C193`, mapper bank 7.
- [`FadeInPalette`](reference/source/game.asm#L10861) — line 10861, physical `$1C277..$1C2D8`, mapper bank 7.
- [`FadeOutPalette`](reference/source/game.asm#L10868) — line 10868, physical `$1C277..$1C2D8`, mapper bank 7.
- [`LoadScenePalettes`](reference/source/game.asm#L21213) — line 21213, physical `$1F77D..$1F780`, mapper bank 7.

**Dependencies, state and order.** B01 commits queued presentation at frame boundary; B04 supplies terrain tiles/cache; B14 status buffers; B16 scene scripts. Animation timing affects B07/B08/B09/B11/B17 decisions, so snapshot mutation/animation advancement occurs during tick, rendering is read-only.

**Proposed interface and tests.** `port/src/presentation` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/presentation/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B03.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove counted animation loop/end behavior, signed piece offsets/flips/wrap rejection, world/screen origins, overlay admission, slot ordering, palette black clamp/four-frame fade waits, queue ownership and split row boundaries. Derive exact update order before actor frame-trigger contracts. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker pending; trace evidence pending; tests pending/not yet authored; test compile/run not executed; implementation pending.

**Status.** Worker `/root/behavior_contracts/b03_presentation` complete. Adjacent full trace `port/src/presentation/trace.md`,53 substantive contracts `contracts.rs`, semantic pending declarations `mod.rs`; independent assigned-unit/consumer scan `planning/contracts/B03-consumers.json` and whole-area handoff/test names `planning/contracts/B03.md`. Rust1.90 locked compile/list exit0;109 tests discovered (53 B03+33 B02+21 B01+2 infrastructure). B03 run exit101:0 passed/53 explicit unimplemented failures/0 ignored; infrastructure exit0:2/2 pass. Exact evidence `checks/contracts/B03-results.json` and sibling logs. No game implementation or QA claim. Corrections: animation compares old tick/frame; rejected sprite pieces reuse OAM slot; RAM frame selectors0..3; script5 reads B9F0 outside main script region; raw queue count80 copies129; FillPpuArea attributes use CMP carry, base+3C0. B01 owns ordered NMI effects; B03 owns content, logical CHR/nametable/palette/OAM and raster snapshots; B02 owns prepared decoded uploads; no duplicate queues. B01-frame-boundary-integration reconciled for B03 source content, pending B12/B13+stage4. B10 must observe same-frame inactive/cull/spawn cleanup; B07..B11 animation logic observes composition/flicker timing. B14 supplies overlays/runtime frames/status tail; B16/B17 own enclosing screen/ending/race continuations including E212 deliberate waits, requesting B03 raster output. No new behavior area discovered. Sole editor relinquished; implementation pending.

## B04 — Room layouts, terrain decoding, scrolling and mutable world

**Scope and observations.** Own all ten source layout selectors and their room lists, 200 logical 72-byte room records (64 block selectors/eight high-bit bitmap bytes), terrain planes/properties, 64-column streaming ring, 128-byte collision cache, attributes, camera bounds and scroll. Room records cross maintained incbin fragments and uniform slices; decode logical consumers rather than fragment boundaries. Expand four quadrants into tile/palette/collision planes. Implement stage3/4 progression substitutions, reversible streaming, missing-room FF bounds, sixty-tick vertical transitions and all sixteen phases. Own persistent changed terrain and matching collision/palette updates.

**Source entry points and data.**
- [`TerrainBlockPlanesAndProperties_0F310Bytes`](reference/source/game.asm#L2010) — line 2010, physical `$0F310..$10010`, mapper bank 3.
- [`PackedRoomPayloadFragment_10010Bytes`](reference/source/game.asm#L2016) — line 2016, physical `$10010..$13850`, mapper bank 4.
- [`LoadTerrainLayoutPointers`](reference/source/game.asm#L11310) — line 11310, physical `$1C79F..$1C7DC`, mapper bank 7.
- [`BuildInitialTerrainColumns`](reference/source/game.asm#L11218) — line 11218, physical `$1C72D..$1C796`, mapper bank 7.
- [`BuildWholeTerrainRing`](reference/source/game.asm#L11290) — line 11290, physical `$1C79F..$1C7DC`, mapper bank 7.
- [`ResolveTerrainColumn`](reference/source/game.asm#L11444) — line 11444, physical `$1C840..$1C852`, mapper bank 7.
- [`ExpandTerrainBlock`](reference/source/game.asm#L12125) — line 12125, physical `$1CCCD..$1CCE8`, mapper bank 7.
- [`DecodeTerrainQuadrant`](reference/source/game.asm#L12164) — line 12164, physical `$1CCF7..$1CD46`, mapper bank 7.
- [`TerrainStageThreeOverride`](reference/source/game.asm#L12218) — line 12218, physical `$1CD46..$1CD60`, mapper bank 7.
- [`TerrainStageFourOverride`](reference/source/game.asm#L12250) — line 12250, physical `$1CD64..$1CD8E`, mapper bank 7.
- [`ResolveTerrainBlockPointer`](reference/source/game.asm#L12316) — line 12316, physical `$1CDC4..$1CDF2`, mapper bank 7.
- [`ResolveRoomPayloadPointer`](reference/source/game.asm#L12434) — line 12434, physical `$1CE3C..$1CE67`, mapper bank 7.
- [`StageRoomLayoutLists_1CE67Bytes`](reference/source/game.asm#L12465) — line 12465, physical `$1CE67..$1CFA3`, mapper bank 7.
- [`AnimateVerticalRoomTransition`](reference/source/game.asm#L11678) — line 11678, physical `$1C986..$1CA0B`, mapper bank 7.
- [`VerticalScrollPhasePointers`](reference/source/game.asm#L11783) — line 11783, physical `$1CA3A..$1CA5A`, mapper bank 7.
- [`UpdateCameraMode`](reference/source/game.asm#L19608) — line 19608, physical `$1EFF2..$1EFF8`, mapper bank 7.
- [`ReplaceTerrainForActor`](reference/source/game.asm#L20598) — line 20598, physical `$1F4E8..$1F511`, mapper bank 7.
- [`BuildChangedTerrainRecord`](reference/source/game.asm#L20725) — line 20725, physical `$1F584..$1F616`, mapper bank 7.
- [`TerrainReplacementLookupData`](reference/source/game.asm#L20869) — line 20869, physical `$1F663..$1F6A7`, mapper bank 7.

**Dependencies, state and order.** B02 provides immutable named source/extension assets; B05 queries terrain and B08/B15 mutate it; B13 requests room transitions; B17 builds race terrain through B16. Camera follows B06 position before B10 spawn-window update.

**Proposed interface and tests.** `port/src/world` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/world/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B04.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove each source stage list extent, row/column flattening, 9-bit block selection, quadrant masks, both camera directions, boundary/FF room behavior, vertical phases/attribute trimming, stage substitutions and replacement cache/upload agreement. Added levels use dynamic typed lists, not raw source selector arithmetic. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b04_world` complete and editing ownership relinquished. Adjacent full trace `port/src/world/trace.md`,60 native substantive cases `world/contracts.rs`, pending native types/stubs `world/mod.rs`; handoff/test names `planning/contracts/B04.md`;86 preserved region/107 unit consumer records `B04-consumers.json`. Rust1.90 locked compile/discovery exit0;169 discovered (60 B04+107 retained previous behavior+2 infrastructure). B04 exit101:0 passed/60 explicit unimplemented failures/0 ignored; infrastructure `game::tests` exit0:2/2pass. Exact logs/results `checks/contracts/B04-*`; initial53-case and incorrect zero-test-filter logs retained separately. All10 source layouts/all200 logical72-byte rooms/all512 descriptors per terrain bank0..3, quadrant planes/properties, stage3/4 substitutions, initial/reverse streaming, camera normal/automatic/clamps,16 vertical phases/staged/neighbor rows and sixty-frame ordering, recovery allocation and mutation/cache/upload/burst branches contracted. B01/B03 prepared-vs-committed queue/raster integration, B05 collision sampling/dispatch, B06 vertical eligibility, B08/B10 actor suppression/recovery, B13 reload/checkpoint/frame reset and B15/B16/B17 scene/ending contexts remain assigned owner dependencies. Source residue assumption `B04-discarded-bottom-staging` tracked in uncertainties.json. No gameplay implementation or QA claim.

## B05 — Shared collision, integration and terrain reactions

**Scope and observations.** Own shared fixed-point integration, collision profiles/sample geometry, cache queries, special terrain response dispatch, penetration/snap results, solid handler-eight actor overlap and player contact/damage predicates. Preserve source carry/borrow semantics as explicit results including boundary escape separately from hit. Terrain reactions can queue state6/7/8, set render bits and momentum. Own generic burst/particle mechanics and side effects, not object reward policy.

**Source entry points and data.**
- [`ProbeTerrainCollision`](reference/source/game.asm#L12711) — line 12711, physical `$1D130..$1D17B`, mapper bank 7.
- [`TerrainCollisionHandlerPointers`](reference/source/game.asm#L12847) — line 12847, physical `$1D1F2..$1D212`, mapper bank 7.
- [`SampleCollisionCache`](reference/source/game.asm#L13144) — line 13144, physical `$1D360..$1D3D6`, mapper bank 7.
- [`CollisionGeometryLookupData`](reference/source/game.asm#L13231) — line 13231, physical `$1D3DE..$1D4AA`, mapper bank 7.
- [`PlayerHorizontalOverlapScan`](reference/source/game.asm#L12857) — line 12857, physical `$1D212..$1D23D`, mapper bank 7.
- [`PlayerVerticalOverlapScan`](reference/source/game.asm#L12990) — line 12990, physical `$1D2A1..$1D2FC`, mapper bank 7.
- [`SnapActorPositiveX`](reference/source/game.asm#L13241) — line 13241, physical `$1D4AA..$1D4C2`, mapper bank 7.
- [`MovePositiveXWithCollision`](reference/source/game.asm#L19815) — line 19815, physical `$1F108..$1F139`, mapper bank 7.
- [`MoveNegativeXWithCollision_F13B`](reference/source/game.asm#L19870) — line 19870, physical `$1F14A..$1F15D`, mapper bank 7.
- [`FallWithCollision`](reference/source/game.asm#L20011) — line 20011, physical `$1F1CE..$1F219`, mapper bank 7.
- [`RiseWithCollision`](reference/source/game.asm#L20042) — line 20042, physical `$1F1CE..$1F219`, mapper bank 7.
- [`IntegratePositiveX`](reference/source/game.asm#L20118) — line 20118, physical `$1F243..$1F2F6`, mapper bank 7.
- [`IntegrateNegativeX`](reference/source/game.asm#L20134) — line 20134, physical `$1F243..$1F2F6`, mapper bank 7.
- [`IntegratePositiveY`](reference/source/game.asm#L20150) — line 20150, physical `$1F243..$1F2F6`, mapper bank 7.
- [`IntegrateNegativeY`](reference/source/game.asm#L20161) — line 20161, physical `$1F243..$1F2F6`, mapper bank 7.
- [`ApplyActorGravity`](reference/source/game.asm#L20191) — line 20191, physical `$1F243..$1F2F6`, mapper bank 7.
- [`TestHorizontalActorOverlap`](reference/source/game.asm#L20298) — line 20298, physical `$1F342..$1F34B`, mapper bank 7.
- [`TestVerticalActorOverlap`](reference/source/game.asm#L20398) — line 20398, physical `$1F3C0..$1F3CC`, mapper bank 7.
- [`TestPlayerActorOverlap`](reference/source/game.asm#L20482) — line 20482, physical `$1F3CE..$1F466`, mapper bank 7.
- [`ObjectCollisionExtentData`](reference/source/game.asm#L20588) — line 20588, physical `$1F4A8..$1F4E8`, mapper bank 7.
- [`Bank04_SpawnTerrainParticles`](reference/source/game.asm#L2161) — line 2161, physical `$13D90..$13DD7`, mapper bank 4.
- [`BuildChangedTerrainBurst`](reference/source/game.asm#L12613) — line 12613, physical `$1D078..$1D120`, mapper bank 7.

**Dependencies, state and order.** B04 supplies cache and world mutation; B06 owns player state changes; B10 owns slot allocation/removal; actor families own reaction choice. Protection/special-mode gates and queued versus immediate state writes must be reconciled with B13.

**Proposed interface and tests.** `port/src/collision` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/collision/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B05.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove all sixteen terrain dispatch entries, sample ordering/early exit, coordinate wraps, profile flag masks, exact contact inequalities, damage gates, handler-eight scan order and penetration signs. Trace SEC/CLC and flag-preserving calls; native arithmetic tests must cover page and sign boundaries. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b05_collision` complete, sole edit ownership relinquished. Adjacent `collision/{mod.rs,contracts.rs,trace.md}`, handoff `planning/contracts/B05.md`,75-region/94-unit consumer index `B05-consumers.json`. Rust1.90 locked compile/discovery exit0;238 discovered (69 B05+167 prior behavior+2 infrastructure). B05 exit101:0passed/69 explicit unimplemented failures/0ignored; `game::tests` exit0:2/2pass. Exact logs/results `checks/contracts/B05-*`; initial63 logs retained, interim67 estimate corrected. All16 terrain dispatch/101 signed sample geometry/128cache×4quadrants/32profiles, fixed-point integration/six wrappers, lifetime, solid/player predicates, source carry quirks, transition clear/timers, particles covered. Source-comment corrections: pose ADC7 adds8 after compare carry; clear preserves slot1; reachable preset1 reads bounded ROM-backed instruction-overlap data. B04 owns cache/decoder/burst-video, B10 slot/lifetime and distinct bank6 actor-actor helper, B06 player merges/unusual E8C2 caller, B13 queued reactions, actor families caller policy and B18 sound integration remain required. No gameplay/QA/audit/new area or source/ledger identity changes.

## B06 — Player action state machine and life/death motion

**Scope and observations.** Own all seventeen source player selectors, standing/walking/crouch, jump/pogo, climb, swing and terrain-object strike, momentum surfaces, special pose, damage/protection, death/fall/life loss, held/carried/external launch and transition alignment. Preserve 8.8 velocity and signed conventions, input edge/held distinctions and animation-trigger action windows. State0F dialogue is delegated to B15 while dispatcher and transition interface remain here.

**Source entry points and data.**
- [`UpdatePlayerProtection`](reference/source/game.asm#L16603) — line 16603, physical `$1E364..$1E36C`, mapper bank 7.
- [`UpdatePlayerSpecialMode`](reference/source/game.asm#L16623) — line 16623, physical `$1E370..$1E374`, mapper bank 7.
- [`DispatchPlayerState`](reference/source/game.asm#L16657) — line 16657, physical `$1E397..$1E3A6`, mapper bank 7.
- [`PlayerStateHandlerPointers_1E3A6Bytes`](reference/source/game.asm#L16669) — line 16669, physical `$1E3A6..$1E3C8`, mapper bank 7.
- [`PlayerStandingEntry`](reference/source/game.asm#L16677) — line 16677, physical `$1E3C8..$1E3D8`, mapper bank 7.
- [`PlayerJumpEdge`](reference/source/game.asm#L16728) — line 16728, physical `$1E3FC..$1E402`, mapper bank 7.
- [`PlayerMomentumAcceleration`](reference/source/game.asm#L16977) — line 16977, physical `$1E539..$1E561`, mapper bank 7.
- [`PlayerMomentumDeceleration`](reference/source/game.asm#L16889) — line 16889, physical `$1E4B6..$1E520`, mapper bank 7.
- [`PlayerAirborneEntry`](reference/source/game.asm#L17218) — line 17218, physical `$1E64B..$1E659`, mapper bank 7.
- [`PlayerLandOrPogo`](reference/source/game.asm#L17295) — line 17295, physical `$1E6AE..$1E6FE`, mapper bank 7.
- [`FindPlayerClimbAlignment`](reference/source/game.asm#L17434) — line 17434, physical `$1E763..$1E765`, mapper bank 7.
- [`UpdatePlayerClimbing`](reference/source/game.asm#L17644) — line 17644, physical `$1E816..$1E888`, mapper bank 7.
- [`PlayerRoomTransition`](reference/source/game.asm#L17712) — line 17712, physical `$1E888..$1E8D8`, mapper bank 7.
- [`UpdatePlayerSwing`](reference/source/game.asm#L17799) — line 17799, physical `$1E8FB..$1E980`, mapper bank 7.
- [`PlayerVerticalPhysics`](reference/source/game.asm#L17876) — line 17876, physical `$1E980..$1EA05`, mapper bank 7.
- [`StartPlayerDamageArc`](reference/source/game.asm#L18152) — line 18152, physical `$1EAC9..$1EB92`, mapper bank 7.
- [`EnterPlayerDeathArc`](reference/source/game.asm#L18190) — line 18190, physical `$1EAC9..$1EB92`, mapper bank 7.
- [`PlayerSnowLikePose`](reference/source/game.asm#L18230) — line 18230, physical `$1EB92..$1EBB3`, mapper bank 7.
- [`PlayerFallOutOfScreen`](reference/source/game.asm#L18271) — line 18271, physical `$1EBB7..$1ED15`, mapper bank 7.
- [`WaitForPlayerA`](reference/source/game.asm#L18291) — line 18291, physical `$1EBB7..$1ED15`, mapper bank 7.
- [`UpdatePlayerCarriedState`](reference/source/game.asm#L18303) — line 18303, physical `$1EBB7..$1ED15`, mapper bank 7.
- [`UpdatePlayerExternalLaunch`](reference/source/game.asm#L18348) — line 18348, physical `$1EBB7..$1ED15`, mapper bank 7.
- [`UpdatePlayerDeathTrajectory`](reference/source/game.asm#L18402) — line 18402, physical `$1EBB7..$1ED15`, mapper bank 7.
- [`HoldPlayerAnimation`](reference/source/game.asm#L18448) — line 18448, physical `$1EBB7..$1ED15`, mapper bank 7.

**Dependencies, state and order.** B05 gives typed contacts/motion bounds; B04/B13 room and reload requests; B09/B16 can carry/launch/hold; B14 costs/lives/health and special timer; B15 dialogue continuation. B13 applies queued reactions after composition/spawn as source orders.

**Proposed interface and tests.** `port/src/player` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/player/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B06.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove every pointer selector entry, jump/pogo hold/release and landing, swing impact frame, climbing alignment/upward/downward room thresholds, momentum reversal/clamping, protection expiry and all damage/death variants. Keep provisional selector IDs until source trace establishes descriptive enums. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status (supersedes earlier pending/active entries).** Worker `/root/behavior_contracts/b06_player` complete and sole editor relinquished. Adjacent `player/{mod.rs,contracts.rs,trace.md}`; full handoff/discovered test names `planning/contracts/B06.md`; original91region/104unit/34routine-call/159state-consumer index `B06-consumers.json`. Rust1.90 locked compile/list0,485total discovered (128 B06+355 retained previous behavior+2 infrastructure). B06 area101:0pass/128explicit unimplemented failures/0ignored; infrastructure `game::tests`0:2pass. Exact final `checks/contracts/B06-{results.json,compile.log,discovery.log,area.log,infrastructure.log}`; initial fixture/accessor compile failures and superseded125-case results retained separately. All17targets/all control, animation, momentum, collision/terrain merge, climb/room/swing, protection/damage/pose/life-loss/carry/launch/death branches and full-call immediate-vs-queued effects source traced and contracted. B01 writable held/pressed plus port2 gate; B03 canonical animation_restart projection; B04 source upward slot2Y=01 indexed write/nonlocal transition; B05 borrowed collision adapter; B07..B11 external actor eligibility/immediate writes; B13 tail reaction/reload; B14 borrowed Health/timed special/conversion; B15 dialogue; B16 runtime status-frame author; B17 holds/goals and B18 ordered audio remain integration obligations. No new area/gameplay/QA/toolkit/check-in/publication. All implementation remains pending.

## B07 — Ordinary enemies, projectiles and reward stealing

**Scope and observations.** Own ordinary ground/jumping/ledge-aware, thrower/projectile, animation-gated proximity, delayed flight/preset, paired, ambush, eruption/palette, repeated jump/patrol/approach actors and their local state dispatches. Include timed/jumping thieves and decimal subtraction helper, spawn-on-animation, companion allocation and offscreen survival rules. Address-derived names remain valid identities until visual/source evidence resolves species.

**Source entry points and data.**
- [`Actor8225VisibilityAndHit`](reference/source/game.asm#L3483) — line 3483, physical `$18235..$1823F`, mapper bank 6.
- [`Bank06_WalkingBounceActor`](reference/source/game.asm#L3609) — line 3609, physical `$182BF..$18309`, mapper bank 6.
- [`Bank06_ProjectileThrower`](reference/source/game.asm#L3687) — line 3687, physical `$1831E..$184A1`, mapper bank 6.
- [`Bank06_ThrownProjectile`](reference/source/game.asm#L3849) — line 3849, physical `$1831E..$184A1`, mapper bank 6.
- [`Actor8491AnimationGate`](reference/source/game.asm#L3879) — line 3879, physical `$184A1..$184A6`, mapper bank 6.
- [`Bank06_ProximityRisingActor`](reference/source/game.asm#L3957) — line 3957, physical `$184EC..$18671`, mapper bank 6.
- [`Bank06_PlayerContactAnimationActor`](reference/source/game.asm#L4026) — line 4026, physical `$184EC..$18671`, mapper bank 6.
- [`Actor9313Wait`](reference/source/game.asm#L6161) — line 6161, physical `$19323..$1935D`, mapper bank 6.
- [`Bank06_DelayedPresetFlyingActor`](reference/source/game.asm#L6340) — line 6340, physical `$193AE..$19554`, mapper bank 6.
- [`Bank06_PairedActorController`](reference/source/game.asm#L6486) — line 6486, physical `$19562..$196CB`, mapper bank 6.
- [`Bank06_LedgeAwareWalker`](reference/source/game.asm#L6674) — line 6674, physical `$196E5..$1994C`, mapper bank 6.
- [`Bank06_FivePhaseAmbushActor`](reference/source/game.asm#L6725) — line 6725, physical `$196E5..$1994C`, mapper bank 6.
- [`Bank06_ProximityEruptingActor`](reference/source/game.asm#L6981) — line 6981, physical `$19966..$19A85`, mapper bank 6.
- [`Bank06_RepeatingJumpActor`](reference/source/game.asm#L7133) — line 7133, physical `$19A91..$19B20`, mapper bank 6.
- [`Actor9B10Patrol`](reference/source/game.asm#L7234) — line 7234, physical `$19B2D..$19B94`, mapper bank 6.
- [`Bank06_FastLedgeAwareWalker`](reference/source/game.asm#L7606) — line 7606, physical `$19D9A..$19E05`, mapper bank 6.
- [`Bank06_TimedHorizontalPatrol`](reference/source/game.asm#L7669) — line 7669, physical `$19E07..$19E61`, mapper bank 6.
- [`Actor9E51Initialize`](reference/source/game.asm#L7716) — line 7716, physical `$19E61..$19E8A`, mapper bank 6.
- [`Bank06_TriggeredJumpingPatrol`](reference/source/game.asm#L7855) — line 7855, physical `$19EC3..$1A075`, mapper bank 6.
- [`Bank06_LeftwardWalkingActor`](reference/source/game.asm#L7944) — line 7944, physical `$19EC3..$1A075`, mapper bank 6.
- [`ActorA065HitGate`](reference/source/game.asm#L8009) — line 8009, physical `$1A075..$1A07F`, mapper bank 6.
- [`Bank06_TimedRewardStealingPatrol`](reference/source/game.asm#L9579) — line 9579, physical `$1AC4D..$1ACCC`, mapper bank 6.
- [`Bank06_JumpingRewardStealer`](reference/source/game.asm#L9650) — line 9650, physical `$1ACCE..$1AE24`, mapper bank 6.
- [`Bank06_HitOnlyReactionActor`](reference/source/game.asm#L9788) — line 9788, physical `$1ACCE..$1AE24`, mapper bank 6.

**Dependencies, state and order.** B10 supplies stable slots, spawn IDs and RNG inputs from B01; B05 shared physics/hit tests; B03 animation frames are logic inputs; B14 owns balances and atomic theft operation; B18 queues sounds. Carry/contact platforms are B09 and boss routines B11.

**Proposed interface and tests.** `port/src/actors/enemies` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/actors/enemies/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B07.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Enumerate every gameplay pointer targeting this area, all inline state tables and local data presets. Prove initialized/resting/moving/hit/recovery/removal phases, unavailable slots, companion links, proximity edges, timing and theft borrow/blank-digit cases. Do not treat visually unnamed handlers as out of scope. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b07_enemies` complete, sole editor relinquished. Full adjacent `actors/enemies/{mod.rs,contracts.rs,trace.md}`, whole-area handoff/discovered names `planning/contracts/B07.md`, independent81primary/related-region/115unit/56caller/2789state-reference index `B07-consumers.json`. Actual67entry table proves26B07selectors/25distinct targets (30/31alias); earlier30/29communication superseded as counting error with no removed targets.97substantive contracts cover all source families/local phases, motion/contact/animation, full-slot/stale-child fields, RNG carries, player attach/release/launch and immediate theft trigger effects. Rust1.90locked compile/list0:582total (97B07+483prior behavior+2infra); B07exit101:0pass/97explicit pending-stub failures/0ignored; infrastructure game::tests0:2pass. Exact final commands/results/hashes/logs `checks/contracts/B07/results.json`; initial/superseded logs preserved. B05preset1 named recipe, B08reaction/terrainlaunch, B10canonical live dispatch, B06borrowedPlayer, B14same-actor immediate theft, B01distinct caller RNG, B03palette/clocks and B18audio remain implementation dependencies. Timed theft cancels same-frame movement decrement and can repeat; jumping thief alone latches. Ambush2/3alias/twostepadvance, pairedfullpoollink0, eruptionoldY/pauseevents, Approach restoredoldfacing, ContactPoseFFwrap and triggered-phase3retention corrected from instructions. No new cohesive area/assumption requiring emulator; no gameplay/QA/toolkit/check-in/publication. Implementation pending.

## B08 — Interactive objects, recoveries, items and treasure collection

**Worker status.** Complete: `/root/behavior_contracts/b08_objects`; sole editor relinquished. Implementation pending.

**Scope and observations.** Own swing-activated terrain objects and nine local motion states, launched/sliding/falling items, hit/transform/recovery reveal paths, secondary rewards, collectible dispatch by animation, four-item dispenser and dispensed-item bounce/contact. Apply source health/life/special/value rewards and stage treasure gates including protection restrictions, stage completion bits and return transitions. Terrain changes must update visual and collision representations together.

**Source entry points and data.**
- [`Actor8668Cull`](reference/source/game.asm#L4163) — line 4163, physical `$18678..$1867D`, mapper bank 6.
- [`Bank06_StartTerrainObjectMotion`](reference/source/game.asm#L4215) — line 4215, physical `$1869D..$18701`, mapper bank 6.
- [`ActorHitResponseEntry_86F2`](reference/source/game.asm#L4268) — line 4268, physical `$18701..$18715`, mapper bank 6.
- [`Bank06_UpdateMovingTerrainObject`](reference/source/game.asm#L4320) — line 4320, physical `$18732..$188EF`, mapper bank 6.
- [`ActorLaunchedMotion`](reference/source/game.asm#L4666) — line 4666, physical `$1898B..$189DE`, mapper bank 6.
- [`ActorTrySecondarySpawn`](reference/source/game.asm#L4744) — line 4744, physical `$189EA..$189F6`, mapper bank 6.
- [`Bank06_SelectTerrainObjectReward`](reference/source/game.asm#L4804) — line 4804, physical `$18A24..$18A69`, mapper bank 6.
- [`Actor8A59AnimationGate`](reference/source/game.asm#L4842) — line 4842, physical `$18A69..$18A6E`, mapper bank 6.
- [`ActorCollectDispatch`](reference/source/game.asm#L5013) — line 5013, physical `$18B31..$18B45`, mapper bank 6.
- [`ActorApplyReward`](reference/source/game.asm#L5057) — line 5057, physical `$18B6A..$18B87`, mapper bank 6.
- [`AddRewardDigit`](reference/source/game.asm#L5133) — line 5133, physical `$18BCF..$18BE5`, mapper bank 6.
- [`Bank06_RewardTimedSpecialMode`](reference/source/game.asm#L5106) — line 5106, physical `$18B87..$18BCF`, mapper bank 6.
- [`Bank06_RewardLifeCounter`](reference/source/game.asm#L5117) — line 5117, physical `$18B87..$18BCF`, mapper bank 6.
- [`Bank06_CollectStageTreasure`](reference/source/game.asm#L5205) — line 5205, physical `$18C23..$18C55`, mapper bank 6.
- [`Actor8C45ContactEntry`](reference/source/game.asm#L5239) — line 5239, physical `$18C55..$18C5A`, mapper bank 6.
- [`ActorSpecialRecoveryHandler`](reference/source/game.asm#L5388) — line 5388, physical `$18CF1..$18CF6`, mapper bank 6.
- [`Actor8D19ContactProfile`](reference/source/game.asm#L5434) — line 5434, physical `$18D29..$18D45`, mapper bank 6.
- [`Bank06_TransformHitTerrainObject`](reference/source/game.asm#L5467) — line 5467, physical `$18D4F..$18DA6`, mapper bank 6.
- [`Bank06_RevealTransformedObject`](reference/source/game.asm#L5569) — line 5569, physical `$18DE1..$18EEE`, mapper bank 6.
- [`Bank06_SwingReleasedCollectible`](reference/source/game.asm#L5712) — line 5712, physical `$18EF6..$18FC8`, mapper bank 6.
- [`Bank06_UpdateReleasedCollectible`](reference/source/game.asm#L5782) — line 5782, physical `$18EF6..$18FC8`, mapper bank 6.
- [`Bank06_FourItemDispenser`](reference/source/game.asm#L8322) — line 8322, physical `$1A1F9..$1A2BE`, mapper bank 6.
- [`Bank06_UpdateDispensedItem`](reference/source/game.asm#L8427) — line 8427, physical `$1A2CA..$1A595`, mapper bank 6.

**Dependencies, state and order.** B05 contacts; B04 terrain replacement; B10 allocation/suppression; B06 swing frame/special state; B14 atomic rewards, saturating digits and option-dependent remap; B13 completion/selection; B15 reward dialogue.

**Proposed interface and tests.** `port/src/actors/objects` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/actors/objects/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B08.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove all collection animation classes, reward values/caps, recovery countdowns, nine movement-state dispatches, duplicate collection prevention, terrain mutation/reward allocation failure, treasure flags and protected/nonprotected contact. Reconcile shared reward arithmetic with B14 without duplicate owner. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Whole-area source trace and94native substantive contracts complete, implementation pending. Adjacent `actors/objects/{mod.rs,contracts.rs,trace.md}`, handoff `planning/contracts/B08.md`, index `B08-consumers.json` preserves78regions/94units with48directcaller and2812state references. All12targets,9motionentries/5distinctbodies and shared reaction/secondary/collection branches covered. Rust1.90locked compile/list0:676total=94B08+580priorbehavior+2infra; area101:0pass/94explicit unimplemented/0ignored; game::tests0:2pass. Exact logs/results/hashes `checks/contracts/B08/results.json`; initial90case warning log retained. B07 trace/handoff sentinelFF claim explicitly superseded: audio empty88/occupied<88/occupiedFF determine distinct secondary RNG increment even whenFF drops. Reveal index7 adds24 viaCPYcarry; contact delay begins29 samecall; both slidingpaths call player; released nofloorlatches and landedtail retains42; dispenser fullpool/retry/countdown and arithmetic wrap covered. B04/B05/B06/B10/B13/B14/B15/B03/B18 immediate owner dependencies enumerated in handoff. No new area/emulator uncertainty; no gameplay/QA/toolkit/check-in/publication.

## B09 — Platforms, carrying and externally imposed movement

**Scope and observations.** Own vertical/horizontal/triggered/preset/rideable/four-phase platforms, pushing/braking contact actor, player carry/lift, external launch triggers and displacement transfer. Four-phase direction changes use forty-eight-tick holds; carry uses signed 16-bit player-X addition and collision profile swap. Keep player-linked scripted departure in B16.

**Source entry points and data.**
- [`Bank06_VerticalCarryingPlatform`](reference/source/game.asm#L5856) — line 5856, physical `$19075..$19225`, mapper bank 6.
- [`Bank06_TriggeredWalkingPlatform`](reference/source/game.asm#L5899) — line 5899, physical `$19075..$19225`, mapper bank 6.
- [`Bank06_PresetOscillatingActor`](reference/source/game.asm#L5963) — line 5963, physical `$19075..$19225`, mapper bank 6.
- [`Bank06_RideablePresetActor`](reference/source/game.asm#L6056) — line 6056, physical `$19235..$192FD`, mapper bank 6.
- [`Bank06_PushingContactActor`](reference/source/game.asm#L7440) — line 7440, physical `$19C56..$19D96`, mapper bank 6.
- [`Bank06_PlayerCarryingActor`](reference/source/game.asm#L7791) — line 7791, physical `$19EC3..$1A075`, mapper bank 6.
- [`Bank06_PlayerExternalMotionTrigger`](reference/source/game.asm#L7970) — line 7970, physical `$19EC3..$1A075`, mapper bank 6.
- [`Bank06_PlayerLaunchContactActor`](reference/source/game.asm#L9794) — line 9794, physical `$1ACCE..$1AE24`, mapper bank 6.
- [`UpdatePlayerLiftActor`](reference/source/game.asm#L21854) — line 21854, physical `$1FA08..$1FA4F`, mapper bank 7.
- [`UpdateFourPhaseMovingPlatform`](reference/source/game.asm#L22892) — line 22892, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`MovingPlatformPhaseData`](reference/source/game.asm#L22949) — line 22949, physical `$1FEE6..$1FEF0`, mapper bank 7.

**Dependencies, state and order.** B06 owns held/carried/external player selectors; B05 supplies contact and penetration; B03 animation triggers; B10 allocation; B13 completion requests. Must preserve update-slot order when both platform and player move in one frame.

**Proposed interface and tests.** `port/src/actors/platforms` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/actors/platforms/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B09.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove all phase/preset transitions and endpoints, rider acquire/release, player X page carry/borrow, profile restoration, offscreen carry/life-loss variants and occupied-slot behavior. Trace pushing actor animation events and launch-only gates. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker pending; trace evidence pending; tests pending/not yet authored; test compile/run not executed; implementation pending.

**Stage3 whole-area handoff.** Worker `/root/behavior_contracts/b09_platforms`
completed all nine source selectors and full local phase/data/caller branches.
Evidence `port/src/actors/platforms/{trace.md,contracts.rs,mod.rs}` and
`planning/contracts/B09.md`;9region/19unit/179caller/2650state-ref index
`planning/contracts/B09-consumers.json`.81B09contracts discovered; whole native
763tests compile. B09area81explicitunimplemented failures/0ignored; infra2pass.
B08actual869Dgate+38tuple datarecipe adds5contracts (99total), B05actualrepeat
count adds1 (70total); earlier94/69case records retained with superseding links.
No source implementation/QA claim. Cross-effects: same-frame currentplayeraction,
postmotioncontact, unmasked3Flaunch tuple25hex, signedfraction/pagewrap, push02FF,
B05repeatstart1/actualreturnedcount, live slot mutation. B07selector34 ownslaunch.
No new cohesive area or emulator uncertainty. Complete check commands/results
`checks/contracts/B09/results.json`; implementation and integration pending.

## B10 — Actor lifecycle, spawn tables and suppression

**Scope and observations.** Own sixteen object slots (player0, actors1..15), active/render/local-state distinction, 67 source handler selectors, update order, free-slot policies, templates, spawn IDs and suppression bitset. Own stage main/special spawn pointer planes, row/window cursors both camera directions, duplicate identity checks, cooldown/negative occupied-slot latch, progression override to final handler3A and boss HP5, stage-one suppression and special slot preservation. Final six zero bytes of spawn data are generic non-code, not an assumed padding contract.

**Source entry points and data.**
- [`UpdateActorHandlers`](reference/source/game.asm#L3305) — line 3305, physical `$18010..$1803C`, mapper bank 6.
- [`GameplayActorHandlerPointers_1803CBytes`](reference/source/game.asm#L3334) — line 3334, physical `$1803C..$180C2`, mapper bank 6.
- [`EmptyActorHandler`](reference/source/game.asm#L3342) — line 3342, physical `$180C2..$180C3`, mapper bank 6.
- [`Bank06_FindInactiveActor`](reference/source/game.asm#L3351) — line 3351, physical `$180C3..$1815A`, mapper bank 6.
- [`Bank06_LoadMotionPreset`](reference/source/game.asm#L3458) — line 3458, physical `$1819A..$181B5`, mapper bank 6.
- [`BuildSpawnSuppression`](reference/source/game.asm#L3067) — line 3067, physical `$17770..$17790`, mapper bank 5.
- [`StageSpawnSuppressionRecords_177AFBytes`](reference/source/game.asm#L3123) — line 3123, physical `$177AF..$17810`, mapper bank 5.
- [`LoadStageSpawnTables`](reference/source/game.asm#L9845) — line 9845, physical `$1AE30..$1AE65`, mapper bank 6.
- [`FindSpawnWindowStart`](reference/source/game.asm#L9881) — line 9881, physical `$1AE70..$1AE7F`, mapper bank 6.
- [`ResetSpawnWindow`](reference/source/game.asm#L9907) — line 9907, physical `$1AE84..$1AE8D`, mapper bank 6.
- [`UpdateSpawnWindow`](reference/source/game.asm#L9925) — line 9925, physical `$1AEB0..$1AED0`, mapper bank 6.
- [`SpawnCandidateIfEligible`](reference/source/game.asm#L10040) — line 10040, physical `$1AF0E..$1AF6C`, mapper bank 6.
- [`InitializeSpawnedActor`](reference/source/game.asm#L10088) — line 10088, physical `$1AF71..$1B005`, mapper bank 6.
- [`SpawnProgressionOverride`](reference/source/game.asm#L10165) — line 10165, physical `$1B005..$1B01A`, mapper bank 6.
- [`Bank06_InitializeFinalBossOverride`](reference/source/game.asm#L10184) — line 10184, physical `$1B01A..$1B02C`, mapper bank 6.
- [`FindSpecialSpawn`](reference/source/game.asm#L10256) — line 10256, physical `$1B0BB..$1B0FB`, mapper bank 6.
- [`CreateSpecialSpawn`](reference/source/game.asm#L10343) — line 10343, physical `$1B115..$1B174`, mapper bank 6.
- [`SpecialAndMainSpawnRecords_1B174Bytes`](reference/source/game.asm#L10392) — line 10392, physical `$1B174..$1C010`, mapper bank 6.
- [`ClearNonplayerActors`](reference/source/game.asm#L20202) — line 20202, physical `$1F243..$1F2F6`, mapper bank 7.
- [`FindFreeActorSlot`](reference/source/game.asm#L21389) — line 21389, physical `$1F7CE..$1F7D8`, mapper bank 7.
- [`SuppressActorRespawn`](reference/source/game.asm#L21465) — line 21465, physical `$1F800..$1F803`, mapper bank 7.
- [`ObjectRemovalBitMasks`](reference/source/game.asm#L21555) — line 21555, physical `$1F81A..$1F822`, mapper bank 7.
- [`FaceActorTowardPlayer`](reference/source/game.asm#L20909) — line 20909, physical `$1F6CD..$1F6E7`, mapper bank 7.

**Dependencies, state and order.** Calls B07/B08/B09/B11/B15/B16/B17 by cohesive handler family; source gravity seed40 precedes actor slots ascending. B04 camera/world and B13 stage/progression choose spawn lists. B01 RNG and B03 visibility gate some lifetimes.

**Proposed interface and tests.** `port/src/actors` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/actors/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B10.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove all 67 target ownership entries, 121 four-byte templates, each of nine stage spawn pointer entries/row list extents and special pair path; no-slot returns, forward/back cursor bounds, persistent suppression and clearing exceptions. Preserve source overload of negative special-spawn latch as typed occupied versus cooldown state. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b10_lifecycle` complete; sole editor relinquished. Full adjacent `actors/{mod.rs,contracts.rs,trace.md}`,58 substantive native contracts; handoff `planning/contracts/B10.md`,66-region/69-unit/813-use consumer index `B10-consumers.json`. Rust1.90 locked compile/discovery exit0;296 discovered (58 B10+236 prior behavior+2 infrastructure). B10 exit101:0passed/58 explicit unimplemented failures/0ignored; `game::tests` exit0:2/2pass. Exact logs/results `checks/contracts/B10-*`,initial55 evidence retained. Canonical ActorContact moved to actors and collision reexports it; presentation borrowed view requires stage4 immediate merge. All67 target owners/all121 templates/all source main/special rows and dynamic named catalogs,dispatch/allocation/identity/suppression/cursor/pair/full-pool/contact/preset/facing/distance branches contracted. Corrections: forward special edge+00F8,paired full-pool writes playerY=C0,ordinary-vs-special initialization differences,highest-match duplicate rule. B11 semantically owns full boss-hit helper including five original B10 metadata fragments; no source/ledger rewrite. B01/B03/B04/B05/B06/families/B13/B15/B16/B17/B18 dependencies in handoff. No new area or gameplay/QA claim.

## B11 — Land boss encounters and shared hit/treasure rules

**Scope and observations.** Own bank-six attack/preset/triplet, jumping-impact/quake, perimeter path, bouncing-wave/children and center-return encounter families, shared boss HP/hit/invulnerability rules and normal treasure spawn fallback. Exact land/species association requires tracing spawn templates, not folklore names. Final animation84 bypasses ordinary treasure allocation and delegates B17.

**Source entry points and data.**
- [`Bank06_FivePhaseAttackActor`](reference/source/game.asm#L8486) — line 8486, physical `$1A2CA..$1A595`, mapper bank 6.
- [`Bank06_AttackActorChooseHeight`](reference/source/game.asm#L8527) — line 8527, physical `$1A2CA..$1A595`, mapper bank 6.
- [`Bank06_AttackActorPresetPath`](reference/source/game.asm#L8575) — line 8575, physical `$1A2CA..$1A595`, mapper bank 6.
- [`Bank06_SpawnAttackTriplet`](reference/source/game.asm#L8712) — line 8712, physical `$1A2CA..$1A595`, mapper bank 6.
- [`Bank06_UpdateAttackProjectile`](reference/source/game.asm#L8776) — line 8776, physical `$1A5CE..$1A6D9`, mapper bank 6.
- [`Bank06_JumpingImpactActor`](reference/source/game.asm#L8794) — line 8794, physical `$1A5CE..$1A6D9`, mapper bank 6.
- [`Bank06_ImpactShakeDisplay`](reference/source/game.asm#L8872) — line 8872, physical `$1A5CE..$1A6D9`, mapper bank 6.
- [`Bank06_PerimeterPathActor`](reference/source/game.asm#L8924) — line 8924, physical `$1A6E3..$1A8BC`, mapper bank 6.
- [`Bank06_AdvancePerimeterPath`](reference/source/game.asm#L9114) — line 9114, physical `$1A6E3..$1A8BC`, mapper bank 6.
- [`Bank06_BouncingWaveActor`](reference/source/game.asm#L9164) — line 9164, physical `$1A8E6..$1AAB8`, mapper bank 6.
- [`Bank06_StartTimedActorWave`](reference/source/game.asm#L9294) — line 9294, physical `$1A8E6..$1AAB8`, mapper bank 6.
- [`Bank06_EmitTimedWaveChildren`](reference/source/game.asm#L9319) — line 9319, physical `$1A8E6..$1AAB8`, mapper bank 6.
- [`Bank06_UpdateWaveChild`](reference/source/game.asm#L9393) — line 9393, physical `$1AAD8..$1AC47`, mapper bank 6.
- [`Bank06_CenterReturningActor`](reference/source/game.asm#L9408) — line 9408, physical `$1AAD8..$1AC47`, mapper bank 6.
- [`Bank06_UpdateCenterActorMotion`](reference/source/game.asm#L9485) — line 9485, physical `$1AAD8..$1AC47`, mapper bank 6.
- [`ValidateBossHitAndDefeat`](reference/source/game.asm#L20929) — line 20929, physical `$1F6E7..$1F6E8`, mapper bank 7.
- [`SpawnBossTreasure`](reference/source/game.asm#L20983) — line 20983, physical `$1F712..$1F715`, mapper bank 7.

**Dependencies, state and order.** B05 collision/hit and B03 animation gates; B10 templates/slots including slot-one fallback; B04 quake/presentation requests through B03; B08 treasure pickup; B18 sounds; B13 progression. Shared surviving-hit flash interval is78 hex; separate boss-local delays remain local.

**Proposed interface and tests.** `port/src/actors/bosses` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/actors/bosses/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B11.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove each encounter local dispatch table, path node/preset limits, opening/attack/recovery/defeat, quake control effects on player/display, wave allocation/countdown, HP/invulnerability acceptance and full-slot treasure fallback. Include all five land routes through spawn-consumer evidence. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b11_bosses` complete; sole editor relinquished. Full adjacent `actors/bosses/{mod.rs,contracts.rs,trace.md}`,81 substantive contracts, whole-area handoff `planning/contracts/B11.md`;44region/62unit/227caller/2652state-reference consumer index `B11-consumers.json`. All7selectors/all5actualattackphases,finitecatalogs/all5naturalmainspawnroutes,fullsharedhit/treasure including originalB10metadata1019..1023,normal/boundary/alternate/fullpool/RNG/animation/audio/immediatealias effects covered. Rust1.90locked finalcompile/list0,zero warnings,844total (81B11+761retainedbehavior+2infra); B11area101:0pass/81explicitpending failures/0ignored;2changedB10focusedcases101explicitpending; game::tests0:2pass. Exactcommands/hashes/names `checks/contracts/B11/results.json`; supersededwarnings/7fixtureborrowerrors/earlier81records retained. Corrections: namedphase labelsattachprecedingRTS butactualtableentersfullbodies;wave6children/lastchild0700alias/equaltimepairs;attackLDA5Enoanimationrewrite;slot1treasurealias;canonical06D0local_aux2 vs06E0sequence_cursor/06F0boss_protection withB10ordinaryclearcorrected. `B11-first-slot-random-increment` explicitly retains B06/B13actualplayer-returnproducer integration; bothpossibleinputscontracted without claiming closure. B08freshB18reactionqueue, B05motion, B03commit, B04camera/cache,B10allocation/dispatch/presets,B13HP/impact/mode,B14reward,B17finalphases remain owner dependencies. No newarea/gameplay/emulator/QA/toolkit/check-in/publication; implementation pending.

## B12 — Title options and land selection

**Scope and observations.** Own title animation/menu, three-choice option wrap/default1 and marker positions, START-to-selection, five selectable lands, directional skip of completed stages and A confirm. Selection uses source layout5 while preserving ordinary selector. Alternate selection presentations transfer value, acknowledge A and trigger completion actor; all five completion bits route to stage1 late setup. No added player level-select/debug menu.

**Source entry points and data.**
- [`InitializeTitle`](reference/source/game.asm#L13334) — line 13334, physical `$1D4E6..$1D544`, mapper bank 7.
- [`PrepareTitleActors`](reference/source/game.asm#L13375) — line 13375, physical `$1D544..$1D5B3`, mapper bank 7.
- [`TitleInputLoop`](reference/source/game.asm#L13430) — line 13430, physical `$1D5B3..$1D5BF`, mapper bank 7.
- [`ChangeTitleOption`](reference/source/game.asm#L13444) — line 13444, physical `$1D5BF..$1D5DE`, mapper bank 7.
- [`UpdateTitleFrame_D5D7`](reference/source/game.asm#L13473) — line 13473, physical `$1D5DE..$1D631`, mapper bank 7.
- [`CopyTitleOverlay`](reference/source/game.asm#L13499) — line 13499, physical `$1D5DE..$1D631`, mapper bank 7.
- [`StageBoundaryPositionData`](reference/source/game.asm#L13517) — line 13517, physical `$1D631..$1D640`, mapper bank 7.
- [`InitializeLandSelect`](reference/source/game.asm#L13527) — line 13527, physical `$1D640..$1D65A`, mapper bank 7.
- [`LoadLandSelectLayout`](reference/source/game.asm#L13546) — line 13546, physical `$1D65A..$1D660`, mapper bank 7.
- [`CreateLandSelectActors`](reference/source/game.asm#L13556) — line 13556, physical `$1D660..$1D6B4`, mapper bank 7.
- [`LandSelectAlternateState`](reference/source/game.asm#L13601) — line 13601, physical `$1D6B4..$1D6B9`, mapper bank 7.
- [`StageSelectCompletionPresentation`](reference/source/game.asm#L13611) — line 13611, physical `$1D6B9..$1D6D1`, mapper bank 7.
- [`PlayAllStagesCompletionScene`](reference/source/game.asm#L13637) — line 13637, physical `$1D6D6..$1D6FF`, mapper bank 7.
- [`FindAvailableLand`](reference/source/game.asm#L13662) — line 13662, physical `$1D6FF..$1D708`, mapper bank 7.
- [`PositionLandSelection`](reference/source/game.asm#L13683) — line 13683, physical `$1D70B..$1D752`, mapper bank 7.
- [`WrapLandSelection`](reference/source/game.asm#L13724) — line 13724, physical `$1D752..$1D756`, mapper bank 7.
- [`SkipCompletedLand`](reference/source/game.asm#L13734) — line 13734, physical `$1D756..$1D760`, mapper bank 7.
- [`LandSelectLoopAndConfirm`](reference/source/game.asm#L13755) — line 13755, physical `$1D765..$1D7E1`, mapper bank 7.
- [`StageProgressionParameterData`](reference/source/game.asm#L13921) — line 13921, physical `$1D8B6..$1D8E9`, mapper bank 7.

**Dependencies, state and order.** B02/B03 assets/display; B14 displayed-value transfer and option-dependent reward semantics; B16 completion actor; B13 stage initialization; B01 edge input/RNG. Five selectable originals do not cap added content catalog.

**Proposed interface and tests.** `port/src/selection` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/selection/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B12.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove option cycling both directions/input masks, START priority, no-input animation, skip/wrap all completion masks, confirmation and alternate/completed route. Establish displayed option descriptions from source assets before replacing provisional labels. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b12_selection` complete. Native pending interfaces/43 contracts/independent detailed trace `port/src/selection/{mod.rs,contracts.rs,trace.md}`; whole-area handoff and exact names `planning/contracts/B12.md`; preserved22 related-region/23unit/40label/70label-reference/710state-reference index `planning/contracts/B12-consumers.json`. Rust1.90 locked no-run/list exit0 warning-free;887 tests discovered (43 B12+842 prior behavior+2 infrastructure). B12 exit101:0 pass/43 explicit unimplemented failures/0 ignored;2 reconciled B14 cases each explicit pending; infrastructure2/2 pass. Exact commands/logs/source/native hashes `checks/contracts/B12/results.json`; early compile/42-case records preserved. Source corrections: actual title entry D4E6/physical1D4F6 follows snap helper; title labels EASY/NORMAL/DIFFICULT proven by script2/CHR; no input reset or demo timeout; equality carry adds1 to title RNG; initial land scan has8 masks, no five-choice cap; confirmation calls terrain builder twice. Explicit continuations prevent actor initialization ahead of outgoing fades. B14 blank/converted/direct-message producer facts survive acknowledgment and commit exact primary pending0/FF/5D. `B12-available-land-entry-precondition` retains B13 natural alternate-selector/caller integration; no invented sixth land or arbitrary FF equivalence. B01/B03 scheduling/display, B02/B04 loading, B10 actor clearing, B13 progression/runtime, B14 transfer/options and B16 completion actor dependencies remain visible. No new cohesive area discovered. Sole editor relinquished; implementation pending.

## B13 — Gameplay orchestration, stage progression, reload and game over

**Scope and observations.** Own run progression, stage setup/defaults, pause/status selector, gameplay freeze gates, late horizontal transition, ordinary/late room requests, death/reload/checkpoint recipes, life-loss variants and game-over reset. Source continuation gates must all be resolved, including $A4/$AB controls and stage1 exceptions. Checkpoints here mean original game respawn records, distinct from debug replay.

**Source entry points and data.**
- [`CheckStageBoundary`](reference/source/game.asm#L12473) — line 12473, physical `$1CFA3..$1CFAC`, mapper bank 7.
- [`TriggerStageBoundary`](reference/source/game.asm#L12512) — line 12512, physical `$1CFC5..$1CFE4`, mapper bank 7.
- [`EnterLateStageHorizontalTransition`](reference/source/game.asm#L12536) — line 12536, physical `$1CFE4..$1D05D`, mapper bank 7.
- [`LateStageTransitionTick`](reference/source/game.asm#L12558) — line 12558, physical `$1CFE4..$1D05D`, mapper bank 7.
- [`StageTransitionParameters_1D05DBytes`](reference/source/game.asm#L12605) — line 12605, physical `$1D05D..$1D078`, mapper bank 7.
- [`InitializeStageRuntime`](reference/source/game.asm#L13822) — line 13822, physical `$1D7E4..$1D8B6`, mapper bank 7.
- [`LoadStagePresentation`](reference/source/game.asm#L14104) — line 14104, physical `$1D9DA..$1DA20`, mapper bank 7.
- [`StagePaletteAndOverlayData`](reference/source/game.asm#L14145) — line 14145, physical `$1DA20..$1DB46`, mapper bank 7.
- [`GameplayLoopEntry`](reference/source/game.asm#L14155) — line 14155, physical `$1DB46..$1DB4A`, mapper bank 7.
- [`GameplayPauseGate`](reference/source/game.asm#L14181) — line 14181, physical `$1DB50..$1DB5C`, mapper bank 7.
- [`UpdateGameplaySimulation`](reference/source/game.asm#L14236) — line 14236, physical `$1DB7C..$1DB92`, mapper bank 7.
- [`CheckGameplayTransitions`](reference/source/game.asm#L14260) — line 14260, physical `$1DB95..$1DBA3`, mapper bank 7.
- [`ComposeAndStreamGameplay`](reference/source/game.asm#L14374) — line 14374, physical `$1DBD5..$1DC0E`, mapper bank 7.
- [`FinishGameplayFrame`](reference/source/game.asm#L14442) — line 14442, physical `$1DC29..$1DC3C`, mapper bank 7.
- [`ReloadStageFromTransitionRequest`](reference/source/game.asm#L14466) — line 14466, physical `$1DC4E..$1DD3F`, mapper bank 7.
- [`InitializeStageReloadState`](reference/source/game.asm#L14486) — line 14486, physical `$1DC4E..$1DD3F`, mapper bank 7.
- [`LoadCheckpointInitialization`](reference/source/game.asm#L14633) — line 14633, physical `$1DD5D..$1DDD1`, mapper bank 7.
- [`CheckpointInitializationRecords`](reference/source/game.asm#L14697) — line 14697, physical `$1DDD1..$1DE7C`, mapper bank 7.
- [`PlayGameOverAndReset`](reference/source/game.asm#L14779) — line 14779, physical `$1DEB3..$1DEF7`, mapper bank 7.
- [`BeginSelectionTransition`](reference/source/game.asm#L14817) — line 14817, physical `$1DF18..$1DF26`, mapper bank 7.
- [`ReturnToLandSelect`](reference/source/game.asm#L14848) — line 14848, physical `$1DF34..$1DF37`, mapper bank 7.

**Dependencies, state and order.** Coordinates B06 player then B10 actors then B14 timer; B04 camera/boundary; deferred reload/status/UI/special-room effects; B03 composition; B10 spawning; queued player reaction; secondary upload; B01 RNG/yield. B17 ending bypass and race latch gate simulation/spawns.

**Proposed interface and tests.** `port/src/progression` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/progression/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B13.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove source order including pause and player0F exception, all freeze combinations, positive/negative transition requests, stage presets, checkpoint/fade/timer/audio restore, life depletion/game-over/title and all completion/key/late-stage branches. No battery SRAM exists (header bit clear), so default Game save hooks remain correct. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Whole-area B13 worker handoff complete (contracts only). Independent
trace: `port/src/progression/trace.md`; pending interfaces/contracts:
`port/src/progression/{mod.rs,contracts.rs}`; full46related-region/55unit consumer
index: `planning/contracts/B13-consumers.json`; handoff: `planning/contracts/B13.md`.
Rust1.90locked no-run exit0 warningfree6.91s; native discovery963total retains887
prior and adds76B13; area exit101:76explicit unimplemented failures,0ignored;
`game::tests`2/2pass. Evidence/hashes `checks/contracts/B13/results.json`.
Contracts cover full stage recipes, all gameplay gates/live owner ordering and
suspensions, raw fallback reads, boundary/horizontal/reload/audio/life/checkpoint/
gameover/selection continuations and canonical initial16/reload12 reset extents.
B12 D79B confirmation/two ordered terrain builds remain shared. B01 frame mapping,
B11 actual B06 first-slot random increment producer and B12 natural all-complete
no-signal entry remain explicitly pending with concrete integration obligations;
no false default, five-land cap or invented menu. B15/B16/B17/B18 owner continuation
integration remains pending. No new behavior area; sole editor relinquished after
handoff. Stage4 implementation and QA remain unstarted.

## B14 — Status, health/lives, value arithmetic and timers

**Scope and observations.** Own health/lives/status and timer quantities, decimal display-code arithmetic, blank24 normalization, saturation, dirty flags/HUD streams, countdown divider and warnings, timer-to-reward conversion and reward-to-total ten-unit sound/frame batches. Own title option2 policy affecting reward animations and damage/cost consumers; do not assume option labels. Original digit operations may preserve blanks/control codes, so normalized numeric state needs a proven display adapter.

**Source entry points and data.**
- [`Bank05_UpdateStatusStreams`](reference/source/game.asm#L2959) — line 2959, physical `$17590..$1762C`, mapper bank 5.
- [`Bank05_ExpandStatusStream`](reference/source/game.asm#L2971) — line 2971, physical `$17590..$1762C`, mapper bank 5.
- [`StatusStreamRecords_1762CBytes`](reference/source/game.asm#L3059) — line 3059, physical `$1762C..$17770`, mapper bank 5.
- [`SelectStatusMessage`](reference/source/game.asm#L3131) — line 3131, physical `$17810..$17818`, mapper bank 5.
- [`BuildStatusUpload`](reference/source/game.asm#L3159) — line 3159, physical `$1782C..$17848`, mapper bank 5.
- [`StatusMessageRecords_1787CBytes`](reference/source/game.asm#L3223) — line 3223, physical `$1787C..$17AC0`, mapper bank 5.
- [`TransferDisplayedValueToTotal`](reference/source/game.asm#L13931) — line 13931, physical `$1D8E9..$1D8FD`, mapper bank 7.
- [`TransferValueDigitBatch`](reference/source/game.asm#L13997) — line 13997, physical `$1D905..$1D9DA`, mapper bank 7.
- [`QueueStatusDigits`](reference/source/game.asm#L14734) — line 14734, physical `$1DE90..$1DEB3`, mapper bank 7.
- [`RefreshStatusWhenIdle`](reference/source/game.asm#L14707) — line 14707, physical `$1DE7C..$1DE8E`, mapper bank 7.
- [`CountdownStateGate`](reference/source/game.asm#L16431) — line 16431, physical `$1E29C..$1E2AD`, mapper bank 7.
- [`TickStageCountdown`](reference/source/game.asm#L16458) — line 16458, physical `$1E2B6..$1E2E7`, mapper bank 7.
- [`CountdownWarning`](reference/source/game.asm#L16507) — line 16507, physical `$1E2F6..$1E2FB`, mapper bank 7.
- [`ApplyOverlayValueCost`](reference/source/game.asm#L16540) — line 16540, physical `$1E30E..$1E364`, mapper bank 7.
- [`ConvertStageTimerToReward`](reference/source/game.asm#L19500) — line 19500, physical `$1EF72..$1EF74`, mapper bank 7.
- [`DecrementTimerDisplay`](reference/source/game.asm#L19539) — line 19539, physical `$1EF7B..$1EFF2`, mapper bank 7.
- [`AddConvertedTimerReward`](reference/source/game.asm#L19561) — line 19561, physical `$1EF7B..$1EFF2`, mapper bank 7.
- [`ClearStatusOverlayFlags`](reference/source/game.asm#L21565) — line 21565, physical `$1F822..$1F832`, mapper bank 7.
- [`ApplyTitleOptionRule`](reference/source/game.asm#L21584) — line 21584, physical `$1F832..$1F838`, mapper bank 7.
- [`ApplyOptionRewardAnimation`](reference/source/game.asm#L21595) — line 21595, physical `$1F838..$1F84E`, mapper bank 7.
- [`SaturateRewardDigits`](reference/source/game.asm#L21623) — line 21623, physical `$1F84F..$1FA00`, mapper bank 7.
- [`SaturateTotalDigits`](reference/source/game.asm#L21633) — line 21633, physical `$1F84F..$1FA00`, mapper bank 7.
- [`PrepareValueTransferDestinations`](reference/source/game.asm#L23162) — line 23162, physical `$1FFD7..$1FFD9`, mapper bank 7.

**Dependencies, state and order.** B08 submits rewards; B07 theft; B15 paid dialogue transactional subtraction; B06 damage/special timers; B13 lifecycle/reset/freeze; B03 queues and overlays; B18 sound commands. B17 ending chooses variations from progression and displayed value.

**Proposed interface and tests.** `port/src/status` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/status/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B14.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove blank/zero/borrow/carry/overflow and saturation across every digit width, no-partial-payment underflow, timer0/divider wrap/expiry/warning, state exemptions and freeze. Trace each status stream/message table entry and original display defaults. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b14_status` complete; full adjacent `port/src/status/trace.md`,61 native contracts and minimal pending declarations; `planning/contracts/B14-consumers.json` covers45 primary/related regions and callers. Whole handoff/test names `planning/contracts/B14.md`. Locked Rust1.90 no-run/list exit0:357 total (61 B14+294 prior behavior+2 infrastructure); area exit101:0 passed/61 explicit unimplemented failures/0 ignored; game::tests exit0:2/2 pass. Evidence checks/contracts/B14-results.json and sibling logs. Corrections: theft upper3 reward digits; payment upper2 reward/total; conversion upper2 timer after units clear; transfer queue-copy source separate from persistent reward, exact-tenth exhaustion silent. Status returns prepared queues; B01/B03 own committed visibility; B10 actor state remains immediate/canonical. B06 special/protection timers, B07/B08 triggers, B13 lifecycle, B15 actor aftermath, B16/B17 scenes/ending consumers and B18 audio scheduling remain caller dependencies. B14-runtime-frame-author supersedes unsupported B03 frame-author claim, retaining aliases and assigning B16 reachability. No new area; sole editor relinquished; implementation pending.

## B15 — Dialogue, progression contacts, choices and escorts

**Scope and observations.** Own eighteen message selectors/encoded strings (AE end, FF line/page,24 blank), glyph cadence/row bounds, page scroll, A/B acceptance and message-specific outcomes. Include hit-activated and contact dialogue actors, prerequisites, per-message bits, reveal/paid reward/choice/special-room requests, escort/companion/terrain opening and returning controls. Message0B constructs race terrain and clears race intro after completion; route implementation remains B17.

**Source entry points and data.**
- [`DialogueTextPointers_13898Bytes`](reference/source/game.asm#L2137) — line 2137, physical `$13898..$138BC`, mapper bank 4.
- [`DialogueEncodedStrings_138BCBytes`](reference/source/game.asm#L2143) — line 2143, physical `$138BC..$13C59`, mapper bank 4.
- [`Bank06_ProgressionContactSelector`](reference/source/game.asm#L5825) — line 5825, physical `$19075..$19225`, mapper bank 6.
- [`RunPlayerDialogue`](reference/source/game.asm#L18459) — line 18459, physical `$1ED15..$1ED17`, mapper bank 7.
- [`PrepareDialogueComposition`](reference/source/game.asm#L18596) — line 18596, physical `$1ED49..$1ED4B`, mapper bank 7.
- [`WaitForDialogueInput`](reference/source/game.asm#L18689) — line 18689, physical `$1ED70..$1ED72`, mapper bank 7.
- [`ApplyDialogueOutcome`](reference/source/game.asm#L18888) — line 18888, physical `$1EDBD..$1EDBF`, mapper bank 7.
- [`DialogueRevealActor`](reference/source/game.asm#L18990) — line 18990, physical `$1EDDC..$1EEBE`, mapper bank 7.
- [`DialogueAcceptStageChoice`](reference/source/game.asm#L19014) — line 19014, physical `$1EDDC..$1EEBE`, mapper bank 7.
- [`DialoguePaidActorReward`](reference/source/game.asm#L19035) — line 19035, physical `$1EDDC..$1EEBE`, mapper bank 7.
- [`ScrollDialogueRows`](reference/source/game.asm#L19096) — line 19096, physical `$1EDDC..$1EEBE`, mapper bank 7.
- [`RenderDialogueLine`](reference/source/game.asm#L19120) — line 19120, physical `$1EEBE..$1EEC0`, mapper bank 7.
- [`NextDialogueGlyph`](reference/source/game.asm#L19189) — line 19189, physical `$1EED7..$1EED9`, mapper bank 7.
- [`FinishDialogueMessage`](reference/source/game.asm#L19405) — line 19405, physical `$1EF4C..$1EF4E`, mapper bank 7.
- [`UpdateHitActivatedDialogueActor`](reference/source/game.asm#L22575) — line 22575, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`DialogueActorArc`](reference/source/game.asm#L22612) — line 22612, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`UpdateProgressionEscort`](reference/source/game.asm#L22653) — line 22653, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`EscortApproachAndCompanion`](reference/source/game.asm#L22685) — line 22685, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`UpdateRightwardTransitionActor`](reference/source/game.asm#L22750) — line 22750, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`UpdateDialogueContactActor`](reference/source/game.asm#L22827) — line 22827, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`UpdateConditionalContactMarker`](reference/source/game.asm#L22877) — line 22877, physical `$1FC0F..$1FEE6`, mapper bank 7.

**Dependencies, state and order.** B06 state0F enters dialogue; B03 text and split presentation; B14 transactional cost; B04 changed cells/burst; B10 actor slots/suppression; B13 special-room/selection requests; B17 race handoff. Message-specific control fields cannot be guessed from wording alone.

**Proposed interface and tests.** `port/src/dialogue` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/dialogue/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B15.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove all eighteen message pointer bounds/control characters, reveal timing/input waits, each outcome including messages03/06/0A/0B/0C/0F, reject B and underfunded payment, contact protection/used phases, escort allocation and exact progression flag writers/readers. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker pending; trace evidence pending; tests pending/not yet authored; test compile/run not executed; implementation pending.

## B16 — Special scenes, departures and scene-actor presentation

**Scope and observations.** Own six scene-actor handlers distinct from gameplay handlers, scene initialization records/scripts, completion animation6F..73/palette flashes, special-room setup/delay/presentation and linked departure/timed reward spawner sequence. BuildSceneScreen prepares ending scene content on B17 request, including terrain, actor records, palettes and split display; progression sequencing belongs B17.

**Source entry points and data.**
- [`Bank04_PlayCompletionActor`](reference/source/game.asm#L2215) — line 2215, physical `$13E10..$13EBB`, mapper bank 4.
- [`Bank04_CompletionActorMotion`](reference/source/game.asm#L2242) — line 2242, physical `$13E10..$13EBB`, mapper bank 4.
- [`Bank04_FlashCompletionPalette`](reference/source/game.asm#L2278) — line 2278, physical `$13E10..$13EBB`, mapper bank 4.
- [`Bank05_InitializeSceneActors`](reference/source/game.asm#L2750) — line 2750, physical `$17390..$173DC`, mapper bank 5.
- [`SceneActorInitializationLists_173DCBytes`](reference/source/game.asm#L2794) — line 2794, physical `$173DC..$17410`, mapper bank 5.
- [`Bank05_UpdateSceneActor`](reference/source/game.asm#L2802) — line 2802, physical `$17410..$1751D`, mapper bank 5.
- [`Bank05_SceneBounce`](reference/source/game.asm#L2812) — line 2812, physical `$17410..$1751D`, mapper bank 5.
- [`Bank05_SceneWalkIn`](reference/source/game.asm#L2855) — line 2855, physical `$17410..$1751D`, mapper bank 5.
- [`Bank05_SceneDelay`](reference/source/game.asm#L2875) — line 2875, physical `$17410..$1751D`, mapper bank 5.
- [`Bank05_SceneMoveToMark`](reference/source/game.asm#L2897) — line 2897, physical `$17410..$1751D`, mapper bank 5.
- [`Bank05_ScenePaletteTransition`](reference/source/game.asm#L2922) — line 2922, physical `$17410..$1751D`, mapper bank 5.
- [`SceneHandlerPointers_1751DBytes`](reference/source/game.asm#L2946) — line 2946, physical `$1751D..$17529`, mapper bank 5.
- [`InitializeSpecialRoom`](reference/source/game.asm#L14857) — line 14857, physical `$1DF37..$1DFDF`, mapper bank 7.
- [`SpecialRoomActorInitializationData`](reference/source/game.asm#L14941) — line 14941, physical `$1DFDF..$1DFFD`, mapper bank 7.
- [`BuildSceneScreen`](reference/source/game.asm#L15246) — line 15246, physical `$1E064..$1E067`, mapper bank 7.
- [`BuildSceneTerrainColumns`](reference/source/game.asm#L15418) — line 15418, physical `$1E0A5..$1E0A7`, mapper bank 7.
- [`RunSceneScreenScripts`](reference/source/game.asm#L15572) — line 15572, physical `$1E0DA..$1E0DD`, mapper bank 7.
- [`ScenePaletteAndParameterData`](reference/source/game.asm#L15806) — line 15806, physical `$1E12B..$1E1BF`, mapper bank 7.
- [`RenderActorsAndReturnBankFour`](reference/source/game.asm#L21643) — line 21643, physical `$1F84F..$1FA00`, mapper bank 7.
- [`UpdatePlayerLinkedDeparture`](reference/source/game.asm#L21654) — line 21654, physical `$1F84F..$1FA00`, mapper bank 7.
- [`UpdateTimedDepartureSpawner`](reference/source/game.asm#L21719) — line 21719, physical `$1F84F..$1FA00`, mapper bank 7.
- [`SpawnDepartureReward`](reference/source/game.asm#L21802) — line 21802, physical `$1F84F..$1FA00`, mapper bank 7.

**Dependencies, state and order.** B03 rendering/animation/script decode; B04 world builder; B10 allocation; B06 held player; B14 displayed-value gates/life loss; B12 completion route; B13 special-room return. Explicit bank-four return in source is a data-context boundary, not a need for native mapper emulation.

**Proposed interface and tests.** `port/src/scenes` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/scenes/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B16.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove six scene targets, scene-list termination/skip records, bounce/walk/delay/palette phases, completion animation end, special-room selector/camera and every departure/spawn/exit/life-loss path. Preserve intentional waits and score-conditioned departure behavior. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker `/root/behavior_contracts/b16_scenes` complete; sole editor relinquished. Full adjacent `scenes/{mod.rs,contracts.rs,trace.md}`,86 substantive native contracts; handoff/test names `planning/contracts/B16.md`,104related/103primaryregion/121unit/956state-reference/124animation-writer index `B16-consumers.json`. Rust1.90 locked finalcompile/list0 warningfree;1154discovered=1067retained+86B16+1B02. Area101:0pass/86explicitpendingfailures/0ignored; B02newjoin/B03allframes/B10empty5..8/B06life-loss each1explicitpending; infrastructure2/2pass. Exact commands/hashes/history `checks/contracts/B16/results.json`. Actual6targets4/5settled,5completionposes/latch5E,bytecursorflip,AE22explicitspawn7/8,source3effectivescripts,all departures/fullpool/RNG and immediate EBB4 contracted. B02canonical scene context binds layout9 with terrain2/1/CHR9/10/no spawn9. B03canonical secondary backing/runtimealias projection sourcezero/tenth+eleventh changed records; earlier informal6/7threshold corrected. `B14-runtime-frame-author` retains full natural animation0A/8C/8D/tenmutation/highcursorcache reachability uncertainty with actualwriters, no false absence/exclusion/statusframes. Existing B06/B15/B13 owners retained. All implementation pending; no new area, gameplay/QA/toolkit/check-in/publication. B17/B18 remain pending.

## B17 — Final boss, rival race, ending variants and terminal screen

**Scope and observations.** Own final handler3A teleport/opening/attack and3B defeat, slot14 attack, shared HP5 integration, B4-tick post-animation pause, dialogue0B/race latch, race terrain construction, encoded multirow rival climbing, loss dialogue and player goal win/F0 ending delay. Own EndingSequenceLatch, three initial ending variations, later scenes/split scroll, ten sixty-tick pauses and permanent yielding End loop. Progress bits7F plus displayed-value tests select variants; exact wealth condition requires tracing display codes.

**Source entry points and data.**
- [`BuildFinalRaceTerrain`](reference/source/game.asm#L14951) — line 14951, physical `$1DFFD..$1E000`, mapper bank 7.
- [`ContinueFinalRaceTerrainSetup`](reference/source/game.asm#L15008) — line 15008, physical `$1E010..$1E013`, mapper bank 7.
- [`BuildFinalRaceTerrainColumns`](reference/source/game.asm#L15065) — line 15065, physical `$1E024..$1E026`, mapper bank 7.
- [`FinishRaceBackgroundUpload`](reference/source/game.asm#L15135) — line 15135, physical `$1E03D..$1E03F`, mapper bank 7.
- [`UpdateEndingSequence`](reference/source/game.asm#L15816) — line 15816, physical `$1E1BF..$1E1C1`, mapper bank 7.
- [`ContinueEndingScene`](reference/source/game.asm#L16022) — line 16022, physical `$1E209..$1E20C`, mapper bank 7.
- [`ScrollEndingSplit`](reference/source/game.asm#L16092) — line 16092, physical `$1E222..$1E225`, mapper bank 7.
- [`AdvanceEndingAfterPause`](reference/source/game.asm#L16326) — line 16326, physical `$1E279..$1E27B`, mapper bank 7.
- [`EndingTerminalLoop`](reference/source/game.asm#L16416) — line 16416, physical `$1E296..$1E299`, mapper bank 7.
- [`UpdateFinalBoss`](reference/source/game.asm#L21894) — line 21894, physical `$1FA4F..$1FA52`, mapper bank 7.
- [`FinalBossTeleportDelay`](reference/source/game.asm#L22060) — line 22060, physical `$1FA92..$1FA95`, mapper bank 7.
- [`FinalBossAttackWindow`](reference/source/game.asm#L22122) — line 22122, physical `$1FAD5..$1FB64`, mapper bank 7.
- [`FinalBossCloseAttackWindow`](reference/source/game.asm#L22164) — line 22164, physical `$1FAD5..$1FB64`, mapper bank 7.
- [`ActorSpawnPositionAndFacingData`](reference/source/game.asm#L22190) — line 22190, physical `$1FB64..$1FB78`, mapper bank 7.
- [`UpdateRaceClimber`](reference/source/game.asm#L22200) — line 22200, physical `$1FB78..$1FB7B`, mapper bank 7.
- [`FinalBossDefeatCountdown`](reference/source/game.asm#L22425) — line 22425, physical `$1FBD9..$1FBDC`, mapper bank 7.
- [`UpdateFinalRaceGoal`](reference/source/game.asm#L22780) — line 22780, physical `$1FC0F..$1FEE6`, mapper bank 7.
- [`FinalGoalEndingDelay`](reference/source/game.asm#L22817) — line 22817, physical `$1FC0F..$1FEE6`, mapper bank 7.

**Dependencies, state and order.** B10 late progression spawn override; B11 shared hit; B15 race dialogue; B16 scene content/handlers; B14 value transfer; B03 split effects; B04 race geometry; B06 climbing; B13 ending bypasses regular sim/spawns. Do not replace natural progression with artificial archive test setup.

**Proposed interface and tests.** `port/src/finale` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/finale/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B17.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Prove teleport positions/delay/vulnerability and repeated hits, attack slot replacement, defeat frame/B4 wait, rival row carry and leader/loss predicates, goal race arbitration/F0 delay, every ending branch/scene and terminal nonrestart behavior. Archive artificial traces guide source questions but do not prove natural full-game reachability. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker pending; trace evidence pending; tests pending/not yet authored; test compile/run not executed; implementation pending.

## B18 — Music/effects sequence engine and synthesis

**Scope and observations.** Own eight-slot command ring sentinel88, full-queue drop, 56 command headers plus FC..FF controls, high/low priority arbitration and four 31-byte channels. Implement primary/secondary interpreters, masks/cursors/loops/durations, tempo/fade, presets/envelope/pitch modulation and native APU register schedules. Header priority and masks determine song/effect interaction; exact musical names remain provisional. Mixed stream/preset/bitmap tail requires consumer-aware decode, not all bytes as audio.

**Source entry points and data.**
- [`AudioFrameEntry`](reference/source/game.asm#L183) — line 183, physical `$0C010..$0C017`, mapper bank 3.
- [`AudioCommandDispatch`](reference/source/game.asm#L187) — line 187, physical `$0C010..$0C017`, mapper bank 3.
- [`AudioSelectStream`](reference/source/game.asm#L257) — line 257, physical `$0C03F..$0C05E`, mapper bank 3.
- [`AudioInstallPrimary`](reference/source/game.asm#L288) — line 288, physical `$0C05F..$0C11F`, mapper bank 3.
- [`AudioInstallSecondary`](reference/source/game.asm#L350) — line 350, physical `$0C05F..$0C11F`, mapper bank 3.
- [`AudioStopHighPriorityStream`](reference/source/game.asm#L442) — line 442, physical `$0C138..$0C15F`, mapper bank 3.
- [`AudioStopLowPriorityStream`](reference/source/game.asm#L520) — line 520, physical `$0C1A2..$0C1B9`, mapper bank 3.
- [`AudioBeginFrame`](reference/source/game.asm#L662) — line 662, physical `$0C21D..$0C260`, mapper bank 3.
- [`AudioVisitChannels`](reference/source/game.asm#L685) — line 685, physical `$0C261..$0C290`, mapper bank 3.
- [`AudioAdvanceFade`](reference/source/game.asm#L777) — line 777, physical `$0C2E2..$0C300`, mapper bank 3.
- [`AudioApplyEnvelope`](reference/source/game.asm#L866) — line 866, physical `$0C34E..$0C522`, mapper bank 3.
- [`AudioIntegratePitch`](reference/source/game.asm#L995) — line 995, physical `$0C34E..$0C522`, mapper bank 3.
- [`AudioPitchOscillator`](reference/source/game.asm#L1022) — line 1022, physical `$0C34E..$0C522`, mapper bank 3.
- [`AudioWritePeriod`](reference/source/game.asm#L1102) — line 1102, physical `$0C34E..$0C522`, mapper bank 3.
- [`AudioInlineDispatch`](reference/source/game.asm#L1210) — line 1210, physical `$0C524..$0C5D3`, mapper bank 3.
- [`AudioSecondaryCommandWords_0C5D3Bytes`](reference/source/game.asm#L1285) — line 1285, physical `$0C5D3..$0C5E1`, mapper bank 3.
- [`AudioPrimaryCommandWords_0C7CBBytes`](reference/source/game.asm#L1662) — line 1662, physical `$0C7CB..$0C7DF`, mapper bank 3.
- [`AudioPrimaryDecode`](reference/source/game.asm#L1519) — line 1519, physical `$0C6CB..$0C756`, mapper bank 3.
- [`AudioSecondaryDecode`](reference/source/game.asm#L1253) — line 1253, physical `$0C524..$0C5D3`, mapper bank 3.
- [`AudioDurationAndPeriodData_0C97BBytes`](reference/source/game.asm#L1986) — line 1986, physical `$0C97B..$0CA50`, mapper bank 3.
- [`AudioCommandHeaderPointers_0CA50Bytes`](reference/source/game.asm#L1992) — line 1992, physical `$0CA50..$0CAC0`, mapper bank 3.
- [`AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes`](reference/source/game.asm#L1998) — line 1998, physical `$0CAC0..$0F110`, mapper bank 3.
- [`QueueAudioCommand`](reference/source/game.asm#L21190) — line 21190, physical `$1F762..$1F77D`, mapper bank 7.
- [`QueueStageAudioCommand`](reference/source/game.asm#L21186) — line 21186, physical `$1F762..$1F77D`, mapper bank 7.

**Dependencies, state and order.** B01 NMI drains commands then advances driver once/frame even when scene/foreground paused; B02 catalog supplies original streams. B03 consumes any graphics payload in mixed bank via B02. All other areas enqueue semantic source IDs. Synthesis sample calls advance APU time only, never command interpreter/game ticks.

**Proposed interface and tests.** `port/src/audio` owns this area with typed state and small request/result interfaces described in architecture.md. Contract tests: `port/src/audio/contracts.rs` or its sibling single-file equivalent; evidence: `planning/contracts/B18.md`. Ledger links: filter `primary_area`/`related_areas`/unit `area` in ledger-area-map.json; no duplicate state owner is authorized.

**Worker scope and unresolved questions.** Trace every primary/secondary dispatch entry and FC..FF, pointer/page carry, finite loop/stop/resume, duration rounding, channel types and cached period writes; contract source-unknown-00c522 physicalC522..C524 and its reaching consumers. Prove queue wrap/full/order and deterministic audio chunking using independent MIT APU, without claiming device fidelity. Planning interpretations above require detailed source traces; original summaries are leads, not proof. No runtime evidence newly gathered in stage 2. Retained archive evidence is supplemental and must be cited specifically by the worker, not generalized to whole-game completion.

**Status.** Worker pending; trace evidence pending; tests pending/not yet authored; test compile/run not executed; implementation pending.

B06 prior pending status is superseded by the active `/root/behavior_contracts/b06_player` worker. B03 status ownership qualification: B14 supplies encoded status stream decoding/overlays, but runtime frame authors at bank5 B580/B700 remain caller contracts for B16; see `B14-runtime-frame-author`, preserving source aliases.

B15 current worker `/root/behavior_contracts/b15_dialogue` active: whole-area source tracing and pending semantic contracts in progress; not a completed handoff.

**B15 whole-area handoff (supersedes active entries).** Worker
`/root/behavior_contracts/b15_dialogue` completed100native substantive source
contracts, no behavior implementation. Full adjacent trace/interfaces/tests
`port/src/dialogue/{mod.rs,contracts.rs,trace.md}`; exact names/dependencies/results
`planning/contracts/B15.md`;148primaryregions/167units/66labels/177labelrefs/1287
state references in `B15-consumers.json`. All18message streams and their actual
pointer bounds/glyphwaits/pages/terminal markers, sixhandler entries and every
meaningful contact/outcome/escort/allocation/wrap branch traced and contracted.
B03 canonical PreparedPrimary owns exactpendingbyte and encodedbytes, B15borrows;
B10 add_suppression handles actualB76A additiveentry. Four adjacent cases added,
prior963tests retained;1067total=100B15+965otherbehavior+2infra. Corrected final
Rust1.90locked compile0warningfree4.83s, discovery0;B15area101:100explicitpending,
0ignored. AdjacentB03two/B10two explicitpending;infra2/2pass. Sourcecachebase
correction05A1/05A9 relative0580→21/29 is tested independently before stub entry;
initial superseded100case records retained. Exact logs/native/source hashes
`checks/contracts/B15/results.json`. Existing first-slot-return/rawselection
uncertainties retain concrete B15source findings; no new area. Soleeditor
relinquished;B16/B17/B18 remain pending; stages4/5unstarted. No gameplay/QAclaim.


B17 worker `/root/behavior_contracts/b17_finale` active; sole editor. Independently tracing full final boss/race/ending area against frozen source; next: complete source/callee/table review then pending native contracts. Stage4/QA not started.


**B17 completed status (supersedes earlier pending/active entries).** Worker
`/root/behavior_contracts/b17_finale` relinquishes sole edit ownership. Adjacent
`finale/{mod.rs,contracts.rs,trace.md}`, whole-area handoff/test names
`planning/contracts/B17.md`, full238primary/239related/252unit/1940field-reference/
124animation-writer consumer index `B17-consumers.json`. Rust1.90locked finalcompile
andlist0 warningfree;1249discovered=1154retained+95B17. B17area101:0pass/95explicit
pending/0ignored;infra2/2pass;seven prior focused B10/B11/B13/B14/B15two/B16 each
one expected pending. Exact commands/logs/native/sourcehashes
`checks/contracts/B17/results.json`. Full3A/3B/3C/40 phases, naturalrace rows,
514-call leader/encodedborrow, simultaneouslosswin arbitration, raw7-code variants,
allscene/split64samepixel waits/10x60pauses/permanentEnd contracted. Canonical
B11hit/B15race/B06loss/B16content/B13suspension/B03split reused, ending fields
borrowed. Actual40entryFD9E/cache05EFcollision[6F] and same-call delay decrements
corrected. Runtimealias naturalco-occurrence/cacheoverlap stays explicit stage4
required uncertainty with all computedwriters indexed, no absence/exclusion.
No new area; B18remaining, implementation/QAunstarted. No toolkit/GUI/emulator/
QA/finalaudit/check-in/publication.

B18 current owner `/root/behavior_contracts/b18_audio` active, sole editor;
whole-area independent source trace and pending executable contracts in progress.

**B18 completed stage3 status (supersedes pending/active entries).** Worker
`/root/behavior_contracts/b18_audio` relinquishes sole edit ownership. Full adjacent
trace/interfaces97native source contracts `port/src/audio/{trace.md,mod.rs,contracts.rs}`;
whole-area handoff/all names `planning/contracts/B18.md`, complete58originalregion/
182label/898state-reference/63queue-driver-call/56header/100root9863node consumer
index `B18-consumers.json`. All10primary/7secondary actual ROMtable targets traced,
including before misleadinglabels; exact queue88hole/full/wrap/CMPreturn,
priorities/install/stop/tempo/fade/envelope/pitch/noise/cache/frame ordering and
actualdriverwrite→independentAPU nonzero/partition/rate contracts. Canonical B08queue
borrow replaces stale producer scalar; typed final Submission adapter retains B15
reveal06/CMP88 into nondefault B06/B13 actorincrement. NMIparameterfirst0 and
subsequentactualproducer chains tested; FE maskhighnibble temporary exact.
C522 originalID/range remains assessedpendingclassification: full positivebounded
natural audio graph/direct targets find no consumer; mixedCHR E910..F110 remains
B02owned, no exclusion. New explicit B18-secondary-reset-seed captures actual16
lowF predecessorselector dependency across channels/runtime state; no default19
or CPU/RAMemulation. No newarea discovered. Final Rust1.90locked compile0warningfree
8.01s (8.65s wall), discovery0/1346total=97B18+1247priorbehavior+2infra;
B18run101/97explicitunimplemented failures0ignored; infra2/2pass; changedB08focused
existingreactionstub1failure. Exact logs/native/source/loghashes
`checks/contracts/B18/results.json`; superseded93records/transientcompileerror
preserved. Implementation/stage4/QA not performed; coordinator next owns combined
18area compile/discovery/ownership reconciliation, not a behavior-pass claim.

## Stage 3 coordinator combined contract completion

All eighteen separate workers completed sequentially; no new cohesive areas were discovered. Current native discovery is 1,346 tests: 1,344 substantive behavior contracts and two infrastructure tests. Full combined run: two infrastructure passed, all 1,344 behavior contracts failed at explicit unimplemented stubs, zero ignored; no compile/fixture/assertion/hang failures. Compilation, discovery and format check passed. This is contract readiness, not implemented fidelity. Current per-area counts in the table include cross-area additions (B02 34, B03 55, B05 70, B08 99, B10 60). Historical area outcomes remain preserved.

Combined handoff/evidence: [planning/contracts/combined.md](planning/contracts/combined.md), [checks/contracts/combined/results.json](checks/contracts/combined/results.json). All original 1,302 region and 1,537 unit identities/extents remain intact and pending. Stage 4 must implement every contract and source area, resolve retained integration questions, and update implementation links/accounting before stage 5 QA.

## Optional modernization iteration008

Background preparation now decodes spans into an ordered update plan at asset
loading; gameplay advances an explicit session. Observable upload cadence, bank
selection, primary/secondary alias effects, final palette timing and suspension
policy remain source-defined. Source contracts and validation are linked in
[assets/trace.md](port/src/assets/trace.md) and
[modernize/changes.json](modernize/changes.json). This is bounded modernization
evidence and does not certify whole-game QA.

## Optional modernization iteration009

The stage timer now stores seconds and explicit expiration; normal time subtraction
and ten-second reward conversion no longer borrow through display glyphs. Entry/
reload/game-over imports and render/debug exports preserve the original codes.
Source warning/death timing and conversion waits remain unchanged. Nonnumeric
inputs preserve the faithful compatibility behavior, including its documented
BLANK subtraction limitation. See [status/trace.md](port/src/status/trace.md) and
[modernize/changes.json](modernize/changes.json) for contract scope and evidence.

## Modernization batch: native health (iteration010)

Health rules now operate on native availability/depletion, with sprite layout
confined to input/output boundaries. Capacity, sparse flags, alternating damage,
locked-slot restoration, extension, payment and player-death order are preserved.
All nine requested conversions are tracked in [modernize/batch.json](modernize/batch.json);
per-change source/build/test evidence is retained in [modernize/changes.json](modernize/changes.json).

## Modernization batch: native money (iteration011)

Reward, total and independent transfer sources now hold native amounts plus the
blank-layout facts needed by payment/ending rules. Credit producers name game
denominations; arithmetic no longer operates on encoded ordinary digits. Source
widths, saturation, partial funding, paid-return provenance, transfer cadence and
nondecimal boundary behavior remain preserved. Contracts/evidence are linked in
[status/trace.md](port/src/status/trace.md) and [modernize/changes.json](modernize/changes.json).

## Modernization batch: native enemy behavior (iteration012)

The complete ambush family now retains explicit phases, delay and preset cursor
in its canonical actor control. Legacy callback, lifecycle and observation
projections preserve exact source fields and partial updates; no duplicate byte
state drives the converted enemy. Same-call transitions, animation gates, wrapped
waits and ordered contact/motion/sound effects retain their source contracts. See
[enemies/trace.md](port/src/actors/enemies/trace.md) and
[modernize/changes.json](modernize/changes.json).

## Modernization batch: decoded movement programs (iteration013)

Perimeter routes now describe headings and contact conditions directly. Wave
programs contain explicit timed spawn and wait-for-last-child instructions. Asset
loading decodes the old packed/parallel tables; gameplay preserves mirrored routes,
corrections, exact byte timestamps, paired emissions and immediate callback order.
Source contracts and evidence are linked in [bosses/trace.md](port/src/actors/bosses/trace.md)
and [modernize/changes.json](modernize/changes.json).

## Modernization batch: native scene actor phases (iteration014)

Scene actor movement now retains native behavior variants with named counters.
Bounce direction, entry progress, walking pauses and move/rebuild waits no longer
depend on packed progress checks in gameplay. Source motion/update/yield order,
all wrapping waits, exact80 entry reset and partial-field handoffs remain covered.
See [scenes/trace.md](port/src/scenes/trace.md) and
[modernize/changes.json](modernize/changes.json) for contract/evidence scope.

Animation modernization015 preserves duration-plus-one updates, dialogue/frozen
holds, tick127 overflow, post-increment frame loop selection and increment-before-
destruction behavior. Shared gameplay callers use native playback resets, hold/
resume and initial-pass events. See `port/src/presentation/trace.md` and its
source-backed contracts; no animation timing or content was intentionally changed.

Modernization016 retains native dialogue programs instead of encoded message
streams. Blank pacing/row-limit bypass, visible glyph17, final unpaced period,
page/scroll and canonical cursor/publication semantics are unchanged. Original18
message contracts and a fresh807-event/42-page renderer comparison cover the text
executor; natural original-emulator dialogue and physical-platform QA remain gaps.
See `port/src/dialogue/trace.md` for scope and source evidence.

Modernization017 replaces flat terrain video writes with native tile/palette
changes across columns, changed blocks, bursts and vertical rows. World clipping,
collision/cache updates, retained staging, byte-wrap aliases, publication timing
and source waits are preserved. Source encoding belongs to Video/presentation;
2610-case direct-owner comparisons and actual Game vertical/neighbor contracts
supplement the fresh gameplay route. Source/physical-platform coverage limits
remain in `port/src/world/trace.md` and the modernization evidence.

Modernization018 replaces raw audio sequence interpretation with immutable typed
primary/secondary programs. Notes carry decoded keys/duration classes, branches
carry decoded destinations, and original resource locations remain only where
cursor/seed or table observations require them. Native actions preserve waits,
repeat/deferred ordering, reset aliases and register/synthesis cadence. Full
source-backed contracts and a58-case/13920-frame direct audio probe supplement
the fresh runtime route; no full soundtrack/device/original-emulator claim is
made. Details and unchanged C522/C523 uncertainty: `port/src/audio/trace.md`.
