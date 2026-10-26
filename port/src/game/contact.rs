//! Immediate collision boundaries over canonical owners. PlayerContact and
//! ActorVisual-style projections are call-local views, committed before return.
use super::content::Content;
use crate::{actors, collision, player, world};

pub fn player_contact(
    player: &player::Player,
    actor: &actors::ActorContact,
) -> collision::PlayerContact {
    collision::PlayerContact {
        animation: actor.animation,
        protection: player.protection,
        special_mode: player.special_mode,
        momentum_control: player.momentum_control,
        immediate_state: player.action as u8,
        queued_state: player.queued.map_or(0, |action| action as u8),
        actor_contact_flags: player.actor_contacts,
        motion: actor.motion_flags,
        vertical_velocity: actor.vertical_velocity,
    }
}
pub fn commit_player_contact(contact: collision::PlayerContact, player: &mut player::Player) {
    player.momentum_control = contact.momentum_control;
    player.action = action(contact.immediate_state);
    player.queued = (contact.queued_state != 0).then(|| action(contact.queued_state));
    player.actor_contacts = contact.actor_contact_flags;
}
pub fn action(value: u8) -> player::Action {
    use player::Action::*;
    match value {
        0 => Standing,
        1 => Airborne,
        2 => ForcedJump,
        3 => Climbing,
        4 => Swing,
        5 => Protected,
        6 => Damage,
        7 => TerrainPose,
        8 => FallOut,
        9 => WaitA,
        10 => CarriedLocked,
        11 => CarriedJumpable,
        12 => ExternalLaunch,
        13 => Death,
        14 => Hold,
        15 => Dialogue,
        16 => ConvertTimer,
        _ => unreachable!("source player action selector"),
    }
}
/// The descriptor consumer uses byte room arithmetic even for a retained raw
/// selector. The normal authored layouts fit the same byte cursor domain.
pub fn descriptor(content: &Content, layout: u8, bank: u8, probe: world::TerrainProbe) -> u8 {
    let layout = if usize::from(layout) < content.world.layouts.len() {
        &content.world.layouts[usize::from(layout)]
    } else {
        content.raw_layouts[usize::from(layout)]
            .as_ref()
            .expect("raw descriptor pointer consumer")
    };
    let index = probe
        .room_row
        .wrapping_mul(layout.columns)
        .wrapping_add(probe.world_page);
    let room = layout
        .rooms
        .get(usize::from(index))
        .copied()
        .flatten()
        .unwrap_or(usize::from(world::MISSING_ROOM));
    let room = &content.raw_rooms[room];
    let coordinate = usize::from(probe.block_coordinate & 63);
    let row = coordinate / 8;
    let column = coordinate % 8;
    let block = usize::from(room.block_at(row, column));
    content.world.terrain[usize::from(bank)].blocks[block][usize::from(probe.quadrant)]
}
/// Semantic integration result consumed at the original player's return boundary.
/// This records a named movement result, not a generic processor carry register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionReturn {
    PositiveXOverflow(bool),
    NegativeXNoBorrow(bool),
    VerticalNoBorrow(bool),
}
/// Final arithmetic of the actor-zero terrain helper. Later player animation
/// comparisons can supersede this return; movement integration cannot stand in
/// for it. This is retained only for the current native player call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainReturn {
    SlotZeroComparison,
    PositiveXSnapNoBorrow(bool),
    NegativeXSnapOverflow(bool),
    PositiveYSnapNoBorrow(bool),
    NegativeYSnapOverflow(bool),
}
impl TerrainReturn {
    pub fn comparison(self) -> bool {
        match self {
            Self::SlotZeroComparison => true,
            Self::PositiveXSnapNoBorrow(value)
            | Self::NegativeXSnapOverflow(value)
            | Self::PositiveYSnapNoBorrow(value)
            | Self::NegativeYSnapOverflow(value) => value,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerCallReturn {
    Policy(player::ReturnOperation),
    Terrain(TerrainReturn),
    Motion(MotionReturn),
    ActorOverlap { no_contact: bool },
    Gravity { no_borrow: bool },
    Audio { command: u8, stage_command: bool },
    ActorZeroWrapper { identity_comparison: Option<bool> },
}
impl PlayerCallReturn {
    pub fn first_actor_increment(
        self,
        audio_return: Option<(u8, bool, crate::audio::Submission)>,
    ) -> crate::progression::FirstActorIncrement {
        use crate::progression::FirstActorIncrement::{One, Zero};
        let comparison = match self {
            Self::Policy(operation) => operation.comparison(),
            Self::Terrain(operation) => operation.comparison(),
            Self::Motion(
                MotionReturn::PositiveXOverflow(value)
                | MotionReturn::NegativeXNoBorrow(value)
                | MotionReturn::VerticalNoBorrow(value),
            ) => value,
            Self::ActorOverlap { no_contact } => no_contact,
            Self::Gravity { no_borrow } => no_borrow,
            Self::ActorZeroWrapper {
                identity_comparison,
            } => identity_comparison.unwrap_or(true),
            Self::Audio {
                command,
                stage_command,
            } => {
                let (submitted_command, submitted_stage, submission) = audio_return
                    .expect("resolve the actual reported player audio producer before return");
                assert_eq!(
                    (command, stage_command),
                    (submitted_command, submitted_stage)
                );
                return submission.first_actor_increment();
            }
        };
        if comparison { One } else { Zero }
    }
}
pub struct PlayerWorld<'a> {
    pub content: &'a Content,
    pub collision: &'a dyn collision::CollisionCells,
    pub row: u8,
    pub layout: u8,
    pub bank: u8,
    pub collision_outcome: &'a mut collision::CollisionOutcome,
    pub last_sample: Option<collision::Sample>,
    pub last_actor: Option<usize>,
    pub last_motion_return: Option<MotionReturn>,
    pub terrain_return: Option<TerrainReturn>,
    pub final_operation: Option<PlayerCallReturn>,
    pub special: &'a mut actors::SpecialControl,
    pub total: &'a mut crate::status::TotalMoney,
    pub ui_pending: &'a mut u8,
}
impl player::ContactWorld for PlayerWorld<'_> {
    fn collision_returned(&mut self, bits: u8) {
        self.collision_outcome.player_aggregate(bits);
    }
    fn returned(&mut self, operation: player::ReturnOperation) {
        self.final_operation = Some(PlayerCallReturn::Policy(operation));
    }
    fn returned_audio(&mut self, command: u8, stage_command: bool) {
        self.final_operation = Some(PlayerCallReturn::Audio {
            command,
            stage_command,
        });
    }
    fn overlay_cost(&mut self, health: &mut crate::status::Health) {
        crate::status::Status::pay_overlay_parts(self.total, health, self.ui_pending);
    }
    fn positive_with_collision(
        &mut self,
        actors: &mut actors::ActorPool,
        player: &mut player::Player,
    ) -> player::CollisionFacts {
        let mut contact = player_contact(player, &actors[0]);
        let mut special = self.special.encode();
        let identity_comparison = actors::ACTIVE >= special;
        let report = collision::move_with_collision(
            0,
            actors,
            &mut contact,
            self.collision,
            self.row,
            &self.content.geometry,
            collision::ProbeRange {
                start: 0,
                remaining: 2,
            },
            collision::Movement::PositiveX,
            player.gravity,
            &mut special,
            &mut |query| descriptor(self.content, self.layout, self.bank, query),
        );
        *self.special = actors::SpecialControl::decode(special);
        commit_player_contact(contact, player);
        // F10D ends accepted actor-zero wrappers with CPX #0, superseding snap
        // arithmetic. The release path instead retains CMP ActiveSlot0,#special.
        self.final_operation = Some(PlayerCallReturn::ActorZeroWrapper {
            identity_comparison: report.removed.then_some(identity_comparison),
        });
        let mut facts = player::CollisionFacts {
            actor_bits: player.actor_contacts,
            render: actors[0].render,
            ..player::CollisionFacts::default()
        };
        self.collision_outcome.movement(&report);
        if let Some(probe) = report.probe {
            self.last_sample = probe.last_sample;
            facts.bits = probe.bits;
            for (index, value) in probe.results {
                if let Some(result) = facts.classes.get_mut(usize::from(index)) {
                    *result = value;
                }
            }
        }
        facts
    }
    fn gravity(&mut self, actor: &mut actors::ActorContact, step: u8) {
        let no_borrow = actor.vertical_velocity as u16 >= u16::from(step);
        collision::gravity(actor, step);
        self.final_operation = Some(PlayerCallReturn::Gravity { no_borrow });
    }
    fn integrate_y(&mut self, actor: &mut actors::ActorContact) {
        let no_borrow = !collision::integrate_vertical(
            actor,
            collision::VerticalOperation::SubtractEncodedVelocity,
        );
        self.last_motion_return = Some(MotionReturn::VerticalNoBorrow(no_borrow));
        self.final_operation = Some(PlayerCallReturn::Motion(self.last_motion_return.unwrap()));
    }
    fn integrate_x(&mut self, actor: &mut actors::ActorContact, direction: player::Horizontal) {
        let boundary = collision::integrate_horizontal(actor, direction);
        self.last_motion_return = Some(match direction {
            collision::HorizontalDirection::Positive => MotionReturn::PositiveXOverflow(boundary),
            collision::HorizontalDirection::Negative => MotionReturn::NegativeXNoBorrow(!boundary),
        });
        self.final_operation = Some(PlayerCallReturn::Motion(self.last_motion_return.unwrap()));
    }
    fn scan(
        &mut self,
        actors: &mut actors::ActorPool,
        player: &mut player::Player,
        vertical: bool,
    ) -> player::CollisionFacts {
        let mut contact = player_contact(player, &actors[0]);
        let report = collision::scan_player(
            if vertical {
                collision::ContactAxis::Vertical
            } else {
                collision::ContactAxis::Horizontal
            },
            actors,
            &mut contact,
            &self.content.geometry,
        );
        commit_player_contact(contact, player);
        self.last_actor = report.last_contact.map(|contact| contact.slot);
        self.final_operation = Some(PlayerCallReturn::ActorOverlap {
            no_contact: self.last_actor.is_none(),
        });
        player::CollisionFacts {
            actor: self.last_actor,
            bits: report.bits,
            actor_bits: player.actor_contacts,
            penetration: report.last_contact.map(|contact| contact.penetration),
            render: actors[0].render,
            ..player::CollisionFacts::default()
        }
    }
    fn snap(&mut self, actor: &mut actors::ActorContact, direction: collision::Snap) {
        let sample = self
            .last_sample
            .expect("terrain snap follows an actual probe");
        // D49A/D4B2/D4C8/D4D6 return the final integer adjustment's
        // subtraction no-borrow or addition overflow, before the actor mutates.
        self.terrain_return = Some(match direction {
            collision::Snap::PositiveX => TerrainReturn::PositiveXSnapNoBorrow(
                actor.position.world_x >= (sample.world_x & 15),
            ),
            collision::Snap::NegativeX => TerrainReturn::NegativeXSnapOverflow(
                actor
                    .position
                    .world_x
                    .overflowing_add(16 - (sample.world_x & 15))
                    .1,
            ),
            collision::Snap::PositiveY => TerrainReturn::PositiveYSnapNoBorrow(
                actor.position.screen_y >= (sample.screen_y & 15),
            ),
            collision::Snap::NegativeY => TerrainReturn::NegativeYSnapOverflow(
                actor
                    .position
                    .screen_y
                    .overflowing_add(16 - (sample.screen_y & 15))
                    .1,
            ),
        });
        collision::snap(actor, sample, direction);
        self.final_operation = Some(PlayerCallReturn::Terrain(self.terrain_return.unwrap()));
    }
    fn probe(
        &mut self,
        actors: &mut actors::ActorPool,
        player: &mut player::Player,
        start: u8,
        remaining: u8,
    ) -> player::CollisionFacts {
        let mut contact = player_contact(player, &actors[0]);
        let report = collision::probe(
            0,
            &mut actors[0],
            &mut contact,
            self.collision,
            self.row,
            &self.content.geometry,
            collision::ProbeRange { start, remaining },
            &mut |query| descriptor(self.content, self.layout, self.bank, query),
        );
        commit_player_contact(contact, player);
        self.collision_outcome.player_aggregate(report.bits);
        self.last_sample = report.last_sample;
        // Every actor-zero probe response rejoins D13E's CPX #0. A subsequent
        // solid snap supersedes this with its actual boundary arithmetic.
        self.terrain_return = Some(TerrainReturn::SlotZeroComparison);
        self.final_operation = Some(PlayerCallReturn::Terrain(TerrainReturn::SlotZeroComparison));
        let mut facts = player::CollisionFacts {
            bits: report.bits,
            actor_bits: player.actor_contacts,
            render: actors[0].render,
            ..player::CollisionFacts::default()
        };
        for (index, value) in report.results {
            if let Some(result) = facts.classes.get_mut(usize::from(index)) {
                *result = value;
            }
        }
        facts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_actor_overlap_does_not_set_the_descending_pogo_latch() {
        use romatlas_port::assets::{MemoryAssets, Rom};
        let bytes =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let rom = Rom::from_bytes(bytes, crate::config::ROM_SHA256).unwrap();
        let overrides = MemoryAssets::default();
        let raw = crate::assets::Assets::load(&rom, &overrides).unwrap();
        let mut content = Content::load(&rom, &raw, &overrides).unwrap();
        content.geometry.extents[1] = collision::Extent {
            half_width: 16,
            half_height: 16,
        };
        let cells = [0; world::CACHE_CELLS];
        let mut special = actors::SpecialControl::Ready;
        let mut total: crate::status::TotalMoney = [0; 7].into();
        let mut ui_pending = 0;
        let mut boundary = PlayerWorld {
            content: &content,
            collision: &cells,
            row: 0,
            layout: 0,
            bank: 0,
            collision_outcome: &mut collision::CollisionOutcome::default(),
            last_sample: None,
            last_actor: None,
            last_motion_return: None,
            terrain_return: None,
            final_operation: None,
            special: &mut special,
            total: &mut total,
            ui_pending: &mut ui_pending,
        };
        let mut actors = [actors::ActorContact::default(); actors::ACTOR_SLOTS];
        let mut player = player::Player::default();
        actors[1].render = collision::VISIBLE;
        actors[1].profile = collision::PROFILE_ENABLED | 1;
        actors[0].vertical_velocity = -1;
        actors[0].motion_flags = collision::DESCENDING_MOTION;
        let ordinary = player::ContactWorld::scan(&mut boundary, &mut actors, &mut player, true);
        assert_eq!(
            (ordinary.bits, ordinary.actor_bits, player.actor_contacts),
            (collision::SOLID_CONTACT, 0, 0)
        );
        actors[0].animation = collision::POGO_POSE;
        let pogo = player::ContactWorld::scan(&mut boundary, &mut actors, &mut player, true);
        assert_eq!(
            (pogo.actor_bits, player.actor_contacts, actors[1].hit_marker),
            (collision::SOLID_CONTACT, collision::SOLID_CONTACT, 1)
        );
        assert_eq!(pogo.actor, Some(1));
        let sample = collision::Sample {
            class: 0,
            world_x: 15,
            screen_y: 15,
            cache_index: None,
            quadrant: None,
            descriptor: None,
        };
        boundary.last_sample = Some(sample);
        for (direction, x, y, expected) in [
            (
                collision::Snap::PositiveX,
                14,
                0,
                TerrainReturn::PositiveXSnapNoBorrow(false),
            ),
            (
                collision::Snap::PositiveX,
                15,
                0,
                TerrainReturn::PositiveXSnapNoBorrow(true),
            ),
            (
                collision::Snap::NegativeX,
                u16::MAX,
                0,
                TerrainReturn::NegativeXSnapOverflow(true),
            ),
            (
                collision::Snap::NegativeX,
                0,
                0,
                TerrainReturn::NegativeXSnapOverflow(false),
            ),
            (
                collision::Snap::PositiveY,
                0,
                14,
                TerrainReturn::PositiveYSnapNoBorrow(false),
            ),
            (
                collision::Snap::PositiveY,
                0,
                15,
                TerrainReturn::PositiveYSnapNoBorrow(true),
            ),
            (
                collision::Snap::NegativeY,
                0,
                255,
                TerrainReturn::NegativeYSnapOverflow(true),
            ),
            (
                collision::Snap::NegativeY,
                0,
                0,
                TerrainReturn::NegativeYSnapOverflow(false),
            ),
        ] {
            actors[0].position.world_x = x;
            actors[0].position.screen_y = y;
            player::ContactWorld::snap(&mut boundary, &mut actors[0], direction);
            assert_eq!(boundary.terrain_return, Some(expected));
        }
        for (control, expected) in [
            (actors::SpecialControl::Occupied(0), true),
            (actors::SpecialControl::Occupied(15), false),
        ] {
            *boundary.special = control;
            {
                let actor_value = actors::ACTIVE;
                actors[0].set_state_code(actor_value);
            };
            actors[0].position.world_x = u16::MAX;
            actors[0].position.x_fraction = u8::MAX;
            actors[0].horizontal_magnitude = player::NORMAL_SPEED;
            let result = player::ContactWorld::positive_with_collision(
                &mut boundary,
                &mut actors,
                &mut player,
            );
            assert_eq!(result.bits, 0);
            assert_eq!(actors[0].state_code(), 0);
            assert_eq!(
                boundary.final_operation,
                Some(PlayerCallReturn::ActorZeroWrapper {
                    identity_comparison: Some(expected)
                })
            );
            assert_eq!(
                *boundary.special,
                if expected {
                    actors::SpecialControl::Cooldown(0x3c)
                } else {
                    control
                }
            );
        }
        *boundary.special = actors::SpecialControl::Ready;
        actors[0].position.world_x = 0x100;
        actors[0].position.x_fraction = 0;
        {
            let actor_value = actors::ACTIVE;
            actors[0].set_state_code(actor_value);
        };
        player::ContactWorld::positive_with_collision(&mut boundary, &mut actors, &mut player);
        assert_eq!(
            boundary.final_operation,
            Some(PlayerCallReturn::ActorZeroWrapper {
                identity_comparison: None
            })
        );
        *boundary.total = [0, 3, crate::status::BLANK, 0, 0, 0, 7].into();
        let mut health = crate::status::Health::from_overlay(
            std::array::from_fn(|index| crate::status::HealthSegment {
                y: if index == 3 { u8::MAX } else { 0x20 },
                tile: 0,
                depleted: u8::from(index == 2),
                x: 0,
            }),
            u8::MAX,
            1,
        );
        player::ContactWorld::overlay_cost(&mut boundary, &mut health);
        assert_eq!(
            boundary.total.display_codes(),
            [
                crate::status::BLANK,
                crate::status::BLANK,
                crate::status::BLANK,
                0,
                0,
                0,
                7
            ]
        );
        assert_eq!(
            (
                *boundary.ui_pending,
                health.alternating,
                health.overlay_segments()[2].depleted
            ),
            (1, 0, 0)
        );
    }
}
