# Differences, assumptions and validation limits

Whole source systems run through shared native Game owners. Focused source
contracts and integration tests validate branches and joins; independent complete
runtime fidelity remains for stage5's two fresh passes. No hardware/device run or
additional gameplay smoke route was performed during stage4.

- Host policy is one source interrupt before resumed foreground work per NTSC
  tick. Internal borrowed-owner requests do not add interrupts. Actual waits do;
  a raster/menu busy loop runs interrupts/audio without controller sampling or
  task0 resume. Hardware reset warm-up vblank polling is omitted before the first
  playable tick. Intentional waits/freezes/flashing remain source-driven.
- Gameplay intentionally extends the original view to 341×240, displayed at 16:9
  with the original 4:3 display pixel proportions. Terrain comes from every visible world room, with full-coordinate
  mutation/collision state. Actor activation and culling follow the wider camera;
  world distances no longer wrap at 256 pixels. This changes encounter timing and
  difficulty. Gameplay sprites have signed positions and no eight-piece scanline
  limit. HUD, menus and scenes retain 4:3 framing. Superseded byte-distance tests
  are preserved in review/widescreen-before/. See WIDESCREEN.md.
- Rendering uses committed logical CHR/nametable/palette/OAM and deterministic
  UI/playfield split evaluation. Sprite-zero hit uses the same pixel policy.
  It approximates source raster timing at scanline/pixel boundaries, without a
  CPU/PPU emulator, hardware-induced lag or sprite-overflow flicker. The RGB
  palette is a conventional approximation; emphasis applies natively.
- Audio interprets the source sequence engine and applies register writes to the
  shared native APU synthesizer. Audio sample callbacks alone advance sample
  clocks. Device mixing/resampling differs from original analog hardware.
- Primary/secondary/terrain-cache overlaps and audio low-F reads borrow canonical
  fields. Reset selector has explicit zero provenance and carries actual later
  producer facts across frames; no default19 or shadow RAM is used.
- Bottom terrain-row expansion keeps the lower staging bytes not rewritten by
  ordinary source callers. Unusual Scratch03 residue from an unestablished caller
  is an assessed source-informed assumption; the ordinary retained-byte policy
  is exposed to audio and covered by composed tests.
- Raw source land/stage selectors5/6/7/21 preserve byte consumer arithmetic and
  adjacent validated instruction-as-data reads. No five-land cap is invented.
  Out-of-domain pointers requiring non-ROM storage remain explicitly unsupported
  at the bounded consumer. This is best-effort source behavior, not a claim that
  every malformed selector is an ordinary authored level.
- PhysicalC522..C524 (`A9 07`) remains semantically unclassified. Source consumer
  review finds no established entry/data read; native implemented audio paths do
  not execute it. This is an assessed absence assumption, not confirmed padding
  or a changed source classification. Original identities/ranges remain intact.
- Authored named levels are a native extension, with shared source rule profiles,
  finite predecessor chains and safe JSON payloads. Their entry checkpoints and
  audio-program stop policy are specified in EXTENSIONS.md. Original global
  progress effects remain; no new content-selection menu is added.

- Countdown now uses native seconds and explicit expiration. For nondecimal
  inputs only, it retains the faithful implementation's shared subtraction rule:
  BLANK24 behaves as zero. Original timer E2B5/EF8D subtracts raw bytes instead.
  Synthetic[00,00,24] consequently ticks to999 natively versus[00,00,23] in source.
  Natural reachability at a timer gate is unestablished; this pre-existing edge
  discrepancy is preserved, not repaired or certified, by iteration009. See
  [status/trace.md](port/src/status/trace.md) for the follow-up probe.

The original has no battery saving. Debug checkpoints use validated deterministic
replay rather than invented cartridge saves or arbitrary memory writes. Desktop
and Wasm compilation are checked headlessly; macOS/Windows/browser/device runtime
recipes are not claims of physical-platform testing. Reference files, local ROM
and superseded evidence remain preserved.
