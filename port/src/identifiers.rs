//! Semantic identities separate original dispatch encodings from extensible content slots.
/// Stable source label for an asset or future extension; collection validation owns syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetId(pub String);
/// Source StageIndex encoding. Exact alternate-route meanings await dispatch tracing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StageId(pub u8);
/// Source ActorHandlerPointerLow/High selector; meanings remain source-label linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorKind(pub u8);
/// Source PlayerStatePointerLow/High selector; do not confuse with actor-local states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStateId(pub u8);
/// Source audio command ID; music/effect association is not yet decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioCommandId(pub u8);
/// Derived native mode; no second retained scene selector belongs to Port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Title,
    LandSelection,
    Confirmation,
    Gameplay,
    Dialogue,
    StatusPanel,
    Transition,
    Ending,
    Stalled,
}
impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::LandSelection => "land_selection",
            Self::Confirmation => "confirmation",
            Self::Gameplay => "gameplay",
            Self::Dialogue => "dialogue",
            Self::StatusPanel => "status_panel",
            Self::Transition => "transition",
            Self::Ending => "ending",
            Self::Stalled => "stalled",
        }
    }
}
/// Boundary names for future bounded semantic events, never player-facing controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    ModeChanged,
    ActorSpawned,
    ActorRemoved,
    ProgressionChanged,
    AudioCommand,
}
