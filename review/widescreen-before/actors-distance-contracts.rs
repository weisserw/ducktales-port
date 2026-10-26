// Superseded source-only expectations, retained before native widescreen.

#[test]
fn overlap_world_distance_uses_full_unsigned_borrow_to_choose_low_byte_magnitude_without_high_rejection()
 {
    for (source, target) in [
        (0x0200, 0x0100),
        (0x0100, 0x0208),
        (0x0000, 0xff08),
        (0xff08, 0x0000),
    ] {
        let mut a = contact_pool();
        a[1].position.world_x = source;
        a[LAST_ACTOR].position.world_x = target;
        assert_eq!(actor_overlap(&mut a, 1, &extents()), Some(LAST_ACTOR));
    }
}
#[test]
fn player_distance_preserves_source_low_byte_alias_and_unsigned_screen_y_magnitude() {
    for (player, actor, expected) in [
        (0x0100, 0x0208, 8),
        (0x0208, 0x0100, 8),
        (0x0000, 0xfff8, 0xf8),
    ] {
        let mut a = pool();
        a[0].position.world_x = player;
        a[1].position.world_x = actor;
        a[0].position.screen_y = 2;
        a[1].position.screen_y = 250;
        assert_eq!(player_distance(&a, 1), (expected, 248));
    }
}
