# Stage 3 combined contract handoff

All 18 whole-area workers completed sequentially under the full contract prompt. Only the active worker edited; coordinator reconciled each handoff. No newly discovered cohesive area required an additional worker. All behavior remains unimplemented and no check-in/publication was performed.

| Area | Separate worker | Current contracts | Whole-area trace/test handoff |
|---|---|---:|---|
| B01 | `/root/behavior_contracts/b01_runtime` | 21 | [B01.md](B01.md) |
| B02 | `/root/behavior_contracts/b02_assets` | 34 | [B02.md](B02.md) |
| B03 | `/root/behavior_contracts/b03_presentation` | 55 | [B03.md](B03.md) |
| B04 | `/root/behavior_contracts/b04_world` | 60 | [B04.md](B04.md) |
| B05 | `/root/behavior_contracts/b05_collision` | 70 | [B05.md](B05.md) |
| B06 | `/root/behavior_contracts/b06_player` | 128 | [B06.md](B06.md) |
| B07 | `/root/behavior_contracts/b07_enemies` | 97 | [B07.md](B07.md) |
| B08 | `/root/behavior_contracts/b08_objects` | 99 | [B08.md](B08.md) |
| B09 | `/root/behavior_contracts/b09_platforms` | 81 | [B09.md](B09.md) |
| B10 | `/root/behavior_contracts/b10_lifecycle` | 60 | [B10.md](B10.md) |
| B11 | `/root/behavior_contracts/b11_bosses` | 81 | [B11.md](B11.md) |
| B12 | `/root/behavior_contracts/b12_selection` | 43 | [B12.md](B12.md) |
| B13 | `/root/behavior_contracts/b13_progression` | 76 | [B13.md](B13.md) |
| B14 | `/root/behavior_contracts/b14_status` | 61 | [B14.md](B14.md) |
| B15 | `/root/behavior_contracts/b15_dialogue` | 100 | [B15.md](B15.md) |
| B16 | `/root/behavior_contracts/b16_scenes` | 86 | [B16.md](B16.md) |
| B17 | `/root/behavior_contracts/b17_finale` | 95 | [B17.md](B17.md) |
| B18 | `/root/behavior_contracts/b18_audio` | 97 | [B18.md](B18.md) |

## Current combined evidence

`checks/contracts/combined/results.json` records exact commands, all native source hashes, frozen source hashes and log hashes. Rust 1.90.0/Cargo.lock; shared target reused; no GUI. Coordinator independently ran locked no-run, discovery, whole native suite, infrastructure filter and format check. Compilation/discovery/format exit 0. Discovered 1,346 tests. Full run exit101: 2 passed infrastructure, 1,344 failed behavior, zero ignored. Every one of the 1,344 failure blocks contains explicit `not implemented`; no other compile, fixture, assertion or hang failure was observed. Infrastructure filter independently passed 2/2. These outcomes prove executable contracts ready for implementation, not source behavior implemented or passing.

Per-area evidence is retained without erasing superseded fixture/compiler errors, corrected source interpretations or earlier discovery totals. Area hashes identify their historical checked files; the combined hash manifest identifies the actual reconciled suite. All 18 adjacent traces were read by coordinator; table/handler domains, cross-system ordering, allocation/error paths and shared state were reconciled through per-area handoffs. No representative-test quota or passing-behavior claim is substituted for whole-area source traces.

## Reconciled ownership and required stage 4 obligations

Canonical owners are Runtime input/scheduler/RNG, Assets named catalogs, World cache/camera, ActorPool physical actors, Player action/timers, Status digit/health/timer state, Progression flags/transitions, Presentation prepared/committed queues and Audio queue/driver/APU. Borrowed views and explicit suspended continuations replace copied state and CPU/RAM maps. Cross-area corrections include 06D0/06E0/06F0 actor aliases; ordinary spawn protection clearing; exact primary pending byte/ACK producer values; additive B76A suppression; B08 actual869D terrain activation and extended ROM-backed tuples; B05 residual sample count; B02 scene9 context joins; shared EBB4 immediate life loss; source04C0 runtime frames backed by the canonical secondary upload tail; B08 fresh per-call canonical audio Submission.

Retained uncertainties are required implementation work, not exclusions: B01 host tick/interrupt/foreground integration; actual B06/B15 caller return provenance into first-slot RNG (B11/B13); natural no-signal all-complete land entry and raw alternate selectors 5/6/7/21 (preserve evidenced behavior, never cap to 5); secondary-buffer/runtime-frame natural co-occurrence/cache overlap; source-unknown-00c522 classification (positive bounded consumer graph finds no current consumer; original identity retained); actual cross-channel secondary lowF reset seed provenance (nondefault ResetLevel, including adjacent runtime state selectors). See uncertainties.json and each area handoff for exact evidence/obligations. None is silently closed by compiling fixtures or exercising both injected values.

All 1,302 original region and 1,537 unit identities/extents match the planning map, all dispositions remain pending, frozen source hashes match workspace.json, and original uncertainty ID remains. No final implementation audit, emulator whole-game comparison, QA pass, toolkit edit, commercial asset embedding or publication occurred.

## Exact resumption

Stage 3 is complete. Coordinator `/root` should launch stage 4 in its own subagent under the entire prompts/start.md/common.md after reviewing this handoff. Implement the whole source game, not only easy-to-pass contracts. Read original source alongside tests, resolve retained uncertainties/caller integrations, implement shared desktop/browser/headless behavior, update ledger/uncertainty and all implementation links/documentation, then require `python3 run.py check` and `python3 run.py audit` before fresh two-pass QA. No stage 4 worker was launched by this stage 3 coordinator.
