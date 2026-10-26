//! Shared desktop/browser/headless game owner and deterministic source host loop.
mod contact;
mod content;
mod dialogue_call;
mod families;
mod foreground;
mod journal;
mod levels;
mod machine;
mod menu;
mod observation;
mod player_call;
mod simulation;
mod tasks;
mod video;
mod widescreen;
use crate::{assets::Assets, identifiers::Mode};
use anyhow::Result;
use romatlas_port::{
    Game,
    assets::{MemoryAssets, Rom},
    graphics::Frame,
    input::{InputState, MultiInputState},
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// The one Machine owns rules, display and audio across every frontend.
#[derive(Clone)]
pub struct Port {
    pub assets: Assets,
    machine: machine::Machine,
}
impl Port {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(rom: &Rom, overrides: Option<&Path>) -> Result<Self> {
        let memory = match overrides {
            Some(root) => MemoryAssets::from_directory(root, &[crate::assets::MANIFEST])?,
            None => MemoryAssets::default(),
        };
        Self::load_assets(rom, &memory)
    }
    pub fn load_assets(rom: &Rom, overrides: &MemoryAssets) -> Result<Self> {
        let assets = Assets::load(rom, overrides)?;
        let content = content::Content::load(rom, &assets, overrides)?;
        Ok(Self {
            assets,
            machine: machine::Machine::new(content),
        })
    }
    pub fn snapshot_json(&self) -> Result<String> {
        self.machine.snapshot_json()
    }
    pub fn restore_snapshot_json(&mut self, json: &str) -> Result<()> {
        self.machine.restore_snapshot_json(json)
    }
    pub fn mode(&self) -> Mode {
        self.machine.mode()
    }
    fn advance_samples(
        &mut self,
        samples: [crate::runtime::PortSample; crate::runtime::CONTROLLER_PORTS],
    ) {
        self.machine.tick(samples);
    }
}
fn sample_actions(held: &std::collections::BTreeSet<String>) -> crate::runtime::PortSample {
    use crate::runtime::{Buttons, PortSample};
    let mut buttons = Buttons::NONE;
    for (action, bit) in [
        ("right", Buttons::RIGHT),
        ("left", Buttons::LEFT),
        ("down", Buttons::DOWN),
        ("up", Buttons::UP),
        ("start", Buttons::START),
        ("select", Buttons::SELECT),
        ("b", Buttons::B),
        ("a", Buttons::A),
    ] {
        if held.contains(action) {
            buttons = buttons | bit;
        }
    }
    // Source edge detection occurs only at a real slot-zero resume. Frontend
    // pressed/released edges cannot replace its game-modifiable held history.
    PortSample {
        standard: buttons,
        auxiliary: Buttons::NONE,
    }
}
impl Game for Port {
    // Pages retains its published runtime pin; its Wasm wrapper uses the same
    // inherent serializers. Native frontends use the new optional Game hooks.
    #[cfg(not(target_arch = "wasm32"))]
    fn snapshot_json(&self) -> Result<String> {
        Port::snapshot_json(self)
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn restore_snapshot_json(&mut self, json: &str) -> Result<()> {
        Port::restore_snapshot_json(self, json)
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn reset(&mut self) -> Result<()> {
        self.machine.cold_reset();
        Ok(())
    }
    fn tick(&mut self, input: &InputState) {
        self.advance_samples([
            sample_actions(&input.held),
            crate::runtime::PortSample::default(),
        ]);
    }
    fn tick_players(&mut self, input: &MultiInputState) {
        self.advance_samples([
            sample_actions(&input.primary.held),
            sample_actions(&input.players[1].held),
        ]);
    }
    #[cfg(feature = "desktop")]
    fn display_aspect_ratio(&self) -> Option<(u32, u32)> {
        Some(crate::config::DISPLAY_ASPECT)
    }
    fn render(&self, frame: &mut Frame) {
        self.machine.services.video.render(frame);
    }
    fn audio(&mut self, frames: usize, sample_rate: u32, samples: &mut Vec<f32>) {
        let mut mono = vec![0.0; frames];
        self.machine
            .services
            .synthesizer
            .render(&mut mono, sample_rate);
        samples.reserve(frames * 2);
        for value in mono {
            samples.extend([value, value]);
        }
    }
    // The original cartridge has no battery; default save hooks remain correct.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets::MANIFEST, config};
    use romatlas_port::assets::Manifest;
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    #[test]
    fn desktop_state_hooks_restore_real_game_audio_and_reset_to_startup() {
        use romatlas_port::{audio::SampleClock, states};
        let rom = rom();
        let mut game = Port::load_assets(&rom, &MemoryAssets::default()).unwrap();
        let initial = Game::snapshot_json(&game).unwrap();
        let mut clock = SampleClock::new(config::RATE, 48000).unwrap();
        for _ in 0..90 {
            game.tick(&InputState::default());
            game.audio(clock.next_tick(), 48000, &mut Vec::new());
        }
        let state = states::capture(&game, &clock, "test-build-rom-assets").unwrap();
        let mut expected = game.clone();
        let mut expected_clock = clock.clone();
        Game::reset(&mut game).unwrap();
        assert_eq!(Game::snapshot_json(&game).unwrap(), initial);
        states::restore(&mut game, &mut clock, "test-build-rom-assets", &state).unwrap();
        for _ in 0..30 {
            game.tick(&InputState::default());
            expected.tick(&InputState::default());
            let (mut a, mut b) = (Vec::new(), Vec::new());
            game.audio(clock.next_tick(), 48000, &mut a);
            expected.audio(expected_clock.next_tick(), 48000, &mut b);
            assert_eq!(
                a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                b.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
            assert_eq!(
                Game::snapshot_json(&game).unwrap(),
                Game::snapshot_json(&expected).unwrap()
            );
        }
    }

    #[test]
    fn browser_snapshot_preserves_fractional_frame_sample_clocks_and_input_edges() {
        use romatlas_port::{browser::Runtime, input::PhysicalInput};
        let rom = rom();
        let new_runtime = || {
            Runtime::new(
                Port::load_assets(&rom, &MemoryAssets::default()).unwrap(),
                config::SIZE,
                config::RATE,
                config::settings(),
                48000,
                rom.sha256.clone(),
            )
            .unwrap()
        };
        let mut original = new_runtime();
        let mut physical: [PhysicalInput; 4] = Default::default();
        physical[0].keys.insert("Z".into());
        original.advance(21.125, physical.clone()).unwrap();
        let (game, driver) = (
            original.game.snapshot_json().unwrap(),
            original.driver_snapshot_json().unwrap(),
        );
        let mut restored = new_runtime();
        restored.game.restore_snapshot_json(&game).unwrap();
        restored.restore_driver_snapshot_json(&driver).unwrap();
        for elapsed in [0.125, 13.25, 35.375, 8.5, 50.125] {
            assert_eq!(
                original.advance(elapsed, physical.clone()).unwrap(),
                restored.advance(elapsed, physical.clone()).unwrap()
            );
            assert_eq!(original.frame.pixels, restored.frame.pixels);
            assert_eq!(
                original
                    .samples
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                restored
                    .samples
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                original.game.snapshot_json().unwrap(),
                restored.game.snapshot_json().unwrap()
            );
            assert_eq!(
                original.driver_snapshot_json().unwrap(),
                restored.driver_snapshot_json().unwrap()
            );
            physical[0].keys.clear();
        }
    }

    #[test]
    fn persistent_snapshot_restores_continuations_frames_and_exact_audio_without_replay() {
        let rom = rom();
        let memory = MemoryAssets::default();
        let mut source = Port::load_assets(&rom, &memory).unwrap();
        for tick in 0..720 {
            let buttons = match tick {
                60..=62 | 150..=152 | 240..=242 => crate::runtime::Buttons::START,
                480..=719 => crate::runtime::Buttons::RIGHT,
                _ => crate::runtime::Buttons::NONE,
            };
            source.advance_samples([
                crate::runtime::PortSample {
                    standard: buttons,
                    ..Default::default()
                },
                Default::default(),
            ]);
            source.audio(800, 48000, &mut Vec::new());
            if [1, 60, 120, 180, 360, 600].contains(&tick) {
                let saved = source.snapshot_json().unwrap();
                let mut restored = Port::load_assets(&rom, &memory).unwrap();
                restored.restore_snapshot_json(&saved).unwrap();
                assert_eq!(restored.snapshot_json().unwrap(), saved);
                let mut expected = source.clone();
                for branch_tick in 0..20 {
                    let input = [
                        crate::runtime::PortSample {
                            standard: if branch_tick % 3 == 0 {
                                crate::runtime::Buttons::A
                            } else {
                                crate::runtime::Buttons::RIGHT
                            },
                            ..Default::default()
                        },
                        Default::default(),
                    ];
                    expected.advance_samples(input);
                    restored.advance_samples(input);
                    let (mut a, mut b) = (
                        Frame::new(config::SIZE.0, config::SIZE.1).unwrap(),
                        Frame::new(config::SIZE.0, config::SIZE.1).unwrap(),
                    );
                    expected.render(&mut a);
                    restored.render(&mut b);
                    assert_eq!(a.pixels, b.pixels, "frame after snapshot at tick {tick}");
                    let (mut a, mut b) = (Vec::new(), Vec::new());
                    expected.audio(800 + branch_tick % 2, 48000, &mut a);
                    restored.audio(800 + branch_tick % 2, 48000, &mut b);
                    assert_eq!(
                        a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                        b.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                        "audio after snapshot at tick {tick}"
                    );
                }
                assert_eq!(
                    expected.snapshot_json().unwrap(),
                    restored.snapshot_json().unwrap()
                );
                let before = restored.snapshot_json().unwrap();
                assert!(restored.restore_snapshot_json("{broken").is_err());
                assert_eq!(before, restored.snapshot_json().unwrap());
            }
        }
    }

    fn rom() -> Rom {
        Rom::open(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../input/game.rom")
                .as_path(),
            config::ROM_SHA256,
        )
        .unwrap()
    }
    fn run(requests: Vec<(&str, Value)>) -> Vec<Value> {
        let input = requests
            .into_iter()
            .enumerate()
            .map(|(id, (method, params))| {
                json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string() + "\n"
            })
            .collect::<String>();
        let mut output = Vec::new();
        let rom = rom();
        romatlas_port::debug::serve_io(
            &mut input.as_bytes(),
            &mut output,
            || Port::load_assets(&rom, &MemoryAssets::default()),
            config::SIZE,
            config::RATE,
            config::settings().bindings.keys().cloned().collect(),
            "infrastructure-test".into(),
            Port::observe,
        )
        .unwrap();
        String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    #[test]
    fn matching_identity_recipe_bounds_and_extension_validation() {
        let rom = rom();
        assert!(Rom::from_bytes(vec![0; 16], config::ROM_SHA256).is_err());
        assert!(rom.read(usize::MAX, 2).is_err());
        assert!(rom.read(131088, 1).is_err());
        let mut overrides = MemoryAssets::default();
        let manifest: Manifest = serde_json::from_value(
            json!({"version":1,"entries":[{"id":"custom/source","file":"custom.bin"}]}),
        )
        .unwrap();
        overrides.manifests = BTreeMap::from([(MANIFEST.into(), manifest)]);
        overrides.files.insert("custom.bin".into(), vec![1]);
        let port = Port::load_assets(&rom, &overrides).unwrap();
        assert!(port.assets.source.get("custom/source").is_some());
        overrides.files.insert("custom.bin".into(), Vec::new());
        assert!(Port::load_assets(&rom, &overrides).is_err());
    }
    #[test]
    fn reset_and_checkpoint_restore_semantics_without_render_mutation() {
        let replies = run(vec![
            ("port.advance", json!({"ticks":3})),
            ("port.checkpoint.export", json!({})),
            ("port.state", json!({})),
            ("port.framebuffer", json!({})),
            ("port.state", json!({})),
            ("port.reset", json!({})),
            ("port.state", json!({})),
        ]);
        assert_eq!(replies[2]["result"], replies[4]["result"]);
        assert_eq!(replies[6]["result"]["game"]["ticks"], 0);
        let checkpoint = replies[1]["result"].clone();
        let mut invalid = checkpoint.clone();
        invalid["identity"] = json!("different-build");
        let restored = run(vec![
            ("port.checkpoint.import", json!({"checkpoint":checkpoint})),
            ("port.state", json!({})),
            ("port.checkpoint.import", json!({"checkpoint":invalid})),
            ("port.state", json!({})),
        ]);
        assert_eq!(replies[2]["result"], restored[1]["result"]);
        assert!(restored[2].get("error").is_some());
        assert_eq!(restored[1]["result"], restored[3]["result"]);
    }
}
