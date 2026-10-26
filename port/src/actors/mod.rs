//! Stable actor identities, lifecycle and immutable spawn definitions (B10).
//! Source-derived lifecycle and spawn ordering are documented in trace.md.
pub mod bosses;
mod catalog;
mod control;
pub use control::{ActorControl, AmbushBehavior, AmbushPhase};
pub mod enemies;
pub mod objects;
pub mod platforms;
pub use crate::collision::{
    ACTIVE, ACTOR_SLOTS, PRESERVED_ACTOR_MARK, SPECIAL_SPAWN_RETRY, VISIBLE,
};
use crate::{
    collision::{Extent, Position},
    identifiers::{AssetId, StageId},
};
use anyhow::{Result, ensure};
pub const PLAYER_SLOT: usize = 0;
pub const FIRST_ACTOR_SLOT: usize = 1;
pub const HANDLER_COUNT: usize = 67;
pub const TEMPLATE_COUNT: usize = 121;
pub const SUPPRESSION_BYTES: usize = 32;
pub const DEFAULT_GRAVITY: u8 = 0x40;
pub const NO_SOURCE_IDENTITY: u8 = 0xff;
pub const FINAL_HANDLER: u8 = 0x3a;
pub const FINAL_ANIMATION: u8 = 0x84;
pub const FINAL_HIT_POINTS: u8 = 5;
pub const CONTACT_ANIMATION: u8 = 0x2a;
pub const MARKED_CONTACT_ANIMATION: u8 = 0x2b;
pub const PRESERVED_ANIMATION: u8 = 0x1b;
pub const PAIRED_FIRST_Y: u8 = 0xc0;
pub const PAIRED_X_TOGGLE: u16 = 0x00f0;
pub const SPECIAL_SOURCE_IDENTITY: u8 = 0xff;
pub const SOURCE_STAGE_CONFIGS: usize = 9;
pub const SUPPRESSION_STAGE_CONFIGS: usize = 5;
/// Canonical mutable actor used by collision/lifecycle/family owners. Presentation
/// borrows a visual projection; neither subsystem allocates another actor array.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActorContact {
    pub position: Position,
    pub horizontal_magnitude: u16,
    pub vertical_velocity: i16,
    pub control: ActorControl,
    pub render: u8,
    pub profile: u8,
    pub handler: u8,
    pub hit_marker: u8,
    pub spawn_identity: Option<u8>,
    pub animation: u8,
    pub playback: crate::presentation::AnimationPlayback,
    pub local_aux2: u8,
    /// Handler-local path cursor (source06E0), borrowed by boss motion.
    pub sequence_cursor: u8,
    /// Shared boss hit interval (source06F0); reset by ordinary spawn.
    pub boss_protection: u8,
    pub spawn_y: u8,
    pub screen_x: i16,
    pub direction: u8,
    pub motion_flags: u8,
}

impl ActorContact {
    pub fn state_code(&self) -> u8 {
        self.control.codes().0
    }
    pub fn local_state_code(&self) -> u8 {
        self.control.codes().1
    }
    pub fn local_counter_code(&self) -> u8 {
        self.control.codes().2
    }
    pub fn set_state_code(&mut self, value: u8) {
        self.control.set_state_code(value);
    }
    pub fn set_local_state_code(&mut self, value: u8) {
        self.control.set_local_state_code(value);
    }
    pub fn set_local_counter_code(&mut self, value: u8) {
        self.control.set_local_counter_code(value);
    }
    pub fn set_handler(&mut self, handler: u8) {
        self.control.select_handler(handler);
        self.handler = handler;
    }
    pub fn select_behavior(&mut self) {
        self.control.select_family(self.handler);
    }
    pub fn ambush_mut(&mut self) -> &mut AmbushBehavior {
        self.control.ambush_mut()
    }
    pub fn advance_ambush(&mut self) {
        self.control.advance_ambush();
    }
    pub fn enter_scene(&mut self) {
        self.control.enter_scene(self.handler);
    }
    pub fn scene_mut(&mut self) -> &mut crate::scenes::SceneActorState {
        self.control.scene_mut(self.handler)
    }
    pub fn scene_for(
        &mut self,
        kind: crate::scenes::SceneHandler,
    ) -> &mut crate::scenes::SceneActorState {
        self.control.scene_mut(kind as u8)
    }
    pub fn advance_scene_bounce(&mut self, forward: bool) {
        self.control.advance_scene_bounce(forward);
    }
    pub fn start_scene_move(&mut self) {
        self.set_handler(crate::scenes::SceneHandler::MoveToMark as u8);
        self.control.start_scene_move();
    }
}

pub type ActorPool = [ActorContact; ACTOR_SLOTS];
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerOwner {
    Lifecycle,
    Enemies,
    Objects,
    Platforms,
    Bosses,
    Dialogue,
    Scenes,
    Finale,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerEntry {
    pub label: String,
    pub owner: HandlerOwner,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Template {
    pub flags_and_local: u8,
    pub animation: u8,
    pub handler: u8,
    pub profile: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpawnRecord {
    pub identity: u8,
    pub world_x: u16,
    pub screen_y: u8,
    pub template: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecialRecord {
    pub page: u8,
    pub paired: bool,
    pub template: usize,
    pub screen_y: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnDefinition {
    pub id: AssetId,
    pub rows: Vec<Vec<SpawnRecord>>,
    pub special_rows: Vec<Vec<SpecialRecord>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnCatalog {
    pub handlers: Vec<HandlerEntry>,
    pub templates: Vec<Template>,
    pub levels: Vec<SpawnDefinition>,
    pub source_levels: Vec<Option<AssetId>>,
    pub extents: [Extent; 32],
    pub vertical_presets: [i16; 32],
    pub horizontal_presets: [u16; 32],
    pub suppression: Vec<Vec<SuppressionRule>>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressField {
    StageBits,
    SourceA1,
    SourceA3,
    SourceA6,
    SourceA8,
    SourceAe,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuppressionRule {
    pub field: ProgressField,
    pub mask: u8,
    pub destination: u8,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SuppressionFlags {
    pub stage_bits: u8,
    pub source_a1: u8,
    pub source_a3: u8,
    pub source_a6: u8,
    pub source_a8: u8,
    pub source_ae: u8,
}
impl SpawnCatalog {
    pub fn original(rom: &[u8]) -> Result<Self> {
        catalog::original(rom)
    }
    pub fn add_named(&mut self, definition: SpawnDefinition) -> Result<()> {
        ensure!(
            !definition.id.0.is_empty()
                && definition
                    .id
                    .0
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_./".contains(&byte)),
            "invalid spawn name"
        );
        ensure!(self.level(&definition.id).is_none(), "duplicate spawn name");
        ensure!(
            definition.rows.len() == definition.special_rows.len(),
            "spawn row join"
        );
        for row in &definition.rows {
            for record in row {
                ensure!(
                    record.template < self.templates.len(),
                    "spawn template bounds"
                );
            }
        }
        for row in &definition.special_rows {
            for record in row {
                ensure!(
                    record.template < self.templates.len(),
                    "special template bounds"
                );
            }
        }
        self.levels.push(definition);
        Ok(())
    }
    pub fn source(&self, stage: StageId) -> Option<&SpawnDefinition> {
        self.source_levels
            .get(usize::from(stage.0))
            .and_then(Option::as_ref)
            .and_then(|id| self.level(id))
    }
    pub fn level(&self, id: &AssetId) -> Option<&SpawnDefinition> {
        self.levels.iter().find(|level| level.id == *id)
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SpecialControl {
    #[default]
    Ready,
    Cooldown(u8),
    Occupied(usize),
}
impl SpecialControl {
    pub fn decode(source: u8) -> Self {
        if source & ACTIVE != 0 {
            Self::Occupied(usize::from(source & 0x7f))
        } else if source == 0 {
            Self::Ready
        } else {
            Self::Cooldown(source)
        }
    }
    pub fn encode(self) -> u8 {
        match self {
            Self::Ready => 0,
            Self::Cooldown(ticks) => ticks,
            Self::Occupied(slot) => ACTIVE | slot as u8,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpawnContext {
    pub stage: u8,
    pub room_row: u8,
    pub camera: u16,
    pub direction: u8,
    pub split_transition: u8,
    pub late_mode: u8,
    pub late_route: u8,
    pub stage_progress: u8,
    pub boss_hit_points: u8,
    pub special: SpecialControl,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Window {
    pub room_start: u8,
    pub upper: u8,
    pub lower: u8,
    pub direction: u8,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SpawnReport {
    pub ordinary: Vec<(u8, usize)>,
    pub special: Vec<usize>,
    pub terrain_replacements: Vec<usize>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationRegister {
    X,
    Y,
}
pub fn allocate(actors: &ActorPool, _register: AllocationRegister) -> Option<usize> {
    (FIRST_ACTOR_SLOT..ACTOR_SLOTS)
        .rev()
        .find(|&slot| actors[slot].state_code() & ACTIVE == 0)
}
pub fn dispatch(
    actors: &mut ActorPool,
    gravity: &mut u8,
    handler: &mut dyn FnMut(usize, u8, &mut ActorPool, &mut u8),
) -> Vec<(usize, u8)> {
    *gravity = DEFAULT_GRAVITY;
    let mut visits = Vec::new();
    for slot in FIRST_ACTOR_SLOT..ACTOR_SLOTS {
        if actors[slot].state_code() & ACTIVE != 0 {
            let selector = actors[slot].handler;
            visits.push((slot, selector));
            handler(slot, selector, actors, gravity);
        }
    }
    visits
}
pub fn remove(
    actors: &mut ActorPool,
    slot: usize,
    special: &mut SpecialControl,
    clear_render: bool,
    state_value: u8,
) {
    {
        let actor_value = state_value;
        actors[slot].set_state_code(actor_value);
    };
    actors[slot].spawn_identity = None;
    if clear_render {
        actors[slot].render = 0;
    }
    if *special == SpecialControl::Occupied(slot) {
        *special = SpecialControl::Cooldown(SPECIAL_SPAWN_RETRY);
    }
}
pub fn suppress(bits: &mut [u8; SUPPRESSION_BYTES], identity: u8) {
    const BITS_PER_BYTE: u8 = 8;
    bits[usize::from(identity / BITS_PER_BYTE)] |= 1 << (identity % BITS_PER_BYTE);
}
pub fn rebuild_suppression(
    bits: &mut [u8; SUPPRESSION_BYTES],
    rules: &[SuppressionRule],
    flags: SuppressionFlags,
) {
    bits.fill(0);
    add_suppression(bits, rules, flags);
}
/// Dialogue ED49 calls actual bank5 B76A, past the full reset loop. Apply the
/// selected rules to the canonical bitmap while preserving all prior bits.
pub fn add_suppression(
    bits: &mut [u8; SUPPRESSION_BYTES],
    rules: &[SuppressionRule],
    flags: SuppressionFlags,
) {
    for rule in rules {
        let value = match rule.field {
            ProgressField::StageBits => flags.stage_bits,
            ProgressField::SourceA1 => flags.source_a1,
            ProgressField::SourceA3 => flags.source_a3,
            ProgressField::SourceA6 => flags.source_a6,
            ProgressField::SourceA8 => flags.source_a8,
            ProgressField::SourceAe => flags.source_ae,
        };
        if value & rule.mask != 0 {
            suppress(bits, rule.destination);
        }
    }
}
pub fn reset_window(room_start: u8, x_high: &[u8], camera: u16) -> Window {
    const FORWARD: u8 = 1;
    let mut cursor = room_start;
    while let Some(&high) = x_high.get(usize::from(cursor)) {
        if high & ACTIVE != 0 || high <= (camera >> 8) as u8 {
            break;
        }
        cursor = cursor.wrapping_add(1);
        if cursor == 0 {
            break;
        }
    }
    Window {
        room_start,
        upper: cursor,
        lower: cursor,
        direction: FORWARD,
    }
}
pub fn candidate(
    actors: &mut ActorPool,
    bits: &[u8; SUPPRESSION_BYTES],
    record: SpawnRecord,
    templates: &[Template],
    context: &mut SpawnContext,
) -> SpawnReport {
    const PERSISTENT: u8 = 0x10;
    const NORMALIZE_ANIMATION: u8 = 0x80;
    const ALL_LANDS: u8 = 0x1f;
    const TRANSYLVANIA: u8 = 1;
    const TRANSYLVANIA_PROGRESS: u8 = 2;
    const TERRAIN_HANDLERS: [u8; 2] = [8, 10];
    const SUPPRESSED_HANDLERS: [u8; 2] = [0x14, 0x15];
    let mut report = SpawnReport::default();
    if bits[usize::from(record.identity >> 3)] & (1 << (record.identity & 7)) != 0
        || record.world_x < context.camera
    {
        return report;
    }
    if let Some(slot) = (FIRST_ACTOR_SLOT..ACTOR_SLOTS)
        .rev()
        .find(|&slot| actors[slot].spawn_identity.unwrap_or(NO_SOURCE_IDENTITY) == record.identity)
        && actors[slot].state_code() & ACTIVE != 0
    {
        return report;
    }
    let Some(slot) = allocate(actors, AllocationRegister::X) else {
        return report;
    };
    let template = templates[record.template];
    let actor = &mut actors[slot];
    actor.spawn_identity = Some(record.identity);
    actor.position.world_x = record.world_x;
    actor.position.screen_y = record.screen_y;
    actor.spawn_y = record.screen_y;
    actor.hit_marker = 0;
    {
        let actor_value = 0;
        actor.set_local_counter_code(actor_value);
    };
    actor.boss_protection = 0;
    {
        let actor_value = template.flags_and_local & 0x0f;
        actor.set_local_state_code(actor_value);
    };
    {
        let actor_value = template.flags_and_local & 0xf0;
        actor.set_state_code(actor_value);
    };
    actor.render = actor.state_code();
    actor.animation = template.animation;
    {
        let actor_value = template.handler;
        actor.set_handler(actor_value);
    };
    actor.profile = template.profile;
    actor.playback.reset_frame();
    actor.playback.restart_clock();
    report.ordinary.push((record.identity, slot));
    if context.split_transition == 0 && actor.state_code() & PERSISTENT != 0 {
        if actor.animation & NORMALIZE_ANIMATION != 0 {
            actor.animation &= !NORMALIZE_ANIMATION;
            {
                let actor_value = u8::MAX;
                actor.set_local_counter_code(actor_value);
            };
        }
        if TERRAIN_HANDLERS.contains(&actor.handler) {
            report.terrain_replacements.push(slot);
            return report;
        }
    }
    if context.late_mode & ACTIVE != 0 {
        context.boss_hit_points = FINAL_HIT_POINTS;
        if context.late_route != 0 || context.stage_progress & ALL_LANDS == ALL_LANDS {
            {
                let actor_value = FINAL_HANDLER;
                actor.set_handler(actor_value);
            };
            actor.profile = 0;
            {
                let actor_value = actor.state_code() | (PERSISTENT);
                actor.set_state_code(actor_value);
            };
            actor.animation = FINAL_ANIMATION;
        }
    }
    if context.stage == TRANSYLVANIA
        && context.stage_progress & TRANSYLVANIA_PROGRESS != 0
        && (SUPPRESSED_HANDLERS.contains(&actor.handler)
            || actor.handler == 8 && actor.animation == PRESERVED_ANIMATION)
    {
        {
            let actor_value = 0;
            actor.set_state_code(actor_value);
        };
    }
    report
}
pub fn scan_window(
    actors: &mut ActorPool,
    bits: &[u8; SUPPRESSION_BYTES],
    records: &[SpawnRecord],
    templates: &[Template],
    window: &mut Window,
    context: &mut SpawnContext,
) -> SpawnReport {
    scan_window_with(
        actors,
        bits,
        records,
        templates,
        window,
        context,
        &mut |_, _, _| {},
    )
}
/// Commit each candidate's terrain/recovery effects before scanning another
/// candidate. The callback receives the same live pool and source context.
pub fn scan_window_with(
    actors: &mut ActorPool,
    bits: &[u8; SUPPRESSION_BYTES],
    records: &[SpawnRecord],
    templates: &[Template],
    window: &mut Window,
    context: &mut SpawnContext,
    committed: &mut dyn FnMut(&mut ActorPool, &mut SpawnContext, &SpawnReport),
) -> SpawnReport {
    scan_window_extent_with(
        actors, bits, records, templates, window, context, 255, committed,
    )
}
/// Native viewport variant; cursors and candidate side effects retain the same
/// ordering while the interval may span more than one source screen.
#[allow(clippy::too_many_arguments)]
pub fn scan_window_extent_with(
    actors: &mut ActorPool,
    bits: &[u8; SUPPRESSION_BYTES],
    records: &[SpawnRecord],
    templates: &[Template],
    window: &mut Window,
    context: &mut SpawnContext,
    visible_interval: u16,
    committed: &mut dyn FnMut(&mut ActorPool, &mut SpawnContext, &SpawnReport),
) -> SpawnReport {
    const FORWARD: u8 = 1;
    const DIRECTION_MASK: u8 = 3;
    let mut report = SpawnReport::default();
    if window.direction & DIRECTION_MASK == 0 || records.is_empty() {
        return report;
    }
    let high = context.camera.wrapping_add(visible_interval);
    let at = |cursor: u8| {
        records
            .iter()
            .find(|record| record.identity == cursor)
            .copied()
    };
    if window.direction & FORWARD != 0 {
        while let Some(record) = at(window.upper) {
            if record.world_x > high {
                break;
            }
            // Wider views can exhaust the original pool. Retain the cursor
            // until a slot opens instead of permanently losing this entry.
            if visible_interval > 255 && allocate(actors, AllocationRegister::X).is_none() {
                break;
            }
            let candidate = candidate(actors, bits, record, templates, context);
            committed(actors, context, &candidate);
            merge_report(&mut report, candidate);
            window.upper = window.upper.wrapping_add(1);
            if window.upper == 0 {
                break;
            }
        }
        while let Some(record) = at(window.lower) {
            if record.world_x > context.camera {
                break;
            }
            window.lower = window.lower.wrapping_add(1);
            if window.lower == 0 {
                break;
            }
        }
    } else {
        while window.lower != window.room_start {
            let previous = window.lower.wrapping_sub(1);
            let Some(record) = at(previous) else {
                break;
            };
            if record.world_x < context.camera {
                break;
            }
            // Wider views can exhaust the original pool. Retain the cursor
            // until a slot opens instead of permanently losing this entry.
            if visible_interval > 255 && allocate(actors, AllocationRegister::X).is_none() {
                break;
            }
            let candidate = candidate(actors, bits, record, templates, context);
            committed(actors, context, &candidate);
            merge_report(&mut report, candidate);
            window.lower = previous;
            if window.lower == 0 {
                break;
            }
        }
        while window.upper != window.room_start {
            let previous = window.upper.wrapping_sub(1);
            let Some(record) = at(previous) else {
                break;
            };
            if record.world_x < high {
                break;
            }
            window.upper = previous;
        }
    }
    report
}
pub fn special_spawn(
    actors: &mut ActorPool,
    rows: &[Vec<SpecialRecord>],
    templates: &[Template],
    context: &mut SpawnContext,
) -> SpawnReport {
    special_spawn_edges(actors, rows, templates, context, None)
}
/// Optional native view bounds move ambient edge spawns with the visible world.
pub fn special_spawn_edges(
    actors: &mut ActorPool,
    rows: &[Vec<SpecialRecord>],
    templates: &[Template],
    context: &mut SpawnContext,
    bounds: Option<(u16, u16)>,
) -> SpawnReport {
    const BACKWARD_BIT: u8 = 2;
    const FORWARD_EDGE: u16 = 0x00f8;
    const BACKWARD_EDGE: u16 = 0x0008;
    let mut report = SpawnReport::default();
    if context.late_mode & ACTIVE != 0 {
        return report;
    }
    match context.special {
        SpecialControl::Occupied(_) => return report,
        SpecialControl::Cooldown(ticks) => {
            context.special = SpecialControl::decode(ticks.wrapping_sub(1));
            return report;
        }
        SpecialControl::Ready => (),
    }
    let edge = if let Some((left, right)) = bounds {
        if context.direction & BACKWARD_BIT != 0 {
            left.saturating_add(8)
        } else {
            right.saturating_sub(7)
        }
    } else {
        context
            .camera
            .wrapping_add(if context.direction & BACKWARD_BIT != 0 {
                BACKWARD_EDGE
            } else {
                FORWARD_EDGE
            })
    };
    let Some(record) = rows
        .get(usize::from(context.room_row))
        .and_then(|row| row.iter().find(|record| record.page == (edge >> 8) as u8))
    else {
        return report;
    };
    let mut create = |actors: &mut ActorPool, edge: u16| -> Option<usize> {
        let slot = allocate(actors, AllocationRegister::X)?;
        let template = templates[record.template];
        let actor = &mut actors[slot];
        {
            let actor_value = template.flags_and_local;
            actor.set_state_code(actor_value);
        };
        actor.render = template.flags_and_local;
        actor.animation = template.animation;
        {
            let actor_value = template.handler;
            actor.set_handler(actor_value);
        };
        actor.profile = template.profile;
        actor.playback.restart_clock();
        actor.playback.reset_frame();
        actor.hit_marker = 0;
        {
            let actor_value = 0;
            actor.set_local_state_code(actor_value);
        };
        {
            let actor_value = 0;
            actor.set_local_counter_code(actor_value);
        };
        actor.position.world_x = edge;
        actor.position.screen_y = record.screen_y;
        actor.spawn_identity = None;
        context.special = SpecialControl::Occupied(slot);
        report.special.push(slot);
        Some(slot)
    };
    let first = create(actors, edge);
    if record.paired {
        // A failed first allocation leaves the source index zero. The subsequent
        // unconditional Y write therefore affects the player even in a full pool.
        actors[first.unwrap_or(PLAYER_SLOT)].position.screen_y = PAIRED_FIRST_Y;
        create(actors, edge ^ PAIRED_X_TOGGLE);
    }
    report
}
/// Foreground AEA0 integration: special effects occur before ordinary cursor work.
pub fn update_window(
    actors: &mut ActorPool,
    bits: &[u8; SUPPRESSION_BYTES],
    definition: &SpawnDefinition,
    templates: &[Template],
    window: &mut Window,
    context: &mut SpawnContext,
) -> SpawnReport {
    let mut report = special_spawn(actors, &definition.special_rows, templates, context);
    if let Some(records) = definition.rows.get(usize::from(context.room_row)) {
        merge_report(
            &mut report,
            scan_window(actors, bits, records, templates, window, context),
        );
    }
    report
}
pub fn actor_overlap(
    actors: &mut ActorPool,
    source: usize,
    extents: &[Extent; 32],
) -> Option<usize> {
    let from = actors[source];
    if from.render & VISIBLE == 0 {
        return None;
    }
    let extent = extents[usize::from(from.profile & 0x7f)];
    let slot = (FIRST_ACTOR_SLOT..ACTOR_SLOTS).rev().find(|&slot| {
        let target = &actors[slot];
        slot != source
            && target.render & VISIBLE != 0
            && from.position.screen_y.abs_diff(target.position.screen_y) < extent.half_height
            && from.position.world_x.abs_diff(target.position.world_x)
                <= u16::from(extent.half_width)
    })?;
    const CONTACT_EXEMPT_HANDLERS: [u8; 3] = [8, 9, 0x25];
    let exempt = CONTACT_EXEMPT_HANDLERS.contains(&actors[slot].handler);
    if !exempt {
        actors[slot].hit_marker = actors[slot].hit_marker.wrapping_add(1);
    }
    if actors[source].animation != PRESERVED_ANIMATION {
        actors[source].animation = if exempt {
            CONTACT_ANIMATION
        } else {
            MARKED_CONTACT_ANIMATION
        };
        actors[source].playback.reset_frame();
        actors[source].playback.restart_clock();
    }
    Some(slot)
}
pub fn motion_preset(actor: &mut ActorContact, catalog: &SpawnCatalog, preset: usize) {
    actor.vertical_velocity = catalog.vertical_presets[preset];
    actor.horizontal_magnitude = catalog.horizontal_presets[preset];
}
pub fn face_player(actors: &mut ActorPool, slot: usize) {
    const TOWARD_RIGHT: u8 = 1;
    const TOWARD_LEFT: u8 = 2;
    actors[slot].direction = if actors[slot].position.world_x < actors[PLAYER_SLOT].position.world_x
    {
        TOWARD_RIGHT
    } else {
        TOWARD_LEFT
    };
}
pub fn player_distance(actors: &ActorPool, slot: usize) -> (u8, u8) {
    let player = actors[PLAYER_SLOT].position;
    let actor = actors[slot].position;
    (
        player.world_x.abs_diff(actor.world_x).min(255) as u8,
        player.screen_y.abs_diff(actor.screen_y),
    )
}
#[cfg(test)]
mod contracts;

fn merge_report(into: &mut SpawnReport, other: SpawnReport) {
    into.ordinary.extend(other.ordinary);
    into.special.extend(other.special);
    into.terrain_replacements.extend(other.terrain_replacements);
}

/// Actual immutable parallel-byte consumers used by source window reset. These
/// retain global record identities and negative sentinels beyond per-room views.
#[derive(Clone)]
pub struct SpawnPlanes {
    pub x_high: [u8; 256],
    pub x_low: [u8; 256],
    pub room_starts: [u8; 256],
    pub y: [u8; 256],
    pub templates: [u8; 256],
}
impl SpawnPlanes {
    pub fn original(rom: &[u8], stage: StageId) -> Result<Self> {
        use anyhow::Context as _;
        const POINTERS: [(usize, usize); 5] = [
            (0x1b060, 0x1b069),
            (0x1b072, 0x1b07b),
            (0x1b084, 0x1b08d),
            (0x1b096, 0x1b09f),
            (0x1b0a8, 0x1b0b1),
        ];
        let mut planes = Vec::new();
        for (low, high) in POINTERS {
            let index = usize::from(stage.0);
            let pointer = u16::from_le_bytes([
                *rom.get(low + index).context("spawn pointer low")?,
                *rom.get(high + index).context("spawn pointer high")?,
            ]);
            let mut plane = [0; 256];
            for (index, byte) in plane.iter_mut().enumerate() {
                let address = usize::from(pointer.wrapping_add(index as u16));
                ensure!(address >= 0x8000, "spawn plane requires a non-ROM consumer");
                let physical = if address >= 0xc000 {
                    0x1c010 + address - 0xc000
                } else {
                    0x18010 + address - 0x8000
                };
                *byte = *rom.get(physical).context("spawn plane data")?;
            }
            planes.push(plane);
        }
        Ok(Self {
            x_high: planes[0],
            x_low: planes[1],
            room_starts: planes[2],
            y: planes[3],
            templates: planes[4],
        })
    }
}
