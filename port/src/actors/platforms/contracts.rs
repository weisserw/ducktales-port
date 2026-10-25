//! Independently derived B09 contracts. Source ranges, all branches and call
//! semantics are in trace.md; fixture boundary outcomes are not game oracles.
use super::*;
use crate::{
    actors::{ACTIVE, ACTOR_SLOTS, ActorContact, ActorPool, SpecialControl, VISIBLE},
    collision::Position,
    player::{Action, CLIMB_ANIMATION},
};
use std::collections::VecDeque;
const SUBJECT: usize = 3;
const TARGET: usize = 7;
const ACTOR_X: u16 = 0x0250;
const ACTOR_Y: u8 = 0x80;
const UP: u8 = 8;
const SOLID: u8 = 1;
const READY: u8 = ACTIVE | 1;
const DAMAGE_PROFILE: u8 = 0x90;
const PUSH_EVENT: u8 = 3;
const PUSH_FINISH: u8 = 5;
const SOUND_WALK: u8 = 0x27;
const SOUND_FLIGHT: u8 = 0x2b;
const SOUND_RIDE: u8 = 0x26;
const SOUND_CARRY: u8 = 0x33;
const FALL_SAMPLE: u8 = 0x31;
const WALK_RIGHT: u8 = 0x34;
const WALK_LEFT: u8 = 0x37;
const PUSH_FALL: u8 = 0x4a;
const PUSH_RIGHT: u8 = 0x44;
const PUSH_LEFT: u8 = 0x47;
const CARRY_LEFT: u8 = 0x38;
const EXTERNAL_RIGHT: u8 = 0x35;
#[derive(Default)]
struct Boundary {
    calls: Vec<Work>,
    facts: VecDeque<MotionFacts>,
    permission: bool,
    contact_action: Option<Action>,
    contact_seen: Vec<Position>,
    target: Option<usize>,
    exit: bool,
    integrated: Option<Position>,
    moved: Option<Position>,
    preset_vertical: i16,
    preset_horizontal: u16,
    before_integrate: Vec<Position>,
    terrain_seen: Vec<(usize, u8, u8)>,
}
impl World for Boundary {
    fn motion(
        &mut self,
        a: &mut ActorPool,
        _: &mut crate::player::Player,
        _: &mut crate::actors::SpecialControl,
        s: usize,
        m: Movement,
        p: u8,
        n: u8,
    ) -> MotionFacts {
        self.calls.push(Work::Motion(m, p, n));
        if let Some(position) = self.moved {
            a[s].position = position;
        }
        self.facts.pop_front().unwrap_or_default()
    }
    fn integrate(&mut self, a: &mut ActorPool, s: usize, o: Integration) -> bool {
        self.calls.push(Work::Integrate(o));
        self.before_integrate.push(a[s].position);
        if let Some(p) = self.integrated {
            a[s].position = p;
        }
        self.exit
    }
    fn player_contact(
        &mut self,
        a: &mut ActorPool,
        p: &mut crate::player::Player,
        s: usize,
    ) -> bool {
        self.calls.push(Work::PlayerContact(a[s].profile));
        if let Some(action) = self.contact_action {
            p.action = action;
        }
        self.contact_seen.push(a[s].position);
        self.permission
    }
    fn actor_contact(&mut self, _: &mut ActorPool, _: usize) -> Option<usize> {
        self.calls.push(Work::ActorContact);
        self.target
    }
    fn preset(&mut self, a: &mut ActorPool, s: usize, p: u8) {
        self.calls.push(Work::Preset(p));
        a[s].vertical_velocity = self.preset_vertical;
        a[s].horizontal_magnitude = self.preset_horizontal;
    }
    fn reaction(
        &mut self,
        a: &mut ActorPool,
        _: &mut crate::player::Player,
        s: usize,
        _: &mut SpecialControl,
    ) -> Effects {
        self.calls.push(Work::Reaction);
        {
            let actor_value = crate::actors::objects::LAUNCHED_HANDLER;
            a[s].set_handler(actor_value);
        };
        Effects {
            sounds: vec![0x35],
            ..Effects::default()
        }
    }
    fn activate_terrain(
        &mut self,
        a: &mut ActorPool,
        p: &mut crate::player::Player,
        s: usize,
        c: &mut u8,
    ) -> Effects {
        self.calls.push(Work::ActivateTerrain(s));
        self.terrain_seen.push((s, p.pose_direction, *c));
        {
            let actor_value = crate::actors::objects::MOVING_HANDLER;
            a[s].set_handler(actor_value);
        };
        *c = 1;
        Effects {
            sounds: vec![0x16],
            ..Effects::default()
        }
    }
}
struct Fixture {
    actors: ActorPool,
    player: crate::player::Player,
    special: SpecialControl,
    changed: u8,
    completion: u8,
    held: u8,
    catalog: Catalog,
    world: Boundary,
}
impl Fixture {
    fn new(handler: u8) -> Self {
        let mut actors = [ActorContact::default(); ACTOR_SLOTS];
        actors[SUBJECT] = ActorContact {
            playback: crate::presentation::AnimationPlayback::from_codes(1, 0, 0),
            control: crate::actors::ActorControl::from_codes(ACTIVE, 0, 0, 0),
            handler,
            render: VISIBLE,
            profile: PLATFORM_PROFILE,
            position: Position {
                world_x: ACTOR_X,
                screen_y: ACTOR_Y,
                ..Position::default()
            },
            spawn_identity: Some(0x42),
            ..ActorContact::default()
        };
        actors[0].position = actors[SUBJECT].position;
        Self {
            actors,
            player: crate::player::Player::default(),
            special: SpecialControl::Occupied(SUBJECT),
            changed: 0,
            completion: 0,
            held: 0,
            catalog: Catalog {
                flight: [2; FLIGHT_ENTRIES],
                ride: [3; RIDE_ENTRIES],
                ride_duration: [15, 5],
                phases: [2, 1, 1, 2],
                player_steps: [1, -1],
                push_steps: [-2, 2],
            },
            world: Boundary::default(),
        }
    }
    fn ready(&mut self) {
        {
            let actor_value = READY;
            self.actors[SUBJECT].set_state_code(actor_value);
        };
    }
    fn actor(&self) -> &ActorContact {
        &self.actors[SUBJECT]
    }
    fn actor_mut(&mut self) -> &mut ActorContact {
        &mut self.actors[SUBJECT]
    }
    fn hit(&mut self, b: u8) {
        self.world.facts.push_back(MotionFacts {
            bits: b,
            continuation_remaining: 0,
        });
    }
    fn run(&mut self) -> Effects {
        self.run_slot(SUBJECT)
    }
    fn run_slot(&mut self, slot: usize) -> Effects {
        step(
            &mut self.actors,
            slot,
            &mut Context {
                player: &mut self.player,
                special: &mut self.special,
                changed: &mut self.changed,
                completion: &mut self.completion,
                held: self.held,
                catalog: &self.catalog,
            },
            &mut self.world,
        )
    }
}
fn contact(profile: u8) -> Work {
    Work::PlayerContact(profile)
}
fn motion(m: Movement, p: u8, n: u8) -> Work {
    Work::Motion(m, p, n)
}
fn integrate(o: Integration) -> Work {
    Work::Integrate(o)
}
#[test]
fn catalog_decodes_all_platform_program_entries_and_signed_pairs_from_matching_local_rom() {
    use romatlas_port::assets::Rom;
    let b =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local matching ROM");
    Rom::from_bytes(b.clone(), crate::config::ROM_SHA256).expect("ROM identity");
    let c = Catalog::original(&b).unwrap();
    const FLIGHT_PHYSICAL: usize = 0x19225;
    const RIDE_PHYSICAL: usize = 0x192fd;
    const DURATION_PHYSICAL: usize = 0x19321;
    const PUSH_PHYSICAL: usize = 0x19d96;
    const PHASE_PHYSICAL: usize = 0x1fee6;
    assert_eq!(
        c.flight.as_slice(),
        &b[FLIGHT_PHYSICAL..FLIGHT_PHYSICAL + FLIGHT_ENTRIES]
    );
    assert_eq!(
        c.ride.as_slice(),
        &b[RIDE_PHYSICAL..RIDE_PHYSICAL + RIDE_ENTRIES]
    );
    assert_eq!(
        c.ride_duration.as_slice(),
        &b[DURATION_PHYSICAL..DURATION_PHYSICAL + 2]
    );
    assert_eq!(c.phases.as_slice(), &b[PHASE_PHYSICAL..PHASE_PHYSICAL + 4]);
    for (index, byte_offset) in [(0, 0), (1, 2)] {
        assert_eq!(
            c.push_steps[index],
            i16::from_le_bytes([
                b[PUSH_PHYSICAL + byte_offset],
                b[PUSH_PHYSICAL + byte_offset + 1]
            ])
        );
        assert_eq!(
            c.player_steps[index],
            i16::from_le_bytes([
                b[PHASE_PHYSICAL + 4 + byte_offset],
                b[PHASE_PHYSICAL + 5 + byte_offset]
            ])
        );
    }
}
#[test]
fn catalog_rejects_truncated_last_consumed_step_without_reading_suffix() {
    const LAST_REQUIRED_PHYSICAL: usize = 0x1feed;
    assert!(Catalog::original(&vec![0; LAST_REQUIRED_PHYSICAL]).is_err());
}
#[test]
fn vertical_ascending_at_upper_threshold_does_not_reverse_or_test_contact() {
    let mut f = Fixture::new(VERTICAL);
    f.world.integrated = Some(Position {
        screen_y: 0x68,
        ..f.actor().position
    });
    f.run();
    assert_eq!(f.actor().vertical_velocity, WALK_SPEED as i16);
    assert_eq!(f.actor().state_code(), ACTIVE);
    assert_eq!(
        f.world.calls,
        vec![integrate(Integration::Vertical(
            VerticalOperation::SubtractEncodedVelocity
        ))]
    );
}
#[test]
fn vertical_ascending_crossing_threshold_reverses_and_carries_same_call() {
    let mut f = Fixture::new(VERTICAL);
    f.actors[0].position.screen_y = 0x67;
    f.world.integrated = Some(Position {
        screen_y: 0x67,
        ..f.actor().position
    });
    f.run();
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.actors[0].position.screen_y, 0x69);
    assert_eq!(f.world.calls.len(), 1);
}
#[test]
fn vertical_downward_carry_is_two_integer_pixels_and_preserves_player_fraction() {
    let mut f = Fixture::new(VERTICAL);
    f.ready();
    f.actors[0].position.y_fraction = 0xe1;
    f.run();
    assert_eq!(
        (
            f.actors[0].position.screen_y,
            f.actors[0].position.y_fraction
        ),
        (0x82, 0xe1)
    );
    assert_eq!(
        f.world.calls,
        vec![integrate(Integration::Vertical(
            VerticalOperation::AddEncodedVelocity
        ))]
    );
}
#[test]
fn vertical_horizontal_distance_equality_rejects_but_one_below_carries() {
    for (distance, want) in [(0x1c, 0x82), (0x1d, 0x80)] {
        let mut f = Fixture::new(VERTICAL);
        f.ready();
        f.actors[0].position.world_x = ACTOR_X + distance;
        f.run();
        assert_eq!(f.actors[0].position.screen_y, want);
    }
}
#[test]
fn vertical_vertical_distance_equality_rejects_but_one_below_carries() {
    for (distance, want) in [(0x17, 0x99), (0x18, 0x98)] {
        let mut f = Fixture::new(VERTICAL);
        f.ready();
        f.actors[0].position.screen_y = ACTOR_Y + distance;
        f.run();
        assert_eq!(f.actors[0].position.screen_y, want);
    }
}
#[test]
fn vertical_distance_is_low_byte_magnitude_not_true_far_page_distance() {
    let mut f = Fixture::new(VERTICAL);
    f.ready();
    f.actors[0].position.world_x = ACTOR_X + 0x100;
    f.run();
    assert_eq!(f.actors[0].position.screen_y, 0x82);
}
#[test]
fn vertical_carry_wraps_player_y_without_reload_or_action_gate() {
    let mut f = Fixture::new(VERTICAL);
    f.ready();
    f.actor_mut().position.screen_y = 0xf0;
    f.actors[0].position.screen_y = 0xff;
    f.player.action = Action::Death;
    f.run();
    assert_eq!(f.actors[0].position.screen_y, 1);
    assert_eq!(f.player.action, Action::Death);
    assert_eq!(f.actor().state_code(), ACTIVE);
}
#[test]
fn vertical_lower_equality_reverses_after_carry_and_preserves_high_state_bits() {
    let mut f = Fixture::new(VERTICAL);
    {
        let actor_value = 0xe1;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().position.screen_y = 0xa8;
    f.actors[0].position.screen_y = 0xa8;
    f.run();
    assert_eq!(f.actor().state_code(), 0xe0);
    assert_eq!(f.actors[0].position.screen_y, 0xaa);
}
#[test]
fn walking_wait_rejects_player_behind_and_low_byte_distance_below_sixteen() {
    for x in [ACTOR_X - 1, ACTOR_X + 15, ACTOR_X + 0x100] {
        let mut f = Fixture::new(WALKING);
        f.actors[0].position.world_x = x;
        let old = f.actors;
        f.run();
        assert_eq!(f.actors, old);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn walking_activation_clears_preserve_bit_and_moves_same_call() {
    let mut f = Fixture::new(WALKING);
    {
        let actor_value = ACTIVE | 0x10;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actors[0].position.world_x = ACTOR_X + 16;
    f.run();
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(
        (
            f.actor().horizontal_magnitude,
            f.actor().vertical_velocity,
            f.actor().direction
        ),
        (WALK_SPEED, 0, 1)
    );
    assert_eq!(
        f.world.calls,
        vec![
            contact(PLATFORM_PROFILE),
            motion(Movement::Fall, FALL_SAMPLE, 2)
        ]
    );
}
#[test]
fn walking_far_positive_page_with_low_sixteen_still_activates() {
    let mut f = Fixture::new(WALKING);
    f.actors[0].position.world_x = ACTOR_X + 0x110;
    f.run();
    assert_eq!(f.actor().state_code(), READY);
}
#[test]
fn walking_airborne_skips_horizontal_and_sound_even_zero_animation_tick() {
    let mut f = Fixture::new(WALKING);
    f.ready();
    f.actor_mut().playback.restart_clock();
    f.hit(2);
    let e = f.run();
    assert!(e.sounds.is_empty());
    assert_eq!(
        f.world.calls,
        vec![
            contact(PLATFORM_PROFILE),
            motion(Movement::Fall, FALL_SAMPLE, 2)
        ]
    );
}
#[test]
fn walking_ground_uses_direction_bit_zero_and_full_collision_bit_one() {
    for (direction, m, p) in [
        (1, Movement::PositiveX, WALK_RIGHT),
        (2, Movement::NegativeX, WALK_LEFT),
    ] {
        let mut f = Fixture::new(WALKING);
        f.ready();
        f.actor_mut().direction = direction;
        f.hit(SOLID);
        f.hit(2);
        f.run();
        assert_eq!(f.actor().direction, direction);
        assert_eq!(f.world.calls[2], motion(m, p, 2));
    }
}
#[test]
fn walking_obstruction_xors_both_direction_bits_and_zero_tick_requests_sound() {
    let mut f = Fixture::new(WALKING);
    f.ready();
    f.actor_mut().direction = 0x81;
    f.actor_mut().playback.restart_clock();
    f.hit(1);
    f.hit(3);
    let e = f.run();
    assert_eq!(f.actor().direction, 0x82);
    assert_eq!(e.sounds, vec![SOUND_WALK]);
}
#[test]
fn walking_contact_immediate_damage_is_not_rolled_back_by_fall() {
    let mut f = Fixture::new(WALKING);
    f.ready();
    f.actor_mut().profile = DAMAGE_PROFILE;
    f.world.contact_action = Some(Action::Damage);
    f.player.queued = Some(Action::TerrainPose);
    f.run();
    assert_eq!(f.player.action, Action::Damage);
    assert_eq!(f.player.queued, Some(Action::TerrainPose));
}
#[test]
fn flight_offscreen_and_hit_set_retry_before_delegating_shared_reaction() {
    for (render, hit) in [(0, 0), (VISIBLE, 1)] {
        let mut f = Fixture::new(FLIGHT);
        f.actor_mut().render = render;
        f.actor_mut().hit_marker = hit;
        let e = f.run();
        assert_eq!(
            f.special,
            SpecialControl::Cooldown(crate::actors::SPECIAL_SPAWN_RETRY)
        );
        assert_eq!(f.world.calls, vec![Work::Reaction]);
        assert_eq!(f.actor().handler, crate::actors::objects::LAUNCHED_HANDLER);
        assert_eq!(e.sounds, vec![0x35]);
    }
}
#[test]
fn flight_initializes_counter_faces_player_and_sets_positive_visual_only() {
    for (player_x, direction, flip) in [(ACTOR_X + 1, 1, 0x40), (ACTOR_X, 2, 0)] {
        let mut f = Fixture::new(FLIGHT);
        f.actors[0].position.world_x = player_x;
        // The negative-X wrapper survives with SEC/SBC no-borrow carry set.
        f.world.exit = direction == 2;
        {
            let actor_value = 99;
            f.actor_mut().set_local_state_code(actor_value);
        };
        {
            let actor_value = 88;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.run();
        assert_eq!(f.actor().state_code(), READY);
        assert_eq!(f.actor().direction, direction);
        assert_eq!(f.actor().render & 0x40, flip);
        assert_eq!(
            (f.actor().local_state_code(), f.actor().local_counter_code()),
            (FLIGHT_HOLD - 1, 1)
        );
        assert_eq!(f.world.calls[0], Work::Preset(2));
    }
}
#[test]
fn flight_negative_preset_normalizes_full_word_before_directional_integration() {
    for (encoded, want) in [(0xff00, 0x0100), (0xff80, 0x0080), (0x8000, 0x8000)] {
        let mut f = Fixture::new(FLIGHT);
        f.ready();
        f.actor_mut().direction = 2;
        f.world.preset_horizontal = encoded;
        f.run();
        assert_eq!(f.actor().horizontal_magnitude, want);
        assert_eq!(
            f.world.calls[1],
            integrate(Integration::Horizontal(HorizontalDirection::Negative))
        );
    }
}
#[test]
fn flight_positive_magnitude_is_retained_and_program_wraps_at_fourteen() {
    let mut f = Fixture::new(FLIGHT);
    f.ready();
    {
        let actor_value = 13;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.catalog.flight[13] = 7;
    f.world.preset_horizontal = 0x017f;
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0x017f);
    assert_eq!(f.actor().local_counter_code(), 0);
    assert_eq!(f.world.calls[0], Work::Preset(7));
}
#[test]
fn flight_hold_does_not_reload_preset_and_y_add_carries_fraction_before_x() {
    let mut f = Fixture::new(FLIGHT);
    f.ready();
    {
        let actor_value = 2;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().position.y_fraction = 0xe0;
    f.actor_mut().vertical_velocity = 0x0180;
    f.run();
    assert_eq!(
        (f.actor().position.screen_y, f.actor().position.y_fraction),
        (0x82, 0x60)
    );
    assert_eq!(f.world.before_integrate[0], f.actor().position);
    assert_eq!(f.actor().local_state_code(), 1);
    assert!(!f.world.calls.iter().any(|v| matches!(v, Work::Preset(_))));
}
#[test]
fn flight_negative_y_increment_wraps_screen_byte_without_y_removal() {
    let mut f = Fixture::new(FLIGHT);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().position.screen_y = 0;
    f.actor_mut().vertical_velocity = -0x80;
    f.run();
    assert_eq!(
        (f.actor().position.screen_y, f.actor().position.y_fraction),
        (0xff, 0x80)
    );
    assert_eq!(f.actor().state_code(), READY);
}
#[test]
fn flight_bound_removal_releases_identity_and_matching_special_before_sound_contact() {
    let mut f = Fixture::new(FLIGHT);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().playback.restart_clock();
    f.world.exit = true;
    let e = f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().render,
            f.actor().spawn_identity
        ),
        (0, 0, None)
    );
    assert_eq!(
        f.special,
        SpecialControl::Cooldown(crate::actors::SPECIAL_SPAWN_RETRY)
    );
    assert!(e.sounds.is_empty());
    assert_eq!(f.world.calls.len(), 1);
}
#[test]
fn flight_removal_preserves_unmatched_special_latch() {
    let mut f = Fixture::new(FLIGHT);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.world.exit = true;
    f.special = SpecialControl::Occupied(TARGET);
    f.run();
    assert_eq!(f.special, SpecialControl::Occupied(TARGET));
}
#[test]
fn flight_zero_tick_sounds_then_contact_and_nonzero_tick_only_contacts() {
    for tick in [0, 1] {
        let mut f = Fixture::new(FLIGHT);
        f.ready();
        {
            let actor_value = 1;
            f.actor_mut().set_local_state_code(actor_value);
        };
        {
            let animation_value = tick;
            f.actor_mut().playback.set_tick_code(animation_value);
        };
        let e = f.run();
        assert_eq!(
            e.sounds,
            if tick == 0 {
                vec![SOUND_FLIGHT]
            } else {
                vec![]
            }
        );
        assert_eq!(f.world.calls.last(), Some(&contact(PLATFORM_PROFILE)));
    }
}
#[test]
fn ride_initialization_clears_own_fractions_before_program_motion() {
    let mut f = Fixture::new(RIDEABLE);
    f.actor_mut().position.x_fraction = 0xff;
    f.actor_mut().position.y_fraction = 0xff;
    {
        let actor_value = 0xaa;
        f.actor_mut().set_local_state_code(actor_value);
    };
    {
        let actor_value = 0xbb;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().position.x_fraction,
            f.actor().position.y_fraction
        ),
        (READY, 0, 0)
    );
    assert_eq!(
        (f.actor().local_state_code(), f.actor().local_counter_code()),
        (14, 1)
    );
    assert_eq!(f.world.calls[0], Work::Preset(3));
}
#[test]
fn ride_packed_low_five_bits_and_original_bit_seven_select_independent_preset_duration() {
    for (packed, preset, duration) in [(0x1f, 31, 14), (0x7f, 31, 14), (0x9f, 31, 4), (0xff, 31, 4)]
    {
        let mut f = Fixture::new(RIDEABLE);
        f.ready();
        f.catalog.ride[0] = packed;
        f.world.preset_horizontal = 0x8000;
        f.run();
        assert_eq!(f.world.calls[0], Work::Preset(preset));
        assert_eq!(f.actor().local_state_code(), duration);
    }
}
#[test]
fn ride_all_thirty_six_steps_use_low_five_and_bit_seven_with_cursor_wrap() {
    for index in 0..RIDE_ENTRIES {
        let mut f = Fixture::new(RIDEABLE);
        f.ready();
        let packed = (index as u8 & 0x1f) | if index % 2 == 0 { 0x80 } else { 0 };
        f.catalog.ride[index] = packed;
        {
            let actor_value = index as u8;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.run();
        assert_eq!(f.world.calls[0], Work::Preset(packed & 0x1f));
        assert_eq!(
            f.actor().local_counter_code(),
            if index + 1 == RIDE_ENTRIES {
                0
            } else {
                index as u8 + 1
            }
        );
        assert_eq!(
            f.actor().local_state_code(),
            if packed & 0x80 != 0 { 4 } else { 14 }
        );
    }
}
#[test]
fn ride_signed_negative_speed_borrows_through_fraction_and_world_page() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().horizontal_magnitude = 0xff80;
    f.actor_mut().position.world_x = 0x0200;
    f.run();
    assert_eq!(
        (f.actor().position.world_x, f.actor().position.x_fraction),
        (0x01ff, 0x80)
    );
    assert!(
        !f.world
            .calls
            .iter()
            .any(|w| matches!(w, Work::Integrate(_) | Work::Preset(_)))
    );
}
#[test]
fn ride_positive_speed_carries_fraction_to_integer_and_world_page() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().horizontal_magnitude = 0x0180;
    f.actor_mut().position.world_x = 0x01ff;
    f.actor_mut().position.x_fraction = 0x90;
    f.run();
    assert_eq!(
        (f.actor().position.world_x, f.actor().position.x_fraction),
        (0x0201, 0x10)
    );
}
#[test]
fn ride_coordinate_wrap_does_not_deactivate_or_release_identity() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().horizontal_magnitude = 0xff80;
    f.actor_mut().vertical_velocity = -0x80;
    f.actor_mut().position = Position::default();
    f.run();
    assert_eq!(
        f.actor().position,
        Position {
            world_x: 0xffff,
            x_fraction: 0x80,
            screen_y: 0xff,
            y_fraction: 0x80
        }
    );
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.actor().spawn_identity, Some(0x42));
}
#[test]
fn ride_damage_and_death_skip_contact_and_copy_but_sound_and_actor_motion_still_run() {
    for action in [Action::Damage, Action::Death] {
        let mut f = Fixture::new(RIDEABLE);
        f.ready();
        {
            let actor_value = 1;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.actor_mut().vertical_velocity = 0x100;
        f.actor_mut().playback.restart_clock();
        f.player.action = action;
        let old = f.actors[0];
        let e = f.run();
        assert_eq!(f.actor().position.screen_y, 0x81);
        assert_eq!(f.actors[0], old);
        assert_eq!(e.sounds, vec![SOUND_RIDE]);
        assert!(f.world.calls.is_empty());
    }
}
#[test]
fn ride_contact_permission_and_up_held_are_both_required_and_contact_effects_survive() {
    for (permission, held, action) in [(false, UP, Action::Standing), (true, 0, Action::Damage)] {
        let mut f = Fixture::new(RIDEABLE);
        f.ready();
        {
            let actor_value = 1;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.world.permission = permission;
        f.held = held;
        if permission {
            f.actor_mut().profile = DAMAGE_PROFILE;
            f.world.contact_action = Some(Action::Damage);
        }
        f.run();
        assert_eq!(f.player.action, action);
        assert_eq!(
            f.world.calls,
            vec![contact(if permission {
                DAMAGE_PROFILE
            } else {
                PLATFORM_PROFILE
            })]
        );
    }
}
#[test]
fn ride_acquisition_overrides_immediate_damage_and_copies_only_integer_position() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.held = UP;
    f.world.permission = true;
    f.actor_mut().profile = DAMAGE_PROFILE;
    f.world.contact_action = Some(Action::Damage);
    f.actors[0].position = Position {
        world_x: 0x1111,
        x_fraction: 0x55,
        screen_y: 0x20,
        y_fraction: 0xaa,
    };
    f.actors[0].vertical_velocity = 0x123;
    f.player.queued = Some(Action::TerrainPose);
    f.run();
    assert_eq!(f.player.action, Action::WaitA);
    assert_eq!(
        f.actors[0].position,
        Position {
            world_x: ACTOR_X,
            x_fraction: 0x55,
            screen_y: ACTOR_Y,
            y_fraction: 0xaa
        }
    );
    assert_eq!(
        (
            f.actors[0].animation,
            f.actors[0].playback.frame_code(),
            f.actors[0].playback.tick_code()
        ),
        (CLIMB_ANIMATION, 0, 0)
    );
    assert_eq!(f.actors[0].vertical_velocity, 0x123);
    assert_eq!(f.player.queued, Some(Action::TerrainPose));
}
#[test]
fn ride_existing_wait_a_bypasses_contact_and_up_but_does_not_reset_animation() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.player.action = Action::WaitA;
    f.actors[0].animation = 9;
    {
        let animation_value = 6;
        f.actors[0].playback.set_tick_code(animation_value);
    };
    f.actors[0].position.world_x = 0x1111;
    f.run();
    assert_eq!(f.actors[0].position.world_x, ACTOR_X);
    assert_eq!(
        (f.actors[0].animation, f.actors[0].playback.tick_code()),
        (9, 6)
    );
    assert!(f.world.calls.is_empty());
}
#[test]
fn ride_wait_a_escape_to_standing_is_not_stale_recopied_without_up() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.player.action = Action::Standing;
    f.actors[0].position.world_x = 0x1111;
    f.world.permission = true;
    f.run();
    assert_eq!(f.actors[0].position.world_x, 0x1111);
    assert_eq!(f.player.action, Action::Standing);
}
#[test]
fn ride_later_slot_can_overwrite_existing_wait_a_position_without_attachment_owner_latch() {
    let mut f = Fixture::new(RIDEABLE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.player.action = Action::WaitA;
    f.actors[TARGET] = *f.actor();
    f.actors[TARGET].position.world_x = 0x0456;
    f.run();
    assert_eq!(f.actors[0].position.world_x, ACTOR_X);
    f.run_slot(TARGET);
    assert_eq!(f.actors[0].position.world_x, 0x0456);
}

#[test]
fn push_offscreen_sets_retry_and_hit_sets_walk_pose_before_shared_reaction() {
    for (visible, hit, want_animation) in [(0, 0, 0), (VISIBLE, 1, WALK_ANIMATION)] {
        let mut f = Fixture::new(PUSHING);
        f.actor_mut().render = visible;
        f.actor_mut().hit_marker = hit;
        f.run();
        assert_eq!(
            f.special,
            SpecialControl::Cooldown(crate::actors::SPECIAL_SPAWN_RETRY)
        );
        assert_eq!(f.world.calls, vec![Work::Reaction]);
        assert_eq!(f.actor().animation, want_animation);
    }
}
#[test]
fn push_no_ground_support_skips_all_phase_and_animation_events() {
    let mut f = Fixture::new(PUSHING);
    {
        let actor_value = ACTIVE | 2;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let animation_value = PUSH_EVENT;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    {
        let actor_value = TARGET as u8;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.world.calls, vec![motion(Movement::Fall, PUSH_FALL, 2)]);
    assert_eq!(f.actor().local_state_code(), TARGET as u8);
}
#[test]
fn push_wait_distance_boundary_freezes_tick_but_preserves_animation_frame() {
    let mut f = Fixture::new(PUSHING);
    f.actors[0].position.world_x = ACTOR_X + 0x50;
    {
        let animation_value = 4;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.hit(1);
    f.run();
    assert_eq!(f.actor().state_code(), ACTIVE);
    assert_eq!(
        (
            f.actor().playback.frame_code(),
            f.actor().playback.tick_code()
        ),
        (4, 0)
    );
    assert_eq!(f.world.calls.len(), 1);
}
#[test]
fn push_initialization_within_range_faces_player_and_returns_without_horizontal() {
    let mut f = Fixture::new(PUSHING);
    f.actors[0].position.world_x = ACTOR_X + 0x4f;
    f.actor_mut().vertical_velocity = 0x222;
    {
        let actor_value = 9;
        f.actor_mut().set_local_state_code(actor_value);
    };
    {
        let actor_value = 8;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.hit(1);
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().horizontal_magnitude,
            f.actor().vertical_velocity,
            f.actor().direction
        ),
        (READY, 0x100, 0, 1)
    );
    assert_eq!(
        (f.actor().local_state_code(), f.actor().local_counter_code()),
        (0, 0)
    );
    assert_eq!(f.world.calls.len(), 1);
}
#[test]
fn push_normal_phase_moves_by_direction_before_testing_obstruction() {
    for (direction, m, p) in [
        (1, Movement::PositiveX, PUSH_RIGHT),
        (2, Movement::NegativeX, PUSH_LEFT),
    ] {
        let mut f = Fixture::new(PUSHING);
        f.ready();
        f.actor_mut().direction = direction;
        f.hit(1);
        f.hit(2);
        f.run();
        assert_eq!(
            f.world.calls,
            vec![motion(Movement::Fall, PUSH_FALL, 2), motion(m, p, 2)]
        );
        assert_eq!(f.actor().local_counter_code(), 0);
    }
}
#[test]
fn push_solid_without_overlap_increments_only_integer_speed_and_counter() {
    let mut f = Fixture::new(PUSHING);
    f.ready();
    f.actor_mut().horizontal_magnitude = 0xff80;
    f.hit(1);
    f.hit(1);
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0x0080);
    assert_eq!(f.actor().local_counter_code(), 1);
    assert_eq!(f.world.calls.last(), Some(&Work::ActorContact));
}
#[test]
fn push_overlap_nonterrain_selects_walk_pose_and_enters_brake_without_saving_target() {
    let mut f = Fixture::new(PUSHING);
    f.ready();
    f.world.target = Some(TARGET);
    {
        let actor_value = crate::actors::objects::MOVING_HANDLER;
        f.actors[TARGET].set_handler(actor_value);
    };
    f.actor_mut().horizontal_magnitude = 0x100;
    f.hit(1);
    f.hit(1);
    f.run();
    assert_eq!(
        (
            f.actor().animation,
            f.actor().local_counter_code(),
            f.actor().local_state_code(),
            f.actor().horizontal_magnitude
        ),
        (WALK_ANIMATION, 1, 0, 0x200)
    );
}
#[test]
fn push_terrain_target_capture_uses_signed_word_offset_and_resets_animation() {
    for (direction, x, want) in [(1, 0x0200, 0x01fe), (2, 0x01ff, 0x0201)] {
        let mut f = Fixture::new(PUSHING);
        f.ready();
        f.actor_mut().direction = direction;
        f.actor_mut().position.world_x = x;
        f.actor_mut().position.x_fraction = 0x77;
        f.world.target = Some(TARGET);
        {
            let actor_value = crate::actors::objects::TERRAIN_HANDLER;
            f.actors[TARGET].set_handler(actor_value);
        };
        f.hit(1);
        f.hit(1);
        f.run();
        assert_eq!(
            (
                f.actor().state_code(),
                f.actor().animation,
                f.actor().local_state_code(),
                f.actor().local_counter_code()
            ),
            (ACTIVE | 2, PUSH_ANIMATION, TARGET as u8, 1)
        );
        assert_eq!(f.actor().position.world_x, want);
        assert_eq!(f.actor().position.x_fraction, 0x77);
        assert_eq!(
            (
                f.actor().playback.frame_code(),
                f.actor().playback.tick_code()
            ),
            (0, 0)
        );
    }
}
#[test]
fn push_brake_subtracts_fraction_and_toggles_state_bit_each_grounded_update() {
    let mut f = Fixture::new(PUSHING);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().horizontal_magnitude = 0x0100;
    f.hit(1);
    f.run();
    assert_eq!(f.actor().horizontal_magnitude, 0x00f8);
    assert_eq!(f.actor().state_code(), READY ^ 0x40);
}
#[test]
fn push_brake_zero_is_not_negative_but_next_step_restarts_at_two_ff() {
    for (speed, want_speed, want_state) in [
        (8, 0, 0),
        (0, BRAKE_RESTART, 0xff),
        (0x8008, BRAKE_RESTART, 0xff),
    ] {
        let mut f = Fixture::new(PUSHING);
        f.ready();
        f.actor_mut().direction = 1;
        {
            let actor_value = 1;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.actor_mut().horizontal_magnitude = speed;
        f.hit(1);
        f.run();
        assert_eq!(f.actor().horizontal_magnitude, want_speed);
        assert_eq!(f.actor().local_state_code(), want_state);
        assert_eq!(f.actor().direction, if want_state == 0 { 1 } else { 2 });
    }
}
#[test]
fn push_nonzero_target_or_ff_sentinel_skips_brake_but_still_moves() {
    for saved in [TARGET as u8, 0xff] {
        let mut f = Fixture::new(PUSHING);
        f.ready();
        {
            let actor_value = 1;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        {
            let actor_value = saved;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.actor_mut().horizontal_magnitude = 0x100;
        f.hit(1);
        f.run();
        assert_eq!(f.actor().horizontal_magnitude, 0x100);
        assert_eq!(f.actor().state_code(), READY);
        assert_eq!(f.world.calls.len(), 2);
    }
}
#[test]
fn push_frame_three_activates_saved_slot_immediately_with_pose_even_nonzero_tick() {
    let mut f = Fixture::new(PUSHING);
    {
        let actor_value = ACTIVE | 2;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let animation_value = PUSH_EVENT;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 7;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    {
        let actor_value = TARGET as u8;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.actor_mut().direction = 2;
    {
        let actor_value = crate::actors::objects::TERRAIN_HANDLER;
        f.actors[TARGET].set_handler(actor_value);
    };
    f.hit(1);
    let e = f.run();
    assert_eq!(f.player.pose_direction, 2);
    assert_eq!(f.world.terrain_seen, vec![(TARGET, 2, 0)]);
    assert_eq!(
        f.actors[TARGET].handler,
        crate::actors::objects::MOVING_HANDLER
    );
    assert_eq!(f.actor().local_state_code(), 0);
    assert_eq!(f.changed, 1);
    assert_eq!(e.sounds, vec![0x16]);
}
#[test]
fn push_frame_three_clears_saved_slot_after_changed_gate_and_does_not_repeat() {
    let mut f = Fixture::new(PUSHING);
    {
        let actor_value = ACTIVE | 2;
        f.actor_mut().set_state_code(actor_value);
    };
    {
        let animation_value = PUSH_EVENT;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    {
        let actor_value = TARGET as u8;
        f.actor_mut().set_local_state_code(actor_value);
    };
    f.changed = 1;
    f.hit(1);
    f.run();
    assert_eq!(f.world.terrain_seen, vec![(TARGET, 0, 1)]);
    assert_eq!(f.actor().local_state_code(), 0);
    f.hit(1);
    f.run();
    assert_eq!(f.world.terrain_seen.len(), 1);
}
#[test]
fn push_other_frames_and_zero_saved_target_skip_event() {
    for (frame, target) in [(2, TARGET as u8), (4, TARGET as u8), (PUSH_EVENT, 0)] {
        let mut f = Fixture::new(PUSHING);
        {
            let actor_value = ACTIVE | 2;
            f.actor_mut().set_state_code(actor_value);
        };
        {
            let animation_value = frame;
            f.actor_mut().playback.set_frame_code(animation_value);
        };
        {
            let actor_value = target;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.hit(1);
        f.run();
        assert!(
            !f.world
                .calls
                .iter()
                .any(|w| matches!(w, Work::ActivateTerrain(_)))
        );
        assert_eq!(f.actor().local_state_code(), target);
    }
}
#[test]
fn push_frame_five_precedes_target_gate_restores_pose_and_only_speed_integer() {
    let mut f = Fixture::new(PUSHING);
    {
        let actor_value = ACTIVE | 2;
        f.actor_mut().set_state_code(actor_value);
    };
    f.actor_mut().animation = PUSH_ANIMATION;
    {
        let animation_value = PUSH_FINISH;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 7;
        f.actor_mut().playback.set_tick_code(animation_value);
    };
    {
        let actor_value = TARGET as u8;
        f.actor_mut().set_local_state_code(actor_value);
    };
    {
        let actor_value = 0xff;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().horizontal_magnitude = 0x0134;
    f.hit(1);
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().animation,
            f.actor().horizontal_magnitude,
            f.actor().local_counter_code()
        ),
        (READY, WALK_ANIMATION, 0x0234, 0)
    );
    assert_eq!(
        (
            f.actor().playback.frame_code(),
            f.actor().playback.tick_code(),
            f.actor().local_state_code()
        ),
        (0, 0, TARGET as u8)
    );
    assert_eq!(f.world.calls.len(), 1);
}
#[test]
fn carrying_initial_overlap_denied_leaves_everything_unchanged() {
    let mut f = Fixture::new(CARRYING);
    let old = f.actors;
    f.run();
    assert_eq!(f.actors, old);
    assert_eq!(f.world.calls, vec![contact(PLATFORM_PROFILE)]);
}
#[test]
fn carrying_initial_permission_has_no_action_gate_and_overrides_contact_damage() {
    let mut f = Fixture::new(CARRYING);
    f.player.action = Action::Death;
    f.world.permission = true;
    f.actor_mut().profile = DAMAGE_PROFILE;
    f.world.contact_action = Some(Action::Damage);
    {
        let animation_value = 9;
        f.actors[0].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 8;
        f.actor_mut().playback.set_frame_code(animation_value);
    };
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().animation,
            f.actor().horizontal_magnitude,
            f.actor().vertical_velocity
        ),
        (READY, CARRY_ANIMATION, WALK_SPEED, 0)
    );
    assert_eq!(f.player.action, Action::CarriedJumpable);
    assert_eq!(f.actors[0].animation, CARRIED_ANIMATION);
    assert_eq!(
        (
            f.actors[0].playback.frame_code(),
            f.actors[0].playback.tick_code(),
            f.actor().playback.frame_code(),
            f.actor().playback.tick_code()
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        f.world.calls,
        vec![
            contact(DAMAGE_PROFILE),
            motion(Movement::Fall, FALL_SAMPLE, 2)
        ]
    );
}
#[test]
fn carrying_airborne_selects_jumpable_and_copies_actor_x_y_minus_eleven_only() {
    let mut f = Fixture::new(CARRYING);
    f.ready();
    f.player.action = Action::CarriedLocked;
    f.actors[0].position.x_fraction = 0x77;
    f.actors[0].position.y_fraction = 0x55;
    f.actors[0].horizontal_magnitude = 0x221;
    f.actors[0].vertical_velocity = 0x123;
    f.run();
    assert_eq!(f.player.action, Action::CarriedJumpable);
    assert_eq!(
        f.actors[0].position,
        Position {
            world_x: ACTOR_X,
            x_fraction: 0x77,
            screen_y: ACTOR_Y - 11,
            y_fraction: 0x55
        }
    );
    assert_eq!(
        (
            f.actors[0].horizontal_magnitude,
            f.actors[0].vertical_velocity
        ),
        (0x221, 0x123)
    );
}
#[test]
fn carrying_ground_calls_negative_probe_then_locks_even_when_horizontal_bits_differ() {
    let mut f = Fixture::new(CARRYING);
    f.ready();
    f.player.action = Action::CarriedJumpable;
    f.hit(1);
    f.hit(0);
    f.run();
    assert_eq!(f.player.action, Action::CarriedLocked);
    assert_eq!(f.actor().local_state_code(), Action::CarriedLocked as u8);
    assert_eq!(
        f.world.calls,
        vec![
            motion(Movement::Fall, FALL_SAMPLE, 2),
            motion(Movement::NegativeX, CARRY_LEFT, 0)
        ]
    );
}
#[test]
fn carrying_player_jump_escape_before_actor_loop_prevents_position_recopy_and_sound() {
    let mut f = Fixture::new(CARRYING);
    f.ready();
    f.player.action = Action::Airborne;
    f.actors[0].position.world_x = 0x1234;
    f.actor_mut().playback.restart_clock();
    f.hit(1);
    let e = f.run();
    assert_eq!(f.player.action, Action::Airborne);
    assert_eq!(f.actors[0].position.world_x, 0x1234);
    assert!(e.sounds.is_empty());
    assert_eq!(f.world.calls.len(), 2);
}
#[test]
fn carrying_y_offset_wraps_and_zero_tick_sound_requires_current_carried_action() {
    let mut f = Fixture::new(CARRYING);
    f.ready();
    f.actor_mut().position.screen_y = 5;
    f.actor_mut().playback.restart_clock();
    f.player.action = Action::CarriedJumpable;
    let e = f.run();
    assert_eq!(f.actors[0].position.screen_y, 0xfa);
    assert_eq!(e.sounds, vec![SOUND_CARRY]);
}
#[test]
fn carrying_movement_commits_before_position_copy_and_queue_is_retained() {
    let mut f = Fixture::new(CARRYING);
    f.ready();
    f.player.action = Action::CarriedLocked;
    f.player.queued = Some(Action::Damage);
    f.world.moved = Some(Position {
        world_x: 0x0400,
        screen_y: 0x71,
        x_fraction: 9,
        y_fraction: 8,
    });
    f.run();
    assert_eq!(
        (f.actors[0].position.world_x, f.actors[0].position.screen_y),
        (0x0400, 0x66)
    );
    assert_eq!(f.player.queued, Some(Action::Damage));
}
#[test]
fn external_trigger_strict_four_pixel_start_and_low_byte_distance_alias() {
    for (offset, active) in [(3, true), (4, false), (0x100, true)] {
        let mut f = Fixture::new(EXTERNAL);
        f.actors[0].position.world_x = ACTOR_X + offset;
        f.run();
        assert_eq!(f.actor().state_code(), if active { READY } else { ACTIVE });
        assert_eq!(f.world.calls.len(), if active { 1 } else { 0 });
        if active {
            assert_eq!(f.actor().horizontal_magnitude, WALK_SPEED);
        }
    }
}
#[test]
fn external_active_actor_moves_even_if_player_is_far_and_proximity_uses_post_motion() {
    let mut f = Fixture::new(EXTERNAL);
    f.ready();
    f.actors[0].position.world_x = ACTOR_X + 0x40;
    f.world.moved = Some(Position {
        world_x: ACTOR_X + 0x40,
        ..f.actor().position
    });
    f.run();
    assert_eq!(f.player.external_motion, 1);
    assert_eq!(
        f.world.calls,
        vec![motion(Movement::PositiveX, EXTERNAL_RIGHT, 0)]
    );
}
#[test]
fn external_proximity_requires_both_axes_strictly_below_twenty_one() {
    for (dx, dy, want) in [(20, 20, 1), (21, 0, 0), (0, 21, 0)] {
        let mut f = Fixture::new(EXTERNAL);
        f.ready();
        f.actors[0].position.world_x = ACTOR_X + dx;
        f.actors[0].position.screen_y = ACTOR_Y + dy;
        f.run();
        assert_eq!(f.player.external_motion, want);
    }
}
#[test]
fn external_latch_increment_wraps_and_ignores_damage_death_and_protection() {
    let mut f = Fixture::new(EXTERNAL);
    f.ready();
    f.player.action = Action::Death;
    f.player.protection = 0xff;
    f.player.external_motion = 0xff;
    f.run();
    assert_eq!(f.player.external_motion, 0);
    assert_eq!(f.player.action, Action::Death);
}
#[test]
fn external_solid_repeats_wrapper_with_start_one_and_actual_residual_count_then_disables_handler() {
    let mut f = Fixture::new(EXTERNAL);
    f.ready();
    f.world.facts.push_back(MotionFacts {
        bits: 3,
        continuation_remaining: 5,
    });
    f.hit(0);
    f.run();
    assert_eq!(
        f.world.calls,
        vec![
            motion(Movement::PositiveX, EXTERNAL_RIGHT, 0),
            motion(Movement::PositiveX, 1, 5)
        ]
    );
    assert_eq!(f.actor().handler, 0);
    assert_eq!(f.player.external_motion, 1);
}
#[test]
fn external_ordinary_solid_return_leaves_zero_residual_count_and_secondary_bit_alone_does_not_repeat()
 {
    for (bits, calls, handler) in [(1, 2, 0), (2, 1, EXTERNAL)] {
        let mut f = Fixture::new(EXTERNAL);
        f.ready();
        f.hit(bits);
        f.run();
        assert_eq!(f.world.calls.len(), calls);
        assert_eq!(f.actor().handler, handler);
        if bits == 1 {
            assert_eq!(f.world.calls[1], motion(Movement::PositiveX, 1, 0));
        }
    }
}
#[test]
fn lift_initial_denied_overlap_does_nothing_and_death_is_checked_after_contact() {
    for permission in [false, true] {
        let mut f = Fixture::new(LIFT);
        f.player.action = Action::Death;
        f.world.permission = permission;
        let old = f.actors;
        f.run();
        assert_eq!(f.actors, old);
        assert_eq!(f.world.calls, vec![contact(PLATFORM_PROFILE)]);
        assert_eq!(f.completion, 0);
    }
}
#[test]
fn lift_contact_can_replace_death_with_damage_before_death_gate_and_then_acquire_hold() {
    let mut f = Fixture::new(LIFT);
    f.player.action = Action::Death;
    f.actor_mut().profile = DAMAGE_PROFILE;
    f.world.contact_action = Some(Action::Damage);
    f.world.permission = true;
    f.actors[0].position.world_x = 0x1111;
    f.actors[0].position.x_fraction = 0x78;
    {
        let animation_value = 7;
        f.actors[0].playback.set_frame_code(animation_value);
    };
    f.run();
    assert_eq!(f.player.action, Action::Hold);
    assert_eq!(f.actor().state_code(), READY);
    assert_eq!(f.actors[0].position.world_x, ACTOR_X);
    assert_eq!(f.actors[0].position.x_fraction, 0x78);
    assert_eq!(
        (
            f.actors[0].animation,
            f.actors[0].playback.frame_code(),
            f.actors[0].playback.tick_code()
        ),
        (CLIMB_ANIMATION, 0, 0)
    );
    assert_eq!(
        (f.actor().position.screen_y, f.actors[0].position.screen_y),
        (ACTOR_Y - 2, ACTOR_Y - 2)
    );
}
#[test]
fn lift_existing_phase_ignores_player_action_and_does_not_recopy_x_or_animation() {
    let mut f = Fixture::new(LIFT);
    f.ready();
    f.player.action = Action::Airborne;
    f.actors[0].position.world_x = 0x1111;
    f.actors[0].animation = 9;
    {
        let animation_value = 7;
        f.actors[0].playback.set_frame_code(animation_value);
    };
    f.run();
    assert_eq!(f.player.action, Action::Airborne);
    assert_eq!(
        (
            f.actors[0].position.world_x,
            f.actors[0].animation,
            f.actors[0].playback.frame_code()
        ),
        (0x1111, 9, 7)
    );
    assert_eq!(f.actors[0].position.screen_y, ACTOR_Y - 2);
    assert!(f.world.calls.is_empty());
}
#[test]
fn lift_completion_strict_postmove_below_sixteen_and_byte_wrap() {
    for (y, post, latch) in [(0x12, 0x10, 0), (0x11, 0x0f, 1), (1, 0xff, 0), (0, 0xfe, 0)] {
        let mut f = Fixture::new(LIFT);
        f.ready();
        f.actor_mut().position.screen_y = y;
        f.run();
        assert_eq!(
            (
                f.actor().position.screen_y,
                f.actors[0].position.screen_y,
                f.completion
            ),
            (post, post, latch)
        );
    }
}
#[test]
fn lift_below_threshold_increments_completion_each_update_including_wrap() {
    let mut f = Fixture::new(LIFT);
    f.ready();
    f.actor_mut().position.screen_y = 0x0e;
    f.completion = 0xff;
    f.run();
    assert_eq!(f.completion, 0);
    f.run();
    assert_eq!(f.completion, 1);
}
#[test]
fn four_phase_initializes_speed_cursor_and_first_phase_then_decrements_hold() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.actor_mut().horizontal_magnitude = 0xffff;
    {
        let actor_value = 77;
        f.actor_mut().set_local_state_code(actor_value);
    };
    {
        let actor_value = 88;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.actor().state_code(),
            f.actor().horizontal_magnitude,
            f.actor().local_state_code(),
            f.actor().local_counter_code(),
            f.actor().direction
        ),
        (READY, 0x100, 1, PHASE_TICKS - 1, 2)
    );
    assert_eq!(
        f.world.calls,
        vec![
            integrate(Integration::Horizontal(HorizontalDirection::Negative)),
            contact(CARRY_CONTACT_PROFILE)
        ]
    );
    assert_eq!(f.actor().profile, PLATFORM_PROFILE);
}
#[test]
fn four_phase_all_directions_select_cursor_modulo_four_and_full_cursor_wrap() {
    for cursor in [0, 1, 2, 3, 4, 0xff] {
        let mut f = Fixture::new(FOUR_PHASE);
        f.ready();
        {
            let actor_value = cursor;
            f.actor_mut().set_local_state_code(actor_value);
        };
        f.run();
        assert_eq!(
            f.actor().direction,
            f.catalog.phases[usize::from(cursor & 3)]
        );
        assert_eq!(f.actor().local_state_code(), cursor.wrapping_add(1));
        assert_eq!(f.actor().local_counter_code(), PHASE_TICKS - 1);
    }
}
#[test]
fn four_phase_nonzero_hold_reaches_zero_without_reloading_until_next_call() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.ready();
    {
        let actor_value = 2;
        f.actor_mut().set_local_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().direction = 2;
    f.run();
    assert_eq!(
        (
            f.actor().local_state_code(),
            f.actor().local_counter_code(),
            f.actor().direction
        ),
        (2, 0, 2)
    );
    f.run();
    assert_eq!(
        (
            f.actor().local_state_code(),
            f.actor().local_counter_code(),
            f.actor().direction
        ),
        (3, PHASE_TICKS - 1, 1)
    );
}
#[test]
fn four_phase_holds_each_choice_for_exact_forty_eight_calls() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.ready();
    for phase in 0..4 {
        for frame in 0..PHASE_TICKS {
            f.run();
            assert_eq!(f.actor().direction, f.catalog.phases[phase]);
            assert_eq!(f.actor().local_counter_code(), PHASE_TICKS - 1 - frame);
        }
        assert_eq!(f.actor().local_state_code(), phase as u8 + 1);
    }
}
#[test]
fn four_phase_carry_adds_signed_integer_step_across_page_without_fraction_change() {
    for (direction, old, want) in [
        (1, 0x01ff, 0x0200),
        (2, 0x0200, 0x01ff),
        (2, 0, 0xffff),
        (1, 0xffff, 0),
    ] {
        let mut f = Fixture::new(FOUR_PHASE);
        f.ready();
        f.actor_mut().direction = direction;
        {
            let actor_value = 1;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.actors[0].position.world_x = old;
        f.actors[0].position.x_fraction = 0xe1;
        f.world.permission = true;
        f.run();
        assert_eq!(f.actors[0].position.world_x, want);
        assert_eq!(f.actors[0].position.x_fraction, 0xe1);
    }
}
#[test]
fn four_phase_contact_denial_still_restores_profile_and_moves_phase() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.ready();
    {
        let actor_value = 2;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().profile = 0xff;
    let old = f.actors[0];
    f.run();
    assert_eq!(f.actors[0], old);
    assert_eq!(f.actor().local_counter_code(), 1);
    assert_eq!(f.actor().profile, PLATFORM_PROFILE);
}
#[test]
fn four_phase_contact_observes_post_motion_actor_with_temporary_profile() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().direction = 1;
    f.world.integrated = Some(Position {
        world_x: 0x0400,
        ..f.actor().position
    });
    f.world.permission = true;
    f.actors[0].position.world_x = 0x02ff;
    f.actors[0].position.x_fraction = 0x77;
    f.run();
    assert_eq!(f.actor().position.world_x, 0x0400);
    assert_eq!(f.world.contact_seen[0].world_x, 0x0400);
    assert_eq!(f.actors[0].position.world_x, 0x0300);
    assert_eq!(f.actors[0].position.x_fraction, 0x77);
    assert_eq!(f.player.action, Action::Standing);
    assert_eq!(f.actor().profile, PLATFORM_PROFILE);
}

#[test]
fn four_phase_integrator_bounds_are_ignored_without_releasing_identity() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.ready();
    {
        let actor_value = 1;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.world.exit = true;
    f.run();
    assert_eq!(
        (f.actor().state_code(), f.actor().spawn_identity),
        (READY, Some(0x42))
    );
    assert_eq!(f.special, SpecialControl::Occupied(SUBJECT));
    assert_eq!(f.world.calls.len(), 2);
}
#[test]
fn four_phase_later_live_slot_moves_same_player_again_with_no_deduplication() {
    let mut f = Fixture::new(FOUR_PHASE);
    f.ready();
    {
        let actor_value = 2;
        f.actor_mut().set_local_counter_code(actor_value);
    };
    f.actor_mut().direction = 1;
    f.actors[TARGET] = *f.actor();
    f.world.permission = true;
    f.run();
    f.run_slot(TARGET);
    assert_eq!(f.actors[0].position.world_x, ACTOR_X + 2);
}
#[test]
fn flight_each_program_entry_is_loaded_and_held_for_five_calls() {
    for index in 0..FLIGHT_ENTRIES {
        let mut f = Fixture::new(FLIGHT);
        f.ready();
        {
            let actor_value = index as u8;
            f.actor_mut().set_local_counter_code(actor_value);
        };
        f.catalog.flight[index] = index as u8;
        for frame in 0..FLIGHT_HOLD {
            f.run();
            assert_eq!(f.actor().local_state_code(), FLIGHT_HOLD - 1 - frame);
        }
        assert_eq!(
            f.world
                .calls
                .iter()
                .filter(|w| matches!(w, Work::Preset(_)))
                .count(),
            1
        );
        assert_eq!(f.world.calls[0], Work::Preset(index as u8));
        assert_eq!(
            f.actor().local_counter_code(),
            if index + 1 == FLIGHT_ENTRIES {
                0
            } else {
                index as u8 + 1
            }
        );
    }
}
