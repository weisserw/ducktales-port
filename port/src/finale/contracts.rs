//! Source-derived expectations; see trace.md for addresses, flags and consumers.
//! Boundary doubles return only external B05 contact facts, never game behavior.
use super::*;
use crate::{
    actors::{ActorContact, FINAL_ANIMATION, FINAL_HANDLER},
    collision::Position,
    player::Action,
    presentation::{ACTIVE, FLASH, FLIP_HORIZONTAL, PROJECTION_ONLY, WORLD_POSITIONED},
    status::{BLANK, Health, HealthSegment},
};
const SUBJECT: usize = 3;
const READY: u8 = ACTIVE | WORLD_POSITIONED | 1;
const HIDDEN: u8 = READY | PROJECTION_ONLY;
const RETAINED: u8 = 0x59;
const OTHER_X: u16 = 0x03f8;
fn status() -> Status {
    Status {
        reward: [BLANK; 6].into(),
        total: [BLANK; 7].into(),
        lives: 3,
        health: Health::from_overlay(
            [HealthSegment {
                y: 0,
                tile: 0,
                depleted: 0,
                x: 0,
            }; 5],
            0xff,
            0,
        ),
        countdown: [2, 0, 0].into(),
        divider: RETAINED,
        dirty: 0,
        ui_pending: 0,
    }
}
fn catalog() -> Catalog {
    Catalog {
        positions: std::array::from_fn(|i| i as u8 * 8),
        attack_offsets: [0x10, 0xf0],
        facing: [0, FLIP_HORIZONTAL],
    }
}
#[derive(Default)]
struct ContactFacts {
    overlap: bool,
    calls: Vec<usize>,
    post_action: Option<Action>,
}
impl Contact for ContactFacts {
    fn overlaps(&mut self, _: &mut ActorPool, p: &mut Player, s: usize) -> bool {
        self.calls.push(s);
        if let Some(a) = self.post_action {
            p.action = a;
        }
        self.overlap
    }
}
struct Fixture {
    pool: ActorPool,
    player: Player,
    progression: Progression,
    dialogue: Dialogue,
    status: Status,
    random: RandomState,
    terrain: WorldCache,
    contact: ContactFacts,
    catalog: Catalog,
    increment: bool,
    row: u8,
}
impl Fixture {
    fn new(handler: u8) -> Self {
        let mut pool = [ActorContact::default(); 16];
        pool[SUBJECT] = ActorContact {
            playback: crate::presentation::AnimationPlayback::from_codes(RETAINED, 0, 0),
            control: crate::actors::ActorControl::from_codes(READY, 0, 0, 0),
            handler,
            animation: FINAL_ANIMATION,
            position: Position {
                world_x: OTHER_X,
                screen_y: OPENING_Y,
                ..Position::default()
            },
            ..ActorContact::default()
        };
        let progression = Progression {
            boss_hit_points: crate::actors::FINAL_HIT_POINTS,
            ..Default::default()
        };
        Self {
            pool,
            player: Player::default(),
            progression,
            dialogue: Dialogue::default(),
            status: status(),
            random: RandomState::default(),
            terrain: WorldCache::default(),
            contact: ContactFacts::default(),
            catalog: catalog(),
            increment: false,
            row: RACE_ROW,
        }
    }
    fn run(&mut self) -> ActorEffects {
        self.run_at(SUBJECT)
    }
    fn run_at(&mut self, slot: usize) -> ActorEffects {
        actor_step(
            &mut self.pool,
            slot,
            &mut ActorContext {
                player: &mut self.player,
                progression: &mut self.progression,
                dialogue: &mut self.dialogue,
                status: &mut self.status,
                random: &mut self.random,
                terrain: &mut self.terrain,
                current_row: self.row,
                camera_page: 3,
                random_increment: self.increment,
                catalog: &self.catalog,
            },
            &mut self.contact,
        )
    }
    fn boss() -> Self {
        let mut f = Self::new(FINAL_HANDLER);
        {
            let actor_value = 2;
            f.pool[SUBJECT].set_local_state_code(actor_value);
        };
        f
    }
    fn climber() -> Self {
        let mut f = Self::new(CLIMBER_HANDLER);
        {
            let actor_value = 0x80;
            f.pool[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let actor_value = 2;
            f.pool[SUBJECT].set_local_counter_code(actor_value);
        };
        {
            let actor_value = 2;
            f.pool[LEADER_SLOT].set_local_counter_code(actor_value);
        };
        f
    }
    fn goal() -> Self {
        let mut f = Self::new(GOAL_HANDLER);
        {
            let actor_value = ACTIVE | WORLD_POSITIONED;
            f.pool[SUBJECT].set_state_code(actor_value);
        };
        f
    }
}
#[test]
fn original_catalog_reads_actual_fb54_table_and_adjacent_offsets() {
    let rom = source_rom();
    let c = Catalog::original(&rom).unwrap();
    assert_eq!(c.positions.len(), TELEPORT_POSITIONS);
    assert_eq!((c.positions[0], c.positions[15]), (0x20, 0xe0));
    assert_eq!(c.attack_offsets, [0x10, 0xf0]);
    assert_eq!(c.facing, [0, FLIP_HORIZONTAL]);
}
#[test]
fn catalog_rejects_truncation_before_last_facing_mask() {
    let r = vec![0; TELEPORT_TABLE_OFFSET + 19];
    assert!(Catalog::original(&r).is_err());
}
#[test]
fn opening_sets_terrain_y_and_decrements_initial_delay_immediately() {
    let mut f = Fixture::boss();
    {
        let actor_value = ACTIVE | WORLD_POSITIONED | PROJECTION_ONLY;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    f.pool[SUBJECT].position.screen_y = RETAINED;
    f.run();
    let a = f.pool[SUBJECT];
    assert_eq!(
        (
            a.position.screen_y,
            a.local_state_code(),
            a.state_code(),
            a.playback.tick_code()
        ),
        (OPENING_Y, INITIAL_DELAY - 1, HIDDEN, 0)
    );
    assert_eq!(f.terrain.collision[BOSS_TERRAIN_CELL], BOSS_TERRAIN_CODE);
    assert!(f.terrain.attributes.iter().all(|&v| v == 0));
}
#[test]
fn opening_wait_preserves_position_and_does_not_advance_rng() {
    let mut f = Fixture::boss();
    f.random.words = [1, 2, 3, 4];
    let before = f.pool[SUBJECT];
    f.run();
    assert_eq!(f.pool[SUBJECT].local_state_code(), 1);
    assert_eq!(f.pool[SUBJECT].position, before.position);
    assert_eq!(f.random.words, [1, 2, 3, 4]);
    assert_eq!(f.pool[SUBJECT].playback.tick_code(), 0);
}
#[test]
fn teleport_uses_all_sixteen_choices_including_wraparound_sum() {
    for index in 0..TELEPORT_POSITIONS {
        let mut f = Fixture::boss();
        {
            let actor_value = 1;
            f.pool[SUBJECT].set_local_state_code(actor_value);
        };
        f.random.words = [index as u8, 0, RETAINED, RETAINED];
        f.run();
        assert_eq!(
            f.pool[SUBJECT].position.world_x,
            0x0300 | u16::from(f.catalog.positions[index])
        );
        assert_eq!(f.pool[SUBJECT].local_counter_code(), TELEPORT_FLASH);
        assert_eq!(f.pool[SUBJECT].state_code(), READY);
        assert_eq!(f.random.words, [index as u8, 0, RETAINED, RETAINED]);
    }
}
#[test]
fn teleport_adc_includes_caller_increment_and_wraps_byte() {
    let mut f = Fixture::boss();
    {
        let actor_value = 1;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.increment = true;
    f.random.words = [0xff, 1, 3, 4];
    f.run();
    assert_eq!(f.random.words, [1, 1, 3, 4]);
    assert_eq!(f.pool[SUBJECT].position.world_x, 0x0308);
}
#[test]
fn teleport_does_not_count_flash_on_same_call() {
    let mut f = Fixture::boss();
    {
        let actor_value = 1;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_state_code(),
            f.pool[SUBJECT].local_counter_code()
        ),
        (0, TELEPORT_FLASH)
    );
    assert_eq!(f.pool[SUBJECT].profile, 0);
}
#[test]
fn facing_compares_full_world_x_including_equality_and_page() {
    for (player_x, direction, flip) in [
        (0x0300, 2, 0),
        (0x0400, 1, FLIP_HORIZONTAL),
        (0x0301, 1, FLIP_HORIZONTAL),
        (0x0200, 2, 0),
    ] {
        let mut f = Fixture::boss();
        {
            let actor_value = 1;
            f.pool[SUBJECT].set_local_state_code(actor_value);
        };
        f.pool[0].position.world_x = player_x;
        f.pool[SUBJECT].render = 0xbf;
        f.run();
        assert_eq!(
            (f.pool[SUBJECT].direction, f.pool[SUBJECT].render),
            (direction, 0xbf & !FLIP_HORIZONTAL | flip)
        );
    }
}
#[test]
fn marked_surviving_hit_overwrites_inherited_rng_increment() {
    let mut f = Fixture::boss();
    {
        let actor_value = 1;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.pool[SUBJECT].hit_marker = 1;
    f.pool[SUBJECT].boss_protection = 2;
    f.increment = true;
    f.run();
    assert_eq!(f.random.words, [0; 4]);
    assert_eq!(f.pool[SUBJECT].boss_protection, 1);
    assert_eq!(f.progression.boss_hit_points, 5);
}
#[test]
fn first_hit_consumes_shared_hp_and_continues_final_handler() {
    let mut f = Fixture::boss();
    f.pool[SUBJECT].hit_marker = 1;
    f.run();
    assert_eq!(f.progression.boss_hit_points, 4);
    assert_eq!(
        f.pool[SUBJECT].boss_protection,
        crate::actors::bosses::HIT_FLASH
    );
    assert_eq!(f.pool[SUBJECT].local_state_code(), 1);
    assert_eq!(f.pool[SUBJECT].handler, FINAL_HANDLER);
}
#[test]
fn absent_hit_marker_does_not_decay_boss_hit_protection() {
    let mut f = Fixture::boss();
    f.pool[SUBJECT].boss_protection = 2;
    f.run();
    assert_eq!(f.pool[SUBJECT].boss_protection, 2);
}
#[test]
fn final_hit_reuses_b11_without_treasure_and_returns_before_opening() {
    let mut f = Fixture::boss();
    for a in &mut f.pool {
        {
            let actor_value = READY;
            a.set_state_code(actor_value);
        };
    }
    f.pool[SUBJECT].hit_marker = 1;
    f.progression.boss_hit_points = 1;
    let before = f.pool;
    f.run();
    let a = f.pool[SUBJECT];
    assert_eq!(
        (
            a.handler,
            a.animation,
            a.playback.tick_code(),
            a.playback.frame_code(),
            a.profile,
            a.state_code()
        ),
        (
            DEFEATED_HANDLER,
            crate::actors::bosses::REVEAL,
            0,
            0,
            0,
            ACTIVE | WORLD_POSITIONED
        )
    );
    assert_eq!(a.local_state_code(), before[SUBJECT].local_state_code());
    assert_eq!(f.progression.boss_hit_points, 0);
    for (slot, previous) in before.iter().enumerate() {
        if slot != SUBJECT {
            assert_eq!(&f.pool[slot], previous);
        }
    }
    assert_eq!(f.terrain.collision[BOSS_TERRAIN_CELL], 0);
}
#[test]
fn shared_hp_zero_wrap_is_survivor_not_another_defeat() {
    let mut f = Fixture::boss();
    f.pool[SUBJECT].hit_marker = 1;
    f.progression.boss_hit_points = 0;
    f.run();
    assert_eq!(f.progression.boss_hit_points, 0xff);
    assert_eq!(f.pool[SUBJECT].handler, FINAL_HANDLER);
}
#[test]
fn flash_wait_does_not_spawn_attack_until_counter_becomes_zero() {
    let mut f = Fixture::boss();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 2;
        f.pool[SUBJECT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = RETAINED;
        f.pool[ATTACK_SLOT].set_handler(actor_value);
    };
    f.run();
    assert_eq!(f.pool[SUBJECT].local_counter_code(), 1);
    assert_eq!(f.pool[ATTACK_SLOT].handler, RETAINED);
}
#[test]
fn attack_overwrites_slot_fourteen_and_only_source_written_fields() {
    let mut f = Fixture::boss();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 1;
        f.pool[SUBJECT].set_local_counter_code(actor_value);
    };
    f.pool[SUBJECT].direction = 1;
    f.pool[SUBJECT].render = 0xff;
    f.pool[ATTACK_SLOT] = ActorContact {
        control: crate::actors::ActorControl::from_codes(0, RETAINED, RETAINED, RETAINED),
        handler: RETAINED,
        local_aux2: RETAINED,
        boss_protection: RETAINED,
        spawn_identity: Some(RETAINED),
        position: Position {
            x_fraction: RETAINED,
            y_fraction: RETAINED,
            ..Position::default()
        },
        ..ActorContact::default()
    };
    f.run();
    let a = f.pool[ATTACK_SLOT];
    assert_eq!(
        (
            a.animation,
            a.handler,
            a.profile,
            a.state_code(),
            a.render,
            a.hit_marker,
            a.playback.tick_code(),
            a.playback.frame_code()
        ),
        (
            ATTACK_ANIMATION,
            ATTACK_HANDLER,
            ATTACK_PROFILE,
            ACTIVE | WORLD_POSITIONED,
            ACTIVE | WORLD_POSITIONED,
            0,
            0,
            0
        )
    );
    assert_eq!(
        (a.position.world_x, a.position.screen_y, a.direction),
        (0x03e8, ATTACK_Y, 1)
    );
    assert_eq!(
        (
            a.local_state_code(),
            a.local_counter_code(),
            a.local_aux2,
            a.boss_protection,
            a.position.x_fraction,
            a.position.y_fraction,
            a.spawn_identity
        ),
        (
            RETAINED,
            RETAINED,
            RETAINED,
            RETAINED,
            RETAINED,
            RETAINED,
            Some(RETAINED)
        )
    );
    let b = f.pool[SUBJECT];
    assert_eq!(
        (b.render, b.profile, b.playback.frame_code(), b.local_aux2),
        (0xf0, EXPOSED_PROFILE, 1, ATTACK_DURATION)
    );
}
#[test]
fn attack_offset_wraps_low_byte_without_incrementing_page() {
    for (direction, start, expected) in [(2, 0xf8, 0x0308), (1, 8, 0x03f8)] {
        let mut f = Fixture::boss();
        {
            let actor_value = 0;
            f.pool[SUBJECT].set_local_state_code(actor_value);
        };
        {
            let actor_value = 1;
            f.pool[SUBJECT].set_local_counter_code(actor_value);
        };
        f.pool[SUBJECT].direction = direction;
        f.pool[SUBJECT].position.world_x = 0x0300 | start;
        f.run();
        assert_eq!(f.pool[ATTACK_SLOT].position.world_x, expected);
    }
}
#[test]
fn exposed_timer_above_sixty_preserves_frame_and_flash() {
    let mut f = Fixture::boss();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.pool[SUBJECT].local_aux2 = 61;
    {
        let animation_value = 1;
        f.pool[SUBJECT].playback.set_frame_code(animation_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_aux2,
            f.pool[SUBJECT].playback.frame_code(),
            f.pool[SUBJECT].render
        ),
        (60, 1, 0)
    );
}
#[test]
fn exposed_sixty_restores_flash_and_opening_frame_before_decrement() {
    let mut f = Fixture::boss();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.pool[SUBJECT].local_aux2 = 60;
    {
        let animation_value = 1;
        f.pool[SUBJECT].playback.set_frame_code(animation_value);
    };
    f.pool[SUBJECT].profile = EXPOSED_PROFILE;
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_aux2,
            f.pool[SUBJECT].playback.frame_code(),
            f.pool[SUBJECT].render,
            f.pool[SUBJECT].profile
        ),
        (59, 0, FLASH, EXPOSED_PROFILE)
    );
}
#[test]
fn attack_expiry_hides_boss_disables_collision_and_starts_sixty_delay() {
    let mut f = Fixture::boss();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.pool[SUBJECT].local_aux2 = 1;
    f.pool[SUBJECT].profile = EXPOSED_PROFILE;
    f.run();
    let a = f.pool[SUBJECT];
    assert_eq!(
        (
            a.local_aux2,
            a.local_state_code(),
            a.profile,
            a.state_code()
        ),
        (0, TELEPORT_FLASH, 0, HIDDEN)
    );
}
#[test]
fn zero_exposure_timer_wraps_without_resetting_cycle() {
    let mut f = Fixture::boss();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_aux2,
            f.pool[SUBJECT].local_state_code()
        ),
        (0xff, 0)
    );
}
#[test]
fn defeat_wait_requires_nonzero_frame_and_exact_tick_five() {
    for (frame, tick) in [(0, 5), (1, 4), (1, 6), (0xff, 0x85)] {
        let mut f = Fixture::new(DEFEATED_HANDLER);
        {
            let actor_value = ACTIVE | WORLD_POSITIONED;
            f.pool[SUBJECT].set_state_code(actor_value);
        };
        {
            let animation_value = frame;
            f.pool[SUBJECT].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = tick;
            f.pool[SUBJECT].playback.set_tick_code(animation_value);
        };
        f.run();
        assert_eq!(
            (
                f.pool[SUBJECT].state_code(),
                f.pool[SUBJECT].local_state_code()
            ),
            (ACTIVE | WORLD_POSITIONED, 0)
        );
    }
}
#[test]
fn defeat_initializes_b4_and_decrements_on_same_tick() {
    let mut f = Fixture::new(DEFEATED_HANDLER);
    {
        let actor_value = ACTIVE | WORLD_POSITIONED;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    {
        let animation_value = 2;
        f.pool[SUBJECT].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = DEFEAT_FRAME_TICK;
        f.pool[SUBJECT].playback.set_tick_code(animation_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.pool[SUBJECT].local_state_code()
        ),
        (0xb1, DEFEAT_PAUSE - 1)
    );
}
#[test]
fn defeat_last_nonzero_tick_only_decrements_then_next_call_enters_dialogue() {
    let mut f = Fixture::new(DEFEATED_HANDLER);
    {
        let actor_value = 1;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.pool[SUBJECT].local_state_code(), 0);
    assert_eq!(f.player.action, Action::Standing);
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.player.action,
            f.dialogue.message,
            f.progression.race_intro
        ),
        (
            0,
            Action::Dialogue,
            crate::dialogue::OutcomeMessage::RaceIntro as u8,
            1
        )
    );
}
#[test]
fn defeat_race_latch_increment_wraps_and_preserves_ending_latch() {
    let mut f = Fixture::new(DEFEATED_HANDLER);
    f.progression.race_intro = 0xff;
    f.progression.ending_latch = RETAINED;
    f.run();
    assert_eq!(
        (f.progression.race_intro, f.progression.ending_latch),
        (0, RETAINED)
    );
}
#[test]
fn climber_initializes_from_screen_y_then_moves_in_same_call() {
    let mut f = Fixture::climber();
    {
        let actor_value = ACTIVE | WORLD_POSITIONED;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    f.pool[SUBJECT].position.screen_y = 0x80;
    f.pool[SUBJECT].local_aux2 = RETAINED;
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_state_code(),
            f.pool[SUBJECT].local_counter_code(),
            f.pool[SUBJECT].local_aux2,
            f.pool[SUBJECT].state_code(),
            f.pool[SUBJECT].position.screen_y
        ),
        (0x7f, 2, 0, READY, 0x7f)
    );
}
#[test]
fn climber_borrow_skips_sixteen_codes_and_decrements_row() {
    let mut f = Fixture::climber();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_state_code(),
            f.pool[SUBJECT].local_counter_code()
        ),
        (0xef, 1)
    );
}
#[test]
fn climber_nonborrow_preserves_row_including_low_one() {
    let mut f = Fixture::climber();
    {
        let actor_value = 1;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_state_code(),
            f.pool[SUBJECT].local_counter_code()
        ),
        (0, 2)
    );
}
#[test]
fn climber_row_underflows_when_above_top_without_leader_hold() {
    let mut f = Fixture::climber();
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    {
        let actor_value = 0;
        f.pool[SUBJECT].set_local_counter_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_state_code(),
            f.pool[SUBJECT].local_counter_code()
        ),
        (0xef, 0xff)
    );
}
#[test]
fn leader_below_ninety_latches_exact_nonzero_y_and_freezes_all_climbers() {
    for y in [1, 0x8f] {
        let mut f = Fixture::climber();
        {
            let actor_value = 0;
            f.pool[LEADER_SLOT].set_local_counter_code(actor_value);
        };
        {
            let actor_value = y;
            f.pool[LEADER_SLOT].set_local_state_code(actor_value);
        };
        f.run();
        assert_eq!(f.pool[LEADER_SLOT].local_aux2, y);
        assert_eq!(
            (
                f.pool[SUBJECT].local_state_code(),
                f.pool[SUBJECT].local_counter_code()
            ),
            (0x80, 2)
        );
    }
}
#[test]
fn leader_at_ninety_does_not_latch_and_climber_still_moves() {
    let mut f = Fixture::climber();
    {
        let actor_value = 0;
        f.pool[LEADER_SLOT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = LEADER_FINISH_Y;
        f.pool[LEADER_SLOT].set_local_state_code(actor_value);
    };
    f.run();
    assert_eq!(f.pool[LEADER_SLOT].local_aux2, 0);
    assert_eq!(f.pool[SUBJECT].local_state_code(), 0x7f);
}
#[test]
fn leader_zero_writes_zero_marker_and_does_not_take_nonzero_freeze_branch() {
    let mut f = Fixture::climber();
    {
        let actor_value = 0;
        f.pool[LEADER_SLOT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = 0;
        f.pool[LEADER_SLOT].set_local_state_code(actor_value);
    };
    f.pool[LEADER_SLOT].local_aux2 = RETAINED;
    f.run();
    assert_eq!(f.pool[LEADER_SLOT].local_aux2, 0);
    assert_eq!(f.pool[SUBJECT].local_state_code(), 0x7f);
}
#[test]
fn leader_nonzero_row_ignores_low_threshold_and_preserves_previous_marker() {
    let mut f = Fixture::climber();
    {
        let actor_value = 1;
        f.pool[LEADER_SLOT].set_local_counter_code(actor_value);
    };
    {
        let actor_value = 1;
        f.pool[LEADER_SLOT].set_local_state_code(actor_value);
    };
    f.pool[LEADER_SLOT].local_aux2 = RETAINED;
    f.run();
    assert_eq!(f.pool[LEADER_SLOT].local_aux2, RETAINED);
    assert_eq!(f.pool[SUBJECT].local_state_code(), 0x7f);
}
#[test]
fn offrow_climber_hides_without_overwriting_stale_screen_y() {
    let mut f = Fixture::climber();
    f.row = 1;
    let y = f.pool[SUBJECT].position.screen_y;
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.pool[SUBJECT].position.screen_y
        ),
        (HIDDEN, y)
    );
}
#[test]
fn onrow_climber_clears_only_hidden_bit_and_projects_encoded_y() {
    let mut f = Fixture::climber();
    {
        let actor_value = HIDDEN;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.pool[SUBJECT].position.screen_y
        ),
        (READY, 0x7f)
    );
}
#[test]
fn goal_without_leader_only_checks_overlap_and_preserves_position() {
    let mut f = Fixture::goal();
    f.run();
    assert_eq!(f.contact.calls, [SUBJECT]);
    assert_eq!(f.pool[SUBJECT].position.world_x, OTHER_X);
    assert_eq!(f.player.action, Action::Standing);
}
#[test]
fn leader_goal_moves_low_x_only_and_still_allows_player_contact_before_boundary() {
    let mut f = Fixture::goal();
    f.pool[LEADER_SLOT].local_aux2 = 1;
    f.pool[SUBJECT].position.world_x = 0x03ae;
    f.contact.overlap = true;
    f.run();
    assert_eq!(f.pool[SUBJECT].position.world_x, 0x03af);
    assert_eq!(f.player.action, Action::Hold);
}
#[test]
fn leader_boundary_wins_arbitration_before_player_overlap() {
    let mut f = Fixture::goal();
    f.pool[LEADER_SLOT].local_aux2 = 1;
    f.pool[SUBJECT].position.world_x = 0x03af;
    f.contact.overlap = true;
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.player.action,
            f.dialogue.message
        ),
        (
            0,
            Action::Dialogue,
            crate::dialogue::OutcomeMessage::LifeLoss as u8
        )
    );
    assert!(f.contact.calls.is_empty());
    assert_eq!(f.status.divider, RETAINED);
}
#[test]
fn leader_goal_increment_wraps_without_world_page_carry() {
    let mut f = Fixture::goal();
    f.pool[LEADER_SLOT].local_aux2 = 1;
    f.pool[SUBJECT].position.world_x = 0x03ff;
    f.run();
    assert_eq!(f.pool[SUBJECT].position.world_x, 0x0300);
    assert_eq!(f.contact.calls, [SUBJECT]);
}
#[test]
fn dead_player_still_removes_losing_goal_but_does_not_open_dialogue() {
    let mut f = Fixture::goal();
    f.player.action = Action::Death;
    f.pool[LEADER_SLOT].local_aux2 = 1;
    f.pool[SUBJECT].position.world_x = 0x03af;
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.player.action,
            f.dialogue.message
        ),
        (0, Action::Death, 0)
    );
    assert!(f.contact.calls.is_empty());
}
#[test]
fn overlap_with_dead_player_does_not_start_ending_or_sound() {
    let mut f = Fixture::goal();
    f.player.action = Action::Death;
    f.contact.overlap = true;
    let effects = f.run();
    assert_eq!(f.pool[SUBJECT].state_code(), ACTIVE | WORLD_POSITIONED);
    assert_eq!(f.status.divider, RETAINED);
    assert_eq!(effects, ActorEffects::default());
}
#[test]
fn goal_rereads_player_action_after_overlap_owner_returns() {
    let mut f = Fixture::goal();
    f.contact.overlap = true;
    f.contact.post_action = Some(Action::Death);
    let e = f.run();
    assert_eq!(f.player.action, Action::Death);
    assert!(e.audio.is_empty());
    assert_eq!(f.pool[SUBJECT].state_code(), ACTIVE | WORLD_POSITIONED);
}
#[test]
fn player_win_queues_stop_then_goal_sound_and_counts_delay_immediately() {
    let mut f = Fixture::goal();
    f.contact.overlap = true;
    let e = f.run();
    assert_eq!(
        e.audio,
        [
            AudioRequest::Stage(crate::status::STOP_STAGE_SOUND),
            AudioRequest::Effect(GOAL_SOUND)
        ]
    );
    assert_eq!(
        (
            f.pool[SUBJECT].state_code(),
            f.pool[SUBJECT].local_state_code(),
            f.player.action,
            f.status.divider
        ),
        (READY, ATTACK_DURATION - 1, Action::Hold, 0)
    );
}
#[test]
fn win_delay_ignores_later_leader_loss_marker_and_death_state() {
    let mut f = Fixture::goal();
    {
        let actor_value = READY;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    {
        let actor_value = 2;
        f.pool[SUBJECT].set_local_state_code(actor_value);
    };
    f.pool[LEADER_SLOT].local_aux2 = 1;
    f.player.action = Action::Death;
    f.run();
    assert_eq!(f.pool[SUBJECT].local_state_code(), 1);
    assert_eq!(f.pool[SUBJECT].state_code(), READY);
    assert!(f.contact.calls.is_empty());
    assert_eq!(f.status.divider, 0);
}
#[test]
fn last_win_delay_increments_ending_latch_with_byte_wrap() {
    for (old, next) in [(0, 1), (0xff, 0)] {
        let mut f = Fixture::goal();
        {
            let actor_value = READY;
            f.pool[SUBJECT].set_state_code(actor_value);
        };
        {
            let actor_value = 1;
            f.pool[SUBJECT].set_local_state_code(actor_value);
        };
        f.progression.ending_latch = old;
        f.run();
        assert_eq!(
            (
                f.progression.ending_latch,
                f.pool[SUBJECT].local_state_code(),
                f.status.divider
            ),
            (next, 0, 0)
        );
    }
}
#[test]
fn zero_win_delay_wraps_to_ff_without_triggering_ending() {
    let mut f = Fixture::goal();
    {
        let actor_value = READY;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    f.run();
    assert_eq!(
        (
            f.pool[SUBJECT].local_state_code(),
            f.progression.ending_latch
        ),
        (0xff, 0)
    );
}
struct RaceFixture {
    session: RaceTerrain,
    pool: ActorPool,
    p: Progression,
    camera: Camera,
    cursor: StreamCursor,
    row: u8,
    background: u8,
    display: Display,
}
impl RaceFixture {
    fn new() -> Self {
        Self {
            session: RaceTerrain::default(),
            pool: [ActorContact::default(); 16],
            p: Progression::default(),
            camera: Camera::default(),
            cursor: StreamCursor::default(),
            row: 0,
            background: 0,
            display: Display::default(),
        }
    }
    fn resume(&mut self, result: OwnerResult) -> RaceWork {
        self.session.resume(
            &mut RaceContext {
                pool: &mut self.pool,
                progression: &mut self.p,
                camera: &mut self.camera,
                cursor: &mut self.cursor,
                row: &mut self.row,
                background_state: &mut self.background,
                display: &mut self.display,
            },
            result,
        )
    }
    fn call(&mut self) -> RaceWork {
        self.resume(OwnerResult::Complete)
    }
    fn after_clear() -> Self {
        let mut f = Self::new();
        assert_eq!(f.call(), RaceWork::ClearNonplayer);
        assert_eq!(f.call(), RaceWork::PollBackground);
        f
    }
}
#[test]
fn race_clear_must_return_before_changing_stage_cursor_or_player() {
    let mut f = RaceFixture::new();
    f.p.stage = crate::identifiers::StageId(4);
    f.pool[0].position.world_x = OTHER_X;
    assert_eq!(f.call(), RaceWork::ClearNonplayer);
    assert_eq!(f.resume(OwnerResult::Running), RaceWork::Suspended);
    assert_eq!(f.p.stage, crate::identifiers::StageId(4));
    assert_eq!(f.pool[0].position.world_x, OTHER_X);
    assert_eq!(f.cursor, StreamCursor::default());
}
#[test]
fn race_initial_poll_uses_temporary_stage_five_row_six_and_packed_room_page() {
    let f = RaceFixture::after_clear();
    assert_eq!(
        (
            f.p.stage.0,
            f.row,
            f.cursor.ring,
            f.cursor.room,
            f.cursor.room_in_row,
            f.background
        ),
        (
            RACE_BACKGROUND_STAGE,
            RACE_BACKGROUND_ROW,
            0,
            RACE_ROOM,
            7,
            0
        )
    );
    assert_eq!(
        (f.camera.x, f.camera.room, f.pool[0].position.world_x),
        (0x0700, RACE_ROOM, 0x0700)
    );
}
#[test]
fn race_preserves_player_and_camera_low_bytes_while_replacing_page() {
    let mut f = RaceFixture::new();
    f.camera.x = OTHER_X;
    f.pool[0].position.world_x = 0x0219;
    f.call();
    f.call();
    assert_eq!((f.camera.x, f.pool[0].position.world_x), (0x07f8, 0x0719));
}
#[test]
fn race_poll_suspension_keeps_background_context_until_return() {
    let mut f = RaceFixture::after_clear();
    assert_eq!(f.resume(OwnerResult::Running), RaceWork::Suspended);
    assert_eq!(
        (f.p.stage.0, f.row, f.cursor.ring),
        (RACE_BACKGROUND_STAGE, RACE_BACKGROUND_ROW, 0)
    );
    assert_eq!(f.call(), RaceWork::ResolveForwardColumn);
    assert_eq!(f.p.stage.0, RACE_STAGE);
}
#[test]
fn race_each_of_sixtyfour_columns_resolves_then_yields_before_next() {
    let mut f = RaceFixture::after_clear();
    assert_eq!(f.call(), RaceWork::ResolveForwardColumn);
    for column in 0..RACE_COLUMNS {
        assert_eq!(f.cursor.ring, column);
        assert_eq!(f.call(), RaceWork::Yield);
        assert_eq!(f.cursor.ring, column + 1);
        let next = f.call();
        assert_eq!(
            next,
            if column + 1 == RACE_COLUMNS {
                RaceWork::PollBackground
            } else {
                RaceWork::ResolveForwardColumn
            }
        );
    }
    assert_eq!(f.p.stage.0, RACE_BACKGROUND_STAGE);
}
#[test]
fn race_resolve_suspension_cannot_increment_cursor_or_yield() {
    let mut f = RaceFixture::after_clear();
    assert_eq!(f.call(), RaceWork::ResolveForwardColumn);
    for _ in 0..3 {
        assert_eq!(f.resume(OwnerResult::Running), RaceWork::Suspended);
        assert_eq!(f.cursor.ring, 0);
    }
    assert_eq!(f.call(), RaceWork::Yield);
    assert_eq!(f.cursor.ring, 1);
}
#[test]
fn race_final_poll_nonnegative_waits_and_repeats_under_stage_five() {
    let mut f = RaceFixture::new();
    f.session.phase = RacePhase::FinalUpload;
    f.cursor.ring = RACE_COLUMNS;
    f.p.stage = crate::identifiers::StageId(RACE_BACKGROUND_STAGE);
    for value in [0, 1, 0x7f] {
        f.background = value;
        assert_eq!(f.call(), RaceWork::Yield);
        assert_eq!(f.p.stage.0, RACE_STAGE);
        assert_eq!(f.call(), RaceWork::PollBackground);
        assert_eq!(f.p.stage.0, RACE_BACKGROUND_STAGE);
    }
}
#[test]
fn race_final_negative_completion_restores_row_two_before_spawn_search() {
    let mut f = RaceFixture::new();
    f.session.phase = RacePhase::FinalUpload;
    f.p.stage = crate::identifiers::StageId(RACE_BACKGROUND_STAGE);
    f.row = RACE_BACKGROUND_ROW;
    f.background = 0x80;
    f.p.late_mode = RETAINED;
    assert_eq!(f.call(), RaceWork::FindSpawnWindow);
    assert_eq!(
        (f.p.stage.0, f.row, f.p.late_mode),
        (RACE_STAGE, RACE_ROW, RETAINED)
    );
    assert!(!f.display.palette_pending);
    assert_eq!(f.call(), RaceWork::SpawnWindow);
    assert_eq!(f.p.late_mode, 0);
    assert!(!f.display.palette_pending);
    assert_eq!(f.call(), RaceWork::Complete);
    assert!(f.display.palette_pending);
}
#[test]
fn race_spawn_search_suspension_preserves_late_mode_until_return() {
    let mut f = RaceFixture::new();
    f.session.phase = RacePhase::SpawnStart;
    f.p.late_mode = RETAINED;
    assert_eq!(f.resume(OwnerResult::Running), RaceWork::Suspended);
    assert_eq!(f.p.late_mode, RETAINED);
    assert_eq!(f.call(), RaceWork::SpawnWindow);
    assert_eq!(f.p.late_mode, 0);
}
#[test]
fn race_spawn_suspension_does_not_submit_palette_early() {
    let mut f = RaceFixture::new();
    f.session.phase = RacePhase::Spawn;
    assert_eq!(f.resume(OwnerResult::Running), RaceWork::Suspended);
    assert!(!f.display.palette_pending);
    assert_eq!(f.call(), RaceWork::Complete);
    assert!(f.display.palette_pending);
}
struct EndingFixture {
    scene_cursor: u8,
    display_mode: u8,
    ending: Ending,
    p: Progression,
    status: Status,
    pool: ActorPool,
    composition: Composition,
    display: Display,
    audio: u8,
}
impl EndingFixture {
    fn new() -> Self {
        let p = Progression {
            ending_latch: 1,
            ..Default::default()
        };
        Self {
            ending: Ending::default(),
            scene_cursor: 0,
            display_mode: 0,
            p,
            status: status(),
            pool: [ActorContact::default(); 16],
            composition: Composition::default(),
            display: Display::default(),
            audio: 0,
        }
    }
    fn resume(&mut self, result: OwnerResult) -> EndingWork {
        self.ending.resume(
            &mut EndingContext {
                scene_cursor: &mut self.scene_cursor,
                display_mode: &mut self.display_mode,
                progression: &mut self.p,
                status: &mut self.status,
                pool: &mut self.pool,
                composition: &mut self.composition,
                display: &mut self.display,
                current_stage_audio: &mut self.audio,
            },
            result,
        )
    }
    fn call(&mut self) -> EndingWork {
        self.resume(OwnerResult::Complete)
    }
    fn before_choice() -> Self {
        let mut f = Self::new();
        assert_eq!(f.call(), EndingWork::TransferValues);
        assert_eq!(f.call(), EndingWork::Wait(ENDING_ENTRY_WAIT));
        assert_eq!(f.call(), EndingWork::StageAudio(ENDING_AUDIO));
        assert_eq!(f.call(), EndingWork::Layout(crate::scenes::ENDING_LAYOUT));
        f
    }
    fn continued() -> Self {
        let mut f = Self::new();
        f.p.ending_latch = 0x81;
        {
            let actor_value = crate::scenes::SceneHandler::Settled as u8;
            f.pool[0].set_handler(actor_value);
        };
        f
    }
    fn split(cursor: u8, y: u8) -> Self {
        let mut f = Self::continued();
        f.scene_cursor = cursor;
        f.ending.phase = EndingPhase::SplitPixel;
        f.composition.oam[0].y = y;
        f
    }
}
#[test]
fn ending_marks_latch_before_value_transfer_and_preserves_low_bits() {
    for value in [1, 0x7f] {
        let mut f = EndingFixture::new();
        f.p.ending_latch = value;
        assert_eq!(f.call(), EndingWork::TransferValues);
        assert_eq!(f.p.ending_latch, value | 0x80);
        assert_eq!(f.audio, 0);
        assert_eq!(f.p.stage.0, 0);
    }
}
#[test]
fn ending_transfer_suspension_cannot_start_wait_audio_or_variant() {
    let mut f = EndingFixture::new();
    f.call();
    for _ in 0..3 {
        assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
        assert_eq!((f.ending.variation, f.p.stage.0, f.audio), (0, 0, 0));
    }
    assert_eq!(f.call(), EndingWork::Wait(ENDING_ENTRY_WAIT));
}
#[test]
fn ending_entry_wait_precedes_audio_and_layout_requests() {
    let mut f = EndingFixture::new();
    assert_eq!(f.call(), EndingWork::TransferValues);
    assert_eq!(f.call(), EndingWork::Wait(ENDING_ENTRY_WAIT));
    assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
    assert_eq!(f.call(), EndingWork::StageAudio(ENDING_AUDIO));
    assert_eq!(f.call(), EndingWork::Layout(crate::scenes::ENDING_LAYOUT));
    assert_eq!(f.p.stage.0, crate::scenes::ENDING_LAYOUT);
}
#[test]
fn ending_choice_waits_for_layout_return_and_reads_fresh_total_after_transfer() {
    let mut f = EndingFixture::before_choice();
    f.p.progress_bits = 0x7f;
    f.status.total.edit_display_codes(|codes| codes[0] = 1);
    assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
    assert_eq!(f.call(), EndingWork::Build(Scene::OpeningThird));
    assert_eq!((f.ending.variation, f.scene_cursor), (2, 2));
}
#[test]
fn all_completion_low_seven_bits_plus_nonblank_first_total_select_third() {
    for progress in [0x7f, 0xff] {
        for first in [0, 1, 9, 0x25] {
            let mut f = EndingFixture::before_choice();
            f.p.progress_bits = progress;
            f.status.total.edit_display_codes(|codes| codes[0] = first);
            assert_eq!(f.call(), EndingWork::Build(Scene::OpeningThird));
            assert_eq!(f.ending.variation, 2);
        }
    }
}
#[test]
fn incomplete_progress_ignores_first_total_and_uses_last_blank_for_second() {
    for progress in [0, 0x1f, 0x3f, 0x7e, 0xfe] {
        let mut f = EndingFixture::before_choice();
        f.p.progress_bits = progress;
        f.status.total.edit_display_codes(|codes| codes[0] = 9);
        assert_eq!(f.call(), EndingWork::Build(Scene::OpeningSecond));
        assert_eq!(f.ending.variation, 1);
    }
}
#[test]
fn complete_progress_but_first_blank_falls_back_to_last_total_code() {
    for (last, scene, variation) in [
        (BLANK, Scene::OpeningSecond, 1),
        (0, Scene::OpeningFirst, 0),
        (9, Scene::OpeningFirst, 0),
    ] {
        let mut f = EndingFixture::before_choice();
        f.p.progress_bits = 0xff;
        f.status.total.edit_display_codes(|codes| codes[6] = last);
        assert_eq!(f.call(), EndingWork::Build(scene));
        assert_eq!(f.ending.variation, variation);
    }
}
#[test]
fn nonblank_last_total_selects_first_even_when_numerically_zero() {
    let mut f = EndingFixture::before_choice();
    f.status.total = [0; 7].into();
    assert_eq!(f.call(), EndingWork::Build(Scene::OpeningFirst));
    assert_eq!(f.ending.variation, 0);
}
#[test]
fn middle_total_and_reward_codes_do_not_choose_ending() {
    let mut f = EndingFixture::before_choice();
    f.status.reward = [9; 6].into();
    f.status.total = [BLANK, 9, 9, 9, 9, 9, BLANK].into();
    assert_eq!(f.call(), EndingWork::Build(Scene::OpeningSecond));
}
#[test]
fn scene_build_must_finish_before_cursor_increment_or_frame_return() {
    let mut f = EndingFixture::before_choice();
    assert_eq!(f.call(), EndingWork::Build(Scene::OpeningSecond));
    assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
    assert_eq!(f.scene_cursor, 1);
    assert_eq!(f.call(), EndingWork::Complete);
    assert_eq!(f.scene_cursor, 2);
}
#[test]
fn scene_build_increments_all_selectors_and_wraps_only_six() {
    for (scene, next) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)] {
        let mut f = EndingFixture::continued();
        f.ending.phase = EndingPhase::Build;
        f.scene_cursor = scene;
        assert_eq!(f.call(), EndingWork::Complete);
        assert_eq!(f.scene_cursor, next);
    }
}
#[test]
fn negative_latch_continuation_does_not_repeat_transfer_or_initial_wait() {
    let mut f = EndingFixture::continued();
    {
        let actor_value = crate::scenes::SceneHandler::Bounce as u8;
        f.pool[0].set_handler(actor_value);
    };
    assert_eq!(f.call(), EndingWork::UpdateSceneActors);
    assert_eq!(f.call(), EndingWork::Complete);
}
#[test]
fn scene_update_returns_to_composition_without_rechecking_settled_in_same_call() {
    let mut f = EndingFixture::continued();
    {
        let actor_value = 0;
        f.pool[0].set_handler(actor_value);
    };
    assert_eq!(f.call(), EndingWork::UpdateSceneActors);
    {
        let actor_value = 5;
        f.pool[0].set_handler(actor_value);
    };
    assert_eq!(f.call(), EndingWork::Complete);
    assert_eq!(f.call(), EndingWork::Wait(ENDING_PAUSE_TICKS));
}
#[test]
fn scene_update_suspension_prevents_composition_and_pause_tail() {
    let mut f = EndingFixture::continued();
    {
        let actor_value = 0;
        f.pool[0].set_handler(actor_value);
    };
    f.call();
    assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
    assert_eq!(f.ending.pauses_left, 0);
}
#[test]
fn settled_scene_handler_requires_exact_five() {
    for handler in [0, 1, 2, 3, 4, 6, 0xff] {
        let mut f = EndingFixture::continued();
        {
            let actor_value = handler;
            f.pool[0].set_handler(actor_value);
        };
        assert_eq!(f.call(), EndingWork::UpdateSceneActors);
    }
}
#[test]
fn display_mode_zero_starts_ten_distinct_sixty_tick_waits() {
    let mut f = EndingFixture::continued();
    f.scene_cursor = 1;
    for _ in 0..ENDING_PAUSE_INTERVALS {
        assert_eq!(f.call(), EndingWork::Wait(ENDING_PAUSE_TICKS));
    }
    assert_eq!(f.call(), EndingWork::Build(Scene::Value));
}
#[test]
fn pause_suspension_cannot_consume_remaining_intervals() {
    let mut f = EndingFixture::continued();
    f.scene_cursor = 1;
    assert_eq!(f.call(), EndingWork::Wait(ENDING_PAUSE_TICKS));
    let remaining = f.ending.pauses_left;
    for _ in 0..3 {
        assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
        assert_eq!(f.ending.pauses_left, remaining);
    }
}
#[test]
fn pause_forces_cursors_one_two_three_to_four_and_preserves_four_five() {
    for (cursor, scene) in [
        (1, Scene::Value),
        (2, Scene::Value),
        (3, Scene::Value),
        (4, Scene::Value),
        (5, Scene::TerminalLead),
    ] {
        let mut f = EndingFixture::continued();
        f.scene_cursor = cursor;
        for _ in 0..ENDING_PAUSE_INTERVALS {
            assert_eq!(f.call(), EndingWork::Wait(ENDING_PAUSE_TICKS));
        }
        assert_eq!(f.call(), EndingWork::Build(scene));
    }
}
#[test]
fn cursor_zero_after_ten_waits_enters_permanent_terminal_yield() {
    let mut f = EndingFixture::continued();
    for _ in 0..ENDING_PAUSE_INTERVALS {
        assert_eq!(f.call(), EndingWork::Wait(ENDING_PAUSE_TICKS));
    }
    for _ in 0..300 {
        assert_eq!(f.call(), EndingWork::Yield);
    }
    assert_eq!(f.ending.phase, EndingPhase::Terminal);
    assert_eq!(f.p.ending_latch, 0x81);
}
#[test]
fn terminal_suspension_never_returns_to_gameplay_or_restarts_scenes() {
    let mut f = EndingFixture::continued();
    f.ending.phase = EndingPhase::Terminal;
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.p.stage.0, 0);
}
#[test]
fn nonzero_display_mode_clears_and_resets_split_delay_before_pixel() {
    let mut f = EndingFixture::continued();
    f.display_mode = 0x40;
    f.ending.split_delay = RETAINED;
    f.composition.oam[0].y = 0x90;
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(
        (f.display_mode, f.ending.split_delay, f.composition.oam[0].y),
        (0, 0, 0x91)
    );
}
#[test]
fn split_increments_sprite_y_before_e8_stop_without_yield() {
    let mut f = EndingFixture::split(0, SPLIT_BOTTOM - 1);
    assert_eq!(f.call(), EndingWork::Complete);
    assert_eq!(f.composition.oam[0].y, SPLIT_BOTTOM);
}
#[test]
fn cursor_five_stops_at_b0_but_other_scenes_continue() {
    for cursor in [0, 1, 2, 3, 4, 5] {
        let mut f = EndingFixture::split(cursor, SPLIT_VALUE_BOTTOM - 1);
        assert_eq!(
            f.call(),
            if cursor == 5 {
                EndingWork::Complete
            } else {
                EndingWork::Yield
            }
        );
        assert_eq!(f.composition.oam[0].y, SPLIT_VALUE_BOTTOM);
    }
}
#[test]
fn split_wraps_byte_y_instead_of_clamping() {
    let mut f = EndingFixture::split(4, 0xff);
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.composition.oam[0].y, 0);
}
#[test]
fn split_yield_precedes_sprite_zero_edge_nametable_switch() {
    let mut f = EndingFixture::split(4, 0x90);
    f.display.control = 0x90;
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.resume(OwnerResult::Running), EndingWork::Suspended);
    assert_eq!(f.call(), EndingWork::SplitRaster(0x91));
    assert_eq!(f.display.control, 0x90);
}
#[test]
fn split_ordinary_pixel_advances_after_completed_raster() {
    let mut f = EndingFixture::split(4, 0x90);
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.call(), EndingWork::SplitRaster(1));
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.composition.oam[0].y, 0x92);
    assert_eq!(f.ending.split_delay, 0);
}
#[test]
fn split_every_eighth_pixel_repeats_sixtyfour_raster_yield_pairs_at_same_y() {
    let mut f = EndingFixture::split(4, 0x96);
    assert_eq!(f.call(), EndingWork::Yield);
    for delay in 1..=SPLIT_DELAY_INTERVALS {
        assert_eq!(f.call(), EndingWork::SplitRaster(1));
        assert_eq!(f.call(), EndingWork::Yield);
        assert_eq!(
            f.composition.oam[0].y,
            if delay == SPLIT_DELAY_INTERVALS {
                0x98
            } else {
                0x97
            }
        );
        assert_eq!(
            f.ending.split_delay,
            if delay == SPLIT_DELAY_INTERVALS {
                0
            } else {
                delay
            }
        );
    }
}
#[test]
fn cursor_zero_below_b7_uses_regular_sixtyfour_delay() {
    let mut f = EndingFixture::split(0, 0xae);
    f.call();
    assert_eq!(f.call(), EndingWork::SplitRaster(1));
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!((f.composition.oam[0].y, f.ending.split_delay), (0xaf, 1));
}
#[test]
fn cursor_zero_at_b7_requests_music_seven_instead_of_delay() {
    let mut f = EndingFixture::split(0, SPLIT_AUDIO_MARK - 1);
    f.call();
    assert_eq!(f.call(), EndingWork::SplitRaster(1));
    assert_eq!(f.call(), EndingWork::StageAudio(SCROLL_AUDIO));
    assert_eq!(f.ending.split_delay, 0);
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(f.composition.oam[0].y, SPLIT_AUDIO_MARK + 1);
}
#[test]
fn existing_music_seven_skips_requeue_and_delay_at_late_eighth_pixel() {
    let mut f = EndingFixture::split(0, 0xbe);
    f.audio = SCROLL_AUDIO;
    f.call();
    assert_eq!(f.call(), EndingWork::SplitRaster(1));
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!((f.composition.oam[0].y, f.ending.split_delay), (0xc0, 0));
}
#[test]
fn other_scene_at_b7_does_not_change_music_and_still_delays() {
    let mut f = EndingFixture::split(4, SPLIT_AUDIO_MARK - 1);
    f.call();
    assert_eq!(f.call(), EndingWork::SplitRaster(1));
    assert_eq!(f.call(), EndingWork::Yield);
    assert_eq!(
        (f.composition.oam[0].y, f.ending.split_delay, f.audio),
        (SPLIT_AUDIO_MARK, 1, 0)
    );
}
fn source_rom() -> Vec<u8> {
    let b = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
    romatlas_port::assets::Rom::from_bytes(b.clone(), crate::config::ROM_SHA256).unwrap();
    b
}
#[test]
fn actual_race_source_rows_join_two_climbers_and_top_goal_templates() {
    let c = crate::actors::SpawnCatalog::original(&source_rom()).unwrap();
    let stage = c.source(crate::identifiers::StageId(RACE_STAGE)).unwrap();
    let row = &stage.rows[RACE_ROW as usize];
    let rivals: Vec<_> = row
        .iter()
        .filter(|r| c.templates[r.template].handler == CLIMBER_HANDLER)
        .collect();
    assert_eq!(rivals.len(), 2);
    assert_eq!((rivals[0].world_x, rivals[0].screen_y), (0x07b0, 0xb0));
    assert_eq!((rivals[1].world_x, rivals[1].screen_y), (0x07ac, 0x94));
    assert_eq!(
        (
            c.templates[rivals[0].template].animation,
            c.templates[rivals[1].template].animation
        ),
        (0x69, 0x5e)
    );
    assert!(stage.rows[1].is_empty());
    let goal = stage.rows[0]
        .iter()
        .find(|r| c.templates[r.template].handler == GOAL_HANDLER)
        .unwrap();
    assert_eq!(
        (
            goal.world_x,
            goal.screen_y,
            c.templates[goal.template].animation
        ),
        (0x0780, 0x98, crate::actors::bosses::TREASURE_ANIMATION)
    );
}
#[test]
fn actual_race_window_allocates_first_rival_to_leader_fifteen_then_second_fourteen() {
    let c = crate::actors::SpawnCatalog::original(&source_rom()).unwrap();
    let stage = c.source(crate::identifiers::StageId(RACE_STAGE)).unwrap();
    let mut pool = [ActorContact::default(); 16];
    // AE60 reads the room's global start identity; row2 is not identity zero.
    let start = stage.rows[RACE_ROW as usize].first().unwrap().identity;
    let mut w = crate::actors::Window {
        room_start: start,
        lower: start,
        upper: start,
        direction: 1,
    };
    let mut context = crate::actors::SpawnContext {
        stage: RACE_STAGE,
        room_row: RACE_ROW,
        camera: 0x0700,
        direction: 1,
        ..Default::default()
    };
    let report = crate::actors::scan_window(
        &mut pool,
        &[0; crate::actors::SUPPRESSION_BYTES],
        &stage.rows[RACE_ROW as usize],
        &c.templates,
        &mut w,
        &mut context,
    );
    assert_eq!(report.ordinary, [(5, LEADER_SLOT), (6, ATTACK_SLOT)]);
    assert_eq!(
        (pool[LEADER_SLOT].handler, pool[ATTACK_SLOT].handler),
        (CLIMBER_HANDLER, CLIMBER_HANDLER)
    );
    assert_eq!(
        (
            pool[LEADER_SLOT].position.screen_y,
            pool[ATTACK_SLOT].position.screen_y
        ),
        (0xb0, 0x94)
    );
}
#[test]
fn leader_initial_position_runs_two_f0_rows_then_freezes_at_eightyf_without_booleanizing_marker() {
    const STEPS_TO_MARK: usize = 514;
    let mut f = Fixture::climber();
    f.pool[LEADER_SLOT] = ActorContact {
        control: crate::actors::ActorControl::from_codes(
            ACTIVE | WORLD_POSITIONED,
            0,
            0,
            CLIMBER_HANDLER,
        ),
        handler: CLIMBER_HANDLER,
        position: Position {
            screen_y: 0xb0,
            ..Default::default()
        },
        ..Default::default()
    };
    for _ in 0..STEPS_TO_MARK - 1 {
        f.run_at(LEADER_SLOT);
    }
    assert_eq!(
        (
            f.pool[LEADER_SLOT].local_state_code(),
            f.pool[LEADER_SLOT].local_counter_code(),
            f.pool[LEADER_SLOT].local_aux2
        ),
        (0x8f, 0, 0)
    );
    f.run_at(LEADER_SLOT);
    assert_eq!(
        (
            f.pool[LEADER_SLOT].local_state_code(),
            f.pool[LEADER_SLOT].local_counter_code(),
            f.pool[LEADER_SLOT].local_aux2
        ),
        (0x8f, 0, 0x8f)
    );
}
#[test]
fn live_ascending_rivals_read_leader_before_its_move_then_latch_next_call() {
    let mut f = Fixture::climber();
    f.pool[LEADER_SLOT] = ActorContact {
        control: crate::actors::ActorControl::from_codes(
            READY,
            LEADER_FINISH_Y,
            0,
            CLIMBER_HANDLER,
        ),
        handler: CLIMBER_HANDLER,
        ..Default::default()
    };
    f.pool[ATTACK_SLOT] = ActorContact {
        control: crate::actors::ActorControl::from_codes(READY, 0x60, 0, CLIMBER_HANDLER),
        handler: CLIMBER_HANDLER,
        ..Default::default()
    };
    f.run_at(ATTACK_SLOT);
    f.run_at(LEADER_SLOT);
    assert_eq!(
        (
            f.pool[ATTACK_SLOT].local_state_code(),
            f.pool[LEADER_SLOT].local_state_code(),
            f.pool[LEADER_SLOT].local_aux2
        ),
        (0x5f, 0x8f, 0)
    );
    f.run_at(ATTACK_SLOT);
    assert_eq!(
        (
            f.pool[ATTACK_SLOT].local_state_code(),
            f.pool[LEADER_SLOT].local_aux2
        ),
        (0x5f, 0x8f)
    );
}
#[test]
fn whole_b4_defeat_pause_requires_following_zero_entry_to_start_intro() {
    let mut f = Fixture::new(DEFEATED_HANDLER);
    {
        let actor_value = ACTIVE | WORLD_POSITIONED;
        f.pool[SUBJECT].set_state_code(actor_value);
    };
    {
        let animation_value = 1;
        f.pool[SUBJECT].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = DEFEAT_FRAME_TICK;
        f.pool[SUBJECT].playback.set_tick_code(animation_value);
    };
    for _ in 0..DEFEAT_PAUSE {
        f.run();
        assert_eq!(f.progression.race_intro, 0);
    }
    assert_eq!(f.pool[SUBJECT].local_state_code(), 0);
    f.run();
    assert_eq!(f.progression.race_intro, 1);
    assert_eq!(f.player.action, Action::Dialogue);
}
#[test]
fn whole_f0_win_delay_includes_contact_call_and_never_requeues_sounds() {
    let mut f = Fixture::goal();
    f.contact.overlap = true;
    assert_eq!(f.run().audio.len(), 2);
    for _ in 1..ATTACK_DURATION - 1 {
        assert!(f.run().audio.is_empty());
        assert_eq!(f.progression.ending_latch, 0);
    }
    assert_eq!(f.pool[SUBJECT].local_state_code(), 1);
    assert!(f.run().audio.is_empty());
    assert_eq!(f.progression.ending_latch, 1);
    assert_eq!(f.contact.calls, [SUBJECT]);
}
#[test]
fn scene_build_uses_live_b16_cursor_after_owner_mutation_not_saved_requested_scene() {
    let mut f = EndingFixture::before_choice();
    assert_eq!(f.call(), EndingWork::Build(Scene::OpeningSecond));
    f.scene_cursor = 5;
    assert_eq!(f.call(), EndingWork::Complete);
    assert_eq!(f.scene_cursor, 0);
}
#[test]
fn split_completion_returns_before_pause_then_next_entry_starts_waits() {
    let mut f = EndingFixture::split(5, SPLIT_VALUE_BOTTOM - 1);
    assert_eq!(f.call(), EndingWork::Complete);
    assert_eq!(f.ending.pauses_left, 0);
    assert_eq!(f.call(), EndingWork::Wait(ENDING_PAUSE_TICKS));
}
#[test]
fn terminal_ignores_external_actor_and_progress_changes_and_preserves_shared_display() {
    let mut f = EndingFixture::continued();
    f.ending.phase = EndingPhase::Terminal;
    f.display_mode = RETAINED;
    for action in [Action::Standing, Action::Death, Action::Dialogue] {
        {
            let actor_value = 0;
            f.pool[0].set_handler(actor_value);
        };
        f.p.progress_bits = 0;
        {
            let actor_value = action as u8;
            f.pool[0].set_local_state_code(actor_value);
        };
        assert_eq!(f.call(), EndingWork::Yield);
        assert_eq!(f.display_mode, RETAINED);
        assert_eq!(f.ending.phase, EndingPhase::Terminal);
    }
}
