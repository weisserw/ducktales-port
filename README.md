# DuckTales native Rust port

A shared native implementation of the original title, land selection, player and
actor rules, maps, bosses, dialogue, special scenes, progression, ending and audio.
Desktop, browser and headless/debug frontends use the same Game implementation.
Stage4 source contracts and integration checks are recorded in checks/implementation;
independent whole-game QA remains a separate stage in debug/qa.json.

The original cartridge has no battery. The browser page additionally offers a
single Save State / Restore State slot in localStorage, which survives Reset and
page reloads. Restore returns to the saved frame paused; click Start to continue.

```sh
python3 run.py check
python3 run.py desktop
python3 run.py web
cargo +1.90.0 run --locked --manifest-path port/Cargo.toml -- --rom input/game.rom --headless 1
```

Desktop opens a window only on your graphical machine. `run.py web` builds/serves
the browser frontend (default0.0.0.0; `--host` changes binding); the configured ROM is fetched and identity-checked at startup. See BUILDING.md for dependencies, platform recipes and transfer.
CLI options are `--rom FILE`, `--assets DIR`, `--settings FILE`, `--debug-stdio`
and `--headless FRAMES`.

Default controls follow the NES settings: arrows move, Z=A/jump, X=B/cane,
Enter=Start/pause and Backspace=Select. F1 opens desktop settings for remapping,
player/controller assignment, volume and display options. Browser settings use the
shared settings frontend. Source controller2 remains an independent lane where the
original rules consult it. There is no added content-selection menu or debug warp.

Raw replacement files use source-assets.json version1 entries `{id,file}`. IDs
and source extents are in port/src/recipes.rs; replacements retain exact recipe
lengths and decode at startup. They are local extracted resources, not files to
redistribute. Added named resources instead use authored JSON formats and
levels.json, documented in [EXTENSIONS.md](EXTENSIONS.md). The ready-to-copy
[original-only manifest](examples/original-only/source-assets.json) keeps all
original content. The [authored example](examples/authored-level/source-assets.json)
contains only authored pattern/map/music data and source template references.

Rules live in port/src/player, actors, world, collision, selection, progression,
status, dialogue, scenes, finale and audio. Adjacent trace.md and contracts.rs
record source derivations; port/src/game connects their canonical owners and
resumable calls. Identity, dimensions256×240, NTSC cadence and defaults are in
port/src/config.rs. See architecture.md and differences.md for timing/render/audio
policies and assessed assumptions.

For separately requested debug/QA work, read debug/goal.md. Semantic observation
schema2, replay/checkpoint limits and reproducible setup recipes are documented in
debug/api.md, DEBUGGING.md and debug/entry-recipes.md. Stage4 does not substitute
focused fixtures for the two required fresh whole-game QA passes.
