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
