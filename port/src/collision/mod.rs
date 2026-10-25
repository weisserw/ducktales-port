//! Shared motion/contact domain contracts. See trace.md for exact source rules.
//! These are borrowed actor snapshots/effects, not another actor store or cache.
use crate::world::{TerrainProbe, WorldCache};
use anyhow::{Context, Result};
/// Latest semantic collision return shared by player and ascending actors.
/// A movement that skips probing preserves the preceding return.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CollisionOutcome {
    pub bits: u8,
}
impl CollisionOutcome {
    pub fn player_aggregate(&mut self, bits: u8) {
        self.bits = bits;
    }
    pub fn movement(&mut self, report: &MovementReport) -> u8 {
        if let Some(probe) = &report.probe {
            self.bits = probe.bits;
        }
        if report.contact.is_some() {
            self.bits |= SOLID_CONTACT;
        }
        self.bits
    }
}
pub const ACTOR_SLOTS: usize = 16;
pub const SAMPLE_GEOMETRIES: usize = 101;
pub const COLLISION_PROFILES: usize = 32;
pub const ACTIVE: u8 = 0x80;
pub const VISIBLE: u8 = 0x80;
pub const SKIP_VERTICAL_PLAYER_SCAN: u8 = 0x02;
pub const TERRAIN_MARK: u8 = 0x04;
pub const FACE_POSITIVE: u8 = 0x40;
pub const REACTION_PROFILE: u8 = 0x70;
pub const PROFILE_ENABLED: u8 = 0x80;
pub const SOLID_CONTACT: u8 = 0x01;
pub const SECONDARY_CONTACT: u8 = 0x02;
pub const SOLID_HANDLER: u8 = 0x08;
pub const OVERLAP_EXEMPT_HANDLER: u8 = 0x09;
pub const POSITIVE_ONLY_EXEMPT_HANDLER: u8 = 0x24;
pub const PROTECTION_CONTACT_EXCEPTION: u8 = 0x15;
pub const DAMAGE_STATE: u8 = 0x06;
pub const POSE_TERRAIN_STATE: u8 = 0x07;
pub const LOW_EXIT_STATE: u8 = 0x08;
pub const SPECIAL_POSE: u8 = 0x03;
pub const POGO_POSE: u8 = 0x06;
pub const DESCENDING_MOTION: u8 = 0x04;
pub const LOW_EXIT_Y: u8 = 0xd0;
pub const POSE_TERRAIN_SAMPLE: u8 = 0x07;
pub const DAMAGE_POGO_SAMPLE: u8 = 0x0a;
pub const REACTION_SAMPLE_REMAINING: u8 = 0x01;
pub const SPECIAL_SPAWN_RETRY: u8 = 0x3c;
pub const TERRAIN_PARTICLE_ANIMATION: u8 = 0x2a;
pub const TERRAIN_PARTICLE_STATE: u8 = 0xa0;
pub const TERRAIN_PARTICLE_RENDER: u8 = 0xa8;
pub const TERRAIN_PARTICLE_SOUND: u8 = 0x16;
pub const PRESERVED_ACTOR_MARK: u8 = 0x10;
/// A named recipe includes the two extra bytes read by the reachable selector1
/// caller, although they also encode the following LDA instruction. No execution.
pub const PRESET_CONSUMER_PHYSICAL_START: usize = 0x1f340;
pub const PRESET_CONSUMER_BYTES: usize = 4;
pub const SAMPLE_Y_PHYSICAL_START: usize = 0x1d3de;
pub const SAMPLE_X_PHYSICAL_START: usize = 0x1d443;
pub const HEIGHTS_PHYSICAL_START: usize = 0x1f4a8;
pub const WIDTHS_PHYSICAL_START: usize = 0x1f4c8;
pub const RESPONSE_PHYSICAL_START: usize = 0x1d1f2;
pub const PARTICLES_PHYSICAL_START: usize = 0x13dd7;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SampleOffset {
    pub x: i8,
    pub y: i8,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Extent {
    pub half_width: u8,
    pub half_height: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainResponse {
    Pose,
    Mark,
    Momentum,
    Damage,
    LowExit,
    MarkMomentum,
    Solid,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Geometry {
    pub offsets: Vec<SampleOffset>,
    pub extents: [Extent; COLLISION_PROFILES],
    pub responses: [TerrainResponse; 16],
    pub velocity_presets: [u16; 2],
    pub particle_y: [u8; 4],
    pub particle_x: [u8; 4],
    pub particle_ticks: [u8; 4],
}
impl Geometry {
    pub fn original(rom: &[u8]) -> Result<Self> {
        let data = |start: usize, length: usize| {
            rom.get(start..start + length)
                .context("collision recipe bounds")
        };
        let xs = data(SAMPLE_X_PHYSICAL_START, SAMPLE_GEOMETRIES)?;
        let ys = data(SAMPLE_Y_PHYSICAL_START, SAMPLE_GEOMETRIES)?;
        let offsets = xs
            .iter()
            .zip(ys)
            .map(|(&x, &y)| SampleOffset {
                x: x as i8,
                y: y as i8,
            })
            .collect();
        let widths = data(WIDTHS_PHYSICAL_START, COLLISION_PROFILES)?;
        let heights = data(HEIGHTS_PHYSICAL_START, COLLISION_PROFILES)?;
        let extents = std::array::from_fn(|index| Extent {
            half_width: widths[index],
            half_height: heights[index],
        });
        use TerrainResponse::*;
        const RESPONSE_TABLE: [TerrainResponse; 16] = [
            Pose,
            Pose,
            Pose,
            Pose,
            Mark,
            Mark,
            Mark,
            Mark,
            Momentum,
            Damage,
            LowExit,
            LowExit,
            Damage,
            Damage,
            MarkMomentum,
            Solid,
        ];
        let presets = data(PRESET_CONSUMER_PHYSICAL_START, PRESET_CONSUMER_BYTES)?;
        let velocity_presets = std::array::from_fn(|index| {
            u16::from_le_bytes([presets[index * 2], presets[index * 2 + 1]])
        });
        Ok(Self {
            offsets,
            extents,
            responses: RESPONSE_TABLE,
            velocity_presets,
            particle_y: data(PARTICLES_PHYSICAL_START, 4)?.try_into()?,
            particle_x: data(PARTICLES_PHYSICAL_START + 4, 4)?.try_into()?,
            particle_ticks: data(PARTICLES_PHYSICAL_START + 8, 4)?.try_into()?,
        })
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Position {
    pub world_x: u16,
    pub x_fraction: u8,
    pub screen_y: u8,
    pub y_fraction: u8,
}
pub use crate::actors::ActorContact;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerContact {
    pub animation: u8,
    pub protection: u8,
    pub special_mode: u8,
    pub momentum_control: u8,
    pub immediate_state: u8,
    pub queued_state: u8,
    pub actor_contact_flags: u8,
    pub motion: u8,
    pub vertical_velocity: i16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalDirection {
    Positive,
    Negative,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalOperation {
    AddEncodedVelocity,
    SubtractEncodedVelocity,
}
/// Reports full24-bit horizontal overflow/underflow, not mere low-byte crossing.
pub fn integrate_horizontal(actor: &mut ActorContact, direction: HorizontalDirection) -> bool {
    const POSITION_MASK: u32 = 0x00ff_ffff;
    let before = u32::from(actor.position.world_x) << 8 | u32::from(actor.position.x_fraction);
    let speed = u32::from(actor.horizontal_magnitude);
    let (next, boundary) = match direction {
        HorizontalDirection::Positive => {
            (before.wrapping_add(speed), before + speed > POSITION_MASK)
        }
        HorizontalDirection::Negative => (before.wrapping_sub(speed), before < speed),
    };
    actor.position.world_x = (next >> 8) as u16;
    actor.position.x_fraction = next as u8;
    boundary
}
/// Reports unsigned16-bit add overflow/subtract borrow. Signed physical crossing
/// is distinct: falling wrappers remove on *no borrow*, rising wrappers ignore it.
pub fn integrate_vertical(actor: &mut ActorContact, operation: VerticalOperation) -> bool {
    let before = u16::from(actor.position.screen_y) << 8 | u16::from(actor.position.y_fraction);
    let velocity = actor.vertical_velocity as u16;
    let (next, boundary) = match operation {
        VerticalOperation::AddEncodedVelocity => before.overflowing_add(velocity),
        VerticalOperation::SubtractEncodedVelocity => before.overflowing_sub(velocity),
    };
    actor.position.screen_y = (next >> 8) as u8;
    actor.position.y_fraction = next as u8;
    boundary
}
pub fn gravity(actor: &mut ActorContact, fractional_step: u8) {
    actor.vertical_velocity = actor
        .vertical_velocity
        .wrapping_sub(i16::from(fractional_step));
}
pub fn set_velocity_preset(
    actor: &mut ActorContact,
    geometry: &Geometry,
    selector: usize,
    vertical: bool,
) {
    if vertical {
        actor.vertical_velocity = geometry.velocity_presets[selector] as i16;
    } else {
        actor.horizontal_magnitude = geometry.velocity_presets[selector];
    }
}
/// Read-only collision cells borrowed from their canonical cache owner. Family
/// adapters may split this slice from changed/attribute fields without copying
/// mutable cache state or creating another memory representation.
pub trait CollisionCells {
    fn collision_cell(&self, index: usize) -> u8;
}
impl CollisionCells for WorldCache {
    fn collision_cell(&self, index: usize) -> u8 {
        self.collision[index]
    }
}
impl CollisionCells for [u8; crate::world::CACHE_CELLS] {
    fn collision_cell(&self, index: usize) -> u8 {
        self[index]
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sample {
    pub class: u8,
    pub world_x: u16,
    pub screen_y: u8,
    pub cache_index: Option<usize>,
    pub quadrant: Option<u8>,
    pub descriptor: Option<TerrainProbe>,
}
pub fn sample(
    cache: &dyn CollisionCells,
    position: Position,
    room_row: u8,
    offset: SampleOffset,
) -> Sample {
    const CACHE_PAGE_CELLS: usize = 64;
    const CACHE_ROW_CELLS: usize = 8;
    const BLOCK_PIXELS_SHIFT: u8 = 5;
    let world_x = position.world_x.wrapping_add_signed(i16::from(offset.x));
    let vertical = i16::from(position.screen_y) + i16::from(offset.y);
    let screen_y = vertical as u8;
    let mut result = Sample {
        class: 0,
        world_x,
        screen_y,
        cache_index: None,
        quadrant: None,
        descriptor: None,
    };
    if world_x & 0x8000 != 0 {
        return result;
    }
    if !(0..=255).contains(&vertical) {
        result.class = u8::from(offset.y < 0);
        return result;
    }
    let page = (world_x >> 8) as u8;
    let coordinate = usize::from(screen_y >> BLOCK_PIXELS_SHIFT) * CACHE_ROW_CELLS
        + usize::from((world_x as u8) >> BLOCK_PIXELS_SHIFT);
    let index = usize::from(page & 1) * CACHE_PAGE_CELLS + coordinate;
    let quadrant = ((screen_y >> 3) & 2) | ((world_x as u8 >> 4) & 1);
    result.class = (cache.collision_cell(index) >> (quadrant * 2)) & 3;
    result.cache_index = Some(index);
    result.quadrant = Some(quadrant);
    result.descriptor = Some(TerrainProbe {
        room_row,
        world_page: page,
        block_coordinate: coordinate as u8,
        quadrant,
    });
    result
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbeRange {
    pub start: u8,
    pub remaining: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeReport {
    pub bits: u8,
    pub results: Vec<(u8, u8)>,
    pub visited: Vec<u8>,
    pub final_range: ProbeRange,
    pub last_sample: Option<Sample>,
}
// Contract boundary keeps each canonical owner borrow explicit.
#[allow(clippy::too_many_arguments)]
pub fn probe(
    slot: usize,
    actor: &mut ActorContact,
    player: &mut PlayerContact,
    cache: &dyn CollisionCells,
    row: u8,
    geometry: &Geometry,
    range: ProbeRange,
    descriptor: &mut dyn FnMut(TerrainProbe) -> u8,
) -> ProbeReport {
    actor.render &= !TERRAIN_MARK;
    let mut report = ProbeReport {
        bits: 0,
        results: Vec::new(),
        visited: Vec::new(),
        final_range: range,
        last_sample: None,
    };
    loop {
        let range = report.final_range;
        let sample = sample(
            cache,
            actor.position,
            row,
            geometry.offsets[usize::from(range.start)],
        );
        report.visited.push(range.start);
        report.results.push((range.remaining, sample.class));
        // CollisionSampleResults[1] and PlayerActorContactFlags are the same
        // source byte. This applies to every actor's probe, including class3's
        // raw sample before its terrain response. Ranges omitting1 retain it.
        if range.remaining == 1 {
            player.actor_contact_flags = sample.class;
        }
        report.last_sample = Some(sample);
        let mut actor_early_exit;
        if sample.class != 3 {
            report.bits |= sample.class;
            actor_early_exit = report.bits & SOLID_CONTACT != 0;
        } else {
            let selector = descriptor(sample.descriptor.expect("class3 has a descriptor")) >> 4;
            let response = geometry.responses[usize::from(selector)];
            let mut solid = true;
            actor_early_exit = true;
            match response {
                TerrainResponse::Solid => (),
                TerrainResponse::Mark => {
                    actor.render |= TERRAIN_MARK;
                    solid = false;
                    actor_early_exit = actor.render & 1 != 0;
                }
                TerrainResponse::Damage => {
                    if slot == 0
                        && player.protection == 0
                        && player.special_mode == 0
                        && range.remaining == REACTION_SAMPLE_REMAINING
                        && (player.animation != POGO_POSE || range.start == DAMAGE_POGO_SAMPLE)
                    {
                        player.queued_state = DAMAGE_STATE;
                    }
                }
                TerrainResponse::Pose => {
                    if slot == 0
                        && range.start == POSE_TERRAIN_SAMPLE
                        && player.animation == POGO_POSE
                    {
                        player.queued_state = POSE_TERRAIN_STATE;
                        solid = false;
                    }
                }
                TerrainResponse::LowExit => {
                    solid = false;
                    if slot == 0 && actor.position.screen_y >= LOW_EXIT_Y {
                        player.queued_state = LOW_EXIT_STATE;
                    }
                }
                TerrainResponse::Momentum | TerrainResponse::MarkMomentum => {
                    if response == TerrainResponse::MarkMomentum {
                        actor.render |= TERRAIN_MARK;
                    }
                    if slot == 0 && range.remaining == REACTION_SAMPLE_REMAINING {
                        player.momentum_control = player.momentum_control.wrapping_add(1);
                        if player.momentum_control == 0 {
                            solid = false;
                            if actor.position.screen_y >= LOW_EXIT_Y {
                                player.queued_state = LOW_EXIT_STATE;
                            }
                        }
                    }
                }
            }
            if solid {
                report.bits |= SOLID_CONTACT;
                actor_early_exit = report.bits & SOLID_CONTACT != 0;
            }
        }
        if slot != 0 && actor_early_exit {
            break;
        }
        report.final_range.start = range.start.wrapping_add(1);
        report.final_range.remaining = range.remaining.wrapping_sub(1);
        if report.final_range.remaining & 0x80 != 0 {
            break;
        }
    }
    report
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Snap {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
}
pub fn snap(actor: &mut ActorContact, sample: Sample, direction: Snap) {
    const TILE_PIXELS: u8 = 16;
    let x_remainder = sample.world_x as u8 & (TILE_PIXELS - 1);
    let y_remainder = sample.screen_y & (TILE_PIXELS - 1);
    match direction {
        Snap::PositiveX => {
            actor.position.world_x = actor.position.world_x.wrapping_sub(u16::from(x_remainder))
        }
        Snap::NegativeX => {
            actor.position.world_x = actor
                .position
                .world_x
                .wrapping_add(u16::from(TILE_PIXELS - x_remainder))
        }
        Snap::PositiveY => {
            actor.position.screen_y = actor.position.screen_y.wrapping_sub(y_remainder)
        }
        Snap::NegativeY => {
            actor.position.screen_y = actor
                .position
                .screen_y
                .wrapping_add(TILE_PIXELS - y_remainder)
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactAxis {
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Contact {
    pub slot: usize,
    pub penetration: u8,
}
/// Carry-clear source success becomes Some; source slot identity is preserved.
pub fn solid_overlap(
    source_slot: usize,
    actors: &[ActorContact; ACTOR_SLOTS],
    geometry: &Geometry,
    axis: ContactAxis,
) -> Option<Contact> {
    let source = &actors[source_slot];
    if source.render & VISIBLE == 0 {
        return None;
    }
    (1..ACTOR_SLOTS).rev().find_map(|slot| {
        let target = &actors[slot];
        if slot == source_slot
            || target.state_code() & ACTIVE == 0
            || target.render & VISIBLE == 0
            || target.handler != SOLID_HANDLER
        {
            return None;
        }
        let extent = geometry.extents[usize::from(target.profile & 0x7f)];
        let dx = horizontal_separation(source.position.world_x, target.position.world_x);
        let dy = source.position.screen_y.abs_diff(target.position.screen_y);
        contact_penetration(axis, dx, dy, extent).map(|penetration| Contact { slot, penetration })
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerScan {
    pub contact_count: u8,
    pub last_contact: Option<Contact>,
    pub bits: u8,
    pub any_contact: bool,
}
pub fn scan_player(
    axis: ContactAxis,
    actors: &mut [ActorContact; ACTOR_SLOTS],
    player: &mut PlayerContact,
    geometry: &Geometry,
) -> PlayerScan {
    let position = actors[0].position;
    let mut result = PlayerScan {
        contact_count: 0,
        last_contact: None,
        bits: 0,
        any_contact: false,
    };
    let mut pogo_claimed = false;
    for slot in (1..ACTOR_SLOTS).rev() {
        let actor = &mut actors[slot];
        if actor.render & VISIBLE == 0
            || actor.profile & PROFILE_ENABLED == 0
            || axis == ContactAxis::Vertical && actor.render & SKIP_VERTICAL_PLAYER_SCAN != 0
        {
            continue;
        }
        let flagged = actor.profile & REACTION_PROFILE != 0;
        let extent = geometry.extents[usize::from(actor.profile & 0x7f)];
        let dx = horizontal_separation(position.world_x, actor.position.world_x);
        let mut dy = position.screen_y.abs_diff(actor.position.screen_y);
        const POSE_DISTANCE_ADJUSTMENT: u8 = 8;
        if flagged && player.animation == SPECIAL_POSE {
            dy = dy.wrapping_add(POSE_DISTANCE_ADJUSTMENT);
        }
        let Some(penetration) = contact_penetration(axis, dx, dy, extent) else {
            continue;
        };
        if player.protection != 0 {
            if axis == ContactAxis::Horizontal
                && !matches!(actor.handler, SOLID_HANDLER | PROTECTION_CONTACT_EXCEPTION)
            {
                continue;
            }
            if axis == ContactAxis::Vertical && flagged {
                continue;
            }
        }
        result.contact_count = result.contact_count.wrapping_add(1);
        result.last_contact = Some(Contact { slot, penetration });
        result.bits |= SOLID_CONTACT;
        result.any_contact = true;
        if axis == ContactAxis::Vertical
            && !pogo_claimed
            && player.vertical_velocity < 0
            && player.animation == POGO_POSE
            && player.motion & DESCENDING_MOTION != 0
        {
            player.actor_contact_flags = SOLID_CONTACT;
            pogo_claimed = true;
            actor.hit_marker = actor.hit_marker.wrapping_add(1);
            if actor.hit_marker != 0 {
                continue;
            }
        }
        if flagged {
            if player.special_mode == 0 {
                player.immediate_state = DAMAGE_STATE;
            } else {
                actor.hit_marker = actor.hit_marker.wrapping_add(1);
            }
        }
        if axis == ContactAxis::Horizontal {
            break;
        }
    }
    result
}
/// Return is source caller permission, not geometric overlap. Special-mode
/// flagged contact increments hit marker and returns false on purpose.
pub fn actor_player_contact(
    actor: &mut ActorContact,
    player_position: Position,
    player: &mut PlayerContact,
    geometry: &Geometry,
) -> bool {
    if actor.render & VISIBLE == 0 {
        return false;
    }
    let extent = geometry.extents[usize::from(actor.profile & 0x7f)];
    let dx = horizontal_separation(player_position.world_x, actor.position.world_x);
    let mut dy = player_position.screen_y.abs_diff(actor.position.screen_y);
    const POSE_DISTANCE_ADJUSTMENT: u8 = 8;
    if player.animation == SPECIAL_POSE {
        dy = dy.wrapping_add(POSE_DISTANCE_ADJUSTMENT);
    }
    if contact_penetration(ContactAxis::Horizontal, dx, dy, extent).is_none() {
        return false;
    }
    if actor.profile & REACTION_PROFILE == 0 {
        return true;
    }
    if player.special_mode != 0 {
        actor.hit_marker = actor.hit_marker.wrapping_add(1);
        return false;
    }
    if player.protection != 0 {
        return false;
    }
    player.immediate_state = DAMAGE_STATE;
    true
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Movement {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    Fall,
    Rise,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovementPhase {
    Gravity,
    Integrate,
    Terrain,
    SolidActors,
    ReleaseIdentity,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementReport {
    /// Accepted positive handler24 wrapper exposes the count consumed by A04C's
    /// immediate repeat. Ordinary solid returns0; special response preserves its
    /// descriptor selector. None where that caller continuation is inapplicable.
    pub repeat_positive_count: Option<u8>,
    pub phases: Vec<MovementPhase>,
    pub probe: Option<ProbeReport>,
    pub contact: Option<Contact>,
    pub removed: bool,
}
/// Movement borrows the canonical caller-owned actor/contact views and world's
/// cache. B10 owns allocation/suppression; removed identity effects are immediate.
// Shared wrapper accepts borrowed owners rather than retaining duplicate state.
#[allow(clippy::too_many_arguments)]
pub fn move_with_collision(
    slot: usize,
    actors: &mut [ActorContact; ACTOR_SLOTS],
    player: &mut PlayerContact,
    cache: &dyn CollisionCells,
    row: u8,
    geometry: &Geometry,
    range: ProbeRange,
    movement: Movement,
    gravity_step: u8,
    special_spawn: &mut u8,
    descriptor: &mut dyn FnMut(TerrainProbe) -> u8,
) -> MovementReport {
    use MovementPhase as P;
    let mut result = MovementReport {
        repeat_positive_count: None,
        phases: Vec::new(),
        probe: None,
        contact: None,
        removed: false,
    };
    if movement == Movement::Fall {
        gravity(&mut actors[slot], gravity_step);
        result.phases.push(P::Gravity);
    }
    result.phases.push(P::Integrate);
    let boundary = match movement {
        Movement::PositiveX => {
            integrate_horizontal(&mut actors[slot], HorizontalDirection::Positive)
        }
        Movement::NegativeX => {
            integrate_horizontal(&mut actors[slot], HorizontalDirection::Negative)
        }
        Movement::PositiveY => {
            integrate_vertical(&mut actors[slot], VerticalOperation::AddEncodedVelocity)
        }
        _ => integrate_vertical(
            &mut actors[slot],
            VerticalOperation::SubtractEncodedVelocity,
        ),
    };
    // Falling/rising always probe before testing the saved unsigned boundary;
    // direct coordinate wrappers release immediately and never probe on escape.
    let direct = !matches!(movement, Movement::Fall | Movement::Rise);
    let mut last_descriptor = None;
    if !direct || !boundary {
        result.phases.push(P::Terrain);
        result.probe = Some(probe(
            slot,
            &mut actors[slot],
            player,
            cache,
            row,
            geometry,
            range,
            &mut |query| {
                let value = descriptor(query);
                last_descriptor = Some(value >> 4);
                value
            },
        ));
    }
    let remove = if movement == Movement::Fall {
        !boundary
    } else {
        direct && boundary
    };
    if remove {
        let actor = &mut actors[slot];
        actor.render = 0;
        {
            let actor_value = 0;
            actor.set_state_code(actor_value);
        };
        actor.spawn_identity = None;
        if *special_spawn == ACTIVE | slot as u8 {
            *special_spawn = SPECIAL_SPAWN_RETRY;
        }
        result.removed = true;
        result.phases.push(P::ReleaseIdentity);
        return result;
    }
    if movement == Movement::Rise {
        gravity(&mut actors[slot], gravity_step);
        result.phases.push(P::Gravity);
    }
    let terrain = result
        .probe
        .as_ref()
        .expect("accepted movement probes terrain");
    let terrain_solid = terrain.bits & SOLID_CONTACT != 0;
    if terrain_solid {
        let snap_direction = match movement {
            Movement::PositiveX => Snap::PositiveX,
            Movement::NegativeX => Snap::NegativeX,
            Movement::PositiveY | Movement::Fall => Snap::PositiveY,
            _ => Snap::NegativeY,
        };
        snap(
            &mut actors[slot],
            terrain
                .last_sample
                .expect("probe visited at least one sample"),
            snap_direction,
        );
    }
    match movement {
        Movement::PositiveX | Movement::NegativeX => {
            let positive = movement == Movement::PositiveX;
            if positive {
                actors[slot].render |= FACE_POSITIVE;
            } else {
                actors[slot].render &= !FACE_POSITIVE;
            }
            if positive && actors[slot].handler == POSITIVE_ONLY_EXEMPT_HANDLER {
                result.repeat_positive_count = Some(
                    if terrain.last_sample.is_some_and(|sample| sample.class == 3) {
                        last_descriptor.expect("special sample decoded")
                    } else if terrain_solid {
                        terrain.final_range.remaining
                    } else {
                        0xff
                    },
                );
            }
            let scan = slot != 0
                && actors[slot].handler != OVERLAP_EXEMPT_HANDLER
                && !(positive && actors[slot].handler == POSITIVE_ONLY_EXEMPT_HANDLER);
            if scan {
                result.phases.push(P::SolidActors);
                result.contact = solid_overlap(slot, actors, geometry, ContactAxis::Horizontal);
                if let Some(contact) = result.contact {
                    let x = &mut actors[slot].position.world_x;
                    *x = if positive {
                        x.wrapping_sub(u16::from(contact.penetration))
                    } else {
                        x.wrapping_add(u16::from(contact.penetration))
                    };
                }
            }
        }
        Movement::Fall | Movement::Rise => {
            let falling = movement == Movement::Fall;
            let mut collided = terrain_solid;
            if !terrain_solid
                && actors[slot].profile != 0
                && actors[slot].handler != OVERLAP_EXEMPT_HANDLER
            {
                result.phases.push(P::SolidActors);
                result.contact = solid_overlap(slot, actors, geometry, ContactAxis::Vertical);
                if let Some(contact) = result.contact {
                    let y = &mut actors[slot].position.screen_y;
                    *y = if falling {
                        y.wrapping_sub(contact.penetration)
                    } else {
                        y.wrapping_add(contact.penetration).wrapping_add(1)
                    };
                    collided = true;
                }
            }
            if collided {
                actors[slot].vertical_velocity = if falling { 0 } else { -0x0100 };
            }
        }
        _ => (),
    }
    result
}
/// B10 consumes this snapshot operation; ordinary clear preserves slots0/1.
pub fn clear_transition_actors(actors: &mut [ActorContact; ACTOR_SLOTS], stage: u8, row: u8) {
    const PRESERVED_STAGE: u8 = 1;
    const PRESERVED_ROWS: u8 = 2;
    const TIMER_STEP: u8 = 60;
    const TIMER_BORROW_CORRECTION: u8 = 16;
    if stage == PRESERVED_STAGE && row < PRESERVED_ROWS {
        for actor in &mut actors[ACTOR_SLOTS - 2..] {
            {
                let actor_value = actor.state_code() | (PRESERVED_ACTOR_MARK);
                actor.set_state_code(actor_value);
            };
            if actor.local_counter_code() != 0 {
                let (low, borrow) = actor.local_state_code().overflowing_sub(TIMER_STEP);
                {
                    let actor_value = if borrow {
                        low.wrapping_sub(TIMER_BORROW_CORRECTION)
                    } else {
                        low
                    };
                    actor.set_local_state_code(actor_value);
                };
                {
                    let actor_value = actor.local_counter_code().wrapping_sub(u8::from(borrow));
                    actor.set_local_counter_code(actor_value);
                };
            }
        }
    } else {
        for actor in &mut actors[2..] {
            {
                let actor_value = 0;
                actor.set_state_code(actor_value);
            };
            actor.render = 0;
            actor.spawn_identity = None;
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticleBatch {
    pub slots: Vec<usize>,
    pub audio_commands: Vec<u8>,
}
pub fn terrain_particles(
    actors: &mut [ActorContact; ACTOR_SLOTS],
    geometry: &Geometry,
    origin_y: u8,
    origin_x: u8,
    player_page: u8,
) -> ParticleBatch {
    let mut slots = Vec::new();
    for index in (0..4).rev() {
        let Some(slot) = (1..ACTOR_SLOTS)
            .rev()
            .find(|&slot| actors[slot].state_code() & ACTIVE == 0)
        else {
            break;
        };
        let actor = &mut actors[slot];
        let (y, y_carry) = origin_y.overflowing_add(geometry.particle_y[index]);
        actor.position.screen_y = y;
        let x = origin_x
            .wrapping_add(geometry.particle_x[index])
            .wrapping_add(u8::from(y_carry));
        actor.position.world_x = u16::from(player_page) << 8 | u16::from(x);
        {
            let actor_value = TERRAIN_PARTICLE_STATE;
            actor.set_state_code(actor_value);
        };
        actor.render = TERRAIN_PARTICLE_RENDER;
        actor.animation = TERRAIN_PARTICLE_ANIMATION;
        actor.playback.reset_frame();
        {
            let animation_value = geometry.particle_ticks[index];
            actor.playback.set_tick_code(animation_value);
        };
        actor.profile = 0;
        {
            let actor_value = 0;
            actor.set_handler(actor_value);
        };
        slots.push(slot);
    }
    ParticleBatch {
        slots,
        audio_commands: vec![TERRAIN_PARTICLE_SOUND],
    }
}
#[cfg(test)]
mod contracts;

fn horizontal_separation(first: u16, second: u16) -> u8 {
    if first >= second {
        first.wrapping_sub(second) as u8
    } else {
        second.wrapping_sub(first) as u8
    }
}
fn contact_penetration(axis: ContactAxis, dx: u8, dy: u8, extent: Extent) -> Option<u8> {
    match axis {
        ContactAxis::Horizontal
            if dy < extent.half_height && dx < extent.half_width.wrapping_add(1) =>
        {
            Some(extent.half_width.wrapping_sub(dx))
        }
        ContactAxis::Vertical
            if dx < extent.half_width && dy < extent.half_height.wrapping_add(1) =>
        {
            Some(extent.half_height.wrapping_sub(dy))
        }
        _ => None,
    }
}
