# Dispatch and data consumer survey

Metadata decoded from matching local ROM and checked against maintained source consumers. No original payload bytes are reproduced. Exact target instructions and all branch semantics remain stage-3 contract work. Selector numbering below is hexadecimal.

## Gameplay actor handlers

Table physical `$1803C`, 67 entries, split low/high planes.

- `00` → CPU `$80B2`, physical `$180C2`, B10: EmptyActorHandler
- `01` → CPU `$8225`, physical `$18235`, B07: Actor8225VisibilityAndHit
- `02` → CPU `$82AF`, physical `$182BF`, B07: Bank06_WalkingBounceActor
- `03` → CPU `$830E`, physical `$1831E`, B07: Bank06_ProjectileThrower
- `04` → CPU `$8462`, physical `$18472`, B07: Bank06_ThrownProjectile
- `05` → CPU `$8491`, physical `$184A1`, B07: Actor8491AnimationGate
- `06` → CPU `$84E5`, physical `$184F5`, B07: Bank06_ProximityRisingActor
- `07` → CPU `$8567`, physical `$18577`, B07: Bank06_PlayerContactAnimationActor
- `08` → CPU `$8668`, physical `$18678`, B08: Actor8668Cull
- `09` → CPU `$8722`, physical `$18732`, B08: Bank06_UpdateMovingTerrainObject
- `0A` → CPU `$8845`, physical `$18855`, B08: Bank06_ProximityFallingObject
- `0B` → CPU `$897B`, physical `$1898B`, B08: ActorLaunchedMotion
- `0C` → CPU `$90EA`, physical `$190FA`, B09: Bank06_TriggeredWalkingPlatform
- `0D` → CPU `$916C`, physical `$1917C`, B09: Bank06_PresetOscillatingActor
- `0E` → CPU `$9225`, physical `$19235`, B09: Bank06_RideablePresetActor
- `0F` → CPU `$9093`, physical `$190A3`, B09: Bank06_VerticalCarryingPlatform
- `10` → CPU `$9313`, physical `$19323`, B07: Actor9313Wait
- `11` → CPU `$9430`, physical `$19440`, B07: Bank06_DelayedPresetFlyingActor
- `12` → CPU `$9552`, physical `$19562`, B07: Bank06_PairedActorController
- `13` → CPU `$8A59`, physical `$18A69`, B08: Actor8A59AnimationGate
- `14` → CPU `$8C45`, physical `$18C55`, B08: Actor8C45ContactEntry
- `15` → CPU `$8D19`, physical `$18D29`, B08: Actor8D19ContactProfile
- `16` → CPU `$8DD1`, physical `$18DE1`, B08: Bank06_RevealTransformedObject
- `17` → CPU `$8E23`, physical `$18E33`, B08: Bank06_SlidingTransformedObject
- `18` → CPU `$96D5`, physical `$196E5`, B07: Bank06_LedgeAwareWalker
- `19` → CPU `$973C`, physical `$1974C`, B07: Bank06_FivePhaseAmbushActor
- `1A` → CPU `$9956`, physical `$19966`, B07: Bank06_ProximityEruptingActor
- `1B` → CPU `$9A81`, physical `$19A91`, B07: Bank06_RepeatingJumpActor
- `1C` → CPU `$9B10`, physical `$19B20`, B07: Actor9B10HitAnimation
- `1D` → CPU `$9C46`, physical `$19C56`, B09: Bank06_PushingContactActor
- `1E` → CPU `$9D8A`, physical `$19D9A`, B07: Bank06_FastLedgeAwareWalker
- `1F` → CPU `$9DF7`, physical `$19E07`, B07: Bank06_TimedHorizontalPatrol
- `20` → CPU `$9E51`, physical `$19E61`, B07: Actor9E51Initialize
- `21` → CPU `$9EB3`, physical `$19EC3`, B09: Bank06_PlayerCarryingActor
- `22` → CPU `$9F3D`, physical `$19F4D`, B07: Bank06_TriggeredJumpingPatrol
- `23` → CPU `$9FEC`, physical `$19FFC`, B07: Bank06_LeftwardWalkingActor
- `24` → CPU `$A024`, physical `$1A034`, B09: Bank06_PlayerExternalMotionTrigger
- `25` → CPU `$A065`, physical `$1A075`, B07: ActorA065HitGate
- `26` → CPU `$A335`, physical `$1A345`, B11: Bank06_FivePhaseAttackActor
- `27` → CPU `$A5BE`, physical `$1A5CE`, B11: Bank06_UpdateAttackProjectile
- `28` → CPU `$A6D3`, physical `$1A6E3`, B11: Bank06_PerimeterPathActor
- `29` → CPU `$AAE1`, physical `$1AAF1`, B11: Bank06_CenterReturningActor
- `2A` → CPU `$A8D6`, physical `$1A8E6`, B11: Bank06_BouncingWaveActor
- `2B` → CPU `$AAC8`, physical `$1AAD8`, B11: Bank06_UpdateWaveChild
- `2C` → CPU `$A1E9`, physical `$1A1F9`, B08: Bank06_FourItemDispenser
- `2D` → CPU `$A2BA`, physical `$1A2CA`, B08: Bank06_UpdateDispensedItem
- `2E` → CPU `$A5DB`, physical `$1A5EB`, B11: Bank06_JumpingImpactActor
- `2F` → CPU `$9065`, physical `$19075`, B15: Bank06_ProgressionContactSelector
- `30` → CPU `$AC3D`, physical `$1AC4D`, B07: Bank06_TimedRewardStealingPatrol
- `31` → CPU `$AC3D`, physical `$1AC4D`, B07: Bank06_TimedRewardStealingPatrol
- `32` → CPU `$ACBE`, physical `$1ACCE`, B07: Bank06_JumpingRewardStealer
- `33` → CPU `$ADB9`, physical `$1ADC9`, B07: Bank06_HitOnlyReactionActor
- `34` → CPU `$ADC1`, physical `$1ADD1`, B07: Bank06_PlayerLaunchContactActor
- `35` → CPU `$FDF5`, physical `$1FE05`, B15: UpdateDialogueContactActor
- `36` → CPU `$F869`, physical `$1F879`, B16: UpdatePlayerLinkedDeparture
- `37` → CPU `$F8F2`, physical `$1F902`, B16: UpdateTimedDepartureSpawner
- `38` → CPU `$F9F8`, physical `$1FA08`, B09: UpdatePlayerLiftActor
- `39` → CPU `$FE69`, physical `$1FE79`, B09: UpdateFourPhaseMovingPlatform
- `3A` → CPU `$FA3F`, physical `$1FA4F`, B17: UpdateFinalBoss
- `3B` → CPU `$FBC9`, physical `$1FBD9`, B17: FinalBossDefeatCountdown
- `3C` → CPU `$FB68`, physical `$1FB78`, B17: UpdateRaceClimber
- `3D` → CPU `$FBFF`, physical `$1FC0F`, B15: UpdateHitActivatedDialogueActor
- `3E` → CPU `$FCA0`, physical `$1FCB0`, B15: UpdateProgressionEscort
- `3F` → CPU `$FD68`, physical `$1FD78`, B15: UpdateRightwardTransitionActor
- `40` → CPU `$FD9E`, physical `$1FDAE`, B17: UpdateFinalRaceGoal
- `41` → CPU `$FE50`, physical `$1FE60`, B15: UpdateConditionalContactMarker
- `42` → CPU `$8EE6`, physical `$18EF6`, B08: Bank06_SwingReleasedCollectible
## Player states

Table physical `$1E3A6`, 17 entries, split low/high planes.

- `00` → CPU `$E3B8`, physical `$1E3C8`, B06: PlayerStandingEntry
- `01` → CPU `$E63B`, physical `$1E64B`, B06: PlayerAirborneEntry
- `02` → CPU `$E7EA`, physical `$1E7FA`, B06: PlayerForcedJumpInput
- `03` → CPU `$E806`, physical `$1E816`, B06: UpdatePlayerClimbing
- `04` → CPU `$E8EB`, physical `$1E8FB`, B06: UpdatePlayerSwing
- `05` → CPU `$EAB9`, physical `$1EAC9`, B06: UpdatePlayerProtectedMotion
- `06` → CPU `$EB00`, physical `$1EB10`, B06: StartPlayerDamageArc
- `07` → CPU `$EB82`, physical `$1EB92`, B06: PlayerSnowLikePose
- `08` → CPU `$EBA7`, physical `$1EBB7`, B06: PlayerFallOutOfScreen
- `09` → CPU `$EBC9`, physical `$1EBD9`, B06: WaitForPlayerA
- `0A` → CPU `$EBD9`, physical `$1EBE9`, B06: UpdatePlayerCarriedState
- `0B` → CPU `$EBD9`, physical `$1EBE9`, B06: UpdatePlayerCarriedState
- `0C` → CPU `$EC2D`, physical `$1EC3D`, B06: UpdatePlayerExternalLaunch
- `0D` → CPU `$EC9B`, physical `$1ECAB`, B06: UpdatePlayerDeathTrajectory
- `0E` → CPU `$ECFF`, physical `$1ED0F`, B06: HoldPlayerAnimation
- `0F` → CPU `$ED05`, physical `$1ED15`, B15: RunPlayerDialogue
- `10` → CPU `$EF62`, physical `$1EF72`, B14: ConvertStageTimerToReward
## Scene actor handlers

Table physical `$1751D`, 6 entries, split low/high planes.

- `00` → CPU `$B412`, physical `$17422`, B16: Bank05_SceneBounce
- `01` → CPU `$B46A`, physical `$1747A`, B16: Bank05_SceneWalkIn
- `02` → CPU `$B48F`, physical `$1749F`, B16: Bank05_SceneDelay
- `03` → CPU `$B4B5`, physical `$174C5`, B16: Bank05_SceneMoveToMark
- `04` → CPU `$B507`, physical `$17517`, B16: Bank05_UpdateSceneActor
- `05` → CPU `$B507`, physical `$17517`, B16: Bank05_UpdateSceneActor
## Terrain responses

Table physical `$1D1F2`, 16 entries, split low/high planes.

- `00` → CPU `$D199`, physical `$1D1A9`, B05: TerrainPlayerPoseResponse
- `01` → CPU `$D199`, physical `$1D1A9`, B05: TerrainPlayerPoseResponse
- `02` → CPU `$D199`, physical `$1D1A9`, B05: TerrainPlayerPoseResponse
- `03` → CPU `$D199`, physical `$1D1A9`, B05: TerrainPlayerPoseResponse
- `04` → CPU `$D16B`, physical `$1D17B`, B05: TerrainResponseSetRenderBitTwo
- `05` → CPU `$D16B`, physical `$1D17B`, B05: TerrainResponseSetRenderBitTwo
- `06` → CPU `$D16B`, physical `$1D17B`, B05: TerrainResponseSetRenderBitTwo
- `07` → CPU `$D16B`, physical `$1D17B`, B05: TerrainResponseSetRenderBitTwo
- `08` → CPU `$D1B1`, physical `$1D1C1`, B05: TerrainResponsePlayerMomentum
- `09` → CPU `$D176`, physical `$1D186`, B05: TerrainResponseDamagePlayer
- `0A` → CPU `$D1BF`, physical `$1D1CF`, B05: TerrainResponseLowScreenExit
- `0B` → CPU `$D1BF`, physical `$1D1CF`, B05: TerrainResponseLowScreenExit
- `0C` → CPU `$D176`, physical `$1D186`, B05: TerrainResponseDamagePlayer
- `0D` → CPU `$D176`, physical `$1D186`, B05: TerrainResponseDamagePlayer
- `0E` → CPU `$D1D1`, physical `$1D1E1`, B05: TerrainResponseMarkAndMomentum
- `0F` → CPU `$D162`, physical `$1D172`, B05: ProbeTerrainCollision
## Vertical scroll phases

Table physical `$1CA3A`, 16 entries, little-endian words.

- `00` → CPU `$CA4A`, physical `$1CA5A`, B04: VerticalPhaseBuildRoomRow
- `01` → CPU `$CB5A`, physical `$1CB6A`, B04: VerticalPhaseNextRoom
- `02` → CPU `$CB6E`, physical `$1CB7E`, B04: VerticalPhasePreviousRoom
- `03` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `04` → CPU `$CA4E`, physical `$1CA5E`, B04: VerticalPhaseCopyStagedRows
- `05` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `06` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `07` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `08` → CPU `$CB5A`, physical `$1CB6A`, B04: VerticalPhaseNextRoom
- `09` → CPU `$CB6E`, physical `$1CB7E`, B04: VerticalPhasePreviousRoom
- `0A` → CPU `$CA8B`, physical `$1CA9B`, B04: VerticalPhaseWrapBoundary
- `0B` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `0C` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `0D` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
- `0E` → CPU `$CA4A`, physical `$1CA5A`, B04: VerticalPhaseBuildRoomRow
- `0F` → CPU `$C42C`, physical `$1C43C`, B02: PollBackgroundUpload
## Audio primary opcode targets

Table physical `$0C7CB`, 10 entries, little-endian words.

- `00` → CPU `$87CF`, physical `$0C7DF`, B18: AudioPrimarySetDurationScale
- `01` → CPU `$87D9`, physical `$0C7E9`, B18: AudioBank03_SetPitchStep
- `02` → CPU `$87E3`, physical `$0C7F3`, B18: AudioPrimarySetControl
- `03` → CPU `$87F3`, physical `$0C803`, B18: AudioPrimarySetControl
- `04` → CPU `$880D`, physical `$0C81D`, B18: AudioPrimaryLoop
- `05` → CPU `$8855`, physical `$0C865`, B18: AudioSelectPeriodTable
- `06` → CPU `$8872`, physical `$0C882`, B18: AudioSelectPeriodTable
- `07` → CPU `$8885`, physical `$0C895`, B18: AudioSelectPeriodTable
- `08` → CPU `$88BF`, physical `$0C8CF`, B18: AudioSelectModulationPreset
- `09` → CPU `$890F`, physical `$0C91F`, B18: AudioBank03_StopPrimaryStream
## Audio secondary opcode targets

Table physical `$0C5D3`, 7 entries, little-endian words.

- `00` → CPU `$85D1`, physical `$0C5E1`, B18: AudioSetDelay
- `01` → CPU `$85D9`, physical `$0C5E9`, B18: AudioSetDelay
- `02` → CPU `$85E3`, physical `$0C5F3`, B18: AudioSetDelay
- `03` → CPU `$85F3`, physical `$0C603`, B18: AudioSetDelay
- `04` → CPU `$860D`, physical `$0C61D`, B18: AudioSecondaryLoop
- `05` → CPU `$863E`, physical `$0C64E`, B18: AudioSetDelay
- `06` → CPU `$8654`, physical `$0C664`, B18: AudioSetDelay
## Audio command headers

Table physical `$0CA50`, 56 entries, little-endian words.

- `00` → CPU `$8AB0`, physical `$0CAC0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0
- `01` → CPU `$8EF7`, physical `$0CF07`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `02` → CPU `$924D`, physical `$0D25D`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `03` → CPU `$9457`, physical `$0D467`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `04` → CPU `$97F0`, physical `$0D800`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `05` → CPU `$9A54`, physical `$0DA64`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `06` → CPU `$9A95`, physical `$0DAA5`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `07` → CPU `$9C69`, physical `$0DC79`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `08` → CPU `$A0A4`, physical `$0E0B4`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `09` → CPU `$A103`, physical `$0E113`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `0A` → CPU `$A16F`, physical `$0E17F`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `0B` → CPU `$A1BA`, physical `$0E1CA`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `0C` → CPU `$8AB0`, physical `$0CAC0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0
- `0D` → CPU `$8AB0`, physical `$0CAC0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0
- `0E` → CPU `$8AB0`, physical `$0CAC0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0
- `0F` → CPU `$8AB0`, physical `$0CAC0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0
- `10` → CPU `$A3B2`, physical `$0E3C2`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `11` → CPU `$A3C9`, physical `$0E3D9`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `12` → CPU `$A40C`, physical `$0E41C`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `13` → CPU `$A41F`, physical `$0E42F`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `14` → CPU `$A442`, physical `$0E452`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `15` → CPU `$A45F`, physical `$0E46F`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `16` → CPU `$A478`, physical `$0E488`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `17` → CPU `$A4A7`, physical `$0E4B7`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `18` → CPU `$A4D0`, physical `$0E4E0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `19` → CPU `$A50D`, physical `$0E51D`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `1A` → CPU `$A542`, physical `$0E552`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `1B` → CPU `$A553`, physical `$0E563`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `1C` → CPU `$A562`, physical `$0E572`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `1D` → CPU `$A562`, physical `$0E572`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `1E` → CPU `$A571`, physical `$0E581`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `1F` → CPU `$A571`, physical `$0E581`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `20` → CPU `$A571`, physical `$0E581`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `21` → CPU `$A5C2`, physical `$0E5D2`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `22` → CPU `$A5C2`, physical `$0E5D2`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `23` → CPU `$A5C2`, physical `$0E5D2`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `24` → CPU `$A5E5`, physical `$0E5F5`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `25` → CPU `$A61C`, physical `$0E62C`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `26` → CPU `$A634`, physical `$0E644`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `27` → CPU `$A64B`, physical `$0E65B`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `28` → CPU `$A64B`, physical `$0E65B`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `29` → CPU `$A64B`, physical `$0E65B`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `2A` → CPU `$A6AE`, physical `$0E6BE`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `2B` → CPU `$A6C4`, physical `$0E6D4`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `2C` → CPU `$A6CD`, physical `$0E6DD`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `2D` → CPU `$A6D8`, physical `$0E6E8`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `2E` → CPU `$A6F3`, physical `$0E703`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `2F` → CPU `$A6FE`, physical `$0E70E`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `30` → CPU `$A721`, physical `$0E731`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `31` → CPU `$A73C`, physical `$0E74C`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `32` → CPU `$A74F`, physical `$0E75F`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `33` → CPU `$A75C`, physical `$0E76C`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `34` → CPU `$A771`, physical `$0E781`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `35` → CPU `$A782`, physical `$0E792`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `36` → CPU `$A7D0`, physical `$0E7E0`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes
- `37` → CPU `$A7F9`, physical `$0E809`, B18: AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes

## Full named table/data index

Every Data/Image/Unknown region remains assigned in ledger-area-map.json, including anonymous uniform fragments. The named entries below link consumers by source labels and physical extents; all direct and indirect consumer contracts remain mandatory. Audio inline dispatch tables and actor local split tables must be derived from their callers as well as these named data slices.

- `region-1` physical `$00000..$00010` B01: INESHeader. iNES header: mapper 2, 128KiB PRG, CHR RAM and vertical mirroring. Cartridge metadata is retained verbatim.
- `region-5` physical `$00010..$00810` B02: ChrTiles_00010. CHR-RAM upload payload: 128 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-20` physical `$00810..$00B10` B02: AssetData_00810. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-6` physical `$00B10..$00C10` B02: ChrTiles_00B10. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-21` physical `$00C10..$00F10` B02: AssetData_00C10. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-7` physical `$00F10..$01010` B02: ChrTiles_00F10. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-22` physical `$01010..$01410` B02: AssetData_01010. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-8` physical `$01410..$01910` B02: ChrTiles_01410. CHR-RAM upload payload: 80 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-23` physical `$01910..$01D10` B02: AssetData_01910. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-9` physical `$01D10..$02110` B02: ChrTiles_01D10. CHR-RAM upload payload: 64 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-24` physical `$02110..$02410` B02: AssetData_02110. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-10` physical `$02410..$02510` B02: ChrTiles_02410. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-25` physical `$02510..$02610` B02: AssetData_02510. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-11` physical `$02610..$02810` B02: ChrTiles_02610. CHR-RAM upload payload: 32 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-26` physical `$02810..$02910` B02: AssetData_02810. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-12` physical `$02910..$02A10` B02: ChrTiles_02910. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-27` physical `$02A10..$02D10` B02: AssetData_02A10. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-13` physical `$02D10..$03010` B02: ChrTiles_02D10. CHR-RAM upload payload: 48 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-28` physical `$03010..$03510` B02: AssetData_03010. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-14` physical `$03510..$03610` B02: ChrTiles_03510. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-29` physical `$03610..$03E10` B02: AssetData_03610. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-15` physical `$03E10..$03F10` B02: ChrTiles_03E10. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-30` physical `$03F10..$04010` B02: AssetData_03F10. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-16` physical `$04010..$07010` B02: ChrTiles_04010. CHR-RAM upload payload: 768 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-31` physical `$07010..$08010` B02: AssetData_07010. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-17` physical `$08010..$0B110` B02: ChrTiles_08010. CHR-RAM upload payload: 784 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-32` physical `$0B110..$0B210` B02: AssetData_0B110. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `region-18` physical `$0B210..$0B310` B02: ChrTiles_0B210. CHR-RAM upload payload: 16 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-33` physical `$0B310..$0C010` B02: AssetData_0B310. Generic graphics-bank data. Tile sheets, adjacent upload descriptors and candidate references were checked; no conflicting code structure was found. Additional tile-like and packed records are visible, but their format, consumers and exact semantic objects remain unresolved.
- `unknown-00c522` physical `$0C522..$0C524` B18: . Two isolated bytes A9 07 at physical 50466..50468 / bank6 CPU8512..8514. A legal immediate-load decoding has no independently supported entry; the next routine overwrites A and the preceding JMP cannot fall through. Data role is equally unresolved. Preserve unknown exactly.
- `region-483` physical `$0C5D3..$0C5E1` B18: AudioSecondaryCommandWords_0C5D3. Seven secondary-command inline target words consumed by8554. Individual game meanings remain unresolved.
- `region-484` physical `$0C7CB..$0C7DF` B18: AudioPrimaryCommandWords_0C7CB. Ten primary-command inline target words consumed by8554. Individual game meanings remain unresolved.
- `region-485` physical `$0C97B..$0CA50` B18: AudioDurationAndPeriodData_0C97B. Audio duration-class bytes followed by period-word lookup data. The command interpreter uses these to scale duration and select 16-bit periods. Independent boundaries end at the command-header pointer table; exact note names, tuning model and original generator are unresolved.
- `region-486` physical `$0CA50..$0CAC0` B18: AudioCommandHeaderPointers_0CA50. 56 command stream header pointers; first encoded header8AB0 closes domain. Individual game meanings remain unresolved.
- `region-487` physical `$0CAC0..$0F110` B18: AudioStreamsAndUnresolvedBitmapData_0CAC0. Bounded non-code audio-bank payload: encoded channel streams and presets, with bitmap-like data near the tail. Command-header pointers and interpreter consumers were inspected, but individual stream/preset/bitmap object boundaries and musical names are not established across this whole area.
- `region-19` physical `$0F110..$0F310` B02: ChrTiles_0F110. CHR-RAM upload payload: 32 native 16-byte 2bpp tiles. Descriptor-selected source pages are copied by UploadChrLists. Exact tile identities/animation associations are unresolved; see the local tile-bank sheets and graphics-structure.json.
- `region-488` physical `$0F310..$10010` B04: TerrainBlockPlanesAndProperties_0F310. Terrain block records and four tile planes plus palette/collision plane; stage3/4 consumer context. Individual game meanings remain unresolved.
- `region-489` physical `$10010..$104F8` B04: PackedRoomPayloadFragment_10010. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-453` physical `$104F8..$10518` B04: GenericData_104F8. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-490` physical `$10518..$106D0` B04: PackedRoomPayloadFragment_10518. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-454` physical `$106D0..$106FF` B04: GenericData_106D0. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-491` physical `$106FF..$10718` B04: PackedRoomPayloadFragment_106FF. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-455` physical `$10718..$1073A` B04: GenericData_10718. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-492` physical `$1073A..$10EF8` B04: PackedRoomPayloadFragment_1073A. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-456` physical `$10EF8..$10F18` B04: GenericData_10EF8. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-493` physical `$10F18..$12020` B04: PackedRoomPayloadFragment_10F18. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-457` physical `$12020..$1204A` B04: GenericData_12020. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-494` physical `$1204A..$12068` B04: PackedRoomPayloadFragment_1204A. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-458` physical `$12068..$12098` B04: GenericData_12068. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-495` physical `$12098..$120B0` B04: PackedRoomPayloadFragment_12098. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-459` physical `$120B0..$120D4` B04: GenericData_120B0. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-496` physical `$120D4..$136E8` B04: PackedRoomPayloadFragment_120D4. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-460` physical `$136E8..$1370C` B04: GenericData_136E8. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-497` physical `$1370C..$13730` B04: PackedRoomPayloadFragment_1370C. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-461` physical `$13730..$13758` B04: GenericData_13730. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-498` physical `$13758..$13848` B04: PackedRoomPayloadFragment_13758. Fragment of the bank-four room area (physical 65552..79952), containing 200 records of 72 bytes. Each record has 64 low block-index bytes and eight bitmap bytes supplying bit eight. Stage lists cover selectors 0..199; fragment boundaries here preserve earlier independently classified slices, rather than dividing logical rooms.
- `region-462` physical `$13848..$1386F` B04: GenericData_13848. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-499` physical `$1386F..$13870` B04: BoundedData_1386F. Zero/FF runs with isolated tag after final room record; generic non-code, no purpose asserted. Individual game meanings remain unresolved.
- `region-463` physical `$13870..$13898` B04: GenericData_13870. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-500` physical `$13898..$138BC` B15: DialogueTextPointers_13898. Eighteen dialogue pointers split into low/high byte planes at CPU $B888/$B89A in mapper bank four. DialogueMessage selects the same index in both planes; targets are the following terminated encoded strings.
- `region-501` physical `$138BC..$13C59` B15: DialogueEncodedStrings_138BC. Eighteen contiguous dialogue strings in the game tile-code alphabet. $AE terminates a message; $FF controls line/page breaks and $24 is blank. The fixed dialogue renderer limits each displayed row and waits between visible glyphs; no decoded wording is claimed here.
- `region-464` physical `$13C59..$13D10` B15: GenericData_13C59. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-465` physical `$13D10..$13D90` B15: GenericData_13D10. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-503` physical `$13DD7..$13DE3` B05: TerrainParticleOffsetAndTickArrays_13DD7. Three four-byte particle parameter arrays: screen-Y offsets at $BDC7, world-X offsets at $BDCB and initial animation ticks at $BDCF. SpawnTerrainParticles uses Y=3..0; this data starts after its complete JSR/RTS tail.
- `region-466` physical `$13DE3..$13E10` B05: GenericData_13DE3. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-467` physical `$13EBB..$13F10` B16: GenericData_13EBB. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-505` physical `$13F10..$14010` B16: BoundedData_13F10. Uniform and bitmap-like bank-tail data; no executable references found in bounded caller/consumer review. Individual game meanings remain unresolved.
- `region-509` physical `$1424B..$15691` B03: CompositeSpriteFrameRecords_1424B. Composite-sprite frame records in mapper bank five. Header bytes hold last piece index and coordinate-stream selector, followed by count-plus-one tile/attribute byte pairs. The 244 ROM frame records exactly cover this area; the other four frame selectors point to RAM $04C0.
- `region-510` physical `$15691..$15881` B03: CompositeSpriteFramePointers_15691. 248 frame pointers split into low bytes at $9681 and high bytes at $9779 in mapper bank five. First four select RAM $04C0; all other targets lie in the classified composite-frame record area.
- `region-511` physical `$15881..$16DF1` B03: SpritePieceCoordinateStreams_15881. Signed Y/X piece-offset byte pairs for composite sprites. Frame headers and their flip variants establish the referenced lengths. Local unreferenced gaps remain generic data; no shared schema beyond those referenced streams is asserted.
- `region-512` physical `$16DF1..$16FB1` B03: SpritePieceCoordinatePointers_16DF1. 224 coordinate-stream pointers, low bytes at $ADE1 and high bytes at $AEC1 in mapper bank five. A frame chooses the base selector and flip variant; its piece count bounds the signed Y/X byte pairs consumed.
- `region-513` physical `$16FB1..$171D7` B03: CountedAnimationSequences_16FB1. 144 counted animation sequences. Each header stores last frame index and ticks per frame, followed by count-plus-one frame-selector bytes. The referenced records form an exact packed union; original animation names are unresolved.
- `region-514` physical `$171D7..$17278` B03: AnimationPointerFragment_171D7. Fragment of 144 low/high animation pointers at CPU $B1C7/$B257 in mapper bank five. Separate old classified slices interleave this physical extent; pointer index is an animation selector, not a frame number.
- `region-468` physical `$17278..$172BC` B03: GenericData_17278. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-469` physical `$172BC..$172F5` B03: GenericData_172BC. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-515` physical `$172F5..$172F7` B03: AnimationPointerFragment_172F5. Fragment of 144 low/high animation pointers at CPU $B1C7/$B257 in mapper bank five. Separate old classified slices interleave this physical extent; pointer index is an animation selector, not a frame number.
- `region-516` physical `$172F7..$17310` B03: BoundedData_172F7. Bounded FF/zero sparse non-code region preceding established scene initializer. Individual game meanings remain unresolved.
- `region-470` physical `$17310..$17390` B03: GenericData_17310. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-517` physical `$173DC..$17410` B16: SceneActorInitializationLists_173DC. Six scene offsets and variable three-byte actor initialization lists. Individual game meanings remain unresolved.
- `region-518` physical `$1751D..$17529` B16: SceneHandlerPointers_1751D. Six split scene handler pointers followed by scene parameter data. Individual game meanings remain unresolved.
- `region-471` physical `$17529..$17590` B16: GenericData_17529. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-519` physical `$1762C..$17770` B14: StatusStreamRecords_1762C. Status stream pointers, counted inserts and encoded PPU payloads. Individual game meanings remain unresolved.
- `region-520` physical `$177AF..$17810` B10: StageSpawnSuppressionRecords_177AF. Five stage suppression-list indices, count/address/mask/destination planes and bit masks. Individual game meanings remain unresolved.
- `region-521` physical `$1787C..$17AC0` B14: StatusMessageRecords_1787C. Status message offsets/records, counted text payloads and23split pointers. Individual game meanings remain unresolved.
- `region-522` physical `$17B0A..$17FEE` B03: ScreenScriptRecords_17B0A. 12split screen script pointers and destination/payload scripts with FE/00 terminators; uniform suffix. Individual game meanings remain unresolved.
- `region-472` physical `$17FEE..$18010` B03: GenericData_17FEE. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-590` physical `$1803C..$180C2` B10: GameplayActorHandlerPointers_1803C. 67 actor handler pointers: low bytes at $802C and high bytes at $806F in mapper bank six. ObjectHandler indexes both planes; dispatch pushes a synthetic next-slot return before jumping. The independently supported empty handler at $80B2 bounds the table.
- `region-591` physical `$1815A..$1819A` B10: BoundedData_1815A. Two 32-byte actor contact extents: vertical at $814A and horizontal at $816A in mapper bank six. The source actor collision profile, masked to seven bits, selects the tolerated absolute position separation; callers establish the supported profile domain.
- `region-592` physical `$181B5..$18235` B10: BoundedData_181B5. Two 32-word motion-preset tables in mapper bank six: vertical velocity at $81A5 and horizontal speed at $81E5. SetMotionPreset doubles A for word indexing; individual handlers may normalize signs or interpret the words as signed increments.
- `region-593` physical `$18671..$18678` B07: BoundedData_18671. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-594` physical `$1890C..$18942` B08: BoundedData_1890C. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-595` physical `$18C0E..$18C23` B08: BoundedData_18C0E. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-596` physical `$18EEE..$18EF6` B08: BoundedData_18EEE. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-597` physical `$18FC8..$19075` B08: BoundedData_18FC8. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-598` physical `$19225..$19235` B09: BoundedData_19225. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-599` physical `$192FD..$19323` B09: BoundedData_192FD. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-600` physical `$19554..$19562` B07: BoundedData_19554. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-601` physical `$196CB..$196E5` B07: BoundedData_196CB. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-602` physical `$1994C..$19966` B07: BoundedData_1994C. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-603` physical `$19A85..$19A91` B07: BoundedData_19A85. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-604` physical `$19D96..$19D9A` B07: BoundedData_19D96. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-605` physical `$19E05..$19E07` B07: BoundedData_19E05. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-606` physical `$1A2BE..$1A2CA` B08: BoundedData_1A2BE. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-607` physical `$1A595..$1A5CE` B11: BoundedData_1A595. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-608` physical `$1A6D9..$1A6E3` B11: BoundedData_1A6D9. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-609` physical `$1A8BC..$1A8E6` B11: BoundedData_1A8BC. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-610` physical `$1AAB8..$1AAD8` B11: BoundedData_1AAB8. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-611` physical `$1AC47..$1AC4D` B11: BoundedData_1AC47. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-612` physical `$1ACCC..$1ACCE` B07: BoundedData_1ACCC. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-613` physical `$1AE24..$1AE30` B07: BoundedData_1AE24. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-614` physical `$1AE65..$1AE70` B10: BoundedData_1AE65. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-473` physical `$1AE8D..$1AEB0` B10: GenericData_1AE8D. Bounded uniform generic data; exact run endpoints and conflict checks are recorded in uniform-data-review.json. Uniformity establishes non-code here, without identifying padding purpose, consumer or neighboring object boundaries.
- `region-615` physical `$1B058..$1B0BB` B10: BoundedData_1B058. Bounded actor lookup/parameter data or uniform gap; consumers and adjoining supported entries inspected. Individual game meanings remain unresolved.
- `region-616` physical `$1B174..$1C010` B10: SpecialAndMainSpawnRecords_1B174. Special-spawn stage/room/page/template/Y metadata, the finite main-spawn parallel position/template arrays, and 121 four-byte actor templates through CPU $BFFA. Template fields are flags/local-state, animation, handler and collision profile. The final six zero bytes are generic non-code; no padding purpose is inferred.
- `region-3` physical `$1C368..$1C37E` B02: ChrUploadListPointers. Eleven low pointer bytes at C358, then eleven high bytes at C363; X is the list selector. Repeated list origins are permitted.
- `region-4` physical `$1C37E..$1C43C` B02: ChrUploadLists. Three-byte records: [16KiB bank<<4 | pages_minus_one, source_CPU_page, PPU_page]. Source/destination low bytes are zero. A tag with bit 7 set ends each list. Encoded origins and consumer bounds establish this complete 190-byte descriptor area.
- `region-1261` physical `$1C518..$1C72D` B02: StagePaletteConfiguration. Stage configuration indices, split palette-list pointers/terminated palette-list records and four-byte palette values; C435/C43C/C45C/C461/C4E9/C4F0/C4F8 consumers; bounded repeated records support non-code without claiming original layout for every gap.
- `region-1262` physical `$1C796..$1C79F` B04: TerrainScrollClampData. Three parallel clamp/step byte arrays read C745/C74A/C752.
- `region-1263` physical `$1C97A..$1C986` B04: TerrainDirectionAndRowSteps. Signed direction/room-step parameters consumed C813/C823/C82B/C8B0, before codeC976.
- `region-1264` physical `$1CA3A..$1CA5A` B04: VerticalScrollPhasePointers. Sixteen little-endian phase handlers; counter30 masked7 and doubled, plus direction offsetCC8D/X0or4 equal10or00; first handlerCA4A closes endpoint; synthetic C9FA return resumesC9FB.
- `region-1265` physical `$1CC95..$1CCCD` B04: VerticalScrollParameters. Direction, phase, signed displacement, mask and nametable address parameters; C989/C9A3/C9B4/C9B9/C9C2/C9CC/C9D3/C9DC/C9E4 plus connected CAxx/CBxx consumers; exact code neighborsCC84RTS/CCBDentry.
- `region-617` physical `$1CE30..$1CE3C` B04: TerrainBitmapAndQuadrantMasks_1CE30. Eight bitmap masks and four quadrant indices. Individual game meanings remain unresolved.
- `region-618` physical `$1CE67..$1CFA3` B04: StageRoomLayoutLists_1CE67. Ten row counts/column counts, packed stage room lists, ten split list pointers. Individual game meanings remain unresolved.
- `region-619` physical `$1D05D..$1D078` B13: StageTransitionParameters_1D05D. Five transition rows/pages/direction flags and signed camera/player/boundary offsets. Individual game meanings remain unresolved.
- `region-1266` physical `$1D120..$1D130` B05: ChangedTerrainBurstParameters. Count/destination/tile-sequence and particle origin parameters consumed D071/D076/D07F/D098/D0FE/D103.
- `region-1267` physical `$1D1F2..$1D212` B05: TerrainCollisionHandlerPointers. Sixteen split low/high handler pointers consumed D155/D15A; independently encoded D1E2/D1F2 origins and D202entry bound32bytes; D1DC CPX0/BNE then D1E0BEQ has no feasible fallthrough into pointer bytes.
- `region-1268` physical `$1D3DE..$1D4AA` B05: CollisionGeometryLookupData. Signed collision geometry offsets and flag lookup bytes read D354/D35F/D382/D38D/D3A6; repeated signed values and adjoining code bounds exclude instruction body.
- `region-1269` physical `$1D631..$1D640` B12: StageBoundaryPositionData. Position/actor parameters and signed step pairs consumed D54D/D553/D559/D5B5/D5BD/D5C3.
- `region-1270` physical `$1D8B6..$1D8E9` B12: StageProgressionParameterData. Signed progression adjustments, actor parameter records and8bit masks consumed D661/D667/D66D/D6F3/D6FD/D70D/D713/D73B/D743/D74B/D755/D765/D76B; exact code neighbors.
- `region-1271` physical `$1DA20..$1DB46` B13: StagePaletteAndOverlayData. Stage lookup bytes, four-byte NES palettes, FFterminated display/object records and timer pairs; consumers D593/D60C/D617/D85A/D868/D875/D87B/D9D7/D9DC/D9EA/D9F0/D9F7/DA00/DCF3/DCF9; no single schema asserted across mixed data.
- `region-1272` physical `$1DC3C..$1DC4E` B13: RoomTransitionComparisonData. Two9byte row/column comparison arrays, includingFFsentinels, consumed DC08/DC10.
- `region-1273` physical `$1DDD1..$1DE7C` B13: CheckpointInitializationRecords. Nineteen contiguous9byte camera/row/player initialization records; DD4E stride9 then DD5A..DD88 fields, next independent codeDE6C bounds extent; valid structural slice, full natural selector domain not claimed.
- `region-1274` physical `$1DEF7..$1DF18` B13: GameOverOverlayData. Eight4byte OAM-like records plusFFterminator copied DEC0 withY20..0.
- `region-1275` physical `$1DFDF..$1DFFD` B16: SpecialRoomActorInitializationData. Three list offsets, terminated6byte actor initialization records and player-state selectors; DF73/DF78/DF7D..DF9D consumers, no executable overlap.
- `region-1276` physical `$1E12B..$1E1BF` B16: ScenePaletteAndParameterData. Scene handler parameters, palette values and final four-byte color groups; E08A/E0C5/E0CA/E0F1/E106/F76D/F772/F77F/F78A/F7AE consumers; repeated palettes inspected between code endpoints.
- `region-620` physical `$1E3A6..$1E3C8` B06: PlayerStateHandlerPointers_1E3A6. Seventeen player-state handler pointers, split into low and high byte planes at $E396/$E3A7. PlayerState selects both planes; the first handler at $E3B8 independently bounds their end.
- `region-1277` physical `$1E649..$1E64B` B06: PlayerFacingBitValues. Two orientation flag bytes00/40 consumed E51A withY input bit.
- `region-1278` physical `$1E74E..$1E750` B06: PlayerClimbThresholds. Two threshold bytes consumed E731 indexed branch.
- `region-1279` physical `$1E7F6..$1E7FA` B06: PlayerGridAlignmentOffsets. Four offsets consumed E75F indexed directional alignment.
- `region-1280` physical `$1E8F3..$1E8FB` B06: PlayerTransitionAdjustments. Eight signed/XOR/row adjustment bytes consumed E89F/E8A3/E8B9/E8C2.
- `region-1281` physical `$1EAC3..$1EAC9` B06: PlayerVerticalMotionParameters. Six velocity/threshold bytes consumed E9CC/E9D2/EA81.
- `region-1282` physical `$1EF69..$1EF72` B15: DialogueLinePositionData. Three3byte parallel line position/end/count arrays consumed EEB0/EEC1/EF0F.
- `region-1283` physical `$1F0F1..$1F108` B04: CameraEdgeAdjustmentData. Signed camera/player adjustment bytes read F022/F02B/F034/F0D3/F0DB plus boundedFFsuffix; generic non-code for unreferenced suffix, no fill-purpose claim.
- `region-1284` physical `$1F340..$1F342` B05: ObjectVelocityPresetWord. One16bit velocity preset0133 consumed F314/F31A and parallel staticF323/F329; supported setter preconditionA0; no expanded selector domain claim.
- `region-1285` physical `$1F4A8..$1F4E8` B05: ObjectCollisionExtentData. Two32byte collision extent arrays consumed D232/D237/D26D/D2C2/D2C9/D30A/F377/F37C/F39C/F3FF/F404/F41D/F45B/F475.
- `region-1286` physical `$1F663..$1F6A7` B04: TerrainReplacementLookupData. Replacement-tile/animation and attribute masks consumed F549/F5AC/F5BA/F5DB/F600/F62F/F63A; bounded numeric/mask arrays, generic localized gaps.
- `region-1287` physical `$1F81A..$1F822` B10: ObjectRemovalBitMasks. Eight successive1bit masks consumed F7F7.
- `region-1288` physical `$1FA00..$1FA08` B16: ActorSpawnDelayChoices. Eight bounded cooldown values selected byF9E6AND07/F9E9.
- `region-1289` physical `$1FB64..$1FB78` B17: ActorSpawnPositionAndFacingData. Sixteen Xpositions selected byFA92AND0F/FA95 plus two signed offsetsFB1B and two facing masksFABE.
- `region-1290` physical `$1FEE6..$1FEF0` B09: MovingPlatformPhaseData. Four phase direction bytesFE89AND03/FE8C, two signed16bit position adjustments FEC1/F ECA indexed0or2, and boundedFFsuffix.
- `region-1291` physical `$1FFF5..$2000A` B01: MapperWriteValuesAndSparseTail. Seven bus-conflict-safe bank bytes0..6 consumed FFE1, followed sparse zero/word-like non-code suffix before independently established vectorsFFFA. Consumer and boundary inspected; generic data only for trailing14bytes, no seed/checksum/schema claim.
- `region-2` physical `$2000A..$20010` B01: InterruptVectors. Little-endian NMI/RESET/IRQ words: C000/FEE0/C000 in the fixed window. IRQ shares the NMI service entry.

## Stage and inline domains requiring explicit contract expansion

- StageLayoutRows/Columns CE57/CE61 and StageRoomListPointerLow/High CF7F/CF89: ten selectors; room selectors span 0..199. Five land choices use0..4; source5 selects land screen,9 ending. Resolve6..8 through special/final scene callers, without presuming extra selectable lands.
- StageSpawn*PointerLow/High B050..B0A1: stage main parallel position/template arrays and row-start offsets; B10 must cross-check its nine selector entries (pointer-plane stride9) against ten room-layout selectors and prove caller restrictions, 121 four-byte templates, suppression records B7AF and special-spawn lists B174.
- StageTerrainBanks DA10 onward, initial room/player/checkpoint data DDCF onward, stage palette/background configurations C508 onward and stage boundary parameters D04D onward require per-selector consumer joins. Source bank offsets and CPU addresses remain distinct.
- Actor local dispatches: moving terrain objects (nine), five-phase ambush/attack and boss paths, platform phases and all split local handler tables must be traced from indirect JMP operands even where table bytes are generic incbin.
- AudioInlineDispatch and PrimaryCommandWords physicalC7CB / SecondaryCommandWords physicalC5D3: inline dispatch removes caller return and redirects by encoded selector. B18 must enumerate supported opcode classes and all command parameter consumers, including FC..FF control commands and unresolved C522..C524.
- Screen script records17B0A, status records1762C/1787C and scene initialization173DC: enumerate every pointer/selector reachable from title, selection, ordinary gameplay, special rooms, all ending variations and terminal screen. Sentinel and count termination are format-specific.
- No table target or data interval is excluded for lack of observed execution. Numeric banked calls (e.g. bank5 $8000/$B580/$B800/$BAB0, bank6 $8000/$AE20/$AE60/$AEA0, bank4 $BE00) must resolve by current bank at each caller; the symbolic-use index alone does not resolve them.

## Important unequal selector domains

The StageSpawn pointer plane starts are B050/B059/B062/B06B/B074/B07D/B086/B08F/B098/B0A1: **nine** entries per plane, not ten. StageTerrainBanks/StageInitialRoomIndices/StageInitialRoomRows/StageInitialPlayerX start DA10/DA19/DA22/DA2B: also nine-entry strides. Layout rows/columns and room-list pointer planes have **ten** entries. Ending selector9 has a layout but must not be sent through a presumed ten-entry spawn/initialization table. B10/B13/B16/B17 must trace caller domains, including selectors6..8 and all scene routes, and expose supported states explicitly. This is a source-visible boundary, not an exclusion of any route.

## Stage layout and spawn pointer joins

All addresses below are CPU addresses in their stated mapper bank; physical mapping follows the inventory convention. Dimensions are source table quantities; exact unit interpretation/bounds are contract work.

| Selector | Layout rows/columns | Room-list pointer (bank7) | Main spawn pointer domains (bank6) |
|---|---|---|---|
| `00` | 7/8 | `$CE6B` | XHi `$B205`, XLo `$B2BA`, Room `$B424`, Y `$B36F`, Template `$B42B` |
| `01` | 8/8 | `$CEA3` | XHi `$B4E0`, XLo `$B57C`, Room `$B6B4`, Y `$B618`, Template `$B6BC` |
| `02` | 6/8 | `$CEE3` | XHi `$B758`, XLo `$B7D6`, Room `$B8D2`, Y `$B854`, Template `$B8D8` |
| `03` | 6/8 | `$CF13` | XHi `$B956`, XLo `$B9CD`, Room `$BABB`, Y `$BA44`, Template `$BAC1` |
| `04` | 6/8 | `$CF43` | XHi `$BB38`, XLo `$BBEE`, Room `$BD5A`, Y `$BCA4`, Template `$BD60` |
| `05` | 1/1 | `$CF73` | XHi `$B0AA`, XLo `$B0AA`, Room `$B0AA`, Y `$B0AA`, Template `$B0AA` |
| `06` | 1/1 | `$CF73` | XHi `$B0AA`, XLo `$B0AA`, Room `$B0AA`, Y `$B0AA`, Template `$B0AA` |
| `07` | 1/1 | `$CF74` | XHi `$B0AA`, XLo `$B0AA`, Room `$B0AA`, Y `$B0AA`, Template `$B0AA` |
| `08` | 1/4 | `$CF75` | XHi `$B0AA`, XLo `$B0AA`, Room `$B0AA`, Y `$B0AA`, Template `$B0AA` |
| `09` | 1/6 | `$CF79` | no ninth-index entry: prove ending caller separation |
