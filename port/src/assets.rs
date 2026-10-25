//! Startup owns raw ROM recipes; later decoders consume these without banked reads in rules.
use crate::{config, recipes};
use anyhow::{Result, ensure};
use romatlas_port::assets::{Asset, Collection, MemoryAssets, Origin, Rom};

pub const MANIFEST: &str = "source-assets.json";
/// Validated raw source assets and extensible named slots. No decoding is implied.
#[derive(Clone)]
pub struct Assets {
    pub source: Collection,
}
impl Assets {
    pub fn load(rom: &Rom, overrides: &MemoryAssets) -> Result<Self> {
        ensure!(
            rom.sha256 == config::ROM_SHA256,
            "matching DuckTales ROM required"
        );
        let defaults = recipes::original_assets()
            .into_iter()
            .map(|recipe| {
                Ok(Asset {
                    bytes: rom.read(recipe.offset, recipe.length)?.to_vec(),
                    id: recipe.id,
                    origin: Origin::Rom {
                        offset: recipe.offset,
                        length: recipe.length,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let source = overrides.collection(MANIFEST, defaults, |asset| {
            ensure!(
                !asset.bytes.is_empty(),
                "raw source asset must not be empty"
            );
            if let Some(recipe) = recipes::original_assets().iter().find(|r| r.id == asset.id) {
                ensure!(
                    asset.bytes.len() == recipe.length,
                    "replacement must preserve raw source record length"
                );
            }
            Ok(())
        })?;
        Ok(Self { source })
    }
}

pub mod catalog;
#[cfg(test)]
mod contracts;
pub mod file_resources;
pub mod formats;
