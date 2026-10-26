# Native widescreen gameplay

The internal frame is **341×240**, presented at **16:9** in the browser and
desktop. This preserves the pixel proportions of the original 256×240 image
presented at 4:3. The ideal width is 341⅓ pixels; rounding to 341 introduces less
than 0.1% pixel-aspect error. Away from map edges, the viewport adds 42 columns
on the left and 43 on the right. Dimensions, source/display aspects, NTSC cadence
and defaults live in `port/src/config.rs`.

The HUD, title, selection, dialogue, status panels and ending scenes retain their
original 256 columns, centered inside the wider frame. Each source UI pixel maps
to one native frame pixel without horizontal resampling. Gameplay and UI therefore
use the same pixel scale, with the original 4:3 presentation retained for UI.

Gameplay samples the complete visible map instead of extending the cartridge's
two-page nametable ring. Connected rooms clamp the camera at map boundaries;
short arenas remain centered. Missing rooms remain missing. Ordinary enemies,
objects and ambient edge spawns activate and cull against the same viewport.
Signed sprite pieces draw without OAM piece/scanline limits. The original sixteen
actor slots, including the player, remain the simulation pool; pending visible
spawns retry when capacity becomes available.

Horizontal collision and proximity use world distances instead of wrapping at
256 pixels. Terrain replacements are tracked by layout, room row and tile so
visible rooms sharing the source cache parity retain independent edits/collision.
The original camera path still drives movement, exits and progression. Vertical
transitions retain their source rows and cadence. Native terrain, sprites and
presentation mode commit at interrupt boundaries; rendering is read-only and
snapshots retain that committed state. Opening dialogue changes its framing
without despawning actors in the wider side areas. Visibility intentionally
changes encounter timing and difficulty.

The complete sprite-zero hit scanline belongs to widescreen gameplay. It already
contains scrolled playfield pixels; keeping it at the HUD's scale produced the
moving seam. The boundary regression covers tile/nametable scroll crossings.
Matched Wasm captures of the boundary fix changed only Amazon scanline 46 and
preserved identical game state, recorded in `checks/widescreen-hud-boundary/`.

Rust checks cover pixel proportions and exact UI pixel copies, viewport bounds,
signed clipping, activation/reversal/capacity, independent terrain edits and
collision, vertical transitions, sprite priority, all five stage maps, natural
Amazon entry, dialogue framing, commit timing and deterministic snapshots.
`scripts/test-widescreen.mjs` exercises the real Wasm build, display proportions,
both gameplay edges and deterministic frame/audio continuation. Browser layout
and save-state checks apply. These implementation checks are separate from a
full-game walkthrough or fidelity acceptance pass.

Superseded source-distance expectations remain in `review/widescreen-before/`.
The earlier 720-column viewport code, tests and notes are retained in
`review/widescreen-proportions-before/`. Earlier check records remain in
`checks/widescreen/`; current proportion checks are in
`checks/widescreen-proportions/`. Original ledgers and QA records are unchanged.
Desktop uses the local runtime aspect support recorded in `library-changes.md`;
transfer that runtime source with desktop builds. Nothing is committed or
published as part of these changes.

Proportion follow-up validation: all 1,484 game tests pass. Desktop-feature clippy
with warnings denied, formatting/whitespace, the release Wasm build, Wasm
gameplay/snapshot continuation, and headless browser layout/storage checks pass.
A real browser capture displays 341×240 at 960×540, measuring pixel aspect
1.25122 against the original 1.25 (whole-column rounding). Capture and logs are
in `checks/widescreen-proportions/`. The HUD boundary correction remains active.

HUD foreground follow-up: eligible native gameplay sprites now draw across the
HUD boundary, using the same sprite patterns and viewport coordinates on both
sides. Scrooge draws in front of the HUD; background priority below the split and
the existing HUD-data suppression rules remain intact. Native tests cover opaque
and transparent pixels, flipping, masking, ordinary-actor eligibility, terrain
priority and non-gameplay framing. All 1,485 game tests and desktop-feature
clippy pass. The rebuilt real Wasm regression reproduces the old failure and now
shows 314 Scrooge foreground pixels above the HUD after a raised-snapshot fixture
and four normal ticks. The headless browser capture shows him crossing the split.
Build, test and capture records are in `checks/hud-player-foreground/`, separate
from QA acceptance. Nothing was committed or published.
