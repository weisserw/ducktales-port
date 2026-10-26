//! Native actor-family boundaries. Every callback operates on the live pool;
//! collision/attribute slices are disjoint borrows of the one WorldCache.
use super::{contact, content::Content};
use crate::{
    actors::{self, ActorPool, SpecialControl, bosses, enemies, objects, platforms},
    audio, collision, player, presentation, runtime, world,
};

const RECOVERY_PALETTE_INDICES: [usize; 2] = [29, 31];
const ERUPTION_PALETTE_START: usize = 30;
const STAGE_ONE_EXTENSION_MARK: u8 = 1;

pub struct Terrain<'a> {
    pub content: &'a Content,
    pub attributes: &'a mut [u8; world::CACHE_CELLS],
    pub cells: &'a mut [u8; world::CACHE_CELLS],
    pub native: &'a mut world::NativeTerrain,
    pub secondary: &'a mut presentation::PreparedSecondary,
    pub cursor: &'a mut u8,
    pub row: u8,
    pub layout: u8,
    pub bank: u8,
    pub stage: u8,
    pub gravity: u8,
    pub collision_outcome: &'a mut collision::CollisionOutcome,
}
impl Terrain<'_> {
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        special: &mut u8,
        slot: usize,
        movement: collision::Movement,
        start: u8,
        remaining: u8,
        gravity: u8,
    ) -> collision::MovementReport {
        let mut view = contact::player_contact(player, &actors[0]);
        let result = collision::move_with_collision(
            slot,
            actors,
            &mut view,
            &world::native_collision(self.cells, self.native),
            self.row,
            &self.content.geometry,
            collision::ProbeRange { start, remaining },
            movement,
            gravity,
            special,
            &mut |query| contact::descriptor(self.content, self.layout, self.bank, query),
        );
        contact::commit_player_contact(view, player);
        self.collision_outcome.movement(&result);
        result
    }
    fn replace(&mut self, actors: &mut ActorPool, slot: usize, changed: &mut u8) {
        let actor = actors[slot];
        let active = std::array::from_fn(|index| actors[index].state_code() & actors::ACTIVE != 0);
        if let Some(recovery) = world::recovery_actor(
            actor.local_counter_code(),
            actor.local_state_code(),
            actor.position.world_x,
            actor.position.screen_y,
            &active,
        ) {
            let target = &mut actors[recovery.slot];
            {
                let actor_value = recovery.state;
                target.set_state_code(actor_value);
            };
            {
                let actor_value = recovery.handler;
                target.set_handler(actor_value);
            };
            target.animation = recovery.animation;
            target.position.world_x = recovery.world_x;
            target.position.screen_y = recovery.screen_y;
            {
                let animation_value = recovery.animation_tick;
                target.playback.set_tick_code(animation_value);
            };
            {
                let animation_value = recovery.animation_frame;
                target.playback.set_frame_code(animation_value);
            };
            target.profile = recovery.collision_profile;
            target.vertical_velocity = recovery.vertical_velocity;
        }
        let index = world::replacement_index(world::Replacement::ActorState {
            stage: crate::identifiers::StageId(self.stage),
            local: actor.local_state_code(),
        });
        let selector = self.content.changed_terrain_selectors[usize::from(index)];
        self.native.replace(
            &self.content.world.terrain[usize::from(self.bank)],
            world::Mutation {
                world_x: actor.position.world_x,
                spawn_y: actor.spawn_y,
                selector,
                marks_changed: true,
            },
        );
        let upload = world::replace_cache(
            self.attributes,
            self.cells,
            changed,
            &self.content.world.terrain[usize::from(self.bank)],
            world::Mutation {
                world_x: actor.position.world_x,
                spawn_y: actor.spawn_y,
                selector,
                marks_changed: true,
            },
        );
        self.secondary
            .append_changed_shared(self.cursor, &upload, self.attributes);
    }
}
fn contact_player(
    content: &Content,
    actors: &mut ActorPool,
    player: &mut player::Player,
    slot: usize,
) -> bool {
    let mut view = contact::player_contact(player, &actors[0]);
    let position = actors[0].position;
    let result =
        collision::actor_player_contact(&mut actors[slot], position, &mut view, &content.geometry);
    contact::commit_player_contact(view, player);
    result
}
fn contact_actor(content: &Content, actors: &mut ActorPool, slot: usize) -> Option<usize> {
    // Bank06_TestActorActorOverlap marks enemies and changes the source's impact
    // animation. The fixed-bank solid scan only handles terrain actors.
    actors::actor_overlap(actors, slot, &content.spawns.extents)
}

fn horizontal(actors: &mut ActorPool, slot: usize, positive: bool) -> bool {
    let boundary = collision::integrate_horizontal(
        &mut actors[slot],
        if positive {
            collision::HorizontalDirection::Positive
        } else {
            collision::HorizontalDirection::Negative
        },
    );
    if positive { boundary } else { !boundary }
}
fn object_integration(
    actors: &mut ActorPool,
    slot: usize,
    operation: objects::Integration,
    gravity: u8,
) -> bool {
    match operation {
        objects::Integration::Gravity => {
            collision::gravity(&mut actors[slot], gravity);
            false
        }
        objects::Integration::FallingY => !collision::integrate_vertical(
            &mut actors[slot],
            collision::VerticalOperation::SubtractEncodedVelocity,
        ),
        objects::Integration::PositiveX => horizontal(actors, slot, true),
        objects::Integration::NegativeX => horizontal(actors, slot, false),
    }
}
/// Canonical audio and palette fields used by shared recovery. The effects list
/// remains an observation; source-submitted commands are never submitted twice.
pub struct Recovery<'a> {
    pub queue: &'a mut audio::CommandQueue,
    pub base: &'a mut [u8; presentation::PALETTE_COLORS],
    pub active: &'a mut [u8; presentation::PALETTE_COLORS],
    pub palette_pending: &'a mut bool,
    pub parameters: objects::RecoveryParameters<'a>,
    pub gravity: u8,
}
impl Recovery<'_> {
    fn effects(&mut self, effects: &objects::Effects) {
        for (index, &command) in effects.sounds.iter().enumerate() {
            if !effects.submitted_sounds.contains(&index) {
                self.queue.submit(command);
            }
        }
        if let Some(pair) = effects.palette {
            for (offset, value) in RECOVERY_PALETTE_INDICES.into_iter().zip(pair) {
                self.base[offset] = value;
                self.active[offset] = value;
            }
            *self.palette_pending = true;
        }
    }
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        slot: usize,
        random: &mut runtime::RandomState,
    ) -> objects::Effects {
        let gravity = self.gravity;
        let effects = objects::reaction_shared(
            actors,
            slot,
            random,
            self.queue,
            self.parameters,
            &mut |actors, slot, operation| object_integration(actors, slot, operation, gravity),
        );
        self.effects(&effects);
        effects
    }
}
pub struct ObjectWorld<'a> {
    pub terrain: Terrain<'a>,
    pub special: &'a mut SpecialControl,
}
impl objects::World for ObjectWorld<'_> {
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
        movement: collision::Movement,
        start: u8,
        remaining: u8,
    ) -> u8 {
        let mut special = self.special.encode();
        self.terrain.motion(
            actors,
            player,
            &mut special,
            slot,
            movement,
            start,
            remaining,
            self.terrain.gravity,
        );
        *self.special = SpecialControl::decode(special);
        self.terrain.collision_outcome.bits
    }
    fn integrate(
        &mut self,
        actors: &mut ActorPool,
        slot: usize,
        operation: objects::Integration,
    ) -> bool {
        object_integration(actors, slot, operation, self.terrain.gravity)
    }
    fn player_contact(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
    ) -> bool {
        contact_player(self.terrain.content, actors, player, slot)
    }
    fn actor_contact(&mut self, actors: &mut ActorPool, slot: usize) -> Option<usize> {
        contact_actor(self.terrain.content, actors, slot)
    }
    fn replace_terrain(&mut self, actors: &mut ActorPool, slot: usize, changed: &mut u8) {
        self.terrain.replace(actors, slot, changed);
    }
}
pub struct EnemyWorld<'a> {
    pub terrain: Terrain<'a>,
    pub recovery: Recovery<'a>,
}
impl enemies::World for EnemyWorld<'_> {
    fn submit_sound(&mut self, command: u8) -> bool {
        self.recovery.queue.submit(command);
        true
    }
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        special: &mut u8,
        slot: usize,
        movement: collision::Movement,
        start: u8,
        remaining: u8,
        gravity: u8,
    ) -> enemies::ContactFacts {
        let report = self.terrain.motion(
            actors, player, special, slot, movement, start, remaining, gravity,
        );
        enemies::ContactFacts {
            bits: self.terrain.collision_outcome.bits,
            actor: report.contact.map(|hit| hit.slot),
        }
    }
    fn integrate(
        &mut self,
        actors: &mut ActorPool,
        slot: usize,
        operation: enemies::Integration,
        gravity: u8,
    ) {
        match operation {
            enemies::Integration::Gravity => collision::gravity(&mut actors[slot], gravity),
            enemies::Integration::PositiveX | enemies::Integration::NegativeX => {
                horizontal(actors, slot, operation == enemies::Integration::PositiveX);
            }
            enemies::Integration::PositiveY => {
                collision::integrate_vertical(
                    &mut actors[slot],
                    collision::VerticalOperation::AddEncodedVelocity,
                );
            }
            enemies::Integration::NegativeY | enemies::Integration::FallingY => {
                collision::integrate_vertical(
                    &mut actors[slot],
                    collision::VerticalOperation::SubtractEncodedVelocity,
                );
            }
        }
    }
    fn player_contact(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
    ) -> bool {
        contact_player(self.terrain.content, actors, player, slot)
    }
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        _: &mut player::Player,
        random: &mut runtime::RandomState,
        slot: usize,
    ) {
        self.recovery.reaction(actors, slot, random);
    }
    fn launch_terrain(&mut self, actors: &mut ActorPool, slot: usize, pose_direction: u8) {
        objects::launch_terrain_shared(
            actors,
            slot,
            pose_direction,
            self.recovery.parameters.catalog,
        );
    }
}
pub struct PlatformWorld<'a> {
    pub terrain: Terrain<'a>,
    pub recovery: Recovery<'a>,
    pub random: &'a mut runtime::RandomState,
}
impl platforms::World for PlatformWorld<'_> {
    fn submit_sound(&mut self, command: u8) -> bool {
        self.recovery.queue.submit(command);
        true
    }
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        special: &mut SpecialControl,
        slot: usize,
        movement: collision::Movement,
        start: u8,
        remaining: u8,
    ) -> platforms::MotionFacts {
        let mut raw = special.encode();
        let report = self.terrain.motion(
            actors,
            player,
            &mut raw,
            slot,
            movement,
            start,
            remaining,
            self.terrain.gravity,
        );
        *special = SpecialControl::decode(raw);
        platforms::MotionFacts {
            bits: self.terrain.collision_outcome.bits,
            continuation_remaining: report.repeat_positive_count.unwrap_or(remaining),
        }
    }
    fn integrate(
        &mut self,
        actors: &mut ActorPool,
        slot: usize,
        operation: platforms::Integration,
    ) -> bool {
        match operation {
            platforms::Integration::Horizontal(direction) => {
                let value = collision::integrate_horizontal(&mut actors[slot], direction);
                if direction == collision::HorizontalDirection::Negative {
                    !value
                } else {
                    value
                }
            }
            platforms::Integration::Vertical(operation) => {
                let value = collision::integrate_vertical(&mut actors[slot], operation);
                if operation == collision::VerticalOperation::SubtractEncodedVelocity {
                    !value
                } else {
                    value
                }
            }
        }
    }
    fn player_contact(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
    ) -> bool {
        contact_player(self.terrain.content, actors, player, slot)
    }
    fn actor_contact(&mut self, actors: &mut ActorPool, slot: usize) -> Option<usize> {
        contact_actor(self.terrain.content, actors, slot)
    }
    fn preset(&mut self, actors: &mut ActorPool, slot: usize, selector: u8) {
        preset(self.terrain.content, actors, slot, selector);
    }
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        _: &mut player::Player,
        slot: usize,
        _: &mut SpecialControl,
    ) -> platforms::Effects {
        let effects = self.recovery.reaction(actors, slot, self.random);
        platforms::Effects {
            submitted_sounds: (0..effects.sounds.len()).collect(),
            sounds: effects.sounds,
        }
    }
    fn activate_terrain(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
        changed: &mut u8,
    ) -> platforms::Effects {
        if *changed != 0 {
            return platforms::Effects::default();
        }
        *changed = changed.wrapping_add(1);
        if actors[slot].state_code() & objects::TERRAIN_REPRESENTATION != 0 {
            {
                let actor_value = actors[slot].state_code() & (!objects::TERRAIN_REPRESENTATION);
                actors[slot].set_state_code(actor_value);
            };
            self.terrain.replace(actors, slot, changed);
        }
        objects::launch_terrain_shared(
            actors,
            slot,
            player.pose_direction,
            self.recovery.parameters.catalog,
        );
        platforms::Effects::default()
    }
}
fn preset(content: &Content, actors: &mut ActorPool, slot: usize, selector: u8) {
    actors[slot].horizontal_magnitude = content.spawns.horizontal_presets[usize::from(selector)];
    actors[slot].vertical_velocity = content.spawns.vertical_presets[usize::from(selector)];
}
/// Boss context owns its terrain borrow; motion receives that exact cache at
/// call time rather than retaining another cache or copying collision cells.
pub struct BossWorld<'a> {
    pub content: &'a Content,
    pub row: u8,
    pub layout: u8,
    pub bank: u8,
    pub gravity: u8,
    pub special: &'a mut SpecialControl,
    pub collision_outcome: &'a mut collision::CollisionOutcome,
    pub recovery: Recovery<'a>,
}
impl bosses::World for BossWorld<'_> {
    fn submit_sound(&mut self, command: u8) -> bool {
        self.recovery.queue.submit(command);
        true
    }
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        terrain: &world::WorldCache,
        slot: usize,
        movement: collision::Movement,
        start: u8,
        remaining: u8,
    ) -> bosses::MotionFacts {
        let mut view = contact::player_contact(player, &actors[0]);
        let mut special = self.special.encode();
        let report = collision::move_with_collision(
            slot,
            actors,
            &mut view,
            terrain,
            self.row,
            &self.content.geometry,
            collision::ProbeRange { start, remaining },
            movement,
            self.gravity,
            &mut special,
            &mut |query| contact::descriptor(self.content, self.layout, self.bank, query),
        );
        *self.special = SpecialControl::decode(special);
        contact::commit_player_contact(view, player);
        bosses::MotionFacts {
            bits: self.collision_outcome.movement(&report),
            random_increment: false,
        }
    }
    fn integrate(
        &mut self,
        actors: &mut ActorPool,
        slot: usize,
        operation: bosses::Integration,
    ) -> bool {
        object_integration(
            actors,
            slot,
            match operation {
                bosses::Integration::FallingY => objects::Integration::FallingY,
                bosses::Integration::PositiveX => objects::Integration::PositiveX,
                bosses::Integration::NegativeX => objects::Integration::NegativeX,
            },
            self.gravity,
        )
    }
    fn player_contact(&mut self, actors: &mut ActorPool, player: &mut player::Player, slot: usize) {
        contact_player(self.content, actors, player, slot);
    }
    fn preset(&mut self, actors: &mut ActorPool, slot: usize, selector: u8) {
        preset(self.content, actors, slot, selector);
    }
    fn reaction(
        &mut self,
        actors: &mut ActorPool,
        _: &mut player::Player,
        random: &mut runtime::RandomState,
        slot: usize,
    ) {
        self.recovery.reaction(actors, slot, random);
    }
}

/// Execute one handler from the live ascending dispatcher. False identifies a
/// scene/dialogue/finale handler owned by its separate native policy module.
// Separate borrows keep canonical owners visible across immediate callbacks.
#[allow(clippy::too_many_arguments)]
pub fn update_family(
    frame: &mut crate::progression::Frame<'_>,
    services: &mut super::simulation::Services,
    suppression: &mut [u8; actors::SUPPRESSION_BYTES],
    special: &mut SpecialControl,
    layout: u8,
    bank: u8,
    split_transition: &mut u8,
    slot: usize,
    first_increment: crate::progression::FirstActorIncrement,
) -> bool {
    use crate::progression::FirstActorIncrement;
    let handler = frame.actors[slot].handler;
    let stage = frame.progression.stage.0;
    let gravity = frame.player.gravity;
    let parameters = objects::RecoveryParameters {
        stage,
        late_mode: frame.progression.late_mode != 0,
        rule: services.option.status_rule(),
        catalog: &services.content.objects,
    };
    let display = &mut services.video.display;
    if matches!(
        handler,
        bosses::ATTACK
            | bosses::PROJECTILE
            | bosses::IMPACT
            | bosses::PERIMETER
            | bosses::WAVE
            | bosses::WAVE_CHILD
            | bosses::CENTER
    ) {
        let mut split_low = services.video.split_address as u8;
        let effects = bosses::step(
            frame.actors,
            slot,
            &mut bosses::Context {
                player: frame.player,
                random: &mut services.runtime.random,
                hit_points: &mut frame.progression.boss_hit_points,
                impact: &mut frame.progression.impact_latch,
                camera: frame.camera,
                terrain: frame.cache,
                split_low: &mut split_low,
                split_control: split_transition,
                composition_tick: services.video.composition.clock,
                random_increment: slot == 1 && first_increment == FirstActorIncrement::One,
                catalog: &services.content.bosses,
            },
            &mut BossWorld {
                content: &services.content,
                row: *frame.room_row,
                layout,
                bank,
                gravity,
                special,
                collision_outcome: &mut services.collision_outcome,
                recovery: Recovery {
                    queue: &mut services.commands,
                    base: &mut display.base,
                    active: &mut display.active,
                    palette_pending: &mut display.palette_pending,
                    parameters,
                    gravity,
                },
            },
        );
        services.video.split_address =
            (services.video.split_address & 0xff00) | u16::from(split_low);
        submit_remaining(
            &mut services.commands,
            &effects.sounds,
            &effects.submitted_sounds,
        );
        return true;
    }
    let terrain = Terrain {
        content: &services.content,
        attributes: &mut frame.cache.attributes,
        cells: &mut frame.cache.collision,
        native: &mut frame.cache.native,
        secondary: &mut display.secondary,
        cursor: frame.secondary_cursor,
        row: *frame.room_row,
        layout,
        bank,
        stage,
        gravity,
        collision_outcome: &mut services.collision_outcome,
    };
    if enemies::family(handler).is_some() {
        let mut raw_special = special.encode();
        let effects = enemies::step(
            frame.actors,
            slot,
            &mut enemies::Context {
                player: frame.player,
                random: &mut services.runtime.random,
                status: frame.status,
                special_spawn: &mut raw_special,
                gravity,
                geometry: &services.content.geometry,
                spawn: &services.content.spawns,
                catalog: &services.content.enemies,
            },
            &mut EnemyWorld {
                terrain,
                recovery: Recovery {
                    queue: &mut services.commands,
                    base: &mut display.base,
                    active: &mut display.active,
                    palette_pending: &mut display.palette_pending,
                    parameters,
                    gravity,
                },
            },
        );
        *special = SpecialControl::decode(raw_special);
        // Eruption edits only the active final pair; recovery edits29/31 in both.
        if let Some(pair) = effects.palette_pair {
            display.active[ERUPTION_PALETTE_START..presentation::PALETTE_COLORS]
                .copy_from_slice(&pair);
        }
        if effects.palette_upload {
            display.palette_pending = true;
        }
        submit_remaining(
            &mut services.commands,
            &effects.sounds,
            &effects.submitted_sounds,
        );
        return true;
    }
    if matches!(
        handler,
        objects::TERRAIN_HANDLER
            | objects::MOVING_HANDLER
            | objects::FALLING_HANDLER
            | objects::LAUNCHED_HANDLER
            | objects::RECOVERY_HANDLER
            | objects::CONTACT_RECOVERY_HANDLER
            | objects::TRANSFORM_HANDLER
            | objects::REVEAL_HANDLER
            | objects::SLIDING_HANDLER
            | objects::DISPENSER_HANDLER
            | objects::DISPENSED_HANDLER
            | objects::RELEASED_HANDLER
    ) {
        let mut message = None;
        let mut extension = frame.progression.stage_one_extension != 0;
        let effects = objects::step(
            frame.actors,
            slot,
            &mut objects::Context {
                player: frame.player,
                random: &mut services.runtime.random,
                status: frame.status,
                suppression,
                changed: &mut frame.cache.changed_this_frame,
                stage_bits: &mut frame.progression.progress_bits,
                reward_flags: &mut frame.progression.reward_progress,
                stage_one_extension: &mut extension,
                completion_latch: &mut frame.progression.completion_actor,
                dialogue: &mut message,
                stage,
                late_mode: parameters.late_mode,
                composition_tick: services.video.composition.clock,
                rule: parameters.rule,
                audio_queue: &mut services.commands,
                catalog: &services.content.objects,
                spawn: &services.content.spawns,
            },
            &mut ObjectWorld { terrain, special },
        );
        if extension && frame.progression.stage_one_extension == 0 {
            frame.progression.stage_one_extension = STAGE_ONE_EXTENSION_MARK;
        }
        if let Some(message) = message {
            services.dialogue.message = message;
            services.dialogue.actor_slot = slot;
            services.dialogue.phase = crate::dialogue::Phase::Entry;
        }
        let mut recovery = Recovery {
            queue: &mut services.commands,
            base: &mut display.base,
            active: &mut display.active,
            palette_pending: &mut display.palette_pending,
            parameters,
            gravity,
        };
        recovery.effects(&effects);
        return true;
    }
    if matches!(
        handler,
        platforms::WALKING
            | platforms::FLIGHT
            | platforms::RIDEABLE
            | platforms::VERTICAL
            | platforms::PUSHING
            | platforms::CARRYING
            | platforms::EXTERNAL
            | platforms::LIFT
            | platforms::FOUR_PHASE
    ) {
        let effects = platforms::step(
            frame.actors,
            slot,
            &mut platforms::Context {
                player: frame.player,
                special,
                changed: &mut frame.cache.changed_this_frame,
                completion: &mut frame.progression.completion_actor,
                held: services.runtime.controllers.ports[0].held.0,
                catalog: &services.content.platforms,
            },
            &mut PlatformWorld {
                terrain,
                random: &mut services.runtime.random,
                recovery: Recovery {
                    queue: &mut services.commands,
                    base: &mut display.base,
                    active: &mut display.active,
                    palette_pending: &mut display.palette_pending,
                    parameters,
                    gravity,
                },
            },
        );
        submit_remaining(
            &mut services.commands,
            &effects.sounds,
            &effects.submitted_sounds,
        );
        return true;
    }
    false
}
fn submit_remaining(queue: &mut audio::CommandQueue, sounds: &[u8], submitted: &[usize]) {
    for (index, &command) in sounds.iter().enumerate() {
        if !submitted.contains(&index) {
            queue.submit(command);
        }
    }
}

/// The actor-only B15 entry calls only these non-yielding leaves; the resumable
/// text owner has a separate Systems implementation for its yielding operations.
struct DialogueActorWorld<'a> {
    services: &'a mut super::simulation::Services,
    special: &'a mut SpecialControl,
    row: u8,
    layout: u8,
    bank: u8,
    gravity: u8,
}
impl crate::dialogue::Systems for DialogueActorWorld<'_> {
    fn perform(
        &mut self,
        work: crate::dialogue::Work,
        context: &mut crate::dialogue::Context<'_>,
    ) -> crate::progression::OwnerStep<()> {
        use crate::dialogue::Work;
        match work {
            Work::ChangedBurst(selector) => {
                burst_owners(
                    &self.services.content,
                    &mut self.services.commands,
                    &mut self.services.video.display.primary,
                    &mut self.services.video.display.secondary,
                    context.cache,
                    context.actors,
                    self.bank,
                    selector,
                );
                self.services
                    .runtime
                    .bank
                    .select(runtime::PrgBank::ActorsFour);
            }
            Work::RestoreActorBank => self
                .services
                .runtime
                .bank
                .select(runtime::PrgBank::ActorsSix),
            _ => unreachable!("B15 actor policy invokes only burst and bank leaves"),
        }
        crate::progression::OwnerStep::Complete(())
    }
    fn contact(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
    ) -> bool {
        contact_player(&self.services.content, actors, player, slot)
    }
    fn distance(&mut self, actors: &ActorPool, slot: usize) -> u8 {
        let actor = actors[slot].position.world_x;
        let player = actors[0].position.world_x;
        actor.abs_diff(player).min(255) as u8
    }
    // Separate borrows keep canonical owners visible across immediate callbacks.
    #[allow(clippy::too_many_arguments)]
    fn motion(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        cache: &world::WorldCache,
        slot: usize,
        movement: collision::Movement,
        start: u8,
        remaining: u8,
    ) -> u8 {
        let mut view = contact::player_contact(player, &actors[0]);
        let mut special = self.special.encode();
        let report = collision::move_with_collision(
            slot,
            actors,
            &mut view,
            cache,
            self.row,
            &self.services.content.geometry,
            collision::ProbeRange { start, remaining },
            movement,
            self.gravity,
            &mut special,
            &mut |query| contact::descriptor(&self.services.content, self.layout, self.bank, query),
        );
        *self.special = SpecialControl::decode(special);
        contact::commit_player_contact(view, player);
        self.services.collision_outcome.movement(&report)
    }
    fn integrate(&mut self, actors: &mut ActorPool, slot: usize, positive: bool) {
        horizontal(actors, slot, positive);
    }
    fn allocate(&mut self, actors: &ActorPool) -> Option<usize> {
        (actors::FIRST_ACTOR_SLOT..actors::ACTOR_SLOTS)
            .rev()
            .find(|&slot| actors[slot].state_code() & actors::ACTIVE == 0)
    }
}
/// The burst prepares primary rows before allocating particles and submitting
/// their sound. Return THAT submission's comparison for the dialogue caller.
// Separate borrows keep canonical owners visible across immediate callbacks.
#[allow(clippy::too_many_arguments)]
pub(super) fn burst_owners(
    content: &Content,
    queue: &mut audio::CommandQueue,
    primary: &mut presentation::PreparedPrimary,
    secondary: &mut presentation::PreparedSecondary,
    cache: &mut world::WorldCache,
    actors: &mut ActorPool,
    bank: u8,
    selector: u8,
) -> audio::Submission {
    let burst = world::changed_burst(&content.world.terrain[usize::from(bank)], selector);
    let bytes = presentation::terrain::encoded(&burst.update);
    super::video::prepare_primary_owners(
        primary,
        secondary,
        &mut cache.attributes,
        &bytes,
        u8::MAX,
    );
    let (origin_y, origin_x) = burst.particle_origin;
    let particles = collision::terrain_particles(
        actors,
        &content.geometry,
        origin_y,
        origin_x,
        (actors[0].position.world_x >> 8) as u8,
    );
    let mut submission = None;
    for command in particles.audio_commands {
        submission = Some(queue.submit(command));
    }
    submission
        .expect("source particle helper always submits its sound, even with a full actor pool")
}
struct FinaleContact<'a>(&'a Content);
impl crate::finale::Contact for FinaleContact<'_> {
    fn overlaps(
        &mut self,
        actors: &mut ActorPool,
        player: &mut player::Player,
        slot: usize,
    ) -> bool {
        contact_player(self.0, actors, player, slot)
    }
}
/// Source B10 visits the live pool in ascending order. A newly allocated later
/// slot runs now; a previously visited slot waits for the next invocation.
// Separate borrows keep canonical owners visible across immediate callbacks.
#[allow(clippy::too_many_arguments)]
pub fn update_actors(
    frame: &mut crate::progression::Frame<'_>,
    services: &mut super::simulation::Services,
    suppression: &mut [u8; actors::SUPPRESSION_BYTES],
    special: &mut SpecialControl,
    layout: u8,
    bank: u8,
    split_transition: &mut u8,
    first_increment: crate::progression::FirstActorIncrement,
) {
    frame.player.gravity = actors::DEFAULT_GRAVITY;
    for slot in actors::FIRST_ACTOR_SLOT..actors::ACTOR_SLOTS {
        if frame.actors[slot].state_code() & actors::ACTIVE == 0 {
            continue;
        }
        let before = super::journal::Snapshot::frame(frame);
        let handler = frame.actors[slot].handler;
        update_live_actor(
            frame,
            services,
            suppression,
            special,
            layout,
            bank,
            split_transition,
            slot,
            first_increment,
        );
        before.finish_frame(
            &services.events,
            frame,
            &services.content,
            &format!("actor_slot_{slot}"),
        );
        services.events.record("ActorHandlerBoundary", serde_json::json!({"slot":slot,"handler":handler,"special_control":special.encode(),"split_transition":*split_transition}));
    }
}
// Separate borrows keep canonical owners visible across immediate callbacks.
#[allow(clippy::too_many_arguments)]
fn update_live_actor(
    frame: &mut crate::progression::Frame<'_>,
    services: &mut super::simulation::Services,
    suppression: &mut [u8; actors::SUPPRESSION_BYTES],
    special: &mut SpecialControl,
    layout: u8,
    bank: u8,
    split_transition: &mut u8,
    slot: usize,
    first_increment: crate::progression::FirstActorIncrement,
) {
    use crate::{dialogue, finale, scenes};
    let handler = frame.actors[slot].handler;
    if handler == 0
        || update_family(
            frame,
            services,
            suppression,
            special,
            layout,
            bank,
            split_transition,
            slot,
            first_increment,
        )
    {
        return;
    }
    let family = match handler {
        value if value == dialogue::Family::ProgressionContact as u8 => {
            Some(dialogue::Family::ProgressionContact)
        }
        value if value == dialogue::Family::DialogueContact as u8 => {
            Some(dialogue::Family::DialogueContact)
        }
        value if value == dialogue::Family::HitActivated as u8 => {
            Some(dialogue::Family::HitActivated)
        }
        value if value == dialogue::Family::Escort as u8 => Some(dialogue::Family::Escort),
        value if value == dialogue::Family::RightwardTransition as u8 => {
            Some(dialogue::Family::RightwardTransition)
        }
        value if value == dialogue::Family::ConditionalMarker as u8 => {
            Some(dialogue::Family::ConditionalMarker)
        }
        _ => None,
    };
    if let Some(family) = family {
        // Moving the Dialogue owner gives the callback exclusive Services
        // access. It is restored before this non-yielding invocation returns.
        let mut dialogue = std::mem::take(&mut services.dialogue);
        let gravity = frame.player.gravity;
        dialogue::update_actor(
            family,
            slot,
            &mut dialogue::Context {
                dialogue: &mut dialogue,
                progression: frame.progression,
                player: frame.player,
                actors: frame.actors,
                status: frame.status,
                cache: frame.cache,
                camera: frame.camera,
                suppression,
            },
            &mut DialogueActorWorld {
                services,
                special,
                row: *frame.room_row,
                layout,
                bank,
                gravity,
            },
        );
        services.dialogue = dialogue;
        return;
    }
    match handler {
        scenes::DEPARTURE_HANDLER | scenes::SPAWNER_HANDLER => {
            let gravity = frame.player.gravity;
            let mut context = scenes::DepartureContext {
                progression: frame.progression,
                player: frame.player,
                status: frame.status,
                random: &mut services.runtime.random,
                departure_signal: &mut services.departure_signal,
            };
            if handler == scenes::DEPARTURE_HANDLER {
                scenes::linked_departure(
                    frame.actors,
                    slot,
                    &mut context,
                    &mut super::tasks::NativeMotion,
                    gravity,
                );
            } else {
                scenes::timed_spawner(
                    frame.actors,
                    slot,
                    &mut context,
                    &mut super::tasks::NativeMotion,
                    &services.content.scenes,
                );
            }
        }
        finale::FINAL_BOSS_HANDLER
        | finale::DEFEATED_HANDLER
        | finale::CLIMBER_HANDLER
        | finale::GOAL_HANDLER => {
            let effects = finale::actor_step(
                frame.actors,
                slot,
                &mut finale::ActorContext {
                    player: frame.player,
                    progression: frame.progression,
                    dialogue: &mut services.dialogue,
                    status: frame.status,
                    random: &mut services.runtime.random,
                    terrain: frame.cache,
                    current_row: *frame.room_row,
                    camera_page: (frame.camera.x >> 8) as u8,
                    random_increment: slot == actors::FIRST_ACTOR_SLOT
                        && first_increment == crate::progression::FirstActorIncrement::One,
                    catalog: &services.content.finale,
                },
                &mut FinaleContact(&services.content),
            );
            for request in effects.audio {
                match request {
                    finale::AudioRequest::Stage(command) => {
                        services.commands.submit_stage(command);
                    }
                    finale::AudioRequest::Effect(command) => {
                        services.commands.submit(command);
                    }
                }
            }
        }
        _ => unreachable!("validated original handler selector has no owner: {handler:02x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::simulation::Core;
    use super::*;
    #[test]
    fn moving_barrel_hits_transylvania_guard_and_starts_defeat_reaction() {
        use crate::{identifiers::StageId, progression};
        use romatlas_port::assets::{MemoryAssets, Rom};
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        // Isolate the actor impact from walls/floors. Keep original actor extents.
        content
            .geometry
            .responses
            .fill(collision::TerrainResponse::Pose);
        let barrel = content.spawns.templates[7];
        let guard = content.spawns.templates[36];
        assert_eq!((barrel.animation, barrel.handler), (0x1a, 8));
        assert_eq!((guard.animation, guard.handler), (0x38, 0x33));
        let mut services = super::super::simulation::Services::new(content);
        let content = std::sync::Arc::clone(&services.content);
        let mut core = Core::reset();
        core.progression.stage = StageId(1);
        for (slot, template, handler) in [
            (1, barrel, objects::MOVING_HANDLER),
            (2, guard, guard.handler),
        ] {
            core.actors[slot] = actors::ActorContact {
                control: actors::ActorControl::from_codes(0xa0, 0, 0, handler),
                render: actors::VISIBLE,
                handler,
                animation: template.animation,
                profile: template.profile,
                position: collision::Position {
                    world_x: 0x180,
                    screen_y: 0x80,
                    ..Default::default()
                },
                direction: 1,
                motion_flags: 1,
                ..Default::default()
            };
        }
        core.actors[1].horizontal_magnitude = 0x200;
        let (mut secondary_pending, mut guard_flag) = (0, 0);
        update_actors(
            &mut progression::Frame {
                progression: &mut core.progression,
                player: &mut core.player,
                actors: &mut core.actors,
                camera: &mut core.camera,
                cache: &mut core.cache,
                status: &mut core.status,
                camera_displacement: &mut core.camera_displacement,
                secondary_cursor: &mut core.secondary_cursor,
                secondary_pending: &mut secondary_pending,
                oam_guard: &mut guard_flag,
                room_row: &mut core.row,
                catalog: &content.progression,
            },
            &mut services,
            &mut core.suppression,
            &mut core.special_spawn,
            core.layout,
            core.terrain_bank,
            &mut core.split_transition,
            progression::FirstActorIncrement::Zero,
        );
        assert_eq!(
            core.actors[1].handler, 0,
            "barrel must break on enemy impact"
        );
        assert_eq!(core.actors[1].animation, actors::MARKED_CONTACT_ANIMATION);
        assert_eq!(core.actors[2].hit_marker, 1);
        assert_eq!(
            core.actors[2].handler,
            objects::LAUNCHED_HANDLER,
            "guard must enter defeat trajectory"
        );
        assert_eq!(core.actors[2].profile, 0);
    }
    #[test]
    fn player_collision_result_survives_approach_y_wrap_without_a_new_probe() {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        for (solid, replacement_probe) in [(true, false), (false, false), (true, true)] {
            content.geometry.responses.fill(if solid {
                collision::TerrainResponse::Solid
            } else {
                collision::TerrainResponse::Pose
            });
            let mut core = Core::reset();
            let mut collision_outcome = collision::CollisionOutcome::default();
            core.cache.collision.fill(if solid { 0x55 } else { 0 });
            core.actors[0].position.world_x = 1304;
            core.actors[0].position.screen_y = 180;
            core.actors[0].render = 0xa0;
            core.actors[0].vertical_velocity = -1;
            core.player.gravity = 0x40;
            let player_result = core.player.vertical(
                &mut core.actors,
                player::Vertical::Falling,
                0,
                &content.player,
                &mut contact::PlayerWorld {
                    content: &content,
                    collision: &core.cache.collision,
                    row: 4,
                    layout: 4,
                    bank: 3,
                    collision_outcome: &mut collision_outcome,
                    last_sample: None,
                    last_actor: None,
                    last_motion_return: None,
                    terrain_return: None,
                    final_operation: None,
                    special: &mut core.special_spawn,
                    total: &mut core.status.total,
                    ui_pending: &mut core.status.ui_pending,
                },
            );
            assert_eq!(player_result.collision_bits & 1, u8::from(solid));
            let actor = &mut core.actors[15];
            {
                let actor_value = 0xa2;
                actor.set_state_code(actor_value);
            };
            {
                let actor_value = 0x25;
                actor.set_handler(actor_value);
            };
            actor.render = 0xa0;
            actor.position.world_x = 1368;
            actor.position.screen_y = 254;
            actor.position.y_fraction = 180;
            actor.vertical_velocity = 0x1cc;
            actor.horizontal_magnitude = 0x1cc;
            actor.animation = 0x4c;
            {
                let animation_value = 3;
                actor.playback.set_tick_code(animation_value);
            };
            let mut secondary = presentation::PreparedSecondary::source_shared();
            let mut cursor = 0;
            let mut queue = audio::CommandQueue::default();
            let (mut base, mut active, mut pending) = ([0; 32], [0; 32], false);
            let mut random = runtime::RandomState::default();
            let mut special = core.special_spawn.encode();
            if replacement_probe {
                core.cache.collision.fill(0);
                content
                    .geometry
                    .responses
                    .fill(collision::TerrainResponse::Pose);
                core.actors[14].position.world_x = 1304;
                core.actors[14].position.screen_y = 100;
                let report = Terrain {
                    content: &content,
                    attributes: &mut core.cache.attributes,
                    cells: &mut core.cache.collision,
                    native: &mut world::NativeTerrain::default(),
                    secondary: &mut secondary,
                    cursor: &mut cursor,
                    row: 4,
                    layout: 4,
                    bank: 3,
                    stage: 4,
                    gravity: 0x40,
                    collision_outcome: &mut collision_outcome,
                }
                .motion(
                    &mut core.actors,
                    &mut core.player,
                    &mut special,
                    14,
                    collision::Movement::PositiveY,
                    0x1e,
                    2,
                    0x40,
                );
                assert_eq!(report.probe.as_ref().unwrap().bits, 0);
            }
            let effects = enemies::step(
                &mut core.actors,
                15,
                &mut enemies::Context {
                    player: &mut core.player,
                    random: &mut random,
                    status: &mut core.status,
                    special_spawn: &mut special,
                    gravity: 0x40,
                    geometry: &content.geometry,
                    spawn: &content.spawns,
                    catalog: &content.enemies,
                },
                &mut EnemyWorld {
                    terrain: Terrain {
                        content: &content,
                        attributes: &mut core.cache.attributes,
                        cells: &mut core.cache.collision,
                        native: &mut world::NativeTerrain::default(),
                        secondary: &mut secondary,
                        cursor: &mut cursor,
                        row: 4,
                        layout: 4,
                        bank: 3,
                        stage: 4,
                        gravity: 0x40,
                        collision_outcome: &mut collision_outcome,
                    },
                    recovery: Recovery {
                        queue: &mut queue,
                        base: &mut base,
                        active: &mut active,
                        palette_pending: &mut pending,
                        parameters: objects::RecoveryParameters {
                            stage: 4,
                            late_mode: false,
                            rule: crate::status::TitleRule::OrdinaryDamage,
                            catalog: &content.objects,
                        },
                        gravity: 0x40,
                    },
                },
            );
            let actor = core.actors[15];
            assert_eq!(
                (
                    actor.state_code(),
                    actor.render,
                    actor.position.screen_y,
                    actor.position.y_fraction
                ),
                (0, 0, 0, 128)
            );
            if solid && !replacement_probe {
                assert_eq!(
                    (
                        actor.motion_flags,
                        actor.animation,
                        actor.playback.frame_code(),
                        actor.playback.tick_code()
                    ),
                    (8, 0x4d, 0, 0)
                );
                assert_eq!(
                    (actor.vertical_velocity, actor.horizontal_magnitude),
                    (0x2d4, 0x100)
                );
                assert_eq!(effects.sounds, [0x24]);
                assert_eq!(queue.slots[0], 0x24);
            } else {
                assert_eq!(
                    (
                        actor.motion_flags,
                        actor.animation,
                        actor.playback.tick_code()
                    ),
                    (0, 0x4c, 3)
                );
                assert_eq!(
                    (actor.vertical_velocity, actor.horizontal_magnitude),
                    (0x1cc, 0x1cc)
                );
                assert!(effects.sounds.is_empty());
            }
        }
    }

    #[test]
    fn active_moon_terrain_gate_allocates_recovery_then_uses_actor_state_lookup() {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let content = Content::load(&rom, &raw, &overrides).unwrap();
        let mut core = Core::reset();
        for actor in &mut core.actors[1..] {
            {
                let actor_value = actors::ACTIVE;
                actor.set_state_code(actor_value);
            };
        }
        {
            let actor_value = 0;
            core.actors[14].set_state_code(actor_value);
        };
        core.actors[15].position.world_x = 312;
        core.actors[15].position.screen_y = 184;
        core.actors[15].spawn_y = 184;
        {
            let actor_value = 0xff;
            core.actors[15].set_local_counter_code(actor_value);
        };
        {
            let actor_value = 0;
            core.actors[15].set_local_state_code(actor_value);
        };
        core.actors[15].animation = 0x1a;
        core.cache.attributes[105] = 0xff;
        core.cache.collision[105] = 0xff;
        core.cache.changed_this_frame = 1; // actual caller claimed the frame latch
        let mut secondary = presentation::PreparedSecondary::source_shared();
        let mut cursor = 0;
        let mut terrain = Terrain {
            content: &content,
            attributes: &mut core.cache.attributes,
            cells: &mut core.cache.collision,
            native: &mut world::NativeTerrain::default(),
            secondary: &mut secondary,
            cursor: &mut cursor,
            row: 4,
            layout: 4,
            bank: 3,
            stage: 4,
            gravity: 0x40,
            collision_outcome: &mut collision::CollisionOutcome::default(),
        };
        terrain.replace(&mut core.actors, 15, &mut core.cache.changed_this_frame);
        assert_eq!(
            (
                core.actors[14].state_code(),
                core.actors[14].handler,
                core.actors[14].animation
            ),
            (0xa1, 0x13, 0x24)
        );
        assert_eq!(
            (
                core.actors[14].position.world_x,
                core.actors[14].position.screen_y
            ),
            (312, 184)
        );
        // Source F653[stage4*4+local0] is tile2 with Moon packed property00.
        assert_eq!(
            (core.cache.attributes[105], core.cache.collision[105]),
            (0x3f, 0x3f)
        );
        assert_eq!((core.cache.changed_this_frame, cursor), (2, 14));
    }

    #[test]
    fn waiting_jumper_submits_step_before_reaction_and_never_replays_dropped_sound() {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content
            .geometry
            .responses
            .fill(collision::TerrainResponse::Solid);
        let mut core = Core::reset();
        core.cache.collision.fill(0x55);
        let slot = 3;
        {
            let actor_value = 0x82;
            core.actors[slot].set_state_code(actor_value);
        };
        {
            let actor_value = 0x10;
            core.actors[slot].set_handler(actor_value);
        };
        core.actors[slot].render = 0x80;
        core.actors[slot].hit_marker = 1;
        core.actors[slot].position.world_x = 128;
        core.actors[slot].position.screen_y = 128;
        core.actors[slot].direction = 1;
        let mut queue = audio::CommandQueue::default();
        // The step command occupies slot0; reaction then sees occupied slot1,
        // whose below88 comparison contributes zero to its immediate RNG sum.
        queue.slots[1] = 0x11;
        let mut random = runtime::RandomState {
            words: [1, 1, 0, 0],
        };
        let mut special = 0;
        let mut secondary = presentation::PreparedSecondary::source_shared();
        let mut cursor = 0;
        let (mut base, mut active, mut pending) = ([0; 32], [0; 32], false);
        let effects = enemies::step(
            &mut core.actors,
            slot,
            &mut enemies::Context {
                player: &mut core.player,
                random: &mut random,
                status: &mut core.status,
                special_spawn: &mut special,
                gravity: 0x40,
                geometry: &content.geometry,
                spawn: &content.spawns,
                catalog: &content.enemies,
            },
            &mut EnemyWorld {
                terrain: Terrain {
                    content: &content,
                    attributes: &mut core.cache.attributes,
                    cells: &mut core.cache.collision,
                    native: &mut world::NativeTerrain::default(),
                    secondary: &mut secondary,
                    cursor: &mut cursor,
                    row: 0,
                    layout: 0,
                    bank: 0,
                    stage: 0,
                    gravity: 0x40,
                    collision_outcome: &mut collision::CollisionOutcome::default(),
                },
                recovery: Recovery {
                    queue: &mut queue,
                    base: &mut base,
                    active: &mut active,
                    palette_pending: &mut pending,
                    parameters: objects::RecoveryParameters {
                        stage: 0,
                        late_mode: false,
                        rule: crate::status::TitleRule::OrdinaryDamage,
                        catalog: &content.objects,
                    },
                    gravity: 0x40,
                },
            },
        );
        assert_eq!(effects.sounds, [0x31]);
        assert_eq!(effects.submitted_sounds, [0]);
        assert_eq!(
            (queue.slots[0], queue.slots[1], queue.producer),
            (0x31, 0x11, 1)
        );
        assert_eq!(
            random.words[1], 2,
            "reaction must observe the producer after step submission"
        );
        let before = queue.clone();
        for (index, &command) in effects.sounds.iter().enumerate() {
            if !effects.submitted_sounds.contains(&index) {
                queue.submit(command);
            }
        }
        assert_eq!(
            queue, before,
            "family observations never replay an attempted command"
        );
        assert!(
            enemies::World::submit_sound(
                &mut EnemyWorld {
                    terrain: Terrain {
                        content: &content,
                        attributes: &mut core.cache.attributes,
                        cells: &mut core.cache.collision,
                        native: &mut world::NativeTerrain::default(),
                        secondary: &mut secondary,
                        cursor: &mut cursor,
                        row: 0,
                        layout: 0,
                        bank: 0,
                        stage: 0,
                        gravity: 0x40,
                        collision_outcome: &mut collision::CollisionOutcome::default(),
                    },
                    recovery: Recovery {
                        queue: &mut queue,
                        base: &mut base,
                        active: &mut active,
                        palette_pending: &mut pending,
                        parameters: objects::RecoveryParameters {
                            stage: 0,
                            late_mode: false,
                            rule: crate::status::TitleRule::OrdinaryDamage,
                            catalog: &content.objects
                        },
                        gravity: 0x40,
                    },
                },
                0x27
            ),
            "a dropped attempt is still submitted provenance"
        );
        assert_eq!(queue, before);
    }
    #[test]
    fn stage_one_extension_and_suppression_do_not_overwrite_late_stage_mode() {
        use crate::{identifiers::StageId, progression};
        use romatlas_port::assets::{MemoryAssets, Rom};
        const EXTENSION_ANIMATION: u8 = 0x2b;
        const EXTENSION_SUPPRESSION_ID: u8 = 37;
        let rom = Rom::from_bytes(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap(),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content.geometry.extents[0] = collision::Extent {
            half_width: 16,
            half_height: 16,
        };
        content.spawns.suppression[1] = vec![actors::SuppressionRule {
            field: actors::ProgressField::SourceAe,
            mask: 1,
            destination: EXTENSION_SUPPRESSION_ID,
        }];
        let mut services = super::super::simulation::Services::new(content);
        let content = std::sync::Arc::clone(&services.content);
        let mut core = Core::reset();
        core.progression.stage = StageId(1);
        core.progression.late_mode = 0x80;
        core.actors[3] = actors::ActorContact {
            control: crate::actors::ActorControl::from_codes(0xa2, 0, 0, objects::RECOVERY_HANDLER),
            render: actors::VISIBLE,
            handler: objects::RECOVERY_HANDLER,
            animation: EXTENSION_ANIMATION,
            ..actors::ActorContact::default()
        };
        let (mut secondary_pending, mut guard) = (0, 0);
        assert!(update_family(
            &mut progression::Frame {
                progression: &mut core.progression,
                player: &mut core.player,
                actors: &mut core.actors,
                camera: &mut core.camera,
                cache: &mut core.cache,
                status: &mut core.status,
                camera_displacement: &mut core.camera_displacement,
                secondary_cursor: &mut core.secondary_cursor,
                secondary_pending: &mut secondary_pending,
                oam_guard: &mut guard,
                room_row: &mut core.row,
                catalog: &content.progression
            },
            &mut services,
            &mut core.suppression,
            &mut core.special_spawn,
            core.layout,
            core.terrain_bank,
            &mut core.split_transition,
            3,
            progression::FirstActorIncrement::Zero
        ));
        assert_eq!(
            (
                core.actors[3].state_code(),
                core.progression.stage_one_extension,
                core.progression.late_mode
            ),
            (0, 1, 0x80)
        );
        super::super::tasks::rebuild_suppression(&mut core, &services);
        assert_ne!(
            core.suppression[usize::from(EXTENSION_SUPPRESSION_ID >> 3)]
                & (1 << (EXTENSION_SUPPRESSION_ID & 7)),
            0
        );
    }
}
