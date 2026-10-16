# Complete KFC3/KFC4 Package Audit

As of: October 16, 2026

Parser revision: `859dd7a`

Scope: KFC index, resource blob, reflection mapping, deserialization, byte roundtrip, and available content containers.

## Result

The current KFC4 client and dedicated-server packages pass the full package audit. All **91,812** client and **43,939** server resources were read. Every resource passed normal and human-readable deserialization roundtrips byte for byte. The audit also read all **101,459** client and **39,818** server content entries and checked their size and content hash.

The available KFC3 resource blobs were also read in full. For the KFC3 dedicated server, the old server executable maps **82 of 83** type hashes used by the index. The **38,658** resources with a mapped type pass both roundtrips. Hash `f9e86997` is absent from the old server executable reflection and affects **3,941** entries. The later client definition `keen::UiTextureResource` does not fit these KFC3 records and returns `unexpected end of data` for every tested entry.

The KFC3 client index can be read in full, but its matching old client executable is absent from the backup. A complete build-matched client deserialization proof is therefore unavailable. Targeted cross-build checks pass for 535 records and fail for 4,888 `UiTextureResource` records with the later KFC4 client schema. The `.dat` content containers are also absent for both KFC3 packages.

## Four-package comparison

| Metric | KFC3 client | KFC3 dedicated server | KFC4 client | KFC4 dedicated server |
| --- | ---: | ---: | ---: | ---: |
| Format | KFC3 | KFC3 | KFC4 | KFC4 |
| Build tag | `1076226`, Jun 29, 2026 | `1024233`, May 11, 2026 | `1267041`, Oct 15, 2026 | `1267041`, Oct 15, 2026 |
| Resource IDs in index | 77,810 | 42,599 | 91,812 | 43,939 |
| Unique GUIDs per type, including unmapped types | 69,161 | 36,106 | 83,128 | 37,445 |
| Type hashes in index | 131 | 83 | 146 | 85 |
| Type hashes in matching executable registry | not verifiable | 82/83 | 146/146 | 85/85 |
| Content entries in index | 85,927 | 46,399 | 101,459 | 39,818 |
| Resource data bytes read | 679,527,809 bytes | 459,801,643 bytes | 480,999,979 bytes | 348,545,937 bytes |
| Content raw bytes checked | unavailable | unavailable | 51,283,306,250 bytes | 7,771,581,668 bytes |
| Resource chunks | 82 | 55 | 59 | 42 |
| Compressed chunk bytes | 230,285,230 | 143,770,981 | 214,606,707 | 143,811,280 |
| Decompressed chunk bytes | 680,740,656 | 460,049,423 | 482,450,848 | 348,797,082 |
| Container slots in KFC index | 32 | 32 | 64 | 10 |
| Non-empty containers in KFC index | 32 | 32 | 55 | 10 |
| Logical container size in index | 44,586,704,896 bytes | 8,572,538,880 bytes | 51,500,314,624 bytes | 7,862,956,032 bytes |
| Physical `.dat` files present | 0 | 0 | 64 (55 non-empty, 9 empty) | 10 (all non-empty) |
| Total `.dat` file size | absent | absent | 51,500,314,624 bytes | 7,862,956,032 bytes |
| Resources read | 77,810/77,810 | 42,599/42,599 | 91,812/91,812 | 43,939/43,939 |

These are counts from each individual index. They do not establish completeness across different game versions. The KFC3 client index is dated June 29, 2026, while the KFC3 server index is dated May 11, 2026. Comparing those KFC3 package sizes mixes two builds. KFC4 client and server share a build tag but remain separate package indexes. The KFC4 server index has 47.86% of the KFC4 client resource IDs and was still read, deserialized, and reserialized in full.

## Input files and provenance

| Package file | Size | SHA-256 |
| --- | ---: | --- |
| KFC3 client `enshrouded.kfc` | 8,278,016 bytes | `ef5f2d74dafe01d8acf82154d42f887e28787b45ab9f62e2dd3809a2b24968da` |
| KFC3 client `enshrouded.kfc_resources` | 230,449,152 bytes | `970afcbe7181611588f8f9a82c9b5c48f0c35dbcddbbbd02cbab385f29ce72b7` |
| KFC3 server `enshrouded_server.exe` | 22,557,696 bytes | `001c1b40ed091d8c1aee583adde3800d7c858ae2c7f4dff54fca2938b2be1637` |
| KFC3 server `enshrouded_server.kfc` | 4,415,488 bytes | `bbcfce8901a66d6add2526feb172a346e226fe2c4d90f27ab0830a07bc54cd69` |
| KFC3 server `enshrouded_server.kfc_resources` | 143,880,192 bytes | `867875a6e3d151cec384907c4310ad61b462f52b10561d229c7639b6d19eb639` |
| KFC4 client `enshrouded.exe` | 35,323,464 bytes | `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a` |
| KFC4 client `enshrouded.kfc` | 9,392,128 bytes | `6780779e749f48f0020f3192dc30ee24dd4b93e64fbc2b5854466d0af60d95d1` |
| KFC4 client `enshrouded.kfc_resources` | 214,728,704 bytes | `98710ce035c05cde98e4610d00dea75159d106df8aeae9fbcd106526a24d82f9` |
| KFC4 server `enshrouded_server.exe` | 22,721,024 bytes | `4de2acdd3b18e562f129bc488a2daea2da16b00341497304423e1e18c49a83ba` |
| KFC4 server `enshrouded_server.kfc` | 4,259,840 bytes | `035555bcf225b232ff0aa3abb48b18df3d4780a125ddc53e76234c6e7848248b` |
| KFC4 server `enshrouded_server.kfc_resources` | 143,896,576 bytes | `35d8b5635c9eea9d112db788636da4fef21ef70af883cd801c26f4452d32c3ee` |

The KFC3 client files came from `C:\Program Files (x86)\Steam\steamapps\common\Enshrouded\shroudforge\backups\originals\client\34302536-1782843918`. Its backup manifest records the matching old `enshrouded.exe` as 34,302,536 bytes with SHA-256 `af2f5a1227911d8aa06b3908d6bd0211838211cae14ea91099cb57d0df990781`. That executable is absent from the backup and the checked game and desktop directories. The KFC3 server files are in `A:\17 Cloud\OneDrive\Desktop\alte kfc3`. The KFC4 client files came from `C:\Program Files (x86)\Steam\steamapps\common\Enshrouded`. The KFC4 server files came from `C:\Program Files (x86)\Steam\steamapps\common\EnshroudedServer`. The KFC3 server backup contains the index, resource blob, and executable, but no `.dat` files.

## Full validation results

| Package | Raw resources read and length-checked | Exact reflection mapping | Normal byte roundtrip | Human-readable byte roundtrip | Content hash check |
| --- | --- | --- | --- | --- | --- |
| KFC3 client | 77,810/77,810 | Matching client EXE missing | No full run | No full run | 0/85,927, `.dat` files absent |
| KFC3 dedicated server | 42,599/42,599 | 82/83 hashes, 38,658/42,599 resources | 38,658 passed, 3,941 unresolved | 38,658 passed, 3,941 unresolved | 0/46,399, `.dat` files absent |
| KFC4 client | 91,812/91,812 | 146/146 hashes | 91,812 passed | 91,812 passed | 101,459/101,459 passed |
| KFC4 dedicated server | 43,939/43,939 | 85/85 hashes | 43,939 passed | 43,939 passed | 39,818/39,818 passed |

For every resource that was read, the returned byte length matched the reader's calculated resource size. The KFC4 content audit checked expected length, flags, and stored content hash for every entry. The total sizes of the `.dat` files for both KFC4 packages match their indexes' logical container sizes.

The audit iterated resources sequentially with one cursor per package. For each resource it checked the number of bytes read. For every mapped type it ran normal and human-readable deserialization followed by a byte-for-byte comparison. For the available KFC4 containers it read every content entry and checked size, hash, and flags. These tables document the parser and package-data audit. They do not claim an additional launch of the dedicated-server process.

### Reflection graphs

Reflection was freshly extracted from every available executable. Each registry entry's inner types, field types, and attribute types were checked against the extracted registry.

| Executable source | Reflection type definitions | Unique qualified hashes | Duplicate hashes | Type references checked | Invalid references |
| --- | ---: | ---: | ---: | ---: | ---: |
| KFC3 client executable | unavailable | — | — | — | — |
| KFC3 server executable | 12,969 | 12,969 | 0 | 51,979 | 0 |
| KFC4 client executable | 12,646 | 12,646 | 0 | 58,403 | 0 |
| KFC4 server executable | 11,174 | 11,174 | 0 | 46,219 | 0 |

The type counts cover each complete extracted registry. The four tables below also list every type hash actually used by each package index, with resource ID count, distinct GUID count per type, and total bytes read for those resources. Byte totals are uncompressed resource sizes, not exported file sizes.

## Complete type census by package

### KFC3 client

The matching KFC3 client executable is missing. Names in this table are cross-build mappings from the KFC4 client executable. Where it lacks two hashes, the names come from the older KFC3 server executable. These mappings label hashes and do not replace a roundtrip against the missing client executable.

| Type hash | Available type name | Name source | Resource IDs | GUIDs per type | Resource bytes |
| --- | --- | --- | ---: | ---: | ---: |
| `011252ac` | `keen::AchievementDefinitionResource` | KFC4-Client-EXE† | 1 | 1 | 7,895 |
| `0469d60d` | `keen::SoundWaterfallParametersResource` | KFC4-Client-EXE† | 1 | 1 | 372 |
| `05b67eb2` | `keen::RenderClothColliderResource` | KFC4-Client-EXE† | 19 | 19 | 8,208 |
| `0a7e828e` | `keen::MidiSongCollection` | KFC4-Client-EXE† | 1 | 1 | 360 |
| `0dfcca99` | `keen::VoxelBlueprintItemRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 5,823 |
| `0f2c8a96` | `keen::CardClusterResource` | KFC4-Client-EXE† | 512 | 256 | 79,872 |
| `14cfcd23` | `keen::VolumetricFog3Resource` | KFC4-Client-EXE† | 1 | 1 | 212 |
| `1601da91` | `keen::AttributeContainerResource` | KFC4-Client-EXE† | 1 | 1 | 17,333 |
| `171bf8ae` | `keen::WaterChunkResource` | KFC4-Client-EXE† | 1,604 | 5 | 256,640 |
| `177394c7` | `keen::RenderModel` | KFC4-Client-EXE† | 12,713 | 12,713 | 125,609,876 |
| `1be67cef` | `keen::BuffTypeCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 3,592 |
| `1e5bc159` | `keen::RenderSettingsGpuPresetsResource` | KFC4-Client-EXE† | 1 | 1 | 3,688 |
| `21b2a090` | `keen::VoxelTemperatureResource` | KFC4-Client-EXE† | 5 | 5 | 17,428,604 |
| `2309f23e` | `keen::EmoteResource` | KFC4-Client-EXE† | 1 | 1 | 140 |
| `2569d42a` | `keen::SceneWhiteboxLabelResource` | KFC4-Client-EXE† | 5 | 5 | 326,576 |
| `293464a1` | `keen::VoxelMaterialVfxDataResource` | KFC4-Client-EXE† | 1 | 1 | 1,032 |
| `2bc68ebe` | `keen::SceneResource` | KFC4-Client-EXE† | 5 | 5 | 332,616 |
| `2bd83360` | `keen::LootableItemsResource` | KFC4-Client-EXE† | 1 | 1 | 597,920 |
| `3002d39d` | `keen::RenderTextureResource` | KFC4-Client-EXE† | 59 | 38 | 4,719 |
| `3118478f` | `keen::SceneRandomLootResource` | KFC4-Client-EXE† | 5 | 5 | 820,112 |
| `316be6f1` | `keen::MapMarkerRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 3,848 |
| `33ce35ab` | `keen::GiVoxelBuildingMaterialResource` | KFC4-Client-EXE† | 1 | 1 | 40,836 |
| `358470de` | `keen::ModelHierarchyResource` | KFC4-Client-EXE† | 217 | 217 | 1,517,648 |
| `36a32740` | `keen::ZoneVisualizationResource` | KFC4-Client-EXE† | 1 | 1 | 3,372 |
| `39768775` | `keen::ecs::TemplateResource` | KFC4-Client-EXE† | 10,745 | 10,745 | 38,321,217 |
| `39fe80cb` | `keen::VfxSystemResource` | KFC4-Client-EXE† | 1 | 1 | 52 |
| `3b294073` | `keen::DevJournalRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 162,208 |
| `3d429775` | `keen::RenderWaterResource` | KFC4-Client-EXE† | 1 | 1 | 148 |
| `3e786789` | `keen::RenderModelChunkModelResource` | KFC4-Client-EXE† | 1,609 | 5 | 9,241,478 |
| `3eb1bc48` | `keen::ecs::DefaultInventoryResource` | KFC4-Client-EXE† | 609 | 609 | 111,116 |
| `3ed71098` | `keen::anim_graph::RetargetDataResource` | KFC4-Client-EXE† | 6 | 6 | 128 |
| `3f13ff35` | `keen::LocaTagCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 324 |
| `405940de` | `keen::SceneWaterEntityResource` | KFC4-Client-EXE† | 1,609 | 5 | 38,616 |
| `43ae2af5` | `keen::NpcCollection` | KFC4-Client-EXE† | 1 | 1 | 5,554 |
| `45c13b50` | `keen::MaterialFeedbackCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 2,165,988 |
| `47a54aff` | `keen::VfxMeshData` | KFC3-Server-EXE‡ | 534 | 534 | 21,360 |
| `48ca76fc` | `keen::UiWorldEventTriggerCollection` | KFC4-Client-EXE† | 1 | 1 | 40 |
| `4bc88248` | `keen::TwitchDropCollection` | KFC4-Client-EXE† | 1 | 1 | 3,687 |
| `519eff77` | `keen::RenderDecal` | KFC4-Client-EXE† | 49 | 49 | 24,252 |
| `53e9cf8a` | `keen::VoiceoverResource` | KFC4-Client-EXE† | 2 | 2 | 160 |
| `557ceea0` | `keen::SoundWaterCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 32 |
| `55a453fd` | `keen::WaterDisplacingResource` | KFC4-Client-EXE† | 1,604 | 5 | 410,624 |
| `577a5583` | `keen::WaterWorldResource` | KFC4-Client-EXE† | 5 | 5 | 1,380 |
| `5f5382cf` | `keen::RecipeRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 723,757 |
| `5faac3a8` | `keen::VolumetricFog3ModelResource` | KFC4-Client-EXE† | 157 | 6 | 114,552 |
| `600965c3` | `keen::RenderResources` | KFC4-Client-EXE† | 1 | 1 | 1,224 |
| `6084515a` | `keen::Perk` | KFC4-Client-EXE† | 233 | 233 | 104,143 |
| `60b5ed8a` | `keen::JournalRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 161,064 |
| `647628d6` | `keen::GpcResource` | KFC4-Client-EXE† | 67 | 67 | 337,747 |
| `64958357` | `keen::ImpactRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 7,944 |
| `657da0a9` | `keen::FishingEffectResource` | KFC4-Client-EXE† | 1 | 1 | 1,516 |
| `67695f1f` | `keen::VoxelWorldFog3Resource` | KFC4-Client-EXE† | 5 | 5 | 3,332 |
| `680ee2b8` | `keen::VoicePackDirectoryResource` | KFC4-Client-EXE† | 1 | 1 | 168 |
| `6ad893c9` | `keen::DefaultLootLabelCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 15,512 |
| `6ffb1e9f` | `keen::GameSettingsPresetsResource` | KFC4-Client-EXE† | 1 | 1 | 904 |
| `72ab39b6` | `keen::ItemRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 204,033 |
| `737c2c02` | `keen::RenderWeatherResource` | KFC4-Client-EXE† | 1 | 1 | 24 |
| `74b3e434` | `keen::enemy::DevEnemyArsenalRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 159,040 |
| `75dc346b` | `keen::VoxelBlueprintConfig` | KFC4-Client-EXE† | 7 | 7 | 2,534 |
| `75f48072` | `keen::VoxelWorldResource` | KFC4-Client-EXE† | 135 | 130 | 923,848 |
| `7a305fca` | `keen::AmbientEventSoundsResource` | KFC4-Client-EXE† | 1 | 1 | 48 |
| `7dcd6a42` | `keen::AmbientCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 96 |
| `878aa5ad` | `keen::UiFontResource` | KFC4-Client-EXE† | 1 | 1 | 2,352 |
| `88d6b534` | `keen::NotifierResource` | KFC4-Client-EXE† | 2,051 | 2,051 | 238,076 |
| `88e305c6` | `keen::anim_graph::BoneMaskResource` | KFC4-Client-EXE† | 7 | 7 | 5,592 |
| `8b68a480` | `keen::fishing::TrashLootTableResource` | KFC4-Client-EXE† | 6 | 6 | 1,188 |
| `8bea0afc` | `keen::ColorPaletteCollectionResource` | KFC3-Server-EXE‡ | 1 | 1 | 2,608 |
| `8e67ba8b` | `keen::BuffType` | KFC4-Client-EXE† | 224 | 224 | 116,572 |
| `914bf38c` | `keen::SlopeDefinition` | KFC4-Client-EXE† | 1 | 1 | 16 |
| `93983f76` | `keen::CameraStatesManager` | KFC4-Client-EXE† | 1 | 1 | 63,000 |
| `9398e747` | `keen::BalancingTable` | KFC4-Client-EXE† | 1 | 1 | 2,852 |
| `94708371` | `keen::BaseAttributeResource` | KFC4-Client-EXE† | 55 | 55 | 17,175 |
| `963bc796` | `keen::RootMotionAnimation` | KFC4-Client-EXE† | 418 | 418 | 1,191,196 |
| `98dcb6e2` | `keen::AmbientVfxCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 5,784 |
| `9a20f554` | `keen::GameKnowledgeQueryTriggerResource` | KFC4-Client-EXE† | 1 | 1 | 63,764 |
| `9a2e4101` | `keen::VoxelBlueprintMaterialPoolRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 687 |
| `9a6c99e2` | `keen::CharacterPresetCollection` | KFC4-Client-EXE† | 1 | 1 | 29,840 |
| `9fd4a561` | `keen::actor::ActorSequenceResource` | KFC4-Client-EXE† | 2,202 | 2,202 | 4,863,279 |
| `a2a7a8b6` | `keen::ActorSequenceCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 35,240 |
| `a345050e` | `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` | KFC4-Client-EXE† | 231 | 231 | 1,860,544 |
| `a36f5a94` | `keen::IngameTimeConfig` | KFC4-Client-EXE† | 1 | 1 | 40 |
| `a45aaf6f` | `keen::SoundWaterFlowScenariosResource` | KFC4-Client-EXE† | 1 | 1 | 248 |
| `a8ea5924` | `keen::WorkshopRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 4,904 |
| `ad4d5bf7` | `keen::DetailScatterResource` | KFC4-Client-EXE† | 2 | 2 | 255,816 |
| `aee1894b` | `keen::RenderMaterialResource` | KFC4-Client-EXE† | 2,982 | 2,982 | 1,748,474 |
| `b4ac556f` | `keen::LveAnimation` | KFC4-Client-EXE† | 418 | 418 | 343,924 |
| `b5ce8765` | `keen::ItemInfo` | KFC4-Client-EXE† | 3,609 | 3,609 | 10,028,377 |
| `b887ea1f` | `keen::RenderSystemResource` | KFC4-Client-EXE† | 1 | 1 | 232 |
| `b8e417ac` | `keen::AnimationSequenceContainer` | KFC4-Client-EXE† | 2,624 | 2,624 | 63,056 |
| `bc47511c` | `keen::GameKnowledgeResource` | KFC4-Client-EXE† | 1 | 1 | 542,940 |
| `bea8708a` | `keen::BuildingMaterialBlendingResource` | KFC4-Client-EXE† | 1 | 1 | 320,764 |
| `c416a16b` | `keen::WeatherSystemResource` | KFC4-Client-EXE† | 1 | 1 | 31,456 |
| `c6ac5f3f` | `keen::IblResource` | KFC4-Client-EXE† | 2 | 2 | 32 |
| `c90f5d91` | `keen::debug::EnemyDebugMenu` | KFC4-Client-EXE† | 1 | 1 | 15,248 |
| `c9ce5b84` | `keen::TerraformingEfficiencyRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 36,184 |
| `ca6d44a7` | `keen::ItemKnowledgeResource` | KFC4-Client-EXE† | 1 | 1 | 224,960 |
| `cb9cefc0` | `keen::JointAnimation` | KFC4-Client-EXE† | 4,628 | 4,628 | 135,164,848 |
| `cdcad9f3` | `keen::AmbientEnvironmentMapResource` | KFC4-Client-EXE† | 1 | 1 | 256 |
| `cfa6f845` | `keen::Game38ClientResources` | KFC4-Client-EXE† | 1 | 1 | 2,208 |
| `cff86319` | `keen::AmbientSfxCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 236,852 |
| `d459986d` | `keen::AmbientParameterResource` | KFC4-Client-EXE† | 1 | 1 | 184 |
| `d4922d88` | `keen::ImpactProgram` | KFC4-Client-EXE† | 496 | 496 | 1,028,267 |
| `d72c06b7` | `keen::SamplerInstrumentResource` | KFC4-Client-EXE† | 7 | 7 | 15,279,268 |
| `d80fe371` | `keen::ecs::TemplateCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 171,928 |
| `d8ba2a70` | `keen::MaterialFeedback` | KFC4-Client-EXE† | 249 | 249 | 1,698,940 |
| `dbfc162c` | `keen::WorldMaterialBlending2Resource` | KFC4-Client-EXE† | 1 | 1 | 300,616 |
| `dc8fcef9` | `keen::VoicePackResource` | KFC4-Client-EXE† | 10 | 10 | 2,880 |
| `df08aa5a` | `keen::SceneEntityChunkResource` | KFC4-Client-EXE† | 1,609 | 5 | 235,745,172 |
| `df74e485` | `keen::SoundResource` | KFC4-Client-EXE† | 10,898 | 10,898 | 47,333,675 |
| `e10655ee` | `keen::UiMapResource` | KFC4-Client-EXE† | 1 | 1 | 8 |
| `e3c0c0a3` | `keen::MidiResource` | KFC4-Client-EXE† | 15 | 15 | 157,199 |
| `e4053c64` | `keen::FogVoxelMappingResource` | KFC4-Client-EXE† | 5 | 5 | 7,568 |
| `e4b6994f` | `keen::SkillTreeResource` | KFC4-Client-EXE† | 1 | 1 | 73,820 |
| `e53afbaf` | `keen::VoxelModelResource` | KFC4-Client-EXE† | 78 | 78 | 59,506 |
| `e613a80a` | `keen::GameKnowledgeQueryResourceDb` | KFC4-Client-EXE† | 1 | 1 | 1,091,832 |
| `e64ee036` | `keen::VfxResource` | KFC4-Client-EXE† | 1,324 | 1,324 | 15,028,083 |
| `e6bad4ee` | `keen::MidiSongResource` | KFC4-Client-EXE† | 15 | 15 | 720 |
| `e70d6642` | `keen::PerkCollectionResource` | KFC4-Client-EXE† | 1 | 1 | 3,736 |
| `e835be71` | `keen::Game38ServerResources` | KFC4-Client-EXE† | 1 | 1 | 1,696 |
| `ebdb76f1` | `keen::SimpleWorldMaterialResource` | KFC4-Client-EXE† | 1 | 1 | 73,863 |
| `ed6815bb` | `keen::fishing::FishSpawnTableResource` | KFC4-Client-EXE† | 25 | 25 | 4,152 |
| `ef04eb1d` | `keen::CameraStatesOverrideConfig` | KFC4-Client-EXE† | 1 | 1 | 1,408 |
| `f0bca7dc` | `keen::GuidRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 120 |
| `f1d216bb` | `keen::AmbientPostProcessingResource` | KFC4-Client-EXE† | 1 | 1 | 56 |
| `f257640c` | `keen::SoundContainerResource` | KFC4-Client-EXE† | 3,507 | 3,507 | 783,208 |
| `f67d6fc3` | `keen::enemy::EnemyArsenalRegistryResource` | KFC4-Client-EXE† | 1 | 1 | 145,256 |
| `f7452b6b` | `keen::BuildingMaterialParametersResource` | KFC4-Client-EXE† | 1 | 1 | 93,552 |
| `f89ee913` | `keen::SequenceResource` | KFC4-Client-EXE† | 2,629 | 2,629 | 2,028,680 |
| `f9e86997` | `keen::UiTextureResource` | KFC4-Client-EXE†, Cross-Parse scheitert | 4,888 | 4,682 | 338,068 |
| `fae9a8b8` | `keen::WaterSurfaceEffectMaterialResource` | KFC4-Client-EXE† | 2 | 2 | 1,720 |
| `fef918b8` | `keen::FbUiBundle` | KFC4-Client-EXE† | 1 | 1 | 210,900 |

### KFC3 dedicated server

The table uses reflection from the accompanying KFC3 server executable. `keen::UiTextureResource` is only a later cross-build name from the KFC4 client executable. The KFC3 server registry does not contain that hash.

| Type hash | Available type name | Name source | Resource IDs | GUIDs per type | Resource bytes |
| --- | --- | --- | ---: | ---: | ---: |
| `05b67eb2` | `keen::RenderClothColliderResource` | KFC3-Server-EXE | 19 | 19 | 8,208 |
| `0a7e828e` | `keen::MidiSongCollection` | KFC3-Server-EXE | 1 | 1 | 360 |
| `0dfcca99` | `keen::VoxelBlueprintItemRegistryResource` | KFC3-Server-EXE | 1 | 1 | 5,823 |
| `1601da91` | `keen::AttributeContainerResource` | KFC3-Server-EXE | 1 | 1 | 17,333 |
| `171bf8ae` | `keen::WaterChunkResource` | KFC3-Server-EXE | 1,600 | 1 | 256,000 |
| `1be67cef` | `keen::BuffTypeCollectionResource` | KFC3-Server-EXE | 1 | 1 | 3,592 |
| `21b2a090` | `keen::VoxelTemperatureResource` | KFC3-Server-EXE | 1 | 1 | 17,428,364 |
| `2309f23e` | `keen::EmoteResource` | KFC3-Server-EXE | 1 | 1 | 140 |
| `2569d42a` | `keen::SceneWhiteboxLabelResource` | KFC3-Server-EXE | 1 | 1 | 326,512 |
| `293464a1` | `keen::VoxelMaterialVfxDataResource` | KFC3-Server-EXE | 1 | 1 | 1,032 |
| `2bc68ebe` | `keen::SceneResource` | KFC3-Server-EXE | 1 | 1 | 321,368 |
| `2bd83360` | `keen::LootableItemsResource` | KFC3-Server-EXE | 1 | 1 | 597,920 |
| `3002d39d` | `keen::RenderTextureResource` | KFC3-Server-EXE | 38 | 31 | 3,427 |
| `3118478f` | `keen::SceneRandomLootResource` | KFC3-Server-EXE | 1 | 1 | 820,016 |
| `33ce35ab` | `keen::GiVoxelBuildingMaterialResource` | KFC3-Server-EXE | 1 | 1 | 40,836 |
| `358470de` | `keen::ModelHierarchyResource` | KFC3-Server-EXE | 207 | 207 | 1,453,744 |
| `39768775` | `keen::ecs::TemplateResource` | KFC3-Server-EXE | 10,745 | 10,745 | 38,321,217 |
| `3b294073` | `keen::DevJournalRegistryResource` | KFC3-Server-EXE | 1 | 1 | 162,208 |
| `3eb1bc48` | `keen::ecs::DefaultInventoryResource` | KFC3-Server-EXE | 609 | 609 | 111,116 |
| `3ed71098` | `keen::anim_graph::RetargetDataResource` | KFC3-Server-EXE | 6 | 6 | 128 |
| `405940de` | `keen::SceneWaterEntityResource` | KFC3-Server-EXE | 1,601 | 1 | 38,424 |
| `43ae2af5` | `keen::NpcCollection` | KFC3-Server-EXE | 1 | 1 | 5,554 |
| `47a54aff` | `keen::VfxMeshData` | KFC3-Server-EXE | 224 | 224 | 8,960 |
| `4bc88248` | `keen::TwitchDropCollection` | KFC3-Server-EXE | 1 | 1 | 3,687 |
| `53e9cf8a` | `keen::VoiceoverResource` | KFC3-Server-EXE | 2 | 2 | 160 |
| `55a453fd` | `keen::WaterDisplacingResource` | KFC3-Server-EXE | 1,600 | 1 | 409,600 |
| `577a5583` | `keen::WaterWorldResource` | KFC3-Server-EXE | 1 | 1 | 276 |
| `5f5382cf` | `keen::RecipeRegistryResource` | KFC3-Server-EXE | 1 | 1 | 723,757 |
| `5faac3a8` | `keen::VolumetricFog3ModelResource` | KFC3-Server-EXE | 89 | 2 | 111,288 |
| `6084515a` | `keen::Perk` | KFC3-Server-EXE | 233 | 233 | 104,143 |
| `60b5ed8a` | `keen::JournalRegistryResource` | KFC3-Server-EXE | 1 | 1 | 161,064 |
| `64958357` | `keen::ImpactRegistryResource` | KFC3-Server-EXE | 1 | 1 | 7,944 |
| `67695f1f` | `keen::VoxelWorldFog3Resource` | KFC3-Server-EXE | 1 | 1 | 3,236 |
| `6ad893c9` | `keen::DefaultLootLabelCollectionResource` | KFC3-Server-EXE | 1 | 1 | 15,512 |
| `6ffb1e9f` | `keen::GameSettingsPresetsResource` | KFC3-Server-EXE | 1 | 1 | 904 |
| `72ab39b6` | `keen::ItemRegistryResource` | KFC3-Server-EXE | 1 | 1 | 204,033 |
| `74b3e434` | `keen::enemy::DevEnemyArsenalRegistryResource` | KFC3-Server-EXE | 1 | 1 | 159,040 |
| `75dc346b` | `keen::VoxelBlueprintConfig` | KFC3-Server-EXE | 7 | 7 | 2,534 |
| `75f48072` | `keen::VoxelWorldResource` | KFC3-Server-EXE | 127 | 126 | 887,944 |
| `88d6b534` | `keen::NotifierResource` | KFC3-Server-EXE | 2,051 | 2,051 | 238,076 |
| `88e305c6` | `keen::anim_graph::BoneMaskResource` | KFC3-Server-EXE | 7 | 7 | 5,592 |
| `8b68a480` | `keen::fishing::TrashLootTableResource` | KFC3-Server-EXE | 6 | 6 | 1,188 |
| `8bea0afc` | `keen::ColorPaletteCollectionResource` | KFC3-Server-EXE | 1 | 1 | 2,608 |
| `8e67ba8b` | `keen::BuffType` | KFC3-Server-EXE | 224 | 224 | 116,572 |
| `93983f76` | `keen::CameraStatesManager` | KFC3-Server-EXE | 1 | 1 | 63,000 |
| `9398e747` | `keen::BalancingTable` | KFC3-Server-EXE | 1 | 1 | 2,852 |
| `94708371` | `keen::BaseAttributeResource` | KFC3-Server-EXE | 55 | 55 | 17,175 |
| `963bc796` | `keen::RootMotionAnimation` | KFC3-Server-EXE | 418 | 418 | 1,191,196 |
| `9a20f554` | `keen::GameKnowledgeQueryTriggerResource` | KFC3-Server-EXE | 1 | 1 | 63,764 |
| `9a2e4101` | `keen::VoxelBlueprintMaterialPoolRegistryResource` | KFC3-Server-EXE | 1 | 1 | 687 |
| `9a6c99e2` | `keen::CharacterPresetCollection` | KFC3-Server-EXE | 1 | 1 | 29,840 |
| `9fd4a561` | `keen::actor::ActorSequenceResource` | KFC3-Server-EXE | 2,202 | 2,202 | 4,863,279 |
| `a2a7a8b6` | `keen::ActorSequenceCollectionResource` | KFC3-Server-EXE | 1 | 1 | 35,240 |
| `a345050e` | `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` | KFC3-Server-EXE | 230 | 230 | 1,860,108 |
| `a36f5a94` | `keen::IngameTimeConfig` | KFC3-Server-EXE | 1 | 1 | 40 |
| `a8ea5924` | `keen::WorkshopRegistryResource` | KFC3-Server-EXE | 1 | 1 | 4,904 |
| `b4ac556f` | `keen::LveAnimation` | KFC3-Server-EXE | 418 | 418 | 343,924 |
| `b5ce8765` | `keen::ItemInfo` | KFC3-Server-EXE | 3,609 | 3,609 | 10,028,377 |
| `b8e417ac` | `keen::AnimationSequenceContainer` | KFC3-Server-EXE | 2,624 | 2,624 | 63,056 |
| `bc47511c` | `keen::GameKnowledgeResource` | KFC3-Server-EXE | 1 | 1 | 542,940 |
| `c416a16b` | `keen::WeatherSystemResource` | KFC3-Server-EXE | 1 | 1 | 31,456 |
| `c9ce5b84` | `keen::TerraformingEfficiencyRegistryResource` | KFC3-Server-EXE | 1 | 1 | 36,184 |
| `ca6d44a7` | `keen::ItemKnowledgeResource` | KFC3-Server-EXE | 1 | 1 | 224,960 |
| `cb9cefc0` | `keen::JointAnimation` | KFC3-Server-EXE | 4,628 | 4,628 | 135,164,848 |
| `d4922d88` | `keen::ImpactProgram` | KFC3-Server-EXE | 496 | 496 | 1,028,267 |
| `d80fe371` | `keen::ecs::TemplateCollectionResource` | KFC3-Server-EXE | 1 | 1 | 171,928 |
| `d8ba2a70` | `keen::MaterialFeedback` | KFC3-Server-EXE | 173 | 173 | 1,247,840 |
| `df08aa5a` | `keen::SceneEntityChunkResource` | KFC3-Server-EXE | 1,601 | 1 | 235,744,628 |
| `e3c0c0a3` | `keen::MidiResource` | KFC3-Server-EXE | 15 | 15 | 157,199 |
| `e4053c64` | `keen::FogVoxelMappingResource` | KFC3-Server-EXE | 1 | 1 | 4,240 |
| `e4b6994f` | `keen::SkillTreeResource` | KFC3-Server-EXE | 1 | 1 | 73,820 |
| `e53afbaf` | `keen::VoxelModelResource` | KFC3-Server-EXE | 78 | 78 | 59,506 |
| `e613a80a` | `keen::GameKnowledgeQueryResourceDb` | KFC3-Server-EXE | 1 | 1 | 1,091,832 |
| `e6bad4ee` | `keen::MidiSongResource` | KFC3-Server-EXE | 15 | 15 | 720 |
| `e70d6642` | `keen::PerkCollectionResource` | KFC3-Server-EXE | 1 | 1 | 3,736 |
| `e835be71` | `keen::Game38ServerResources` | KFC3-Server-EXE | 1 | 1 | 1,696 |
| `ed6815bb` | `keen::fishing::FishSpawnTableResource` | KFC3-Server-EXE | 25 | 25 | 4,152 |
| `ef04eb1d` | `keen::CameraStatesOverrideConfig` | KFC3-Server-EXE | 1 | 1 | 1,408 |
| `f0bca7dc` | `keen::GuidRegistryResource` | KFC3-Server-EXE | 1 | 1 | 120 |
| `f67d6fc3` | `keen::enemy::EnemyArsenalRegistryResource` | KFC3-Server-EXE | 1 | 1 | 145,256 |
| `f7452b6b` | `keen::BuildingMaterialParametersResource` | KFC3-Server-EXE | 1 | 1 | 93,552 |
| `f89ee913` | `keen::SequenceResource` | KFC3-Server-EXE | 2,629 | 2,629 | 2,028,680 |
| `f9e86997` | `keen::UiTextureResource†` | KFC4-Client-EXE, Cross-Parse scheitert | 3,941 | 3,941 | 283,793 |

### KFC4 client

All names come from the matching KFC4 client executable. Every type hash used by the index is mapped.

| Type hash | Reflection type | Name source | Resource IDs | GUIDs per type | Resource bytes |
| --- | --- | --- | ---: | ---: | ---: |
| `011252ac` | `keen::AchievementDefinitionResource` | passende EXE | 1 | 1 | 8,623 |
| `03c6aeda` | `keen::CharacterScenePostProcessingResource` | passende EXE | 1 | 1 | 96 |
| `0469d60d` | `keen::SoundWaterfallParametersResource` | passende EXE | 1 | 1 | 400 |
| `05b67eb2` | `keen::RenderClothColliderResource` | passende EXE | 27 | 27 | 11,680 |
| `099621b1` | `keen::ecs::TargetSelectionSettings` | passende EXE | 1 | 1 | 384 |
| `0a7e828e` | `keen::MidiSongCollection` | passende EXE | 1 | 1 | 360 |
| `0af758c1` | `keen::SoundReverbLateReflectionsResource` | passende EXE | 1 | 1 | 408 |
| `0dfcca99` | `keen::VoxelBlueprintItemRegistryResource` | passende EXE | 1 | 1 | 5,997 |
| `0f2c8a96` | `keen::CardClusterResource` | passende EXE | 530 | 265 | 356,814 |
| `10d3c781` | `keen::VfxCurveResource` | passende EXE | 8 | 8 | 4,224 |
| `14cfcd23` | `keen::VolumetricFog3Resource` | passende EXE | 1 | 1 | 280 |
| `1601da91` | `keen::AttributeContainerResource` | passende EXE | 1 | 1 | 17,579 |
| `171bf8ae` | `keen::WaterChunkResource` | passende EXE | 1,604 | 5 | 282,304 |
| `177394c7` | `keen::RenderModel` | passende EXE | 10,470 | 10,470 | 24,275,616 |
| `1be67cef` | `keen::BuffTypeCollectionResource` | passende EXE | 1 | 1 | 142,204 |
| `1e5bc159` | `keen::RenderSettingsGpuPresetsResource` | passende EXE | 1 | 1 | 5,776 |
| `21b2a090` | `keen::VoxelTemperatureResource` | passende EXE | 5 | 5 | 44,660,092 |
| `2309f23e` | `keen::EmoteResource` | passende EXE | 1 | 1 | 140 |
| `2569d42a` | `keen::SceneWhiteboxLabelResource` | passende EXE | 5 | 5 | 388,628 |
| `293464a1` | `keen::VoxelMaterialVfxDataResource` | passende EXE | 1 | 1 | 1,032 |
| `2bc68ebe` | `keen::SceneResource` | passende EXE | 5 | 5 | 396,088 |
| `2bd83360` | `keen::LootableItemsResource` | passende EXE | 1 | 1 | 717,484 |
| `3002d39d` | `keen::RenderTextureResource` | passende EXE | 304 | 265 | 38,704 |
| `3118478f` | `keen::SceneRandomLootResource` | passende EXE | 5 | 5 | 988,696 |
| `316be6f1` | `keen::MapMarkerRegistryResource` | passende EXE | 1 | 1 | 4,652 |
| `33ce35ab` | `keen::GiVoxelBuildingMaterialResource` | passende EXE | 1 | 1 | 83,247 |
| `358470de` | `keen::ModelHierarchyResource` | passende EXE | 257 | 257 | 1,866,784 |
| `36a32740` | `keen::ZoneVisualizationResource` | passende EXE | 1 | 1 | 108 |
| `39768775` | `keen::ecs::TemplateResource` | passende EXE | 12,267 | 12,267 | 43,717,771 |
| `39fe80cb` | `keen::VfxSystemResource` | passende EXE | 1 | 1 | 16 |
| `3b294073` | `keen::DevJournalRegistryResource` | passende EXE | 1 | 1 | 190,620 |
| `3d429775` | `keen::RenderWaterResource` | passende EXE | 1 | 1 | 148 |
| `3e786789` | `keen::RenderModelChunkModelResource` | passende EXE | 1,609 | 5 | 8,271,110 |
| `3eb1bc48` | `keen::ecs::DefaultInventoryResource` | passende EXE | 420 | 420 | 175,236 |
| `3ed71098` | `keen::anim_graph::RetargetDataResource` | passende EXE | 3 | 3 | 1,288 |
| `3f13ff35` | `keen::LocaTagCollectionResource` | passende EXE | 1 | 1 | 640 |
| `405940de` | `keen::SceneWaterEntityResource` | passende EXE | 1,609 | 5 | 64,360 |
| `43ae2af5` | `keen::NpcCollection` | passende EXE | 1 | 1 | 5,868 |
| `459be2bf` | `keen::Ps5ActivityCollection` | passende EXE | 1 | 1 | 4,786 |
| `45c13b50` | `keen::MaterialFeedbackCollectionResource` | passende EXE | 1 | 1 | 2,748,904 |
| `488838e1` | `keen::SoundReverbCollectionResource` | passende EXE | 1 | 1 | 48 |
| `48ca76fc` | `keen::UiWorldEventTriggerCollection` | passende EXE | 1 | 1 | 56 |
| `4bc88248` | `keen::TwitchDropCollection` | passende EXE | 1 | 1 | 4,447 |
| `4c7efd93` | `keen::SoundReverbPresetsResource` | passende EXE | 1 | 1 | 632 |
| `519eff77` | `keen::RenderDecal` | passende EXE | 52 | 52 | 56,436 |
| `53e9cf8a` | `keen::VoiceoverResource` | passende EXE | 1,273 | 1,273 | 117,116 |
| `557ceea0` | `keen::SoundWaterCollectionResource` | passende EXE | 1 | 1 | 32 |
| `55a453fd` | `keen::WaterDisplacingResource` | passende EXE | 1,604 | 5 | 821,248 |
| `577a5583` | `keen::WaterWorldResource` | passende EXE | 5 | 5 | 1,380 |
| `5aff8e0c` | `keen::CommerceProductCollection` | passende EXE | 1 | 1 | 184 |
| `5f5382cf` | `keen::RecipeRegistryResource` | passende EXE | 1 | 1 | 927,539 |
| `5faac3a8` | `keen::VolumetricFog3ModelResource` | passende EXE | 169 | 10 | 172,214 |
| `600965c3` | `keen::RenderResources` | passende EXE | 1 | 1 | 1,352 |
| `6084515a` | `keen::Perk` | passende EXE | 365 | 365 | 166,095 |
| `60b5ed8a` | `keen::JournalRegistryResource` | passende EXE | 1 | 1 | 190,620 |
| `6287e034` | `keen::BiomeMapResource` | passende EXE | 5 | 5 | 1,638,544 |
| `647628d6` | `keen::GpcResource` | passende EXE | 74 | 74 | 464,964 |
| `64958357` | `keen::ImpactRegistryResource` | passende EXE | 1 | 1 | 9,128 |
| `64e45568` | `keen::anim_graph::tool_graph::FeetIKSettings` | passende EXE | 2 | 2 | 184 |
| `657da0a9` | `keen::FishingEffectResource` | passende EXE | 1 | 1 | 1,524 |
| `67695f1f` | `keen::VoxelWorldFog3Resource` | passende EXE | 5 | 5 | 3,396 |
| `680ee2b8` | `keen::VoicePackDirectoryResource` | passende EXE | 1 | 1 | 168 |
| `6ad893c9` | `keen::DefaultLootLabelCollectionResource` | passende EXE | 1 | 1 | 15,372 |
| `6ffb1e9f` | `keen::GameSettingsPresetsResource` | passende EXE | 1 | 1 | 904 |
| `72ab39b6` | `keen::ItemRegistryResource` | passende EXE | 1 | 1 | 247,432 |
| `737c2c02` | `keen::RenderWeatherResource` | passende EXE | 1 | 1 | 28 |
| `74b3e434` | `keen::enemy::DevEnemyArsenalRegistryResource` | passende EXE | 1 | 1 | 220,360 |
| `75dc346b` | `keen::VoxelBlueprintConfig` | passende EXE | 7 | 7 | 2,534 |
| `75f48072` | `keen::VoxelWorldResource` | passende EXE | 145 | 140 | 1,313,576 |
| `7a305fca` | `keen::AmbientEventSoundsResource` | passende EXE | 1 | 1 | 48 |
| `7dcd6a42` | `keen::AmbientCollectionResource` | passende EXE | 1 | 1 | 96 |
| `878aa5ad` | `keen::UiFontResource` | passende EXE | 1 | 1 | 2,928 |
| `88d6b534` | `keen::NotifierResource` | passende EXE | 2,319 | 2,319 | 287,748 |
| `88e305c6` | `keen::anim_graph::BoneMaskResource` | passende EXE | 9 | 9 | 7,264 |
| `89b0ccf6` | `keen::SoundReverbPresetResource` | passende EXE | 7 | 7 | 545 |
| `8b68a480` | `keen::fishing::TrashLootTableResource` | passende EXE | 7 | 7 | 2,264 |
| `8e2c1e03` | `keen::VfxMeshResource` | passende EXE | 638 | 638 | 35,728 |
| `914bf38c` | `keen::SlopeDefinition` | passende EXE | 1 | 1 | 16 |
| `93983f76` | `keen::CameraStatesManager` | passende EXE | 1 | 1 | 81,328 |
| `9398e747` | `keen::BalancingTable` | passende EXE | 1 | 1 | 2,976 |
| `94708371` | `keen::BaseAttributeResource` | passende EXE | 16 | 16 | 3,372 |
| `963bc796` | `keen::RootMotionAnimation` | passende EXE | 578 | 578 | 1,628,496 |
| `97740892` | `keen::VfxJointCollectionResource` | passende EXE | 30 | 30 | 42,176 |
| `98dcb6e2` | `keen::AmbientVfxCollectionResource` | passende EXE | 1 | 1 | 6,176 |
| `9a20f554` | `keen::GameKnowledgeQueryTriggerResource` | passende EXE | 1 | 1 | 67,460 |
| `9a2e4101` | `keen::VoxelBlueprintMaterialPoolRegistryResource` | passende EXE | 1 | 1 | 731 |
| `9a6c99e2` | `keen::CharacterPresetCollection` | passende EXE | 1 | 1 | 31,348 |
| `9fd4a561` | `keen::actor::ActorSequenceResource` | passende EXE | 2,157 | 2,157 | 4,890,572 |
| `a2a7a8b6` | `keen::ActorSequenceCollectionResource` | passende EXE | 1 | 1 | 34,520 |
| `a345050e` | `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` | passende EXE | 279 | 279 | 2,706,168 |
| `a36f5a94` | `keen::IngameTimeConfig` | passende EXE | 1 | 1 | 40 |
| `a45aaf6f` | `keen::SoundWaterFlowScenariosResource` | passende EXE | 1 | 1 | 248 |
| `a8ea5924` | `keen::WorkshopRegistryResource` | passende EXE | 1 | 1 | 5,328 |
| `ad4d5bf7` | `keen::DetailScatterResource` | passende EXE | 2 | 2 | 192,820 |
| `aee1894b` | `keen::RenderMaterialResource` | passende EXE | 3,458 | 3,458 | 3,637,145 |
| `b4ac556f` | `keen::LveAnimation` | passende EXE | 578 | 578 | 501,696 |
| `b5ce8765` | `keen::ItemInfo` | passende EXE | 4,262 | 4,262 | 12,724,537 |
| `b887ea1f` | `keen::RenderSystemResource` | passende EXE | 1 | 1 | 304 |
| `b8e417ac` | `keen::AnimationSequenceContainer` | passende EXE | 2,894 | 2,894 | 69,536 |
| `b9a40b67` | `keen::ecs::EntityProxyModelSetResource` | passende EXE | 1 | 1 | 8,776 |
| `bc47511c` | `keen::GameKnowledgeResource` | passende EXE | 1 | 1 | 913,868 |
| `bea8708a` | `keen::BuildingMaterialBlendingResource` | passende EXE | 1 | 1 | 792,524 |
| `c416a16b` | `keen::WeatherSystemResource` | passende EXE | 1 | 1 | 31,392 |
| `c6ac5f3f` | `keen::IblResource` | passende EXE | 2 | 2 | 32 |
| `c83c0398` | `keen::anim_graph::BlendProfileResource` | passende EXE | 1 | 1 | 824 |
| `c90f5d91` | `keen::debug::EnemyDebugMenu` | passende EXE | 1 | 1 | 17,498 |
| `c9ce5b84` | `keen::TerraformingEfficiencyRegistryResource` | passende EXE | 1 | 1 | 36,740 |
| `ca6d44a7` | `keen::ItemKnowledgeResource` | passende EXE | 1 | 1 | 334,360 |
| `cb9cefc0` | `keen::JointAnimation` | passende EXE | 5,459 | 5,459 | 168,652,464 |
| `cdcad9f3` | `keen::AmbientEnvironmentMapResource` | passende EXE | 1 | 1 | 256 |
| `cfa6f845` | `keen::Game38ClientResources` | passende EXE | 1 | 1 | 2,400 |
| `cff86319` | `keen::AmbientSfxCollectionResource` | passende EXE | 1 | 1 | 139,976 |
| `d459986d` | `keen::AmbientParameterResource` | passende EXE | 1 | 1 | 176 |
| `d4922d88` | `keen::ImpactProgram` | passende EXE | 570 | 570 | 1,263,626 |
| `d72c06b7` | `keen::SamplerInstrumentResource` | passende EXE | 7 | 7 | 15,279,268 |
| `d80fe371` | `keen::ecs::TemplateCollectionResource` | passende EXE | 1 | 1 | 196,280 |
| `d8ba2a70` | `keen::MaterialFeedback` | passende EXE | 274 | 274 | 1,869,312 |
| `dbfc162c` | `keen::WorldMaterialBlending2Resource` | passende EXE | 1 | 1 | 593,428 |
| `dc8fcef9` | `keen::VoicePackResource` | passende EXE | 10 | 10 | 2,880 |
| `dddc9acd` | `keen::LogicSystemResource` | passende EXE | 1 | 1 | 9,056 |
| `df08aa5a` | `keen::SceneEntityChunkResource` | passende EXE | 1,609 | 5 | 49,079,813 |
| `df74e485` | `keen::SoundResource` | passende EXE | 16,472 | 16,472 | 53,291,775 |
| `dff86ba2` | `keen::AtmosphereResource` | passende EXE | 1 | 1 | 132 |
| `e10655ee` | `keen::UiMapResource` | passende EXE | 1 | 1 | 8 |
| `e3c0c0a3` | `keen::MidiResource` | passende EXE | 15 | 15 | 157,199 |
| `e4053c64` | `keen::FogVoxelMappingResource` | passende EXE | 5 | 5 | 7,952 |
| `e4b6994f` | `keen::SkillTreeResource` | passende EXE | 1 | 1 | 78,424 |
| `e53afbaf` | `keen::VoxelModelResource` | passende EXE | 78 | 78 | 59,506 |
| `e613a80a` | `keen::GameKnowledgeQueryResourceDb` | passende EXE | 1 | 1 | 1,294,428 |
| `e64ee036` | `keen::VfxResource` | passende EXE | 1,573 | 1,573 | 19,056,813 |
| `e6bad4ee` | `keen::MidiSongResource` | passende EXE | 15 | 15 | 720 |
| `e70d6642` | `keen::PerkCollectionResource` | passende EXE | 1 | 1 | 5,848 |
| `e835be71` | `keen::Game38ServerResources` | passende EXE | 1 | 1 | 1,712 |
| `ebdb76f1` | `keen::SimpleWorldMaterialResource` | passende EXE | 1 | 1 | 126,643 |
| `ed2fcc5e` | `keen::SoundReverbEarlyReflectionsResource` | passende EXE | 1 | 1 | 1 |
| `ed6815bb` | `keen::fishing::FishSpawnTableResource` | passende EXE | 30 | 30 | 5,256 |
| `ef04eb1d` | `keen::CameraStatesOverrideConfig` | passende EXE | 1 | 1 | 5,312 |
| `f0bca7dc` | `keen::GuidRegistryResource` | passende EXE | 1 | 1 | 120 |
| `f1d216bb` | `keen::AmbientPostProcessingResource` | passende EXE | 1 | 1 | 3,576 |
| `f257640c` | `keen::SoundContainerResource` | passende EXE | 6,940 | 6,940 | 1,440,432 |
| `f67d6fc3` | `keen::enemy::EnemyArsenalRegistryResource` | passende EXE | 1 | 1 | 216,904 |
| `f7452b6b` | `keen::BuildingMaterialParametersResource` | passende EXE | 1 | 1 | 91,684 |
| `f89ee913` | `keen::SequenceResource` | passende EXE | 2,899 | 2,899 | 2,391,085 |
| `f9e86997` | `keen::UiTextureResource` | passende EXE | 5,699 | 5,493 | 513,946 |
| `fae9a8b8` | `keen::WaterSurfaceEffectMaterialResource` | passende EXE | 2 | 2 | 1,736 |
| `fef918b8` | `keen::FbUiBundle` | passende EXE | 1 | 1 | 243,544 |

### KFC4 dedicated server

All names come from the matching KFC4 server executable. Every type hash used by the index is mapped.

| Type hash | Reflection type | Name source | Resource IDs | GUIDs per type | Resource bytes |
| --- | --- | --- | ---: | ---: | ---: |
| `05b67eb2` | `keen::RenderClothColliderResource` | passende EXE | 27 | 27 | 11,680 |
| `099621b1` | `keen::ecs::TargetSelectionSettings` | passende EXE | 1 | 1 | 384 |
| `0a7e828e` | `keen::MidiSongCollection` | passende EXE | 1 | 1 | 360 |
| `0dfcca99` | `keen::VoxelBlueprintItemRegistryResource` | passende EXE | 1 | 1 | 5,997 |
| `1601da91` | `keen::AttributeContainerResource` | passende EXE | 1 | 1 | 17,579 |
| `171bf8ae` | `keen::WaterChunkResource` | passende EXE | 1,600 | 1 | 281,600 |
| `1be67cef` | `keen::BuffTypeCollectionResource` | passende EXE | 1 | 1 | 142,204 |
| `21b2a090` | `keen::VoxelTemperatureResource` | passende EXE | 1 | 1 | 44,659,852 |
| `2309f23e` | `keen::EmoteResource` | passende EXE | 1 | 1 | 140 |
| `2569d42a` | `keen::SceneWhiteboxLabelResource` | passende EXE | 1 | 1 | 388,564 |
| `293464a1` | `keen::VoxelMaterialVfxDataResource` | passende EXE | 1 | 1 | 1,032 |
| `2bc68ebe` | `keen::SceneResource` | passende EXE | 1 | 1 | 384,616 |
| `2bd83360` | `keen::LootableItemsResource` | passende EXE | 1 | 1 | 717,484 |
| `3118478f` | `keen::SceneRandomLootResource` | passende EXE | 1 | 1 | 988,600 |
| `358470de` | `keen::ModelHierarchyResource` | passende EXE | 246 | 246 | 1,801,024 |
| `39768775` | `keen::ecs::TemplateResource` | passende EXE | 12,267 | 12,267 | 43,717,771 |
| `3b294073` | `keen::DevJournalRegistryResource` | passende EXE | 1 | 1 | 190,620 |
| `3eb1bc48` | `keen::ecs::DefaultInventoryResource` | passende EXE | 420 | 420 | 175,236 |
| `3ed71098` | `keen::anim_graph::RetargetDataResource` | passende EXE | 3 | 3 | 1,288 |
| `405940de` | `keen::SceneWaterEntityResource` | passende EXE | 1,601 | 1 | 64,040 |
| `43ae2af5` | `keen::NpcCollection` | passende EXE | 1 | 1 | 5,868 |
| `4bc88248` | `keen::TwitchDropCollection` | passende EXE | 1 | 1 | 4,447 |
| `53e9cf8a` | `keen::VoiceoverResource` | passende EXE | 1,273 | 1,273 | 117,116 |
| `55a453fd` | `keen::WaterDisplacingResource` | passende EXE | 1,600 | 1 | 819,200 |
| `577a5583` | `keen::WaterWorldResource` | passende EXE | 1 | 1 | 276 |
| `5aff8e0c` | `keen::CommerceProductCollection` | passende EXE | 1 | 1 | 184 |
| `5f5382cf` | `keen::RecipeRegistryResource` | passende EXE | 1 | 1 | 927,539 |
| `5faac3a8` | `keen::VolumetricFog3ModelResource` | passende EXE | 101 | 6 | 165,142 |
| `6084515a` | `keen::Perk` | passende EXE | 365 | 365 | 166,095 |
| `60b5ed8a` | `keen::JournalRegistryResource` | passende EXE | 1 | 1 | 190,620 |
| `6287e034` | `keen::BiomeMapResource` | passende EXE | 1 | 1 | 1,638,428 |
| `64958357` | `keen::ImpactRegistryResource` | passende EXE | 1 | 1 | 9,128 |
| `64e45568` | `keen::anim_graph::tool_graph::FeetIKSettings` | passende EXE | 2 | 2 | 184 |
| `67695f1f` | `keen::VoxelWorldFog3Resource` | passende EXE | 1 | 1 | 3,300 |
| `6ad893c9` | `keen::DefaultLootLabelCollectionResource` | passende EXE | 1 | 1 | 15,372 |
| `6ffb1e9f` | `keen::GameSettingsPresetsResource` | passende EXE | 1 | 1 | 904 |
| `72ab39b6` | `keen::ItemRegistryResource` | passende EXE | 1 | 1 | 247,432 |
| `74b3e434` | `keen::enemy::DevEnemyArsenalRegistryResource` | passende EXE | 1 | 1 | 220,360 |
| `75dc346b` | `keen::VoxelBlueprintConfig` | passende EXE | 7 | 7 | 2,534 |
| `75f48072` | `keen::VoxelWorldResource` | passende EXE | 137 | 136 | 1,276,136 |
| `88d6b534` | `keen::NotifierResource` | passende EXE | 2,319 | 2,319 | 287,748 |
| `88e305c6` | `keen::anim_graph::BoneMaskResource` | passende EXE | 9 | 9 | 7,264 |
| `8b68a480` | `keen::fishing::TrashLootTableResource` | passende EXE | 7 | 7 | 2,264 |
| `8e2c1e03` | `keen::VfxMeshResource` | passende EXE | 279 | 279 | 15,624 |
| `93983f76` | `keen::CameraStatesManager` | passende EXE | 1 | 1 | 81,328 |
| `9398e747` | `keen::BalancingTable` | passende EXE | 1 | 1 | 2,976 |
| `94708371` | `keen::BaseAttributeResource` | passende EXE | 16 | 16 | 3,372 |
| `963bc796` | `keen::RootMotionAnimation` | passende EXE | 578 | 578 | 1,628,496 |
| `97740892` | `keen::VfxJointCollectionResource` | passende EXE | 4 | 4 | 2,704 |
| `9a20f554` | `keen::GameKnowledgeQueryTriggerResource` | passende EXE | 1 | 1 | 67,460 |
| `9a2e4101` | `keen::VoxelBlueprintMaterialPoolRegistryResource` | passende EXE | 1 | 1 | 731 |
| `9a6c99e2` | `keen::CharacterPresetCollection` | passende EXE | 1 | 1 | 31,348 |
| `9fd4a561` | `keen::actor::ActorSequenceResource` | passende EXE | 2,157 | 2,157 | 4,890,572 |
| `a2a7a8b6` | `keen::ActorSequenceCollectionResource` | passende EXE | 1 | 1 | 34,520 |
| `a345050e` | `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` | passende EXE | 278 | 278 | 2,705,044 |
| `a36f5a94` | `keen::IngameTimeConfig` | passende EXE | 1 | 1 | 40 |
| `a8ea5924` | `keen::WorkshopRegistryResource` | passende EXE | 1 | 1 | 5,328 |
| `b4ac556f` | `keen::LveAnimation` | passende EXE | 578 | 578 | 501,696 |
| `b5ce8765` | `keen::ItemInfo` | passende EXE | 4,262 | 4,262 | 12,724,537 |
| `b8e417ac` | `keen::AnimationSequenceContainer` | passende EXE | 2,894 | 2,894 | 69,536 |
| `b9a40b67` | `keen::ecs::EntityProxyModelSetResource` | passende EXE | 1 | 1 | 8,776 |
| `bc47511c` | `keen::GameKnowledgeResource` | passende EXE | 1 | 1 | 913,868 |
| `c416a16b` | `keen::WeatherSystemResource` | passende EXE | 1 | 1 | 31,392 |
| `c83c0398` | `keen::anim_graph::BlendProfileResource` | passende EXE | 1 | 1 | 824 |
| `c9ce5b84` | `keen::TerraformingEfficiencyRegistryResource` | passende EXE | 1 | 1 | 36,740 |
| `ca6d44a7` | `keen::ItemKnowledgeResource` | passende EXE | 1 | 1 | 334,360 |
| `cb9cefc0` | `keen::JointAnimation` | passende EXE | 5,459 | 5,459 | 168,652,464 |
| `d4922d88` | `keen::ImpactProgram` | passende EXE | 570 | 570 | 1,263,626 |
| `d80fe371` | `keen::ecs::TemplateCollectionResource` | passende EXE | 1 | 1 | 196,280 |
| `d8ba2a70` | `keen::MaterialFeedback` | passende EXE | 192 | 192 | 1,375,796 |
| `dddc9acd` | `keen::LogicSystemResource` | passende EXE | 1 | 1 | 9,056 |
| `df08aa5a` | `keen::SceneEntityChunkResource` | passende EXE | 1,601 | 1 | 49,078,693 |
| `e3c0c0a3` | `keen::MidiResource` | passende EXE | 15 | 15 | 157,199 |
| `e4053c64` | `keen::FogVoxelMappingResource` | passende EXE | 1 | 1 | 4,624 |
| `e4b6994f` | `keen::SkillTreeResource` | passende EXE | 1 | 1 | 78,424 |
| `e53afbaf` | `keen::VoxelModelResource` | passende EXE | 78 | 78 | 59,506 |
| `e613a80a` | `keen::GameKnowledgeQueryResourceDb` | passende EXE | 1 | 1 | 1,294,428 |
| `e6bad4ee` | `keen::MidiSongResource` | passende EXE | 15 | 15 | 720 |
| `e70d6642` | `keen::PerkCollectionResource` | passende EXE | 1 | 1 | 5,848 |
| `e835be71` | `keen::Game38ServerResources` | passende EXE | 1 | 1 | 1,712 |
| `ed6815bb` | `keen::fishing::FishSpawnTableResource` | passende EXE | 30 | 30 | 5,256 |
| `ef04eb1d` | `keen::CameraStatesOverrideConfig` | passende EXE | 1 | 1 | 5,312 |
| `f0bca7dc` | `keen::GuidRegistryResource` | passende EXE | 1 | 1 | 120 |
| `f67d6fc3` | `keen::enemy::EnemyArsenalRegistryResource` | passende EXE | 1 | 1 | 216,904 |
| `f89ee913` | `keen::SequenceResource` | passende EXE | 2,899 | 2,899 | 2,391,085 |

## KFC3 edge cases

| Package and type hash | Resource count | Result |
| --- | ---: | --- |
| KFC3 client `47a54aff` | 534 | `keen::VfxMeshData` from the older KFC3 server executable. All 534 resources passed both roundtrips with this cross-build reflection. |
| KFC3 client `8bea0afc` | 1 | `keen::ColorPaletteCollectionResource` from the older KFC3 server executable. Both roundtrips passed. |
| KFC3 client `f9e86997` | 4,888 | KFC4 client reflection calls this `keen::UiTextureResource`. All 4,888 resources were read. Normal and human-readable parsing both failed with `unexpected end of data`. |
| KFC3 dedicated server `f9e86997` | 3,941 | The KFC3 server executable does not know this hash. Its ResourceBundle internal-hash value `a863690a` also has no registry match. The later KFC4 client definition `keen::UiTextureResource` failed with `unexpected end of data` for every entry. |

The full KFC3 client attempt with a KFC4 client registry stopped when one cross-version record requested a 34,091,302,912-byte allocation. This run is not counted as a pass. The historical client executable is absent, so KFC3 client deserialization remains open. The targeted checks above limit the result to concrete resources instead of applying the newer type tree to the entire old package.

## Large-resource size correction

The KFC file stores resource sizes in 24 bits. The reader derives the missing amount from the next resource offset. The affected `keen::VoxelTemperatureResource` appears in all four indexes.

| Package | Stored size | Calculated actual size | Recovered amount |
| --- | ---: | ---: | ---: |
| KFC3 client | 651,148 bytes | 17,428,364 bytes | 16,777,216 bytes |
| KFC3 dedicated server | 651,148 bytes | 17,428,364 bytes | 16,777,216 bytes |
| KFC4 client | 11,105,420 bytes | 44,659,852 bytes | 33,554,432 bytes |
| KFC4 dedicated server | 11,105,420 bytes | 44,659,852 bytes | 33,554,432 bytes |

GUID: `509feadb-4c60-425f-9c7c-deeefd9b6920_21b2a090_3`. The KFC4 resource in both packages passed the size, deserialization, and byte-roundtrip checks.

## What is still needed for a complete KFC3 proof

- The KFC3 client executable listed in the backup manifest, SHA-256 `af2f5a1227911d8aa06b3908d6bd0211838211cae14ea91099cb57d0df990781`.
- The `.dat` containers belonging to both KFC3 indexes, for the 85,927 client and 46,399 server content entries.
- A reflection definition matching KFC3 server resource hash `f9e86997`, or a confirmed rule that these 3,941 `UiTextureResource`-like records are handled as raw data by the dedicated server.

The KFC4 indexes and available KFC3 server executable were checked on October 16, 2026. Temporary audit tests were removed after measurement. Only the fully rebuilt comparison documents remain as working-tree changes.