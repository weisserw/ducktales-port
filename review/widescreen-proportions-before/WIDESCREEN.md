# Native widescreen gameplay

The internal frame is 720×240, exactly 3:1, presented at 16:9 in the browser
and desktop. Away from map edges, this adds 232 pixels (29 eight-pixel tiles)
on either side of the original 256-pixel view. Dimensions, presentation aspect,
NTSC cadence and defaults live in `port/src/config.rs`.

Gameplay samples the complete visible map instead of extending the cartridge's
two-page nametable ring. The camera uses connected rooms at level edges, avoiding
empty space when sufficient map exists. An arena narrower than 720 pixels stays
centered with space outside its boundaries. Missing rooms remain missing.

Ordinary enemies/objects and ambient edge spawns activate across the wider view.
Signed sprite coordinates and wider culling keep them visible across the old
screen edges. Gameplay composition lifts OAM piece/scanline drawing limits;
the original sixteen actor slots, including the player, remain the simulation
pool. Pending visible spawns retry when a slot becomes available. Horizontal collision and proximity use world distances, preventing actors
256 pixels apart from touching or carrying each other. Terrain replacements
are tracked by layout, room row and tile, so changes and collision remain correct
when multiple visible rooms share the source cache's parity.

The original camera path still drives movement, room exits and progression. The
native viewport extends and clamps that path for presentation and activation.
Vertical scrolling retains both room rows and its source cadence. Source raster
splits, pattern/palette updates, audio and controls use the shared Game owners.
Rendering is read-only; committed native terrain and sprites are included in
snapshots. Encounter timing and difficulty intentionally change with visibility.

The HUD is centered at its original 4:3 display aspect. Title, selection, dialogue,
status panels and ending scenes use the same centered framing. Only gameplay
fills the 16:9 surface.

Focused Rust checks cover viewport edges, signed clipping, spawning/reversal,
multiple room pages, independent terrain edits/collision, vertical directions,
sprite priority beyond eight pieces, all five stage maps, natural Amazon entry,
and deterministic snapshots. `scripts/test-widescreen.mjs` exercises the real Wasm
build, both gameplay edges and deterministic frame/audio continuation. Existing
browser layout and save-state checks also apply. These implementation checks do
not constitute a new full-game walkthrough or fidelity acceptance pass.

Superseded low-byte-distance test expectations are retained in
`review/widescreen-before/`. Original ledgers and prior QA records are unchanged.
Desktop uses an additive local runtime aspect API; details and check results are
recorded in `library-changes.md`. Transfer the modified runtime with desktop
source; no toolkit publication is required for the browser workflow.

Final validation on 2026-10-26: all 1,483 game tests and 73 shared runtime tests
pass; desktop-feature clippy with warnings denied, formatting/whitespace, release
Wasm build, Wasm gameplay/snapshot checks, and headless browser layout/storage
checks pass. Local implementation logs are in `checks/widescreen/`; QA records
remain separate. No desktop graphics were launched, and nothing was committed
or published.

HUD boundary follow-up: gameplay now widens the sprite-zero hit scanline itself,
which already contains scrolled playfield pixels. Keeping that row in the HUD
previously produced a moving horizontal seam. The focused renderer test passes
at scroll offsets 0, 1, 7, 8, 255 and 256. Matched real Wasm captures change only
scanline 46 and preserve identical game state. The release Wasm build,
formatting and whitespace checks pass; evidence is in
`checks/widescreen-hud-boundary/`. Pixel proportions await a user choice: fitting
720×240 to 16:9 necessarily compresses pixels horizontally regardless of HUD
height, so retaining those exact dimensions and removing compression conflict.
