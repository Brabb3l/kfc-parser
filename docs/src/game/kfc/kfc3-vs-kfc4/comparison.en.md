# KFC3/KFC4 Parser Comparison

Report date: 2026-10-15

This README documents a sample comparison between an original KFC3 backup and the Enshrouded KFC4 file installed on October 15, 2026. It focuses on types present in these two specific packages. The reflection database contains 12,646 type definitions. These tables do not enumerate the entire registry. They cover every type observed in the KFC3/KFC4 packages and the indexed counts for each.

## Summary

| Metric | KFC3 | KFC4 | Delta |
| --- | ---: | ---: | ---: |
| Resource entries | 77,810 | 91,812 | 14,002 |
| Recognized entries | 77,275 | 91,812 | 14,537 |
| Recognized types | 129 | 146 | 17 |
| GUIDs, counted per type | 68,626 | 83,128 | 14,502 |
| Unknown entries | 535 | 0 | -535 |
| Content entries | 85,927 | 101,459 | 15,532 |
| Resource chunks | 82 | 58 | -24 |
| Containers | 32 | 55 | 23 |

The KFC4 package contains 14,002 more resource IDs and 17 more recognized types. In the KFC3 index, 535 resource IDs across two type hashes are absent from the reflection registry used for this comparison. KFC4 has no unknown resource IDs in this comparison.

## Inputs and parser revisions

| Package | File and build | Source and SHA-256 |
| --- | --- | --- |
| KFC3 | `enshrouded.kfc`, 1076226\|^/game38/branches/ea_update_08\|2026-06-29T10:27:39.052394Z | KFC3 original from a ShroudForge backup, `ef5f2d74dafe01d8acf82154d42f887e28787b45ab9f62e2dd3809a2b24968da` |
| KFC4 | `enshrouded.kfc`, 1267041\|^/game38/branches/release_1.0\|2026-10-15T17:07:04.598331Z | Local game installation, `13ca8dace2ddc52e6f83800f6bbd70ef1b1ecc052ed2954ef147eede2b655ec9` |

The builds differ. KFC3 is dated June 29, 2026, and KFC4 is dated October 15, 2026. Count changes between them are not a controlled measurement of parser accuracy. The KFC3 index file is 8,278,016 bytes. KFC4 is 9,392,128 bytes.

The old parser revision `f201f76` predates KFC4 support. The current parser revision `a404b15` accepts KFC3 and KFC4 magic values.

## What the statistics count

**Resource ID / entry.** One item in the resource index. A GUID may be split into multiple resource IDs or parts.

**Object / GUID.** The unique GUID within a type. This README sums unique GUIDs per type. It is not a global GUID count deduplicated across types.

**Recognized type.** The 32-bit type hash on a resource ID could be mapped to a qualified type name in the reflection database extracted from the installed Enshrouded executable.

**Content entry.** An indexed, hashed content block. It is separate from the typed resource IDs.

**Chunk and container.** Compressed resource sections and data containers. Summed container sizes are logical payload sizes from the KFC index, not the total size of the installation directory.

**MiB by type.** Sum of resource sizes recorded in the KFC index. This is not the JSON export size or the compressed on-disk size.

## Important recognized types

The following descriptions are based on qualified type names and field names in the reflection schema. They describe the schema, not verified in-game behavior.

### Gameplay, templates, and inventory

`keen::ecs::TemplateResource` is the second largest KFC4 type with 12,267 entries. Reflection fields include `name`, `family`, `proxyModelId`, and `components`, indicating ECS entity templates with component lists. `keen::ItemInfo` has 4,262 entries and 64 fields. Its schema includes identifiers, stack size, rarity, display text, category, equipment, costs, armor, fuel, commerce references, perks, and fishing setups. `keen::Perk` appears 365 times. `keen::ecs::DefaultInventoryResource` holds a root group and appears 420 times. Names and fields describe schemas, not the behavior of individual records.

### Audio, music, and voice

`keen::SoundResource` is the most common KFC4 type with 16,472 entries. Its eight reflection fields cover channel configuration, format, sample rate, frame count, duration, data, and streamed data. `keen::SoundContainerResource` has 6,940 entries organized through an `entries` field. The package also contains 1,273 `VoiceoverResource` entries, 15 `MidiResource` entries, and 15 `MidiSongResource` entries. Ambient, water, reverb, and voice-pack collections mostly appear as single shared resources.

### Graphics, models, and textures

`keen::RenderModel` has 10,470 entries. Its fields cover bounding volumes, materials, meshes, LODs, hierarchy, blend shapes, geometry, and skinning data. `keen::RenderMaterialResource` appears 3,458 times and references effects, image data, and texture feedbacks. `keen::UiTextureResource` has 5,699 resource IDs across 5,493 GUIDs. `keen::RenderTextureResource` has 304 entries across 265 GUIDs. The difference comes from multiple resource parts per GUID. `keen::CardClusterResource`, `keen::RenderModelChunkModelResource`, and `keen::RenderClothColliderResource` describe specialized model data.

### Animation and effects

The 5,459 `keen::JointAnimation` entries contain animation data associated with hierarchies and joints. There are also 2,894 `AnimationSequenceContainer` and 2,899 `SequenceResource` entries. `keen::actor::ActorSequenceResource` connects sequence IDs and subsequences. Effects include 1,573 `VfxResource` entries and 638 `VfxMeshResource` entries. The VFX schema references particles, models, meshes, decals, images, sounds, curves, and parameters. `keen::ImpactProgram` has 570 entries with code, shutdown code, data layout, and query calls. `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` appears 279 times.

### World, scenes, voxels, and water

`keen::SceneResource` appears five times and references nodes, models, lights, cameras, VFX, sounds, and entity chunks. `SceneEntityChunkResource`, `RenderModelChunkModelResource`, and `SceneWaterEntityResource` each have 1,609 resource IDs but only five unique GUIDs. These are heavily segmented resources, not 1,609 independent scenes. `WaterChunkResource` and `WaterDisplacingResource` also have five GUIDs each with about 1,600 parts. Voxel and fog types expose world origin, dimensions, materials, tiles, and voxel data.

### Global registries and configuration

Many types appear once but contain large registries or global game configuration. Examples include `keen::BalancingTable` with 88 fields, `keen::Game38ClientResources` with references to client assets, `keen::Game38ServerResources`, `keen::ItemRegistryResource`, `keen::RecipeRegistryResource`, `keen::GameKnowledgeResource`, `keen::GameKnowledgeQueryResourceDb`, `keen::SkillTreeResource`, `keen::JournalRegistryResource`, `keen::RenderResources`, and `keen::WeatherSystemResource`. Entry count alone is not a measure of importance.

## Types with the most KFC4 entries

| Type | KFC3 entries | KFC4 entries | Delta entries | KFC4 GUIDs |
| --- | ---: | ---: | ---: | ---: |
| `keen::SoundResource` | 10,898 | 16,472 | 5,574 | 16,472 |
| `keen::ecs::TemplateResource` | 10,745 | 12,267 | 1,522 | 12,267 |
| `keen::RenderModel` | 12,713 | 10,470 | -2,243 | 10,470 |
| `keen::SoundContainerResource` | 3,507 | 6,940 | 3,433 | 6,940 |
| `keen::UiTextureResource` | 4,888 | 5,699 | 811 | 5,493 |
| `keen::JointAnimation` | 4,628 | 5,459 | 831 | 5,459 |
| `keen::ItemInfo` | 3,609 | 4,262 | 653 | 4,262 |
| `keen::RenderMaterialResource` | 2,982 | 3,458 | 476 | 3,458 |
| `keen::SequenceResource` | 2,629 | 2,899 | 270 | 2,899 |
| `keen::AnimationSequenceContainer` | 2,624 | 2,894 | 270 | 2,894 |
| `keen::NotifierResource` | 2,051 | 2,319 | 268 | 2,319 |
| `keen::actor::ActorSequenceResource` | 2,202 | 2,157 | -45 | 2,157 |
| `keen::RenderModelChunkModelResource` | 1,609 | 1,609 | 0 | 5 |
| `keen::SceneEntityChunkResource` | 1,609 | 1,609 | 0 | 5 |
| `keen::SceneWaterEntityResource` | 1,609 | 1,609 | 0 | 5 |
| `keen::WaterChunkResource` | 1,604 | 1,604 | 0 | 5 |
| `keen::WaterDisplacingResource` | 1,604 | 1,604 | 0 | 5 |
| `keen::VfxResource` | 1,324 | 1,573 | 249 | 1,573 |
| `keen::VoiceoverResource` | 2 | 1,273 | 1,271 | 1,273 |
| `keen::VfxMeshResource` | 0 | 638 | 638 | 638 |

Resource IDs and GUIDs differ substantially for segmented types. For example, 5,699 `UiTextureResource` entries map to 5,493 GUIDs. The 1,609 entries for each scene-chunk type map to five GUIDs each. Comparing entry counts alone would mistake this segmentation for object growth.

## Types present in only one package

Present in KFC4 but absent from the KFC3 index: `keen::AtmosphereResource`, `keen::BiomeMapResource`, `keen::CharacterScenePostProcessingResource`, `keen::CommerceProductCollection`, `keen::LogicSystemResource`, `keen::Ps5ActivityCollection`, `keen::SoundReverbCollectionResource`, `keen::SoundReverbEarlyReflectionsResource`, `keen::SoundReverbLateReflectionsResource`, `keen::SoundReverbPresetResource`, `keen::SoundReverbPresetsResource`, `keen::VfxCurveResource`, `keen::VfxJointCollectionResource`, `keen::VfxMeshResource`, `keen::anim_graph::BlendProfileResource`, `keen::anim_graph::tool_graph::FeetIKSettings`, `keen::ecs::EntityProxyModelSetResource`, `keen::ecs::TargetSelectionSettings`.
Present in KFC3 but absent from the KFC4 package: `UNKNOWN_47a54aff`, `UNKNOWN_8bea0afc`, `keen::BuffType`.

The two unknown KFC3 hashes are `UNKNOWN_47a54aff` with 534 entries and `UNKNOWN_8bea0afc` with one entry. The complete counts for added and missing types are in the catalog below.

Eighteen recognized types occur only in the KFC4 index. `keen::BuffType` occurs only in the KFC3 index. This yields the net increase from 129 to 146 recognized types. The two unknown KFC3 hashes are not included in those type totals.

## Full catalog of types observed in the packages

This table covers the union of types found in both package indexes. The two unknown hashes are listed as `UNKNOWN_...`. Counts are zero when a type is absent from that package.

| Type | KFC3 IDs | KFC3 GUIDs | KFC4 IDs | KFC4 GUIDs | Delta IDs | Delta GUIDs | KFC3 MiB | KFC4 MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `keen::SoundResource` | 10,898 | 10,898 | 16,472 | 16,472 | 5,574 | 5,574 | 45.14 | 50.82 |
| `keen::ecs::TemplateResource` | 10,745 | 10,745 | 12,267 | 12,267 | 1,522 | 1,522 | 36.55 | 41.69 |
| `keen::RenderModel` | 12,713 | 12,713 | 10,470 | 10,470 | -2,243 | -2,243 | 119.79 | 23.15 |
| `keen::SoundContainerResource` | 3,507 | 3,507 | 6,940 | 6,940 | 3,433 | 3,433 | 0.75 | 1.37 |
| `keen::UiTextureResource` | 4,888 | 4,682 | 5,699 | 5,493 | 811 | 811 | 0.32 | 0.49 |
| `keen::JointAnimation` | 4,628 | 4,628 | 5,459 | 5,459 | 831 | 831 | 128.90 | 160.84 |
| `keen::ItemInfo` | 3,609 | 3,609 | 4,262 | 4,262 | 653 | 653 | 9.56 | 12.14 |
| `keen::RenderMaterialResource` | 2,982 | 2,982 | 3,458 | 3,458 | 476 | 476 | 1.67 | 3.47 |
| `keen::SequenceResource` | 2,629 | 2,629 | 2,899 | 2,899 | 270 | 270 | 1.93 | 2.28 |
| `keen::AnimationSequenceContainer` | 2,624 | 2,624 | 2,894 | 2,894 | 270 | 270 | 0.06 | 0.07 |
| `keen::NotifierResource` | 2,051 | 2,051 | 2,319 | 2,319 | 268 | 268 | 0.23 | 0.27 |
| `keen::actor::ActorSequenceResource` | 2,202 | 2,202 | 2,157 | 2,157 | -45 | -45 | 4.64 | 4.66 |
| `keen::RenderModelChunkModelResource` | 1,609 | 5 | 1,609 | 5 | 0 | 0 | 8.81 | 7.89 |
| `keen::SceneEntityChunkResource` | 1,609 | 5 | 1,609 | 5 | 0 | 0 | 224.82 | 46.81 |
| `keen::SceneWaterEntityResource` | 1,609 | 5 | 1,609 | 5 | 0 | 0 | 0.04 | 0.06 |
| `keen::WaterChunkResource` | 1,604 | 5 | 1,604 | 5 | 0 | 0 | 0.24 | 0.27 |
| `keen::WaterDisplacingResource` | 1,604 | 5 | 1,604 | 5 | 0 | 0 | 0.39 | 0.78 |
| `keen::VfxResource` | 1,324 | 1,324 | 1,573 | 1,573 | 249 | 249 | 14.33 | 18.17 |
| `keen::VoiceoverResource` | 2 | 2 | 1,273 | 1,273 | 1,271 | 1,271 | 0.00 | 0.11 |
| `keen::VfxMeshResource` | 0 | 0 | 638 | 638 | 638 | 638 | 0.00 | 0.03 |
| `keen::LveAnimation` | 418 | 418 | 578 | 578 | 160 | 160 | 0.33 | 0.48 |
| `keen::RootMotionAnimation` | 418 | 418 | 578 | 578 | 160 | 160 | 1.14 | 1.55 |
| `keen::ImpactProgram` | 496 | 496 | 570 | 570 | 74 | 74 | 0.98 | 1.21 |
| `keen::CardClusterResource` | 512 | 256 | 530 | 265 | 18 | 9 | 0.08 | 0.34 |
| `keen::ecs::DefaultInventoryResource` | 609 | 609 | 420 | 420 | -189 | -189 | 0.11 | 0.17 |
| `keen::Perk` | 233 | 233 | 365 | 365 | 132 | 132 | 0.10 | 0.16 |
| `keen::RenderTextureResource` | 59 | 38 | 304 | 265 | 245 | 227 | 0.00 | 0.04 |
| `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` | 231 | 231 | 279 | 279 | 48 | 48 | 1.77 | 2.58 |
| `keen::MaterialFeedback` | 249 | 249 | 274 | 274 | 25 | 25 | 1.62 | 1.78 |
| `keen::ModelHierarchyResource` | 217 | 217 | 257 | 257 | 40 | 40 | 1.45 | 1.78 |
| `keen::VolumetricFog3ModelResource` | 157 | 6 | 169 | 10 | 12 | 4 | 0.11 | 0.16 |
| `keen::VoxelWorldResource` | 135 | 130 | 145 | 140 | 10 | 10 | 0.88 | 1.25 |
| `keen::VoxelModelResource` | 78 | 78 | 78 | 78 | 0 | 0 | 0.06 | 0.06 |
| `keen::GpcResource` | 67 | 67 | 74 | 74 | 7 | 7 | 0.32 | 0.44 |
| `keen::RenderDecal` | 49 | 49 | 52 | 52 | 3 | 3 | 0.02 | 0.05 |
| `keen::VfxJointCollectionResource` | 0 | 0 | 30 | 30 | 30 | 30 | 0.00 | 0.04 |
| `keen::fishing::FishSpawnTableResource` | 25 | 25 | 30 | 30 | 5 | 5 | 0.00 | 0.01 |
| `keen::RenderClothColliderResource` | 19 | 19 | 27 | 27 | 8 | 8 | 0.01 | 0.01 |
| `keen::BaseAttributeResource` | 55 | 55 | 16 | 16 | -39 | -39 | 0.02 | 0.00 |
| `keen::MidiResource` | 15 | 15 | 15 | 15 | 0 | 0 | 0.15 | 0.15 |
| `keen::MidiSongResource` | 15 | 15 | 15 | 15 | 0 | 0 | 0.00 | 0.00 |
| `keen::VoicePackResource` | 10 | 10 | 10 | 10 | 0 | 0 | 0.00 | 0.00 |
| `keen::anim_graph::BoneMaskResource` | 7 | 7 | 9 | 9 | 2 | 2 | 0.01 | 0.01 |
| `keen::VfxCurveResource` | 0 | 0 | 8 | 8 | 8 | 8 | 0.00 | 0.00 |
| `keen::SamplerInstrumentResource` | 7 | 7 | 7 | 7 | 0 | 0 | 14.57 | 14.57 |
| `keen::SoundReverbPresetResource` | 0 | 0 | 7 | 7 | 7 | 7 | 0.00 | 0.00 |
| `keen::VoxelBlueprintConfig` | 7 | 7 | 7 | 7 | 0 | 0 | 0.00 | 0.00 |
| `keen::fishing::TrashLootTableResource` | 6 | 6 | 7 | 7 | 1 | 1 | 0.00 | 0.00 |
| `keen::BiomeMapResource` | 0 | 0 | 5 | 5 | 5 | 5 | 0.00 | 1.56 |
| `keen::FogVoxelMappingResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.01 | 0.01 |
| `keen::SceneRandomLootResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.78 | 0.94 |
| `keen::SceneResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.32 | 0.38 |
| `keen::SceneWhiteboxLabelResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.31 | 0.37 |
| `keen::VoxelTemperatureResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.62 | 10.59 |
| `keen::VoxelWorldFog3Resource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.00 | 0.00 |
| `keen::WaterWorldResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0.00 | 0.00 |
| `keen::anim_graph::RetargetDataResource` | 6 | 6 | 3 | 3 | -3 | -3 | 0.00 | 0.00 |
| `keen::DetailScatterResource` | 2 | 2 | 2 | 2 | 0 | 0 | 0.24 | 0.18 |
| `keen::IblResource` | 2 | 2 | 2 | 2 | 0 | 0 | 0.00 | 0.00 |
| `keen::WaterSurfaceEffectMaterialResource` | 2 | 2 | 2 | 2 | 0 | 0 | 0.00 | 0.00 |
| `keen::anim_graph::tool_graph::FeetIKSettings` | 0 | 0 | 2 | 2 | 2 | 2 | 0.00 | 0.00 |
| `keen::AchievementDefinitionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.01 |
| `keen::ActorSequenceCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.03 | 0.03 |
| `keen::AmbientCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::AmbientEnvironmentMapResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::AmbientEventSoundsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::AmbientParameterResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::AmbientPostProcessingResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::AmbientSfxCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.23 | 0.13 |
| `keen::AmbientVfxCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.01 |
| `keen::AtmosphereResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::AttributeContainerResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.02 | 0.02 |
| `keen::BalancingTable` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::BuffTypeCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.14 |
| `keen::BuildingMaterialBlendingResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.31 | 0.76 |
| `keen::BuildingMaterialParametersResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.09 | 0.09 |
| `keen::CameraStatesManager` | 1 | 1 | 1 | 1 | 0 | 0 | 0.06 | 0.08 |
| `keen::CameraStatesOverrideConfig` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.01 |
| `keen::CharacterPresetCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0.03 | 0.03 |
| `keen::CharacterScenePostProcessingResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::CommerceProductCollection` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::DefaultLootLabelCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.01 |
| `keen::DevJournalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.15 | 0.18 |
| `keen::EmoteResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::FbUiBundle` | 1 | 1 | 1 | 1 | 0 | 0 | 0.20 | 0.23 |
| `keen::FishingEffectResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::Game38ClientResources` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::Game38ServerResources` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::GameKnowledgeQueryResourceDb` | 1 | 1 | 1 | 1 | 0 | 0 | 1.04 | 1.23 |
| `keen::GameKnowledgeQueryTriggerResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.06 | 0.06 |
| `keen::GameKnowledgeResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.52 | 0.87 |
| `keen::GameSettingsPresetsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::GiVoxelBuildingMaterialResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.04 | 0.08 |
| `keen::GuidRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::ImpactRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.01 |
| `keen::IngameTimeConfig` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::ItemKnowledgeResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.21 | 0.32 |
| `keen::ItemRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.19 | 0.24 |
| `keen::JournalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.15 | 0.18 |
| `keen::LocaTagCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::LogicSystemResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.01 |
| `keen::LootableItemsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.57 | 0.68 |
| `keen::MapMarkerRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::MaterialFeedbackCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 2.07 | 2.62 |
| `keen::MidiSongCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::NpcCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.01 |
| `keen::PerkCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.01 |
| `keen::Ps5ActivityCollection` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::RecipeRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.69 | 0.88 |
| `keen::RenderResources` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::RenderSettingsGpuPresetsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.01 |
| `keen::RenderSystemResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::RenderWaterResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::RenderWeatherResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::SimpleWorldMaterialResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.07 | 0.12 |
| `keen::SkillTreeResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.07 | 0.07 |
| `keen::SlopeDefinition` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::SoundReverbCollectionResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::SoundReverbEarlyReflectionsResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::SoundReverbLateReflectionsResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::SoundReverbPresetsResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::SoundWaterCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::SoundWaterFlowScenariosResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::SoundWaterfallParametersResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::TerraformingEfficiencyRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.03 | 0.04 |
| `keen::TwitchDropCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::UiFontResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::UiMapResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::UiWorldEventTriggerCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::VfxSystemResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::VoicePackDirectoryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::VolumetricFog3Resource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::VoxelBlueprintItemRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.01 |
| `keen::VoxelBlueprintMaterialPoolRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::VoxelMaterialVfxDataResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::WeatherSystemResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.03 | 0.03 |
| `keen::WorkshopRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.01 |
| `keen::WorldMaterialBlending2Resource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.29 | 0.57 |
| `keen::ZoneVisualizationResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.00 | 0.00 |
| `keen::anim_graph::BlendProfileResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::debug::EnemyDebugMenu` | 1 | 1 | 1 | 1 | 0 | 0 | 0.01 | 0.02 |
| `keen::ecs::EntityProxyModelSetResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.01 |
| `keen::ecs::TargetSelectionSettings` | 0 | 0 | 1 | 1 | 1 | 1 | 0.00 | 0.00 |
| `keen::ecs::TemplateCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.16 | 0.19 |
| `keen::enemy::DevEnemyArsenalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.15 | 0.21 |
| `keen::enemy::EnemyArsenalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0.14 | 0.21 |
| `UNKNOWN_47a54aff` | 534 | 534 | 0 | 0 | -534 | -534 | 0.02 | 0.00 |
| `UNKNOWN_8bea0afc` | 1 | 1 | 0 | 0 | -1 | -1 | 0.00 | 0.00 |
| `keen::BuffType` | 224 | 224 | 0 | 0 | -224 | -224 | 0.11 | 0.00 |

## Parser run and limitations

| Parser run | Input | Result |
| --- | --- | --- |
| Old CLI parser `f201f76` | KFC3 backup | The KFC3 header and index were read. Full export stopped at the first resource payload. The program logged “unexpected end of data” followed by an allocation request of 60,363,262,464 bytes. No JSON descriptors were exported. |
| Current parser `a404b15` | KFC4 installation | The KFC4 file, resource IDs, sizes, chunks, and type hashes were read through the current KFC index implementation. This report does not include a full JSON export or field-by-field validation of every descriptor. |

The 535 unknown KFC3 resources are present as index entries. Their two type hashes are not present in the reflection database used here. An unknown label does not mean the resource is absent from the KFC index.

## Sources in this parser repository

The KFC files used for this comparison are not part of this repository. The KFC3 sample came from an original backup created by ShroudForge. The KFC4 sample came from the local game installation on October 15, 2026. The SHA-256 checksums above identify the exact input files.

The [KFC file format](../file.md) and [reflection type format](../types.md) documentation explain the relevant structures. The file header and index records are defined in [`header.rs`](../../../../../crates/kfc-base/src/container/header.rs). KFC index parsing is implemented in [`file.rs`](../../../../../crates/kfc-base/src/container/file.rs). The CLI export path is in [`main.rs`](../../../../../crates/cli-legacy/src/main.rs). Type lookup is implemented in [`registry.rs`](../../../../../crates/kfc-base/src/reflection/registry.rs).
