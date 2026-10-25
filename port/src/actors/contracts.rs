//! Independent executable B10 contracts. Each rule derives from trace.md;
//! ROM fixtures are read locally and no commercial bytes are embedded here.
use super::*;
use crate::config;
use romatlas_port::assets::Rom;
const SOURCE_BANK: usize = 6;
const PRG_BANK_BYTES: usize = 0x4000;
const HEADER_BYTES: usize = 16;
const SWITCH_ORIGIN: usize = 0x8000;
const TEMPLATE_CPU: usize = 0xbe16;
const LAST_ACTOR: usize = ACTOR_SLOTS - 1;
const FORWARD: u8 = 1;
const BACKWARD: u8 = 2;
const WORLD_POSITIONED: u8 = 0x20;
const TERRAIN_REPLACEMENT: u8 = 0x10;
const STAGE_ONE: u8 = 1;
fn rom() -> Vec<u8> {
    let b =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../input/game.rom"))
            .expect("local ROM");
    Rom::from_bytes(b.clone(), config::ROM_SHA256).expect("matching ROM");
    b
}
fn bank_byte(b: &[u8], bank: usize, cpu: usize) -> u8 {
    b[HEADER_BYTES + bank * PRG_BANK_BYTES + cpu - SWITCH_ORIGIN]
}
fn pointer(b: &[u8], low: usize, high: usize, index: usize) -> usize {
    usize::from(bank_byte(b, SOURCE_BANK, low + index))
        | usize::from(bank_byte(b, SOURCE_BANK, high + index)) << 8
}
fn pool() -> ActorPool {
    [ActorContact::default(); ACTOR_SLOTS]
}
fn occupied() -> ActorPool {
    [ActorContact {
        control: crate::actors::ActorControl::from_codes(ACTIVE, 0, 0, 0),
        render: VISIBLE,
        ..ActorContact::default()
    }; ACTOR_SLOTS]
}
fn template() -> Template {
    Template {
        flags_and_local: ACTIVE | WORLD_POSITIONED | 3,
        animation: 7,
        handler: 2,
        profile: 5,
    }
}
fn record(identity: u8, x: u16) -> SpawnRecord {
    SpawnRecord {
        identity,
        world_x: x,
        screen_y: 0x70,
        template: 0,
    }
}
fn context() -> SpawnContext {
    SpawnContext {
        camera: 0x0200,
        direction: FORWARD,
        ..SpawnContext::default()
    }
}
fn bits() -> [u8; SUPPRESSION_BYTES] {
    [0; SUPPRESSION_BYTES]
}
fn special(page: u8, paired: bool) -> SpecialRecord {
    SpecialRecord {
        page,
        paired,
        template: 0,
        screen_y: 0x60,
    }
}
fn contact_pool() -> ActorPool {
    let mut a = pool();
    a[1] = ActorContact {
        render: VISIBLE,
        profile: 1,
        animation: 4,
        position: Position {
            world_x: 0x0200,
            screen_y: 0x80,
            ..Position::default()
        },
        ..ActorContact::default()
    };
    a[LAST_ACTOR] = ActorContact {
        render: VISIBLE,
        handler: 2,
        position: Position {
            world_x: 0x0208,
            screen_y: 0x87,
            ..Position::default()
        },
        ..ActorContact::default()
    };
    a
}
fn extents() -> [Extent; 32] {
    [Extent {
        half_width: 8,
        half_height: 8,
    }; 32]
}
#[test]
fn original_handler_table_resolves_all_sixtyseven_source_entries_and_cohesive_owners() {
    // Labels independently reread at source targets; aliases30/31 share AC3D.
    let catalog = SpawnCatalog::original(&rom()).unwrap();
    let labels = [
        "EmptyActorHandler",
        "Actor8225VisibilityAndHit",
        "Bank06_WalkingBounceActor",
        "Bank06_ProjectileThrower",
        "Bank06_ThrownProjectile",
        "Actor8491AnimationGate",
        "Bank06_ProximityRisingActor",
        "Bank06_PlayerContactAnimationActor",
        "Actor8668Cull",
        "Bank06_UpdateMovingTerrainObject",
        "Bank06_ProximityFallingObject",
        "ActorLaunchedMotion",
        "Bank06_TriggeredWalkingPlatform",
        "Bank06_PresetOscillatingActor",
        "Bank06_RideablePresetActor",
        "Bank06_VerticalCarryingPlatform",
        "Actor9313Wait",
        "Bank06_DelayedPresetFlyingActor",
        "Bank06_PairedActorController",
        "Actor8A59AnimationGate",
        "Actor8C45ContactEntry",
        "Actor8D19ContactProfile",
        "Bank06_RevealTransformedObject",
        "Bank06_SlidingTransformedObject",
        "Bank06_LedgeAwareWalker",
        "Bank06_FivePhaseAmbushActor",
        "Bank06_ProximityEruptingActor",
        "Bank06_RepeatingJumpActor",
        "Actor9B10HitAnimation",
        "Bank06_PushingContactActor",
        "Bank06_FastLedgeAwareWalker",
        "Bank06_TimedHorizontalPatrol",
        "Actor9E51Initialize",
        "Bank06_PlayerCarryingActor",
        "Bank06_TriggeredJumpingPatrol",
        "Bank06_LeftwardWalkingActor",
        "Bank06_PlayerExternalMotionTrigger",
        "ActorA065HitGate",
        "Bank06_FivePhaseAttackActor",
        "Bank06_UpdateAttackProjectile",
        "Bank06_PerimeterPathActor",
        "Bank06_CenterReturningActor",
        "Bank06_BouncingWaveActor",
        "Bank06_UpdateWaveChild",
        "Bank06_FourItemDispenser",
        "Bank06_UpdateDispensedItem",
        "Bank06_JumpingImpactActor",
        "Bank06_ProgressionContactSelector",
        "Bank06_TimedRewardStealingPatrol",
        "Bank06_TimedRewardStealingPatrol",
        "Bank06_JumpingRewardStealer",
        "Bank06_HitOnlyReactionActor",
        "Bank06_PlayerLaunchContactActor",
        "UpdateDialogueContactActor",
        "UpdatePlayerLinkedDeparture",
        "UpdateTimedDepartureSpawner",
        "UpdatePlayerLiftActor",
        "UpdateFourPhaseMovingPlatform",
        "UpdateFinalBoss",
        "FinalBossDefeatCountdown",
        "UpdateRaceClimber",
        "UpdateHitActivatedDialogueActor",
        "UpdateProgressionEscort",
        "UpdateRightwardTransitionActor",
        "UpdateFinalRaceGoal",
        "UpdateConditionalContactMarker",
        "Bank06_SwingReleasedCollectible",
    ];
    assert_eq!(catalog.handlers.len(), HANDLER_COUNT);
    for (i, label) in labels.iter().enumerate() {
        assert_eq!(&catalog.handlers[i].label, label, "selector {i:02X}");
    }
    // Every entry is assigned below rather than treating fixed-bank tails as B10.
    let owners = [
        HandlerOwner::Lifecycle,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Platforms,
        HandlerOwner::Platforms,
        HandlerOwner::Platforms,
        HandlerOwner::Platforms,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Platforms,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Platforms,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Platforms,
        HandlerOwner::Enemies,
        HandlerOwner::Bosses,
        HandlerOwner::Bosses,
        HandlerOwner::Bosses,
        HandlerOwner::Bosses,
        HandlerOwner::Bosses,
        HandlerOwner::Bosses,
        HandlerOwner::Objects,
        HandlerOwner::Objects,
        HandlerOwner::Bosses,
        HandlerOwner::Dialogue,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Enemies,
        HandlerOwner::Dialogue,
        HandlerOwner::Scenes,
        HandlerOwner::Scenes,
        HandlerOwner::Platforms,
        HandlerOwner::Platforms,
        HandlerOwner::Finale,
        HandlerOwner::Finale,
        HandlerOwner::Finale,
        HandlerOwner::Dialogue,
        HandlerOwner::Dialogue,
        HandlerOwner::Dialogue,
        HandlerOwner::Finale,
        HandlerOwner::Dialogue,
        HandlerOwner::Objects,
    ];
    for (i, owner) in owners.iter().enumerate() {
        assert_eq!(catalog.handlers[i].owner, *owner, "owner {i:02X}");
    }
}
#[test]
fn original_all_templates_decode_full_nibbles_and_fields_without_embedded_payload() {
    let b = rom();
    let c = SpawnCatalog::original(&b).unwrap();
    assert_eq!(c.templates.len(), TEMPLATE_COUNT);
    for (i, t) in c.templates.iter().enumerate() {
        let p = TEMPLATE_CPU + i * 4;
        assert_eq!(
            *t,
            Template {
                flags_and_local: bank_byte(&b, SOURCE_BANK, p),
                animation: bank_byte(&b, SOURCE_BANK, p + 1),
                handler: bank_byte(&b, SOURCE_BANK, p + 2),
                profile: bank_byte(&b, SOURCE_BANK, p + 3)
            }
        );
    }
}
#[test]
fn original_nine_stage_pointer_sets_decode_every_row_and_parallel_record_until_sentinel() {
    let b = rom();
    let c = SpawnCatalog::original(&b).unwrap();
    let row_counts = [7, 8, 6, 6, 6];
    let array_lengths = [181, 156, 126, 119, 182];
    for (stage, rows) in row_counts.iter().enumerate() {
        let d = c.source(StageId(stage as u8)).unwrap();
        assert_eq!(d.rows.len(), *rows);
        let high = pointer(&b, 0xb050, 0xb059, stage);
        let low = pointer(&b, 0xb062, 0xb06b, stage);
        let starts = pointer(&b, 0xb074, 0xb07d, stage);
        let ypos = pointer(&b, 0xb086, 0xb08f, stage);
        let templates = pointer(&b, 0xb098, 0xb0a1, stage);
        assert_eq!(low - high, array_lengths[stage]);
        for row in 0..*rows {
            let start = usize::from(bank_byte(&b, SOURCE_BANK, starts + row));
            let mut count = 0;
            for index in start..array_lengths[stage] {
                let hi = bank_byte(&b, SOURCE_BANK, high + index);
                if hi & ACTIVE != 0 {
                    break;
                }
                let actual = d.rows[row][count];
                assert_eq!(
                    actual,
                    SpawnRecord {
                        identity: index as u8,
                        world_x: u16::from(hi) << 8
                            | u16::from(bank_byte(&b, SOURCE_BANK, low + index)),
                        screen_y: bank_byte(&b, SOURCE_BANK, ypos + index),
                        template: usize::from(bank_byte(&b, SOURCE_BANK, templates + index))
                    }
                );
                count += 1;
            }
            assert_eq!(d.rows[row].len(), count);
        }
    }
}
#[test]
fn scene_source_five_through_eight_are_empty_and_ending_nine_has_no_spawn_adapter() {
    let c = SpawnCatalog::original(&rom()).unwrap();
    assert_eq!(c.source_levels.len(), SOURCE_STAGE_CONFIGS);
    for s in 5..SOURCE_STAGE_CONFIGS {
        let d = c.source(StageId(s as u8)).unwrap();
        assert!(d.rows.iter().all(Vec::is_empty));
        assert!(d.special_rows.iter().all(Vec::is_empty));
    }
    assert!(c.source(StageId(SOURCE_STAGE_CONFIGS as u8)).is_none());
}
#[test]
fn special_metadata_decodes_every_stage_room_page_sign_template_and_y() {
    let b = rom();
    let c = SpawnCatalog::original(&b).unwrap();
    let rows = [7, 8, 6, 6, 6];
    for (stage, count) in rows.iter().enumerate() {
        let d = c.source(StageId(stage as u8)).unwrap();
        assert_eq!(d.special_rows.len(), *count);
        let first = usize::from(bank_byte(&b, SOURCE_BANK, 0xb164 + stage));
        for row in 0..*count {
            let cursor = usize::from(bank_byte(&b, SOURCE_BANK, 0xb16d + first + row));
            let mut n = 0;
            for i in cursor..38 {
                let raw = bank_byte(&b, SOURCE_BANK, 0xb18f + i);
                if raw == NO_SOURCE_IDENTITY {
                    break;
                }
                assert_eq!(
                    d.special_rows[row][n],
                    SpecialRecord {
                        page: raw & 0x7f,
                        paired: raw & ACTIVE != 0,
                        template: usize::from(bank_byte(&b, SOURCE_BANK, 0xb1b5 + i)),
                        screen_y: bank_byte(&b, SOURCE_BANK, 0xb1db + i)
                    }
                );
                n += 1;
            }
            assert_eq!(d.special_rows[row].len(), n);
        }
    }
}
#[test]
fn all_thirtytwo_bank_six_contact_extents_and_motion_words_use_distinct_recipes() {
    let b = rom();
    let c = SpawnCatalog::original(&b).unwrap();
    for i in 0..32 {
        assert_eq!(
            c.extents[i],
            Extent {
                half_height: bank_byte(&b, SOURCE_BANK, 0x814a + i),
                half_width: bank_byte(&b, SOURCE_BANK, 0x816a + i)
            }
        );
        let v = u16::from_le_bytes([
            bank_byte(&b, SOURCE_BANK, 0x81a5 + i * 2),
            bank_byte(&b, SOURCE_BANK, 0x81a6 + i * 2),
        ]);
        let h = u16::from_le_bytes([
            bank_byte(&b, SOURCE_BANK, 0x81e5 + i * 2),
            bank_byte(&b, SOURCE_BANK, 0x81e6 + i * 2),
        ]);
        assert_eq!(c.vertical_presets[i], v as i16);
        assert_eq!(c.horizontal_presets[i], h);
    }
}
#[test]
fn bounded_original_decoder_rejects_missing_required_data_instead_of_panicking() {
    let b = rom();
    for end in [
        0,
        HEADER_BYTES,
        0x177af,
        0x1803c,
        0x1815a,
        0x181b5,
        0x1b174,
        0x1c00a,
    ] {
        assert!(
            SpawnCatalog::original(&b[..end]).is_err(),
            "truncation {end:X}"
        );
    }
}
#[test]
fn named_extensions_preserve_originals_allow_more_than_nine_levels_and_validate_atomically() {
    let mut c = SpawnCatalog::original(&rom()).unwrap();
    let originals = c.source_levels.clone();
    for n in 0..12 {
        let d = SpawnDefinition {
            id: AssetId(format!("authored-spawns-{n}")),
            rows: vec![vec![record(0, 0x200)]],
            special_rows: vec![vec![]],
        };
        c.add_named(d.clone()).unwrap();
        assert_eq!(c.level(&d.id), Some(&d));
    }
    assert_eq!(c.source_levels, originals);
    let before = c.clone();
    let duplicate = c.levels.last().unwrap().clone();
    assert!(c.add_named(duplicate).is_err());
    assert_eq!(c, before);
    let mut invalid = SpawnDefinition {
        id: AssetId("bad-template".into()),
        rows: vec![vec![record(0, 0)]],
        special_rows: vec![vec![]],
    };
    invalid.rows[0][0].template = TEMPLATE_COUNT;
    assert!(c.add_named(invalid).is_err());
    assert_eq!(c, before);
}
#[test]
fn both_allocation_entries_choose_highest_sign_clear_slot_without_mutation() {
    for reg in [AllocationRegister::X, AllocationRegister::Y] {
        let mut a = occupied();
        {
            let actor_value = 0x7f;
            a[14].set_state_code(actor_value);
        };
        {
            let actor_value = 0;
            a[3].set_state_code(actor_value);
        };
        let before = a;
        assert_eq!(allocate(&a, reg), Some(14));
        assert_eq!(a, before);
    }
}
#[test]
fn full_pool_never_allocates_player_even_when_player_inactive() {
    let mut a = occupied();
    {
        let actor_value = 0;
        a[PLAYER_SLOT].set_state_code(actor_value);
    };
    for r in [AllocationRegister::X, AllocationRegister::Y] {
        assert_eq!(allocate(&a, r), None);
    }
}
#[test]
fn empty_handler_is_still_a_dispatched_active_slot_and_slot_zero_is_excluded() {
    let mut a = pool();
    {
        let actor_value = ACTIVE;
        a[0].set_state_code(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[1].set_state_code(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[LAST_ACTOR].set_state_code(actor_value);
    };
    {
        let actor_value = 0x42;
        a[LAST_ACTOR].set_handler(actor_value);
    };
    let mut gravity = 0;
    let mut seen = vec![];
    let trace = dispatch(&mut a, &mut gravity, &mut |slot, handler, _, g| {
        seen.push((slot, handler));
        assert_eq!(*g, DEFAULT_GRAVITY)
    });
    assert_eq!(trace, vec![(1, 0), (LAST_ACTOR, 0x42)]);
    assert_eq!(seen, trace);
    assert_eq!(gravity, DEFAULT_GRAVITY);
}
#[test]
fn dispatch_observes_later_slot_spawn_removal_handler_and_gravity_writes_immediately() {
    let mut a = pool();
    {
        let actor_value = ACTIVE;
        a[1].set_state_code(actor_value);
    };
    {
        let actor_value = ACTIVE;
        a[2].set_state_code(actor_value);
    };
    {
        let actor_value = 2;
        a[2].set_handler(actor_value);
    };
    let mut gravity = 0;
    let visits = dispatch(&mut a, &mut gravity, &mut |slot, _, actors, g| {
        if slot == 1 {
            {
                let actor_value = 0;
                actors[2].set_state_code(actor_value);
            };
            {
                let actor_value = ACTIVE;
                actors[3].set_state_code(actor_value);
            };
            {
                let actor_value = 4;
                actors[3].set_handler(actor_value);
            };
            *g = 0x20;
        } else {
            assert_eq!(*g, 0x20);
            {
                let actor_value = 9;
                actors[1].set_handler(actor_value);
            };
        }
    });
    assert_eq!(visits, vec![(1, 0), (3, 4)]);
    assert_eq!(a[1].handler, 9);
}
#[test]
fn dispatch_does_not_revisit_lower_slots_or_follow_callback_slot_changes() {
    let mut a = pool();
    {
        let actor_value = ACTIVE;
        a[8].set_state_code(actor_value);
    };
    let mut g = 0;
    let visits = dispatch(&mut a, &mut g, &mut |_, _, a, _| {
        {
            let actor_value = ACTIVE;
            a[1].set_state_code(actor_value);
        };
        {
            let actor_value = 0;
            a[8].set_state_code(actor_value);
        };
    });
    assert_eq!(visits, vec![(8, 0)]);
    assert_eq!(a[1].state_code(), ACTIVE);
}
#[test]
fn suppression_sets_each_of_all_twohundredfiftysix_identity_bits_preserving_neighbors() {
    for id in 0..=u8::MAX {
        let mut b = [0x55; SUPPRESSION_BYTES];
        let old = b;
        suppress(&mut b, id);
        let index = usize::from(id) / 8;
        for i in 0..SUPPRESSION_BYTES {
            assert_eq!(
                b[i],
                if i == index {
                    old[i] | (1 << (id % 8))
                } else {
                    old[i]
                }
            );
        }
    }
}
#[test]
fn suppression_rules_decode_counts_plus_one_and_all_six_semantic_flag_sources() {
    let c = SpawnCatalog::original(&rom()).unwrap();
    let expected = [
        vec![(ProgressField::SourceA6, 1, 64)],
        vec![
            (ProgressField::SourceAe, 1, 90),
            (ProgressField::SourceA8, 4, 74),
            (ProgressField::SourceA8, 4, 109),
            (ProgressField::SourceA3, 2, 142),
        ],
        vec![
            (ProgressField::SourceA3, 2, 6),
            (ProgressField::StageBits, 0x80, 30),
            (ProgressField::StageBits, 0x20, 40),
            (ProgressField::SourceA6, 4, 84),
        ],
        vec![
            (ProgressField::SourceA8, 0x40, 79),
            (ProgressField::SourceA8, 0x40, 90),
            (ProgressField::SourceA6, 8, 77),
        ],
        vec![
            (ProgressField::SourceA1, 0x7f, 157),
            (ProgressField::SourceA3, 4, 7),
            (ProgressField::SourceA1, 0x7f, 30),
            (ProgressField::SourceA3, 4, 33),
            (ProgressField::StageBits, 0x40, 5),
            (ProgressField::SourceA6, 0x10, 27),
            (ProgressField::SourceA1, 0x80, 156),
        ],
    ];
    assert_eq!(c.suppression.len(), SUPPRESSION_STAGE_CONFIGS);
    for (stage, rules) in expected.iter().enumerate() {
        assert_eq!(
            c.suppression[stage],
            rules
                .iter()
                .map(|(field, mask, destination)| SuppressionRule {
                    field: *field,
                    mask: *mask,
                    destination: *destination
                })
                .collect::<Vec<_>>()
        );
    }
}
#[test]
fn rebuild_clears_stale_bits_then_ors_every_matching_destination_and_keeps_unmatched_clear() {
    let rules = [
        SuppressionRule {
            field: ProgressField::SourceA8,
            mask: 4,
            destination: 74,
        },
        SuppressionRule {
            field: ProgressField::SourceA8,
            mask: 4,
            destination: 109,
        },
        SuppressionRule {
            field: ProgressField::SourceA3,
            mask: 2,
            destination: 142,
        },
    ];
    let mut b = [0xff; SUPPRESSION_BYTES];
    rebuild_suppression(
        &mut b,
        &rules,
        SuppressionFlags {
            source_a8: 4,
            ..SuppressionFlags::default()
        },
    );
    let mut expected = bits();
    expected[9] = 4;
    expected[13] = 0x20;
    assert_eq!(b, expected);
    rebuild_suppression(&mut b, &rules, SuppressionFlags::default());
    assert_eq!(b, bits());
}
#[test]
fn removal_invalidates_identity_preserves_fields_and_releases_only_matching_occupied_latch() {
    for slot in [1, 8, LAST_ACTOR] {
        for matched in [false, true] {
            let mut a = occupied();
            a[slot].spawn_identity = Some(9);
            a[slot].animation = 7;
            {
                let actor_value = 13;
                a[slot].set_local_counter_code(actor_value);
            };
            let mut control = SpecialControl::Occupied(if matched { slot } else { 2 });
            remove(&mut a, slot, &mut control, true, 0);
            assert_eq!(a[slot].state_code(), 0);
            assert_eq!(a[slot].render, 0);
            assert_eq!(a[slot].spawn_identity, None);
            assert_eq!(a[slot].animation, 7);
            assert_eq!(a[slot].local_counter_code(), 13);
            assert_eq!(
                control,
                if matched {
                    SpecialControl::Cooldown(SPECIAL_SPAWN_RETRY)
                } else {
                    SpecialControl::Occupied(2)
                }
            );
        }
    }
}
#[test]
fn direct_release_stores_callers_accumulator_without_clearing_render_or_suppression() {
    let mut a = occupied();
    a[4].spawn_identity = Some(17);
    let mut s = SpecialControl::Cooldown(5);
    remove(&mut a, 4, &mut s, false, 0x20);
    assert_eq!(a[4].state_code(), 0x20);
    assert_eq!(a[4].render, VISIBLE);
    assert_eq!(a[4].spawn_identity, None);
    assert_eq!(s, SpecialControl::Cooldown(5));
}
#[test]
fn reset_window_stops_at_first_high_less_equal_camera_or_negative_sentinel_and_sets_both_cursors() {
    for (camera, expected) in [(0x0200, 3), (0x0300, 3), (0x0400, 2), (0x0100, 4)] {
        let w = reset_window(2, &[0, 0, 4, 2, 0xff], camera);
        assert_eq!(
            w,
            Window {
                room_start: 2,
                upper: expected,
                lower: expected,
                direction: FORWARD
            }
        );
    }
}
#[test]
fn suppressed_candidate_below_camera_duplicate_active_and_full_pool_leave_everything_unchanged() {
    for reason in 0..4 {
        let mut a = if reason == 3 { occupied() } else { pool() };
        let mut b = bits();
        let r = record(9, if reason == 1 { 0x01ff } else { 0x0200 });
        if reason == 0 {
            b[1] = 2;
        }
        if reason == 2 {
            a[4].spawn_identity = Some(9);
            {
                let actor_value = ACTIVE;
                a[4].set_state_code(actor_value);
            };
        }
        let before = a;
        let mut c = context();
        let old = c;
        assert_eq!(
            candidate(&mut a, &b, r, &[template()], &mut c),
            SpawnReport::default()
        );
        assert_eq!(a, before);
        assert_eq!(c, old);
    }
}
#[test]
fn duplicate_scan_checks_highest_matching_identity_only_even_if_lower_matching_slot_is_active() {
    let mut a = pool();
    a[14].spawn_identity = Some(9);
    {
        let actor_value = 0;
        a[14].set_state_code(actor_value);
    };
    a[4].spawn_identity = Some(9);
    {
        let actor_value = ACTIVE;
        a[4].set_state_code(actor_value);
    };
    let mut c = context();
    let out = candidate(&mut a, &bits(), record(9, 0x0200), &[template()], &mut c);
    assert_eq!(out.ordinary, vec![(9, LAST_ACTOR)]);
    assert_eq!(a[4].state_code(), ACTIVE);
}
#[test]
fn ordinary_spawn_splits_template_low_nibble_and_resets_only_source_initialized_fields() {
    let mut a = pool();
    a[LAST_ACTOR] = ActorContact {
        playback: crate::presentation::AnimationPlayback::from_codes(99, 99, 0),
        control: crate::actors::ActorControl::from_codes(0, 99, 99, 0),
        local_aux2: 99,
        sequence_cursor: 98,
        boss_protection: 97,
        hit_marker: 99,
        horizontal_magnitude: 0xbeef,
        vertical_velocity: -17,
        direction: 2,
        motion_flags: 3,
        ..ActorContact::default()
    };
    let mut c = context();
    let report = candidate(&mut a, &bits(), record(8, 0x0200), &[template()], &mut c);
    assert_eq!(report.ordinary, vec![(8, LAST_ACTOR)]);
    let v = a[LAST_ACTOR];
    // B11 independent source reconciliation: AF clear is06F0, not06D0.
    assert_eq!((v.local_aux2, v.sequence_cursor), (99, 98));
    assert_eq!(
        (v.state_code(), v.render, v.local_state_code()),
        (ACTIVE | WORLD_POSITIONED, ACTIVE | WORLD_POSITIONED, 3)
    );
    assert_eq!((v.animation, v.handler, v.profile), (7, 2, 5));
    assert_eq!(
        (
            v.hit_marker,
            v.local_counter_code(),
            v.boss_protection,
            v.playback.frame_code(),
            v.playback.tick_code()
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(
        (
            v.position.world_x,
            v.position.screen_y,
            v.spawn_y,
            v.spawn_identity
        ),
        (0x0200, 0x70, 0x70, Some(8))
    );
    assert_eq!(
        (
            v.horizontal_magnitude,
            v.vertical_velocity,
            v.direction,
            v.motion_flags
        ),
        (0xbeef, -17, 2, 3)
    );
}
#[test]
fn template_page_crossing_selectors_sixtyfour_and_last_decode_the_correct_four_bytes() {
    let mut templates = vec![template(); TEMPLATE_COUNT];
    templates[64].animation = 0x47;
    templates[TEMPLATE_COUNT - 1].animation = 0x71;
    for (index, animation) in [(64, 0x47), (TEMPLATE_COUNT - 1, 0x71)] {
        let mut a = pool();
        let mut c = context();
        let mut r = record(1, 0x0200);
        r.template = index;
        candidate(&mut a, &bits(), r, &templates, &mut c);
        assert_eq!(a[LAST_ACTOR].animation, animation);
    }
}
#[test]
fn persistent_negative_animation_normalizes_and_marks_counter_only_outside_split_transition() {
    for split in [0, 1] {
        for persistent in [false, true] {
            let mut a = pool();
            let mut t = template();
            t.animation = 0x91;
            if persistent {
                t.flags_and_local |= TERRAIN_REPLACEMENT;
            }
            let mut c = context();
            c.split_transition = split;
            candidate(&mut a, &bits(), record(1, 0x0200), &[t], &mut c);
            assert_eq!(
                a[LAST_ACTOR].animation,
                if split == 0 && persistent { 0x11 } else { 0x91 }
            );
            assert_eq!(
                a[LAST_ACTOR].local_counter_code(),
                if split == 0 && persistent { 0xff } else { 0 }
            );
        }
    }
}
#[test]
fn persistent_handlers_eight_and_ten_replace_terrain_and_bypass_progression_suppression() {
    for handler in [8, 10] {
        let mut a = pool();
        let t = Template {
            flags_and_local: ACTIVE | TERRAIN_REPLACEMENT,
            handler,
            ..template()
        };
        let mut c = context();
        c.stage = STAGE_ONE;
        c.stage_progress = 0x1f;
        c.late_mode = ACTIVE;
        c.late_route = 1;
        let out = candidate(&mut a, &bits(), record(1, 0x0200), &[t], &mut c);
        assert_eq!(out.terrain_replacements, vec![LAST_ACTOR]);
        assert_eq!(a[LAST_ACTOR].handler, handler);
        assert_eq!(c.boss_hit_points, 0);
    }
}
#[test]
fn split_transition_bypasses_terrain_replacement_but_not_final_override() {
    let mut a = pool();
    let t = Template {
        flags_and_local: ACTIVE | TERRAIN_REPLACEMENT,
        handler: 8,
        ..template()
    };
    let mut c = context();
    c.split_transition = 1;
    c.late_mode = ACTIVE;
    c.late_route = 1;
    let out = candidate(&mut a, &bits(), record(1, 0x0200), &[t], &mut c);
    assert!(out.terrain_replacements.is_empty());
    assert_eq!(a[LAST_ACTOR].handler, FINAL_HANDLER);
}
#[test]
fn negative_late_mode_seeds_hp_five_even_when_progression_gate_rejects_override() {
    let mut a = pool();
    let mut c = context();
    c.late_mode = ACTIVE;
    c.stage_progress = 0x1e;
    candidate(&mut a, &bits(), record(1, 0x0200), &[template()], &mut c);
    assert_eq!(c.boss_hit_points, FINAL_HIT_POINTS);
    assert_eq!(a[LAST_ACTOR].handler, template().handler);
}
#[test]
fn final_override_accepts_nonzero_route_or_all_five_progress_bits_and_preserves_render_flags() {
    for (route, progress) in [(1, 0), (0, 0x1f), (0, 0xff)] {
        let mut a = pool();
        let mut c = context();
        c.late_mode = ACTIVE;
        c.late_route = route;
        c.stage_progress = progress;
        candidate(&mut a, &bits(), record(1, 0x0200), &[template()], &mut c);
        assert_eq!(
            (
                a[LAST_ACTOR].handler,
                a[LAST_ACTOR].animation,
                a[LAST_ACTOR].profile
            ),
            (FINAL_HANDLER, FINAL_ANIMATION, 0)
        );
        assert_eq!(
            a[LAST_ACTOR].state_code(),
            ACTIVE | WORLD_POSITIONED | TERRAIN_REPLACEMENT
        );
        assert_eq!(a[LAST_ACTOR].render, ACTIVE | WORLD_POSITIONED);
        assert_eq!(c.boss_hit_points, FINAL_HIT_POINTS);
    }
}
#[test]
fn nonnegative_late_mode_neither_overrides_nor_reseeds_hp() {
    for late in [0, 1, 0x7f] {
        let mut a = pool();
        let mut c = context();
        c.late_mode = late;
        c.late_route = 1;
        c.stage_progress = 0x1f;
        c.boss_hit_points = 3;
        candidate(&mut a, &bits(), record(1, 0x0200), &[template()], &mut c);
        assert_eq!(a[LAST_ACTOR].handler, template().handler);
        assert_eq!(c.boss_hit_points, 3);
    }
}
#[test]
fn stage_one_suppression_deactivates_exact_handler_animation_combinations_but_leaves_render_identity()
 {
    for (stage, progress, handler, animation, clear) in [
        (1, 2, 0x14, 7, true),
        (1, 2, 0x15, 7, true),
        (1, 2, 8, 0x1b, true),
        (1, 2, 8, 0x1a, false),
        (1, 0, 0x14, 7, false),
        (0, 2, 0x14, 7, false),
        (1, 2, 9, 0x1b, false),
    ] {
        let mut a = pool();
        let mut c = context();
        c.stage = stage;
        c.stage_progress = progress;
        let t = Template {
            handler,
            animation,
            ..template()
        };
        candidate(&mut a, &bits(), record(1, 0x0200), &[t], &mut c);
        assert_eq!(
            a[LAST_ACTOR].state_code(),
            if clear { 0 } else { ACTIVE | WORLD_POSITIONED }
        );
        assert_eq!(a[LAST_ACTOR].render, ACTIVE | WORLD_POSITIONED);
        assert_eq!(a[LAST_ACTOR].spawn_identity, Some(1));
    }
}
#[test]
fn forward_window_includes_upper_edge_and_advances_lower_past_camera_equality() {
    let records = [
        record(0, 0x01ff),
        record(1, 0x0200),
        record(2, 0x02ff),
        record(3, 0x0300),
    ];
    let mut a = pool();
    let mut c = context();
    let mut w = Window {
        direction: FORWARD,
        ..Window::default()
    };
    let out = scan_window(&mut a, &bits(), &records, &[template()], &mut w, &mut c);
    assert_eq!(out.ordinary, vec![(1, 15), (2, 14)]);
    assert_eq!((w.upper, w.lower), (3, 2));
}
#[test]
fn forward_scan_consumes_suppressed_duplicate_and_full_pool_entries_without_retry_until_reverse() {
    let records = [record(0, 0x0200), record(1, 0x0201), record(2, 0x0300)];
    let mut a = occupied();
    let mut c = context();
    let mut w = Window {
        direction: FORWARD,
        ..Window::default()
    };
    let out = scan_window(&mut a, &bits(), &records, &[template()], &mut w, &mut c);
    assert!(out.ordinary.is_empty());
    assert_eq!((w.upper, w.lower), (2, 1));
    {
        let actor_value = 0;
        a[15].set_state_code(actor_value);
    };
    assert!(
        scan_window(&mut a, &bits(), &records, &[template()], &mut w, &mut c)
            .ordinary
            .is_empty()
    );
}
#[test]
fn backward_scan_includes_camera_equality_and_retracts_upper_at_inclusive_high_edge() {
    let records = [
        record(0, 0x01ff),
        record(1, 0x0200),
        record(2, 0x02ff),
        record(3, 0x0300),
    ];
    let mut a = pool();
    let mut c = context();
    c.direction = BACKWARD;
    let mut w = Window {
        room_start: 0,
        lower: 3,
        upper: 4,
        direction: BACKWARD,
    };
    let out = scan_window(&mut a, &bits(), &records, &[template()], &mut w, &mut c);
    assert_eq!(out.ordinary, vec![(2, 15), (1, 14)]);
    assert_eq!((w.lower, w.upper), (1, 2));
}
#[test]
fn cursor_directions_mask_low_two_bits_with_forward_priority_and_zero_means_no_ordinary_work() {
    for flags in [0, 4, 0x80] {
        let mut a = pool();
        let mut c = context();
        let mut w = Window {
            direction: flags,
            ..Window::default()
        };
        let old = w;
        assert!(
            scan_window(
                &mut a,
                &bits(),
                &[record(0, 0x0200), record(1, 0x0300)],
                &[template()],
                &mut w,
                &mut c
            )
            .ordinary
            .is_empty()
        );
        assert_eq!(w, old);
    }
    let mut a = pool();
    let mut c = context();
    let mut w = Window {
        direction: 3,
        ..Window::default()
    };
    assert_eq!(
        scan_window(
            &mut a,
            &bits(),
            &[record(0, 0x0200), record(1, 0x0300)],
            &[template()],
            &mut w,
            &mut c
        )
        .ordinary,
        vec![(0, LAST_ACTOR)]
    );
}
#[test]
fn backward_cursor_never_reads_before_room_start() {
    let mut a = pool();
    let mut c = context();
    let mut w = Window {
        room_start: 2,
        lower: 2,
        upper: 2,
        direction: BACKWARD,
    };
    let old = w;
    assert!(
        scan_window(
            &mut a,
            &bits(),
            &[record(0, 0), record(1, 0), record(2, 0xffff)],
            &[template()],
            &mut w,
            &mut c
        )
        .ordinary
        .is_empty()
    );
    assert_eq!(w, old);
}
#[test]
fn forward_camera_interval_carries_low_ff_and_wraps_sixteenbit_world() {
    let mut a = pool();
    let mut c = context();
    c.camera = 0x02f8;
    let mut w = Window {
        direction: FORWARD,
        ..Window::default()
    };
    let records = [
        record(0, 0x02f7),
        record(1, 0x02f8),
        record(2, 0x03f7),
        record(3, 0x03f8),
    ];
    assert_eq!(
        scan_window(&mut a, &bits(), &records, &[template()], &mut w, &mut c).ordinary,
        vec![(1, 15), (2, 14)]
    );
    assert_eq!((w.upper, w.lower), (3, 2));
}
#[test]
fn special_control_separates_zero_positive_cooldown_and_negative_slot_latch() {
    assert_eq!(SpecialControl::decode(0), SpecialControl::Ready);
    for ticks in 1..0x80 {
        assert_eq!(
            SpecialControl::decode(ticks),
            SpecialControl::Cooldown(ticks)
        );
        assert_eq!(SpecialControl::Cooldown(ticks).encode(), ticks);
    }
    for slot in 0..0x80 {
        assert_eq!(
            SpecialControl::decode(ACTIVE | slot as u8),
            SpecialControl::Occupied(slot)
        );
        assert_eq!(SpecialControl::Occupied(slot).encode(), ACTIVE | slot as u8);
    }
}
#[test]
fn special_late_gate_precedes_cooldown_and_positive_one_expires_without_spawning_same_call() {
    for late in [0, ACTIVE] {
        let mut a = pool();
        let mut c = context();
        c.late_mode = late;
        c.special = SpecialControl::Cooldown(1);
        let out = special_spawn(&mut a, &[vec![special(2, false)]], &[template()], &mut c);
        assert!(out.special.is_empty());
        assert_eq!(
            c.special,
            if late == 0 {
                SpecialControl::Ready
            } else {
                SpecialControl::Cooldown(1)
            }
        );
    }
}
#[test]
fn occupied_special_latch_freezes_scanning_and_unmatched_pages_leave_ready() {
    let mut a = pool();
    let before = a;
    let mut c = context();
    c.special = SpecialControl::Occupied(3);
    assert!(
        special_spawn(&mut a, &[vec![special(2, false)]], &[template()], &mut c)
            .special
            .is_empty()
    );
    assert_eq!(a, before);
    c.special = SpecialControl::Ready;
    assert!(
        special_spawn(&mut a, &[vec![special(7, false)]], &[template()], &mut c)
            .special
            .is_empty()
    );
    assert_eq!(c.special, SpecialControl::Ready);
}
#[test]
fn special_forward_and_backward_edges_use_plus_twohundredfortyeight_and_plus_eight_with_page_carry()
{
    for (direction, camera, page, x) in [
        (FORWARD, 0x0200, 2, 0x02f8),
        (BACKWARD, 0x02fc, 3, 0x0304),
        (3, 0x02fc, 3, 0x0304),
        (0, 0x0200, 2, 0x02f8),
    ] {
        let mut a = pool();
        let mut c = context();
        c.direction = direction;
        c.camera = camera;
        let out = special_spawn(&mut a, &[vec![special(page, false)]], &[template()], &mut c);
        assert_eq!(out.special, vec![LAST_ACTOR]);
        assert_eq!(a[LAST_ACTOR].position.world_x, x);
    }
}
#[test]
fn special_initialization_copies_entire_flag_byte_zeros_local_state_and_preserves_auxiliary_and_spawn_y()
 {
    let mut a = pool();
    a[LAST_ACTOR].local_aux2 = 77;
    a[LAST_ACTOR].sequence_cursor = 76;
    a[LAST_ACTOR].boss_protection = 75;
    a[LAST_ACTOR].spawn_y = 88;
    {
        let animation_value = 9;
        a[LAST_ACTOR].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 8;
        a[LAST_ACTOR].playback.set_tick_code(animation_value);
    };
    let mut c = context();
    c.camera = 0x0108;
    let out = special_spawn(&mut a, &[vec![special(2, false)]], &[template()], &mut c);
    assert_eq!(out.special, vec![LAST_ACTOR]);
    let actor = a[LAST_ACTOR];
    assert_eq!(
        (actor.state_code(), actor.render),
        (template().flags_and_local, template().flags_and_local)
    );
    assert_eq!(
        (
            actor.local_state_code(),
            actor.local_counter_code(),
            actor.hit_marker,
            actor.playback.frame_code(),
            actor.playback.tick_code()
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(
        (
            actor.local_aux2,
            actor.sequence_cursor,
            actor.boss_protection,
            actor.spawn_y
        ),
        (77, 76, 75, 88)
    );
    assert_eq!(
        (
            actor.position.world_x,
            actor.position.screen_y,
            actor.spawn_identity
        ),
        (0x0200, 0x60, None)
    );
    assert_eq!(c.special, SpecialControl::Occupied(LAST_ACTOR));
}
#[test]
fn special_uses_first_matching_page_and_single_full_pool_keeps_player_unchanged() {
    let mut a = pool();
    let mut c = context();
    c.camera = 0x0108;
    let mut second = special(2, false);
    second.screen_y = 0x99;
    special_spawn(
        &mut a,
        &[vec![special(2, false), second]],
        &[template()],
        &mut c,
    );
    assert_eq!(a[LAST_ACTOR].position.screen_y, 0x60);
    a = occupied();
    let before = a;
    c.special = SpecialControl::Ready;
    assert!(
        special_spawn(&mut a, &[vec![special(2, false)]], &[template()], &mut c)
            .special
            .is_empty()
    );
    assert_eq!(a, before);
    assert_eq!(c.special, SpecialControl::Ready);
}
#[test]
fn paired_special_allocates_highest_two_slots_forces_first_y_and_toggles_only_low_x_nibble_plane() {
    let mut a = pool();
    let mut c = context();
    c.camera = 0x0108;
    let out = special_spawn(&mut a, &[vec![special(2, true)]], &[template()], &mut c);
    assert_eq!(out.special, vec![15, 14]);
    assert_eq!(
        (a[15].position.world_x, a[15].position.screen_y),
        (0x0200, PAIRED_FIRST_Y)
    );
    assert_eq!(
        (a[14].position.world_x, a[14].position.screen_y),
        (0x02f0, 0x60)
    );
    assert_eq!(c.special, SpecialControl::Occupied(14));
}
#[test]
fn paired_one_free_slot_keeps_first_actor_and_first_occupied_latch_after_companion_failure() {
    let mut a = occupied();
    {
        let actor_value = 0;
        a[12].set_state_code(actor_value);
    };
    let mut c = context();
    c.camera = 0x0108;
    let out = special_spawn(&mut a, &[vec![special(2, true)]], &[template()], &mut c);
    assert_eq!(out.special, vec![12]);
    assert_eq!(
        (a[12].position.world_x, a[12].position.screen_y),
        (0x0200, PAIRED_FIRST_Y)
    );
    assert_eq!(c.special, SpecialControl::Occupied(12));
}
#[test]
fn paired_full_pool_source_failure_continuation_writes_player_y_c0_then_fails_second_allocation() {
    let mut a = occupied();
    a[0].position.screen_y = 0x50;
    let before = a;
    let mut c = context();
    c.camera = 0x0108;
    let out = special_spawn(&mut a, &[vec![special(2, true)]], &[template()], &mut c);
    assert!(out.special.is_empty());
    assert_eq!(a[0].position.screen_y, PAIRED_FIRST_Y);
    for slot in 1..ACTOR_SLOTS {
        assert_eq!(a[slot], before[slot]);
    }
    assert_eq!(c.special, SpecialControl::Ready);
}
#[test]
fn bank_six_overlap_requires_source_and_target_visible_but_not_active_or_profile_enable() {
    let mut a = contact_pool();
    let out = actor_overlap(&mut a, 1, &extents());
    assert_eq!(out, Some(LAST_ACTOR));
    assert_eq!(a[LAST_ACTOR].hit_marker, 1);
    assert_eq!(a[1].animation, MARKED_CONTACT_ANIMATION);
    a = contact_pool();
    a[1].render = 0;
    let before = a;
    assert_eq!(actor_overlap(&mut a, 1, &extents()), None);
    assert_eq!(a, before);
    a = contact_pool();
    a[LAST_ACTOR].render = 0;
    assert_eq!(actor_overlap(&mut a, 1, &extents()), None);
}
#[test]
fn bank_six_overlap_uses_source_profile_masked_low_seven_not_targets_profile() {
    let mut a = contact_pool();
    a[1].profile = 0x81;
    a[LAST_ACTOR].profile = 31;
    let mut e = extents();
    e[31] = Extent {
        half_width: 0,
        half_height: 0,
    };
    assert_eq!(actor_overlap(&mut a, 1, &e), Some(LAST_ACTOR));
}
#[test]
fn overlap_height_is_strict_width_inclusive_and_scan_excludes_player_self_chooses_highest() {
    for (dy, dx, hit) in [(7, 8, true), (8, 8, false), (7, 9, false), (0, 0, true)] {
        let mut a = contact_pool();
        a[LAST_ACTOR].position.screen_y = 0x80 + dy;
        a[LAST_ACTOR].position.world_x = 0x0200 + dx;
        assert_eq!(actor_overlap(&mut a, 1, &extents()).is_some(), hit);
    }
    let mut a = contact_pool();
    a[0] = a[LAST_ACTOR];
    a[2] = a[LAST_ACTOR];
    assert_eq!(actor_overlap(&mut a, 1, &extents()), Some(LAST_ACTOR));
    a[LAST_ACTOR].render = 0;
    a[2].render = 0;
    assert_eq!(actor_overlap(&mut a, 1, &extents()), None);
}
#[test]
fn overlap_target_exceptions_eight_nine_twentyfive_skip_hit_marker_choose_animation_2a() {
    for handler in [8, 9, 0x25] {
        let mut a = contact_pool();
        {
            let actor_value = handler;
            a[LAST_ACTOR].set_handler(actor_value);
        };
        a[LAST_ACTOR].hit_marker = 0xff;
        {
            let animation_value = 7;
            a[1].playback.set_frame_code(animation_value);
        };
        {
            let animation_value = 9;
            a[1].playback.set_tick_code(animation_value);
        };
        assert_eq!(actor_overlap(&mut a, 1, &extents()), Some(LAST_ACTOR));
        assert_eq!(a[LAST_ACTOR].hit_marker, 0xff);
        assert_eq!(
            (
                a[1].animation,
                a[1].playback.frame_code(),
                a[1].playback.tick_code()
            ),
            (CONTACT_ANIMATION, 0, 0)
        );
    }
}
#[test]
fn ordinary_overlap_hit_wraps_marker_and_preserved_animation_1b_keeps_clocks() {
    let mut a = contact_pool();
    a[LAST_ACTOR].hit_marker = 0xff;
    a[1].animation = PRESERVED_ANIMATION;
    {
        let animation_value = 7;
        a[1].playback.set_frame_code(animation_value);
    };
    {
        let animation_value = 9;
        a[1].playback.set_tick_code(animation_value);
    };
    assert_eq!(actor_overlap(&mut a, 1, &extents()), Some(LAST_ACTOR));
    assert_eq!(a[LAST_ACTOR].hit_marker, 0);
    assert_eq!(
        (
            a[1].animation,
            a[1].playback.frame_code(),
            a[1].playback.tick_code()
        ),
        (PRESERVED_ANIMATION, 7, 9)
    );
}
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
fn overlap_byte_y_absolute_does_not_wrap_to_shortest_distance() {
    let mut a = contact_pool();
    a[1].position.screen_y = 0;
    a[LAST_ACTOR].position.screen_y = 0xff;
    assert_eq!(actor_overlap(&mut a, 1, &extents()), None);
}
#[test]
fn each_motion_preset_copies_both_words_without_sign_normalization_or_unrelated_changes() {
    let c = SpawnCatalog::original(&rom()).unwrap();
    for i in 0..32 {
        let mut actor = ActorContact {
            control: crate::actors::ActorControl::from_codes(ACTIVE, 0, 0, 0),
            animation: 7,
            ..ActorContact::default()
        };
        motion_preset(&mut actor, &c, i);
        assert_eq!(actor.vertical_velocity, c.vertical_presets[i]);
        assert_eq!(actor.horizontal_magnitude, c.horizontal_presets[i]);
        assert_eq!((actor.state_code(), actor.animation), (ACTIVE, 7));
    }
}
#[test]
fn face_player_uses_unsigned_sixteenbit_compare_with_equality_direction_two() {
    for (player, actor, direction) in [
        (0x0200, 0x01ff, 1),
        (0x0200, 0x0200, 2),
        (0x0200, 0x0201, 2),
        (0xffff, 0, 1),
        (0, 0xffff, 2),
    ] {
        let mut a = pool();
        a[0].position.world_x = player;
        a[1].position.world_x = actor;
        face_player(&mut a, 1);
        assert_eq!(a[1].direction, direction);
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

#[test]
fn integrated_window_allocates_special_before_ordinary_from_the_same_canonical_pool() {
    let mut a = pool();
    let mut c = context();
    let mut w = Window {
        direction: FORWARD,
        ..Window::default()
    };
    let d = SpawnDefinition {
        id: AssetId("ordering-fixture".into()),
        rows: vec![vec![record(0, 0x0200), record(1, 0x0300)]],
        special_rows: vec![vec![special(2, false)]],
    };
    let out = update_window(&mut a, &bits(), &d, &[template()], &mut w, &mut c);
    assert_eq!(out.special, vec![15]);
    assert_eq!(out.ordinary, vec![(0, 14)]);
    assert_eq!(a[15].spawn_identity, None);
    assert_eq!(a[14].spawn_identity, Some(0));
    assert_eq!(c.special, SpecialControl::Occupied(15));
}
#[test]
fn integrated_direction_zero_still_runs_special_cooldown_before_returning_without_cursor_work() {
    let mut a = pool();
    let mut c = context();
    c.special = SpecialControl::Cooldown(2);
    c.direction = 0;
    let mut w = Window::default();
    let before = w;
    let d = SpawnDefinition {
        id: AssetId("stationary-fixture".into()),
        rows: vec![vec![record(0, 0x0200)]],
        special_rows: vec![vec![special(2, false)]],
    };
    let out = update_window(&mut a, &bits(), &d, &[template()], &mut w, &mut c);
    assert_eq!(out, SpawnReport::default());
    assert_eq!(c.special, SpecialControl::Cooldown(1));
    assert_eq!(w, before);
}
#[test]
fn empty_native_rows_have_no_candidate_or_cursor_reads_and_wrapped_high_interval_does_not_become_saturating()
 {
    let mut a = pool();
    let mut c = context();
    let mut w = Window {
        direction: FORWARD,
        ..Window::default()
    };
    assert_eq!(
        scan_window(&mut a, &bits(), &[], &[template()], &mut w, &mut c),
        SpawnReport::default()
    );
    assert_eq!((w.upper, w.lower), (0, 0));
    c.camera = 0xfffe;
    let records = [record(0, 0xfffe), record(1, 0xffff)];
    assert!(
        scan_window(&mut a, &bits(), &records, &[template()], &mut w, &mut c)
            .ordinary
            .is_empty()
    );
    assert_eq!((w.upper, w.lower), (0, 1));
}

#[test]
fn dialogue_additive_suppression_entry_preserves_every_existing_byte_and_adds_selected_rules() {
    // ED49 JSR B76A begins after B760's full32byte clear. Include high identity
    // bits and disabled rules so calling rebuild_suppression cannot pass this.
    let mut bits = [0x25; SUPPRESSION_BYTES];
    let rules = [
        SuppressionRule {
            field: ProgressField::SourceA1,
            mask: 0x80,
            destination: 0xff,
        },
        SuppressionRule {
            field: ProgressField::SourceA8,
            mask: 0x40,
            destination: 0x13,
        },
        SuppressionRule {
            field: ProgressField::SourceA3,
            mask: 2,
            destination: 0x09,
        },
    ];
    add_suppression(
        &mut bits,
        &rules,
        SuppressionFlags {
            source_a1: 0x80,
            source_a8: 0x40,
            ..SuppressionFlags::default()
        },
    );
    let mut expected = [0x25; SUPPRESSION_BYTES];
    expected[31] = 0x25 | 0x80;
    expected[2] = 0x25 | 8;
    assert_eq!(bits, expected);
}
#[test]
fn dialogue_additive_suppression_disabled_rules_do_not_clear_existing_low_or_high_bits() {
    let mut bits = [0xff; SUPPRESSION_BYTES];
    add_suppression(
        &mut bits,
        &[SuppressionRule {
            field: ProgressField::StageBits,
            mask: 0x1f,
            destination: 0,
        }],
        SuppressionFlags::default(),
    );
    assert_eq!(bits, [0xff; SUPPRESSION_BYTES]);
}

#[test]
fn converted_control_roundtrips_source_fields_and_handler_boundaries() {
    // B10 partial writes and B08 reaction preserve fields not explicitly reset.
    // The native family payload must be their single lossless owner.
    for state in 0..=u8::MAX {
        for delay in [0, 1, 0x7f, 0xff] {
            for cursor in [0, 13, 0xff] {
                let mut actor = ActorContact {
                    control: ActorControl::from_codes(state, delay, cursor, 0x19),
                    handler: 0x19,
                    ..Default::default()
                };
                assert_eq!(actor.control.codes(), (state, delay, cursor));
                actor.set_handler(0x0b);
                assert_eq!(actor.control.codes(), (state, delay, cursor));
                actor.set_handler(0x19);
                assert_eq!(actor.control.codes(), (state, delay, cursor));
                actor.set_state_code(0);
                assert_eq!(actor.control.codes(), (0, delay, cursor));
                actor.set_local_state_code(23);
                assert_eq!(actor.control.codes(), (0, 23, cursor));
                actor.set_local_counter_code(7);
                assert_eq!(actor.control.codes(), (0, 23, 7));
            }
        }
    }
}

#[test]
fn native_ambush_payload_survives_reaction_and_partial_slot_reinitialization() {
    let mut actor = ActorContact {
        control: ActorControl::from_codes(0xb2, 3, 13, 0x19),
        handler: 0x19,
        ..Default::default()
    };
    let behavior = actor.ambush_mut();
    behavior.phase = AmbushPhase::Closing;
    behavior.delay_remaining = 120;
    assert_eq!(actor.control.codes(), (0xb3, 120, 13));
    actor.set_handler(0x0b);
    assert_eq!(actor.control.codes(), (0xb3, 120, 13));
    actor.set_state_code(0xa0);
    actor.set_local_state_code(0);
    actor.set_handler(0x19);
    assert_eq!(actor.control.codes(), (0xa0, 0, 13));
    assert_eq!(actor.ambush_mut().phase, AmbushPhase::WaitingForPlayer);
    actor.advance_ambush();
    assert_eq!(actor.control.codes(), (0xa1, 0, 13));
}
