// Superseded source-only expectations, retained before native widescreen.

#[test]
fn world_x_contact_selects_absolute_low_byte_using_full_unsigned_borrow_without_high_distance_rejection()
 {
    let mut all = actors();
    all[2] = actor();
    all[15] = actor();
    {
        let actor_value = SOLID_HANDLER;
        all[15].set_handler(actor_value);
    };
    all[2].position.world_x = 0x0200;
    all[15].position.world_x = 0x0101;
    assert_eq!(
        solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal),
        None
    ); // low differenceFF, not signed -1
    all[2].position.world_x = 0x0101;
    all[15].position.world_x = 0x0200;
    assert_eq!(
        solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal),
        None
    ); // negated low01 ->FF
    all[15].position.world_x = 0x0301;
    assert_eq!(
        solid_overlap(2, &all, &geometry(), ContactAxis::Horizontal),
        Some(Contact {
            slot: 15,
            penetration: TEST_WIDTH
        })
    );
}