//! Original immutable actor recipe decoding. Physical offsets occur only here;
//! native dispatch uses family ownership and source selectors, never ROM code.
use super::*;
use anyhow::ensure;
const BANK_PHYSICAL: usize = 0x18010;
const SWITCH_CPU: usize = 0x8000;
const HANDLERS: [(&str, HandlerOwner); HANDLER_COUNT] = [
    ("EmptyActorHandler", HandlerOwner::Lifecycle),
    ("Actor8225VisibilityAndHit", HandlerOwner::Enemies),
    ("Bank06_WalkingBounceActor", HandlerOwner::Enemies),
    ("Bank06_ProjectileThrower", HandlerOwner::Enemies),
    ("Bank06_ThrownProjectile", HandlerOwner::Enemies),
    ("Actor8491AnimationGate", HandlerOwner::Enemies),
    ("Bank06_ProximityRisingActor", HandlerOwner::Enemies),
    ("Bank06_PlayerContactAnimationActor", HandlerOwner::Enemies),
    ("Actor8668Cull", HandlerOwner::Objects),
    ("Bank06_UpdateMovingTerrainObject", HandlerOwner::Objects),
    ("Bank06_ProximityFallingObject", HandlerOwner::Objects),
    ("ActorLaunchedMotion", HandlerOwner::Objects),
    ("Bank06_TriggeredWalkingPlatform", HandlerOwner::Platforms),
    ("Bank06_PresetOscillatingActor", HandlerOwner::Platforms),
    ("Bank06_RideablePresetActor", HandlerOwner::Platforms),
    ("Bank06_VerticalCarryingPlatform", HandlerOwner::Platforms),
    ("Actor9313Wait", HandlerOwner::Enemies),
    ("Bank06_DelayedPresetFlyingActor", HandlerOwner::Enemies),
    ("Bank06_PairedActorController", HandlerOwner::Enemies),
    ("Actor8A59AnimationGate", HandlerOwner::Objects),
    ("Actor8C45ContactEntry", HandlerOwner::Objects),
    ("Actor8D19ContactProfile", HandlerOwner::Objects),
    ("Bank06_RevealTransformedObject", HandlerOwner::Objects),
    ("Bank06_SlidingTransformedObject", HandlerOwner::Objects),
    ("Bank06_LedgeAwareWalker", HandlerOwner::Enemies),
    ("Bank06_FivePhaseAmbushActor", HandlerOwner::Enemies),
    ("Bank06_ProximityEruptingActor", HandlerOwner::Enemies),
    ("Bank06_RepeatingJumpActor", HandlerOwner::Enemies),
    ("Actor9B10HitAnimation", HandlerOwner::Enemies),
    ("Bank06_PushingContactActor", HandlerOwner::Platforms),
    ("Bank06_FastLedgeAwareWalker", HandlerOwner::Enemies),
    ("Bank06_TimedHorizontalPatrol", HandlerOwner::Enemies),
    ("Actor9E51Initialize", HandlerOwner::Enemies),
    ("Bank06_PlayerCarryingActor", HandlerOwner::Platforms),
    ("Bank06_TriggeredJumpingPatrol", HandlerOwner::Enemies),
    ("Bank06_LeftwardWalkingActor", HandlerOwner::Enemies),
    (
        "Bank06_PlayerExternalMotionTrigger",
        HandlerOwner::Platforms,
    ),
    ("ActorA065HitGate", HandlerOwner::Enemies),
    ("Bank06_FivePhaseAttackActor", HandlerOwner::Bosses),
    ("Bank06_UpdateAttackProjectile", HandlerOwner::Bosses),
    ("Bank06_PerimeterPathActor", HandlerOwner::Bosses),
    ("Bank06_CenterReturningActor", HandlerOwner::Bosses),
    ("Bank06_BouncingWaveActor", HandlerOwner::Bosses),
    ("Bank06_UpdateWaveChild", HandlerOwner::Bosses),
    ("Bank06_FourItemDispenser", HandlerOwner::Objects),
    ("Bank06_UpdateDispensedItem", HandlerOwner::Objects),
    ("Bank06_JumpingImpactActor", HandlerOwner::Bosses),
    ("Bank06_ProgressionContactSelector", HandlerOwner::Dialogue),
    ("Bank06_TimedRewardStealingPatrol", HandlerOwner::Enemies),
    ("Bank06_TimedRewardStealingPatrol", HandlerOwner::Enemies),
    ("Bank06_JumpingRewardStealer", HandlerOwner::Enemies),
    ("Bank06_HitOnlyReactionActor", HandlerOwner::Enemies),
    ("Bank06_PlayerLaunchContactActor", HandlerOwner::Enemies),
    ("UpdateDialogueContactActor", HandlerOwner::Dialogue),
    ("UpdatePlayerLinkedDeparture", HandlerOwner::Scenes),
    ("UpdateTimedDepartureSpawner", HandlerOwner::Scenes),
    ("UpdatePlayerLiftActor", HandlerOwner::Platforms),
    ("UpdateFourPhaseMovingPlatform", HandlerOwner::Platforms),
    ("UpdateFinalBoss", HandlerOwner::Finale),
    ("FinalBossDefeatCountdown", HandlerOwner::Finale),
    ("UpdateRaceClimber", HandlerOwner::Finale),
    ("UpdateHitActivatedDialogueActor", HandlerOwner::Dialogue),
    ("UpdateProgressionEscort", HandlerOwner::Dialogue),
    ("UpdateRightwardTransitionActor", HandlerOwner::Dialogue),
    ("UpdateFinalRaceGoal", HandlerOwner::Finale),
    ("UpdateConditionalContactMarker", HandlerOwner::Dialogue),
    ("Bank06_SwingReleasedCollectible", HandlerOwner::Objects),
];
pub(super) fn original(rom: &[u8]) -> Result<SpawnCatalog> {
    const CARTRIDGE_BYTES: usize = 0x20010;
    const STAGE_ROWS: [usize; SOURCE_STAGE_CONFIGS] = [7, 8, 6, 6, 6, 1, 1, 1, 1];
    const MAIN_POINTERS: [(usize, usize); 5] = [
        (0xb050, 0xb059),
        (0xb062, 0xb06b),
        (0xb074, 0xb07d),
        (0xb086, 0xb08f),
        (0xb098, 0xb0a1),
    ];
    const SPECIAL_STAGE_OFFSETS: usize = 0xb164;
    const SPECIAL_ROOM_OFFSETS: usize = 0xb16d;
    const SPECIAL_PAGES: usize = 0xb18f;
    const SPECIAL_TEMPLATES: usize = 0xb1b5;
    const SPECIAL_Y: usize = 0xb1db;
    const SPECIAL_RECORDS: usize = 38;
    const TEMPLATES: usize = 0xbe16;
    const HEIGHTS: usize = 0x814a;
    const WIDTHS: usize = 0x816a;
    const VERTICAL_PRESETS: usize = 0x81a5;
    const HORIZONTAL_PRESETS: usize = 0x81e5;
    const SUPPRESSION_BANK_BIAS: usize = 0xc010;
    const SUPPRESSION_OFFSETS: usize = 0xb79f;
    const SUPPRESSION_ADDRESSES: usize = 0xb7a4;
    const SUPPRESSION_MASKS: usize = 0xb7bc;
    const SUPPRESSION_DESTINATIONS: usize = 0xb7d4;
    ensure!(
        rom.len() >= CARTRIDGE_BYTES,
        "truncated source actor cartridge"
    );
    let byte = |cpu: usize| rom[BANK_PHYSICAL + cpu - SWITCH_CPU];
    let pointer = |low: usize, high: usize, index: usize| {
        usize::from(byte(low + index)) | usize::from(byte(high + index)) << 8
    };
    let word = |cpu: usize| u16::from_le_bytes([byte(cpu), byte(cpu + 1)]);
    let handlers = HANDLERS
        .iter()
        .map(|&(label, owner)| HandlerEntry {
            label: label.to_owned(),
            owner,
        })
        .collect();
    let templates = (0..TEMPLATE_COUNT)
        .map(|index| {
            let start = TEMPLATES + index * 4;
            Template {
                flags_and_local: byte(start),
                animation: byte(start + 1),
                handler: byte(start + 2),
                profile: byte(start + 3),
            }
        })
        .collect::<Vec<_>>();
    let mut levels = Vec::new();
    let mut source_levels = Vec::new();
    for (stage, &row_count) in STAGE_ROWS.iter().enumerate() {
        let pointers = MAIN_POINTERS.map(|(low, high)| pointer(low, high, stage));
        let mut rows = Vec::new();
        let mut special_rows = Vec::new();
        for row in 0..row_count {
            let first = usize::from(byte(pointers[2] + row));
            let mut records = Vec::new();
            if stage < SUPPRESSION_STAGE_CONFIGS {
                let mut terminated = false;
                for index in first..256 {
                    let high = byte(pointers[0] + index);
                    if high & ACTIVE != 0 {
                        terminated = true;
                        break;
                    }
                    let template = usize::from(byte(pointers[4] + index));
                    ensure!(template < TEMPLATE_COUNT, "source spawn template");
                    records.push(SpawnRecord {
                        identity: index as u8,
                        world_x: u16::from(high) << 8 | u16::from(byte(pointers[1] + index)),
                        screen_y: byte(pointers[3] + index),
                        template,
                    });
                }
                ensure!(terminated, "unterminated source spawn row");
            }
            rows.push(records);
            let mut special = Vec::new();
            if stage < SUPPRESSION_STAGE_CONFIGS {
                let index = usize::from(byte(SPECIAL_STAGE_OFFSETS + stage)) + row;
                let first = usize::from(byte(SPECIAL_ROOM_OFFSETS + index));
                let mut terminated = false;
                for index in first..SPECIAL_RECORDS {
                    let page = byte(SPECIAL_PAGES + index);
                    if page == NO_SOURCE_IDENTITY {
                        terminated = true;
                        break;
                    }
                    special.push(SpecialRecord {
                        page: page & 0x7f,
                        paired: page & ACTIVE != 0,
                        template: usize::from(byte(SPECIAL_TEMPLATES + index)),
                        screen_y: byte(SPECIAL_Y + index),
                    });
                }
                ensure!(terminated, "unterminated special spawn row");
            }
            special_rows.push(special);
        }
        let id = AssetId(format!("original/spawns-{stage}"));
        source_levels.push(Some(id.clone()));
        levels.push(SpawnDefinition {
            id,
            rows,
            special_rows,
        });
    }
    let fixed_suppression = |cpu: usize| rom[SUPPRESSION_BANK_BIAS + cpu];
    let mut suppression = Vec::new();
    for stage in 0..SUPPRESSION_STAGE_CONFIGS {
        let start = usize::from(fixed_suppression(SUPPRESSION_OFFSETS + stage));
        let count = usize::from(fixed_suppression(SUPPRESSION_ADDRESSES + start)) + 1;
        let mut rules = Vec::new();
        for index in start + 1..start + 1 + count {
            let field = match fixed_suppression(SUPPRESSION_ADDRESSES + index) {
                0xa0 => ProgressField::StageBits,
                0xa1 => ProgressField::SourceA1,
                0xa3 => ProgressField::SourceA3,
                0xa6 => ProgressField::SourceA6,
                0xa8 => ProgressField::SourceA8,
                0xae => ProgressField::SourceAe,
                _ => anyhow::bail!("unknown suppression source field"),
            };
            rules.push(SuppressionRule {
                field,
                mask: fixed_suppression(SUPPRESSION_MASKS + index),
                destination: fixed_suppression(SUPPRESSION_DESTINATIONS + index),
            });
        }
        suppression.push(rules);
    }
    Ok(SpawnCatalog {
        handlers,
        templates,
        levels,
        source_levels,
        extents: std::array::from_fn(|index| Extent {
            half_height: byte(HEIGHTS + index),
            half_width: byte(WIDTHS + index),
        }),
        vertical_presets: std::array::from_fn(|index| word(VERTICAL_PRESETS + index * 2) as i16),
        horizontal_presets: std::array::from_fn(|index| word(HORIZONTAL_PRESETS + index * 2)),
        suppression,
    })
}
