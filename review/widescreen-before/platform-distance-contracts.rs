// Superseded byte-distance behavior, retained before native widescreen.

#[test]
fn vertical_distance_is_low_byte_magnitude_not_true_far_page_distance() {
    let mut f = Fixture::new(VERTICAL);
    f.ready();
    f.actors[0].position.world_x = ACTOR_X + 0x100;
    f.run();
    assert_eq!(f.actors[0].position.screen_y, 0x82);
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