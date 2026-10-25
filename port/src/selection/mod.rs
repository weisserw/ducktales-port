//! Title and original land-selection policies (B12). See trace.md.
//! Progression, actors, random and display
//! clocks are borrowed from their canonical owners; no second state store.
use crate::{
    actors::{ACTIVE, ActorPool},
    assets::catalog::DecodedAssets,
    identifiers::{AssetId, StageId},
    presentation::{Composition, PROJECTION_ONLY, VISIBLE},
    runtime::{PortInput, RandomState},
    status::TitleRule,
};
use anyhow::{Context as _, Result};

pub const MENU_SOUND: u8 = 0x2f;
pub const TITLE_AUDIO: u8 = 7;
pub const TITLE_PALETTE_OFFSET: u8 = 0x60;
pub const TITLE_CHR: usize = 6;
pub const LAND_CHR: usize = 5;
pub const TITLE_SCRIPT: usize = 2;
pub const STAGE_UI_SCRIPT: usize = 0;
pub const LAND_NAME_ANIMATION_BASE: u8 = 0x6f;
pub const TITLE_RANDOM_PERIOD: u8 = 0xb4;
pub const COMPLETION_PAUSE: u16 = 60;
pub const ORIGINAL_LAND_BITS: u8 = 0x1f;
pub const TITLE_PARAMETER_OFFSET: usize = 0x1d631;
pub const TITLE_PARAMETER_BYTES: usize = 15;
pub const LAND_PARAMETER_OFFSET: usize = 0x1d8b6;
pub const LAND_PARAMETER_BYTES: usize = 51;

/// Labels are proven by title script2 at PPU2225, not guessed from damage rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleOption {
    Easy,
    Normal,
    Difficult,
}
impl TitleOption {
    pub fn source_selector(self) -> u8 {
        match self {
            Self::Easy => 0,
            Self::Normal => 1,
            Self::Difficult => 2,
        }
    }
    pub fn display_label(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Normal => "NORMAL",
            Self::Difficult => "DIFFICULT",
        }
    }
    pub fn status_rule(self) -> TitleRule {
        match self {
            Self::Easy => TitleRule::AlternatingDamage,
            Self::Normal => TitleRule::OrdinaryDamage,
            Self::Difficult => TitleRule::AlternateRewards,
        }
    }
}
/// The original menu domain is finite. Added named levels belong to B02's dynamic
/// catalog and are not extra entries in the original game's menu.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Land {
    Amazon,
    Transylvania,
    AfricanMines,
    Himalayas,
    Moon,
}
impl Land {
    pub fn source_selector(self) -> StageId {
        StageId(match self {
            Self::Amazon => 0,
            Self::Transylvania => 1,
            Self::AfricanMines => 2,
            Self::Himalayas => 3,
            Self::Moon => 4,
        })
    }
    pub fn level(self, _catalog: &DecodedAssets) -> Result<AssetId> {
        Ok(_catalog
            .original_level(self.source_selector())
            .context("original land catalog identity")?
            .id
            .clone())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActorPlacement {
    pub animation: u8,
    pub y: u8,
    pub x: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub title_actors: [ActorPlacement; 2],
    pub title_marker_x: [u8; 3],
    pub title_steps: [i8; 2],
    pub title_wrap: [i8; 2],
    pub land_actors: [ActorPlacement; 8],
    pub cursor_y: [u8; 5],
    pub name_y: [u8; 5],
    pub name_x: [u8; 5],
    pub completion_masks: [u8; 8],
    pub land_steps: [i8; 2],
    pub land_wrap: [i8; 2],
}
impl Catalog {
    pub fn decode(title: &[u8], lands: &[u8]) -> Result<Self> {
        let title = title
            .get(..TITLE_PARAMETER_BYTES)
            .context("title parameters extent")?;
        let lands = lands
            .get(..LAND_PARAMETER_BYTES)
            .context("land parameters extent")?;
        let placement = |b: &[u8]| ActorPlacement {
            animation: b[0],
            y: b[1],
            x: b[2],
        };
        Ok(Self {
            title_actors: std::array::from_fn(|i| placement(&title[i * 3..])),
            title_marker_x: title[8..11].try_into()?,
            title_steps: [title[11] as i8, title[13] as i8],
            title_wrap: [title[12] as i8, title[14] as i8],
            land_actors: std::array::from_fn(|i| placement(&lands[9 + i * 3..])),
            cursor_y: lands[4..9].try_into()?,
            name_y: lands[41..46].try_into()?,
            name_x: lands[46..51].try_into()?,
            completion_masks: lands[33..41].try_into()?,
            land_steps: [lands[0] as i8, lands[2] as i8],
            land_wrap: [lands[1] as i8, lands[3] as i8],
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    Frame,
    LandSelection,
    ConfirmStage,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputResult {
    pub decision: Decision,
    pub operations: Vec<Operation>,
}
/// Ordered subroutine requests, executed by existing owners before continuation.
/// Fade and acknowledgement suspend the foreground; they are not instant effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    InstallControlShadow,
    Yield,
    CopyBaseOverlays,
    StageAudio(u8),
    FadeOut,
    ClearGameplayAndProtection,
    ClearGameplay,
    DisableRendering,
    ResetTitleScroll,
    FillFirstNametable,
    HideSprites,
    ClearNonplayerActors,
    ScreenScript(usize),
    Chr(usize),
    TitlePalette,
    TitlePaletteTail,
    InitializeTitleActors,
    InitializeLandActors,
    SaveStageSelector,
    UseLandLayoutSelector,
    RestoreStageSelector,
    ClearOverlayGate,
    EnableRendering,
    FadeIn,
    CopyTitleOverlay,
    LoadLandPresentation,
    BuildWholeTerrainRing,
    StatusMessage,
    Compose,
    Sound(u8),
    Wait(u16),
    TransferDisplayedValue,
    Message(u8),
    CompletionActor,
    Acknowledge,
    ClearActorOne,
    LoadStagePresentation,
    BuildInitialTerrainColumns,
    StageChr,
    EnterStageRuntime,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitlePhase {
    Entry,
    AfterEntryYield,
    AfterFadeOut,
    AfterDisplayYield,
    AfterEnableYield,
    AfterFadeIn,
    AfterOverlayYield,
    Input,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleBatch {
    pub operations: Vec<Operation>,
    pub next: TitlePhase,
}
/// Temporary owner views. The enclosing B13 progression retains these fields.
pub struct TitleContext<'a> {
    pub actors: &'a mut ActorPool,
    pub random: &'a mut RandomState,
    pub option: &'a mut TitleOption,
}
/// A source completion path can bypass transfer and low-five-bit testing when
/// completion_actor is nonzero. Preserve the exact distinction from scene_signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionInput {
    pub completion_actor: u8,
    pub scene_signal: u8,
    pub progress_bits: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionRoute {
    AvailableLand,
    OrdinaryAcknowledgement,
    LateStage,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionPlan {
    pub operations: Vec<Operation>,
    pub route: CompletionRoute,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageRequest {
    pub level: AssetId,
    pub selector: StageId,
    pub late_route: u8,
}

pub fn title_resume(
    _phase: TitlePhase,
    _context: &mut TitleContext<'_>,
    _catalog: &Catalog,
) -> TitleBatch {
    use Operation::*;
    let (operations, next) = match _phase {
        TitlePhase::Entry => (
            vec![InstallControlShadow, Yield],
            TitlePhase::AfterEntryYield,
        ),
        TitlePhase::AfterEntryYield => {
            _context.random.words[0] = crate::runtime::TITLE_RANDOM_SEED;
            (
                vec![CopyBaseOverlays, StageAudio(TITLE_AUDIO), FadeOut],
                TitlePhase::AfterFadeOut,
            )
        }
        TitlePhase::AfterFadeOut => (
            vec![ClearGameplayAndProtection, Yield],
            TitlePhase::AfterDisplayYield,
        ),
        TitlePhase::AfterDisplayYield => (
            vec![
                DisableRendering,
                ResetTitleScroll,
                FillFirstNametable,
                HideSprites,
                ClearNonplayerActors,
                ScreenScript(TITLE_SCRIPT),
                Chr(TITLE_CHR),
                TitlePalette,
                InitializeTitleActors,
                TitlePaletteTail,
                EnableRendering,
                Yield,
            ],
            TitlePhase::AfterEnableYield,
        ),
        TitlePhase::AfterEnableYield => (vec![FadeIn], TitlePhase::AfterFadeIn),
        TitlePhase::AfterFadeIn => (vec![CopyTitleOverlay, Yield], TitlePhase::AfterOverlayYield),
        TitlePhase::AfterOverlayYield => {
            *_context.option = TitleOption::Normal;
            (vec![], TitlePhase::Input)
        }
        TitlePhase::Input => (vec![], TitlePhase::Input),
    };
    TitleBatch { operations, next }
}
pub fn title_input(
    _option: &mut TitleOption,
    _actors: &mut ActorPool,
    _input: PortInput,
    _catalog: &Catalog,
) -> InputResult {
    use crate::runtime::Buttons;
    if _input.pressed.0 & Buttons::START.0 != 0 {
        return InputResult {
            decision: Decision::LandSelection,
            operations: vec![],
        };
    }
    let mut operations = Vec::new();
    if _input.pressed.0 & 0x23 != 0 {
        let index = usize::from((_input.pressed.0 & 2) >> 1);
        let mut value = _option
            .source_selector()
            .wrapping_add_signed(_catalog.title_steps[index]);
        if value >= 3 {
            value = value.wrapping_add_signed(_catalog.title_wrap[index]);
        }
        *_option = match value {
            0 => TitleOption::Easy,
            1 => TitleOption::Normal,
            _ => TitleOption::Difficult,
        };
        _actors[1].position.world_x = (_actors[1].position.world_x & 0xff00)
            | u16::from(_catalog.title_marker_x[usize::from(value)]);
        operations.push(Operation::Sound(MENU_SOUND));
    }
    InputResult {
        decision: Decision::Frame,
        operations,
    }
}
/// Called immediately before B03 composition. B03 then increments its own clock.
pub fn title_animation(
    _actors: &mut ActorPool,
    _random: &mut RandomState,
    _composition: &mut Composition,
) -> Vec<Operation> {
    if _actors[0].playback.at_initial_frame(0) && _actors[0].playback.at_running_tick(1) {
        _actors[0].playback.restart_clock();
        if _composition.clock == TITLE_RANDOM_PERIOD {
            _composition.clock = 0;
            _random.words[0] = _random.words[0]
                .wrapping_add(_random.words[1])
                .wrapping_add(1);
            {
                let animation_value = _random.words[0] & 1;
                _actors[0].playback.set_frame_code(animation_value);
            };
        }
    }
    vec![Operation::HideSprites, Operation::Compose, Operation::Yield]
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LandPhase {
    Entry,
    AfterFadeOut,
    AfterStatusYield,
    AfterFadeIn,
    Completion,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandBatch {
    pub operations: Vec<Operation>,
    pub next: LandPhase,
}
/// Initialization operations commit at their source location, after outgoing
/// fade/load work. Building a batch must not mutate actors ahead of its effects.
pub fn land_resume(_phase: LandPhase) -> LandBatch {
    use Operation::*;
    let (operations, next) = match _phase {
        LandPhase::Entry => (vec![FadeOut], LandPhase::AfterFadeOut),
        LandPhase::AfterFadeOut => (
            vec![
                ClearGameplay,
                DisableRendering,
                HideSprites,
                ClearNonplayerActors,
                SaveStageSelector,
                UseLandLayoutSelector,
                Chr(LAND_CHR),
                LoadLandPresentation,
                BuildWholeTerrainRing,
                RestoreStageSelector,
                InitializeLandActors,
                EnableRendering,
                StatusMessage,
                Yield,
            ],
            LandPhase::AfterStatusYield,
        ),
        LandPhase::AfterStatusYield => (vec![FadeIn], LandPhase::AfterFadeIn),
        LandPhase::AfterFadeIn => (vec![ClearOverlayGate], LandPhase::Completion),
        LandPhase::Completion => (vec![], LandPhase::Completion),
    };
    LandBatch { operations, next }
}
pub fn initialize_title_actors(_actors: &mut ActorPool, _catalog: &Catalog) {
    initialize_menu_actors(_actors, &_catalog.title_actors, VISIBLE);
}
pub fn initialize_land_actors(_actors: &mut ActorPool, _catalog: &Catalog) {
    initialize_menu_actors(_actors, &_catalog.land_actors, VISIBLE | PROJECTION_ONLY);
}
/// D545/D657 place the menu's records in the first actor slots, descending.
/// These are partial updates: integer X clears its old high byte, while fractions,
/// handler, restart and other unlisted fields survive. This is not a spawn/reset.
fn initialize_menu_actors(actors: &mut ActorPool, placements: &[ActorPlacement], render: u8) {
    for (actor, placement) in actors.iter_mut().zip(placements).rev() {
        {
            let actor_value = ACTIVE;
            actor.set_state_code(actor_value);
        };
        actor.render = render;
        actor.animation = placement.animation;
        actor.position.screen_y = placement.y;
        actor.position.world_x = u16::from(placement.x);
        actor.playback.restart_clock();
        actor.playback.reset_frame();
    }
}
pub fn completion_plan(_input: CompletionInput) -> CompletionPlan {
    use Operation::*;
    let mut operations = Vec::new();
    if _input.completion_actor == 0 {
        if _input.scene_signal == 0 {
            return CompletionPlan {
                operations,
                route: CompletionRoute::AvailableLand,
            };
        }
        operations.extend([Compose, Wait(COMPLETION_PAUSE), TransferDisplayedValue]);
        if _input.progress_bits & ORIGINAL_LAND_BITS != ORIGINAL_LAND_BITS {
            operations.extend([Acknowledge, Message(0), Yield]);
            return CompletionPlan {
                operations,
                route: CompletionRoute::OrdinaryAcknowledgement,
            };
        }
        operations.push(Message(3));
    }
    operations.extend([
        Wait(COMPLETION_PAUSE),
        CompletionActor,
        Message(4),
        Acknowledge,
    ]);
    CompletionPlan {
        operations,
        route: CompletionRoute::LateStage,
    }
}
/// Source's initial scan has eight masks and assumes a usable caller state;
/// only five results are ordinary player selections. No source cap is invented.
pub fn first_available_selector(_progress_bits: u8, _catalog: &Catalog) -> Option<StageId> {
    _catalog
        .completion_masks
        .iter()
        .position(|mask| _progress_bits & mask == 0)
        .map(|index| StageId(index as u8))
}
pub fn position_land(_land: Land, _actors: &mut ActorPool, _catalog: &Catalog) {
    position_selector(_land.source_selector().0, _actors, _catalog, true);
}
pub fn land_input(
    _land: &mut Land,
    _actors: &mut ActorPool,
    _progress_bits: u8,
    _input: PortInput,
    _catalog: &Catalog,
) -> InputResult {
    use crate::runtime::Buttons;
    let mut operations = Vec::new();
    if _input.pressed.0 & 12 != 0 {
        let index = usize::from((_input.pressed.0 >> 1) & 2) / 2;
        let mut selector = _land.source_selector().0;
        // Valid ordinary callers have an unfinished land; skip without yielding.
        loop {
            selector = selector.wrapping_add_signed(_catalog.land_steps[index]);
            if selector >= 5 {
                selector = selector.wrapping_add_signed(_catalog.land_wrap[index]);
            }
            if _progress_bits & _catalog.completion_masks[usize::from(selector)] == 0 {
                break;
            }
        }
        *_land = land_from_selector(selector);
        position_selector(selector, _actors, _catalog, false);
        for slot in [2, 4] {
            _actors[slot].playback.reset_frame();
            _actors[slot].playback.restart_clock();
        }
        operations.push(Operation::Sound(MENU_SOUND));
    } else if _input.pressed.0 & Buttons::A.0 != 0 {
        return InputResult {
            decision: Decision::ConfirmStage,
            operations: vec![],
        };
    }
    operations.extend([Operation::HideSprites, Operation::Compose, Operation::Yield]);
    InputResult {
        decision: Decision::Frame,
        operations,
    }
}
pub fn confirmation_plan() -> Vec<Operation> {
    {
        use Operation::*;
        vec![
            FadeOut,
            ClearGameplay,
            Yield,
            DisableRendering,
            HideSprites,
            ClearActorOne,
            FillFirstNametable,
            LoadStagePresentation,
            ClearNonplayerActors,
            BuildInitialTerrainColumns,
            ScreenScript(STAGE_UI_SCRIPT),
            StageChr,
            BuildInitialTerrainColumns,
            EnterStageRuntime,
        ]
    }
}
pub fn late_stage_request(_late_route: &mut u8, _catalog: &DecodedAssets) -> Result<StageRequest> {
    {
        *_late_route = _late_route.wrapping_add(1);
        let selector = StageId(1);
        Ok(StageRequest {
            level: _catalog
                .original_level(selector)
                .context("late original level")?
                .id
                .clone(),
            selector,
            late_route: *_late_route,
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Acknowledgement {
    /// Suspend exactly one frame before reading the next sampled edge byte.
    WaitOneFrame,
    /// Source tail writes the retained message/transfer submission value to pending,
    /// including zero. This is not an unconditional true upload flag.
    Accepted { primary_upload_pending: u8 },
}
pub fn acknowledge(_input: PortInput, _retained_submission_value: u8) -> Acknowledgement {
    if _input.pressed.0 & crate::runtime::Buttons::A.0 != 0 {
        Acknowledgement::Accepted {
            primary_upload_pending: _retained_submission_value,
        }
    } else {
        Acknowledgement::WaitOneFrame
    }
}
#[cfg(test)]
mod contracts;

/// Indices into B03's base AND active palette, not new palette storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitlePaletteEdits {
    pub before_actors: Vec<(usize, u8)>,
    pub after_actors: Vec<(usize, u8)>,
}
pub fn title_palette_edits() -> TitlePaletteEdits {
    TitlePaletteEdits {
        before_actors: vec![(16, 0x11)],
        after_actors: vec![(26, 0x2c), (30, 0x20), (31, 0x2c)],
    }
}
/// Source copies fixed extents, including embedded sentinel bytes. B03 decides
/// later where composition stops; decoding does not shorten these templates.
pub fn prepare_base_overlays(_stage_ui: &[u8], _main: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    Ok((
        _stage_ui
            .get(..13)
            .context("stage UI template extent")?
            .to_vec(),
        _main
            .get(..21)
            .context("main overlay template extent")?
            .to_vec(),
    ))
}
pub fn prepare_title_overlay(_template: &[u8]) -> Result<Vec<u8>> {
    Ok(_template
        .get(..13)
        .context("title overlay extent")?
        .to_vec())
}

fn land_from_selector(selector: u8) -> Land {
    match selector {
        0 => Land::Amazon,
        1 => Land::Transylvania,
        2 => Land::AfricanMines,
        3 => Land::Himalayas,
        4 => Land::Moon,
        _ => unreachable!("ordinary land selector"),
    }
}
fn position_selector(selector: u8, actors: &mut ActorPool, catalog: &Catalog, initialize: bool) {
    let index = usize::from(selector);
    actors[0].position.screen_y = catalog.cursor_y[index];
    actors[2].animation = LAND_NAME_ANIMATION_BASE.wrapping_add(selector);
    actors[4].animation = actors[2].animation;
    actors[3].position.screen_y = catalog.name_y[index];
    actors[3].position.world_x =
        (actors[3].position.world_x & 0xff00) | u16::from(catalog.name_x[index]);
    if initialize {
        for slot in [0, 2, 4, 3] {
            {
                let actor_value = 0x80;
                actors[slot].set_state_code(actor_value);
            };
        }
        actors[3].render = 0x90;
    }
}

/// Immutable overlapping ROM parameters read by the initial mask scan and raw
/// selector positioning. The source indexes beyond its five ordinary choices;
/// adjacent instruction bytes are data at these bounded consumers, never code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawLandCatalog {
    bytes: Vec<u8>,
}
impl RawLandCatalog {
    const START: usize = LAND_PARAMETER_OFFSET + 4;
    const MASKS: usize = 29;
    const NAME_Y: usize = 37;
    const NAME_X: usize = 42;
    pub fn original(rom: &[u8]) -> Result<Self> {
        Ok(Self {
            bytes: rom
                .get(Self::START..Self::START + Self::NAME_X + 256)
                .context("raw land selector parameter consumers")?
                .to_vec(),
        })
    }
    pub fn first_available(&self, progress: u8) -> StageId {
        for selector in 0..=u8::MAX {
            if progress & self.bytes[Self::MASKS + usize::from(selector)] == 0 {
                return StageId(selector);
            }
        }
        StageId(0) // Source INY wraps and falls through with Y=0.
    }
    pub fn position(&self, selector: StageId, actors: &mut ActorPool, initialize: bool) {
        let index = usize::from(selector.0);
        actors[0].position.screen_y = self.bytes[index];
        actors[2].animation = LAND_NAME_ANIMATION_BASE.wrapping_add(selector.0);
        actors[4].animation = actors[2].animation;
        actors[3].position.screen_y = self.bytes[Self::NAME_Y + index];
        actors[3].position.world_x =
            (actors[3].position.world_x & 0xff00) | u16::from(self.bytes[Self::NAME_X + index]);
        if initialize {
            for slot in [0, 2, 4, 3] {
                {
                    let actor_value = 0x80;
                    actors[slot].set_state_code(actor_value);
                };
            }
            actors[3].render = 0x90;
        }
    }
    /// None means the source's non-yielding skip loop repeats forever. The host
    /// retains that foreground continuation while interrupts continue normally.
    pub fn navigate(
        &self,
        current: StageId,
        progress: u8,
        down: bool,
        catalog: &Catalog,
    ) -> Option<StageId> {
        let mut visited = [false; 256];
        let index = usize::from(down);
        let mut candidate = current.0;
        loop {
            candidate = candidate.wrapping_add_signed(catalog.land_steps[index]);
            if candidate >= 5 {
                candidate = candidate.wrapping_add_signed(catalog.land_wrap[index]);
            }
            if progress & self.bytes[Self::MASKS + usize::from(candidate)] == 0 {
                return Some(StageId(candidate));
            }
            if visited[usize::from(candidate)] {
                return None;
            }
            visited[usize::from(candidate)] = true;
        }
    }
}
