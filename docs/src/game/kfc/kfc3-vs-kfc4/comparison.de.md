# KFC3/KFC4-Parservergleich

Stand: 15.10.2026

Diese README dokumentiert einen Stichprobenvergleich zwischen einer originalen KFC3-Sicherung und der am 15. Oktober 2026 installierten KFC4-Datei von Enshrouded. Sie konzentriert sich auf Typen, die in diesen beiden konkreten Paketen vorkommen. Die Reflection-Datenbank enthält 12.646 Typdefinitionen. Die Tabellen hier führen nicht alle davon auf, sondern alle im KFC3/KFC4-Bestand beobachteten Typen und ihre Mengen.

## Kurzfassung

| Kennzahl | KFC3 | KFC4 | Differenz |
| --- | ---: | ---: | ---: |
| Ressourceneinträge | 77.810 | 91.812 | 14.002 |
| Erkannte Einträge | 77.275 | 91.812 | 14.537 |
| Erkannte Typen | 129 | 146 | 17 |
| GUIDs, je Typ gezählt | 68.626 | 83.128 | 14.502 |
| Unbekannte Einträge | 535 | 0 | -535 |
| Content-Einträge | 85.927 | 101.459 | 15.532 |
| Ressourcen-Chunks | 82 | 58 | -24 |
| Container | 32 | 55 | 23 |

Das KFC4-Paket enthält 14.002 mehr Ressourcen-IDs und 17 mehr erkannte Typen. Im KFC3-Index sind 535 Ressourcen-IDs aus zwei Typ-Hashes nicht in der verwendeten Reflection-Datenbank vorhanden. KFC4 hat in diesem Abgleich keine unbekannten Ressourcen-IDs.

## Datenbasis und Parserstände

| Paket | Datei und Build | Quelle und SHA-256 |
| --- | --- | --- |
| KFC3 | `enshrouded.kfc`, 1076226\|^/game38/branches/ea_update_08\|2026-06-29T10:27:39.052394Z | KFC3-Original aus ShroudForge-Backup, `ef5f2d74dafe01d8acf82154d42f887e28787b45ab9f62e2dd3809a2b24968da` |
| KFC4 | `enshrouded.kfc`, 1267041\|^/game38/branches/release_1.0\|2026-10-15T17:07:04.598331Z | Lokale Installation, `13ca8dace2ddc52e6f83800f6bbd70ef1b1ecc052ed2954ef147eede2b655ec9` |

Die Builds sind verschieden. KFC3 ist vom 29. Juni 2026, KFC4 vom 15. Oktober 2026. Mengenänderungen zwischen ihnen sind deshalb keine kontrollierte Parser-Genauigkeitsmessung. Das KFC3-Backup umfasst 8.278.016 Byte, KFC4 umfasst 9.392.128 Byte.

Der alte Parserstand `f201f76` liegt vor dem KFC4-Support. Der aktuelle Parserstand `a404b15` akzeptiert KFC3- und KFC4-Dateiköpfe.

## Was wurde gezählt?

**Ressourcen-ID / Eintrag.** Ein Element im Ressourcenindex. Eine GUID kann in mehrere Ressourcen-IDs beziehungsweise Teile zerlegt sein.

**Objekt/GUID.** Die eindeutige GUID innerhalb eines Typs. Die README addiert eindeutige GUIDs je Typ. Das ist keine globale, typübergreifend deduplizierte GUID-Zahl.

**Erkannter Typ.** Der 32-Bit-Typ-Hash einer Ressourcen-ID ließ sich in der Reflection-Datenbank des installierten Enshrouded-Programms einer qualifizierten Typbezeichnung zuordnen.

**Content-Eintrag.** Ein indexierter, gehashter Inhaltsblock. Das ist ein anderer Bestand als die typisierten Ressourcen-IDs.

**Chunk und Container.** Komprimierte Ressourcenabschnitte beziehungsweise Datencontainer. Die aufsummierten Containergrößen sind logische Nutzdaten aus dem KFC-Index und keine Dateigröße des kompletten Installationsordners.

**MiB pro Typ.** Summe der im KFC-Index eingetragenen Ressourcengrößen. Das ist weder eine Messung der JSON-Exportgröße noch ein Maß für komprimierte Dateigröße.

## Wichtige erkannte Typen

Die folgenden Beschreibungen stützen sich auf qualifizierte Typnamen und die Feldnamen im Reflection-Schema. Wo das Format kein Klartextfeld zeigt, beschreiben sie die Struktur und keine geprüfte Spielwirkung.

### Gameplay, Vorlagen und Inventar

`keen::ecs::TemplateResource` ist mit 12.267 KFC4-Einträgen der zweitgrößte Typ. Die Reflection-Daten nennen Felder wie `name`, `family`, `proxyModelId` und `components`. Das deutet auf Vorlagen für ECS-Entitäten mit Komponentenlisten. `keen::ItemInfo` hat 4.262 Einträge und 64 Felder. Dazu zählen Kennungen, Stapelgröße, Seltenheit, Anzeigenamen, Beschreibung, Kategorie, Ausrüstung, Kosten, Rüstung, Brennstoff, Handelsbezüge, Perks und Fischerei-Setups. `keen::Perk` kommt 365-mal vor. `keen::ecs::DefaultInventoryResource` enthält eine Root-Gruppe und liegt 420-mal vor. Die Typnamen und Felder beschreiben Strukturen, nicht die Wirkung einzelner Datensätze.

### Audio, Musik und Sprache

`keen::SoundResource` ist mit 16.472 Einträgen der häufigste KFC4-Typ. Die acht Reflection-Felder umfassen Kanalkonfiguration, Format, Abtastrate, Framezahl, Dauer, Daten und gestreamte Daten. `keen::SoundContainerResource` bündelt 6.940 Einträge in einer `entries`-Liste. Hinzu kommen 1.273 `VoiceoverResource`-Einträge, 15 `MidiResource`- und 15 `MidiSongResource`-Einträge. Ambient-, Wasser-, Reverb- und Voice-Pack-Sammlungen treten meist als einzelne zentrale Ressourcen auf.

### Grafik, Modelle und Texturen

`keen::RenderModel` enthält 10.470 Einträge. Die Felder umfassen Bounding Volumes, Materialien, Meshes, LODs, Hierarchie, Blend Shapes und Geometrie- oder Skinning-Daten. `keen::RenderMaterialResource` kommt 3.458-mal vor und referenziert unter anderem Effekte, Bilddaten und Texture-Feedbacks. `keen::UiTextureResource` umfasst 5.699 Ressourcen-IDs, aber 5.493 GUIDs. `keen::RenderTextureResource` hat 304 Einträge und 265 GUIDs. Die Unterschiede entstehen durch mehrere Teilressourcen pro GUID. `keen::CardClusterResource`, `keen::RenderModelChunkModelResource` und `keen::RenderClothColliderResource` ergänzen spezialisierte Modelldaten.

### Animationen und Effekte

Die 5.459 `keen::JointAnimation`-Einträge enthalten Animationsdaten pro Hierarchie und Gelenk. Dazu kommen 2.894 `AnimationSequenceContainer`- und 2.899 `SequenceResource`-Einträge. `keen::actor::ActorSequenceResource` verbindet Sequenz-IDs und Teilsequenzen. Für Effekte liegen 1.573 `VfxResource`-Einträge und 638 `VfxMeshResource`-Einträge vor. Die VFX-Struktur referenziert Partikel, Modelle, Meshes, Decals, Bilder, Sounds, Kurven und Parameter. `keen::ImpactProgram` umfasst 570 Einträge mit Code, Shutdown-Code, Datenschema und Query-Aufrufen. `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` liegt 279-mal vor.

### Welt, Szenen, Voxel und Wasser

`keen::SceneResource` kommt fünfmal vor und führt unter anderem Knoten, Modelle, Lichter, Kameras, VFX, Sounds sowie Entitäts-Chunks auf. Die Typen `SceneEntityChunkResource`, `RenderModelChunkModelResource` und `SceneWaterEntityResource` haben jeweils 1.609 Ressourcen-IDs, aber nur fünf eindeutige GUIDs. Das zeigt stark segmentierte Ressourcen, keine 1.609 eigenständigen Szenen. Auch `WaterChunkResource` und `WaterDisplacingResource` haben jeweils fünf GUIDs mit rund 1.600 Teil-Einträgen. Voxel- und Fog-Typen enthalten World-Origin-, Größen-, Material-, Tile- und Voxeldaten.

### Zentrale Kataloge und Konfiguration

Viele Typen erscheinen genau einmal, tragen aber umfangreiche Kataloge oder Spielkonfigurationen. Beispiele sind `keen::BalancingTable` mit 88 Feldern, `keen::Game38ClientResources` mit Verweisen auf Client-Ressourcen, `keen::Game38ServerResources`, `keen::ItemRegistryResource`, `keen::RecipeRegistryResource`, `keen::GameKnowledgeResource`, `keen::GameKnowledgeQueryResourceDb`, `keen::SkillTreeResource`, `keen::JournalRegistryResource`, `keen::RenderResources` und `keen::WeatherSystemResource`. Die Eintragszahl allein ist daher kein Maß für inhaltliche Bedeutung.

## Typen mit den meisten KFC4-Einträgen

| Typ | KFC3 Einträge | KFC4 Einträge | Δ Einträge | KFC4 GUIDs |
| --- | ---: | ---: | ---: | ---: |
| `keen::SoundResource` | 10.898 | 16.472 | 5.574 | 16.472 |
| `keen::ecs::TemplateResource` | 10.745 | 12.267 | 1.522 | 12.267 |
| `keen::RenderModel` | 12.713 | 10.470 | -2.243 | 10.470 |
| `keen::SoundContainerResource` | 3.507 | 6.940 | 3.433 | 6.940 |
| `keen::UiTextureResource` | 4.888 | 5.699 | 811 | 5.493 |
| `keen::JointAnimation` | 4.628 | 5.459 | 831 | 5.459 |
| `keen::ItemInfo` | 3.609 | 4.262 | 653 | 4.262 |
| `keen::RenderMaterialResource` | 2.982 | 3.458 | 476 | 3.458 |
| `keen::SequenceResource` | 2.629 | 2.899 | 270 | 2.899 |
| `keen::AnimationSequenceContainer` | 2.624 | 2.894 | 270 | 2.894 |
| `keen::NotifierResource` | 2.051 | 2.319 | 268 | 2.319 |
| `keen::actor::ActorSequenceResource` | 2.202 | 2.157 | -45 | 2.157 |
| `keen::RenderModelChunkModelResource` | 1.609 | 1.609 | 0 | 5 |
| `keen::SceneEntityChunkResource` | 1.609 | 1.609 | 0 | 5 |
| `keen::SceneWaterEntityResource` | 1.609 | 1.609 | 0 | 5 |
| `keen::WaterChunkResource` | 1.604 | 1.604 | 0 | 5 |
| `keen::WaterDisplacingResource` | 1.604 | 1.604 | 0 | 5 |
| `keen::VfxResource` | 1.324 | 1.573 | 249 | 1.573 |
| `keen::VoiceoverResource` | 2 | 1.273 | 1.271 | 1.273 |
| `keen::VfxMeshResource` | 0 | 638 | 638 | 638 |

Ressourcen-IDs und GUIDs unterscheiden sich deutlich bei segmentierten Typen. Zum Beispiel stehen 5.699 `UiTextureResource`-Einträge für 5.493 GUIDs. Die 1.609 Einträge je Scene-Chunk-Typ stehen für jeweils fünf GUIDs. Ein reiner Vergleich der Eintragszahlen würde diese Aufteilung als Objektwachstum fehlinterpretieren.

## Typen nur in einem Paket

Im KFC4-Paket, aber nicht im KFC3-Index: `keen::AtmosphereResource`, `keen::BiomeMapResource`, `keen::CharacterScenePostProcessingResource`, `keen::CommerceProductCollection`, `keen::LogicSystemResource`, `keen::Ps5ActivityCollection`, `keen::SoundReverbCollectionResource`, `keen::SoundReverbEarlyReflectionsResource`, `keen::SoundReverbLateReflectionsResource`, `keen::SoundReverbPresetResource`, `keen::SoundReverbPresetsResource`, `keen::VfxCurveResource`, `keen::VfxJointCollectionResource`, `keen::VfxMeshResource`, `keen::anim_graph::BlendProfileResource`, `keen::anim_graph::tool_graph::FeetIKSettings`, `keen::ecs::EntityProxyModelSetResource`, `keen::ecs::TargetSelectionSettings`.
Im KFC3-Index, aber nicht im KFC4-Paket: `UNKNOWN_47a54aff`, `UNKNOWN_8bea0afc`, `keen::BuffType`.

Die zwei unbekannten KFC3-Hashes sind `UNKNOWN_47a54aff` mit 534 Einträgen und `UNKNOWN_8bea0afc` mit einem Eintrag. Die vollständige Zuordnung neuer beziehungsweise fehlender Typen steht im Katalog unten.

18 erkannte Typen kommen nur im KFC4-Index vor. `keen::BuffType` kommt nur im KFC3-Index vor. Daraus ergibt sich der Nettoanstieg von 129 auf 146 bekannte Typen. Die beiden unbekannten KFC3-Hashes sind in dieser Typenzahl nicht enthalten.

## Vollständiger Katalog der in den Paketen beobachteten Typen

Die Tabelle enthält alle Typen aus der Vereinigung beider Paketindizes. Die zwei unbekannten Hashes stehen als `UNKNOWN_...`. KFC3 und KFC4 stehen auf null, wenn der Typ in diesem Paket fehlt.

| Typ | KFC3 IDs | KFC3 GUIDs | KFC4 IDs | KFC4 GUIDs | Δ IDs | Δ GUIDs | KFC3 MiB | KFC4 MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `keen::SoundResource` | 10.898 | 10.898 | 16.472 | 16.472 | 5.574 | 5.574 | 45,14 | 50,82 |
| `keen::ecs::TemplateResource` | 10.745 | 10.745 | 12.267 | 12.267 | 1.522 | 1.522 | 36,55 | 41,69 |
| `keen::RenderModel` | 12.713 | 12.713 | 10.470 | 10.470 | -2.243 | -2.243 | 119,79 | 23,15 |
| `keen::SoundContainerResource` | 3.507 | 3.507 | 6.940 | 6.940 | 3.433 | 3.433 | 0,75 | 1,37 |
| `keen::UiTextureResource` | 4.888 | 4.682 | 5.699 | 5.493 | 811 | 811 | 0,32 | 0,49 |
| `keen::JointAnimation` | 4.628 | 4.628 | 5.459 | 5.459 | 831 | 831 | 128,90 | 160,84 |
| `keen::ItemInfo` | 3.609 | 3.609 | 4.262 | 4.262 | 653 | 653 | 9,56 | 12,14 |
| `keen::RenderMaterialResource` | 2.982 | 2.982 | 3.458 | 3.458 | 476 | 476 | 1,67 | 3,47 |
| `keen::SequenceResource` | 2.629 | 2.629 | 2.899 | 2.899 | 270 | 270 | 1,93 | 2,28 |
| `keen::AnimationSequenceContainer` | 2.624 | 2.624 | 2.894 | 2.894 | 270 | 270 | 0,06 | 0,07 |
| `keen::NotifierResource` | 2.051 | 2.051 | 2.319 | 2.319 | 268 | 268 | 0,23 | 0,27 |
| `keen::actor::ActorSequenceResource` | 2.202 | 2.202 | 2.157 | 2.157 | -45 | -45 | 4,64 | 4,66 |
| `keen::RenderModelChunkModelResource` | 1.609 | 5 | 1.609 | 5 | 0 | 0 | 8,81 | 7,89 |
| `keen::SceneEntityChunkResource` | 1.609 | 5 | 1.609 | 5 | 0 | 0 | 224,82 | 46,81 |
| `keen::SceneWaterEntityResource` | 1.609 | 5 | 1.609 | 5 | 0 | 0 | 0,04 | 0,06 |
| `keen::WaterChunkResource` | 1.604 | 5 | 1.604 | 5 | 0 | 0 | 0,24 | 0,27 |
| `keen::WaterDisplacingResource` | 1.604 | 5 | 1.604 | 5 | 0 | 0 | 0,39 | 0,78 |
| `keen::VfxResource` | 1.324 | 1.324 | 1.573 | 1.573 | 249 | 249 | 14,33 | 18,17 |
| `keen::VoiceoverResource` | 2 | 2 | 1.273 | 1.273 | 1.271 | 1.271 | 0,00 | 0,11 |
| `keen::VfxMeshResource` | 0 | 0 | 638 | 638 | 638 | 638 | 0,00 | 0,03 |
| `keen::LveAnimation` | 418 | 418 | 578 | 578 | 160 | 160 | 0,33 | 0,48 |
| `keen::RootMotionAnimation` | 418 | 418 | 578 | 578 | 160 | 160 | 1,14 | 1,55 |
| `keen::ImpactProgram` | 496 | 496 | 570 | 570 | 74 | 74 | 0,98 | 1,21 |
| `keen::CardClusterResource` | 512 | 256 | 530 | 265 | 18 | 9 | 0,08 | 0,34 |
| `keen::ecs::DefaultInventoryResource` | 609 | 609 | 420 | 420 | -189 | -189 | 0,11 | 0,17 |
| `keen::Perk` | 233 | 233 | 365 | 365 | 132 | 132 | 0,10 | 0,16 |
| `keen::RenderTextureResource` | 59 | 38 | 304 | 265 | 245 | 227 | 0,00 | 0,04 |
| `keen::anim_graph::runtime_graph::AnimationGraphResource2_0` | 231 | 231 | 279 | 279 | 48 | 48 | 1,77 | 2,58 |
| `keen::MaterialFeedback` | 249 | 249 | 274 | 274 | 25 | 25 | 1,62 | 1,78 |
| `keen::ModelHierarchyResource` | 217 | 217 | 257 | 257 | 40 | 40 | 1,45 | 1,78 |
| `keen::VolumetricFog3ModelResource` | 157 | 6 | 169 | 10 | 12 | 4 | 0,11 | 0,16 |
| `keen::VoxelWorldResource` | 135 | 130 | 145 | 140 | 10 | 10 | 0,88 | 1,25 |
| `keen::VoxelModelResource` | 78 | 78 | 78 | 78 | 0 | 0 | 0,06 | 0,06 |
| `keen::GpcResource` | 67 | 67 | 74 | 74 | 7 | 7 | 0,32 | 0,44 |
| `keen::RenderDecal` | 49 | 49 | 52 | 52 | 3 | 3 | 0,02 | 0,05 |
| `keen::VfxJointCollectionResource` | 0 | 0 | 30 | 30 | 30 | 30 | 0,00 | 0,04 |
| `keen::fishing::FishSpawnTableResource` | 25 | 25 | 30 | 30 | 5 | 5 | 0,00 | 0,01 |
| `keen::RenderClothColliderResource` | 19 | 19 | 27 | 27 | 8 | 8 | 0,01 | 0,01 |
| `keen::BaseAttributeResource` | 55 | 55 | 16 | 16 | -39 | -39 | 0,02 | 0,00 |
| `keen::MidiResource` | 15 | 15 | 15 | 15 | 0 | 0 | 0,15 | 0,15 |
| `keen::MidiSongResource` | 15 | 15 | 15 | 15 | 0 | 0 | 0,00 | 0,00 |
| `keen::VoicePackResource` | 10 | 10 | 10 | 10 | 0 | 0 | 0,00 | 0,00 |
| `keen::anim_graph::BoneMaskResource` | 7 | 7 | 9 | 9 | 2 | 2 | 0,01 | 0,01 |
| `keen::VfxCurveResource` | 0 | 0 | 8 | 8 | 8 | 8 | 0,00 | 0,00 |
| `keen::SamplerInstrumentResource` | 7 | 7 | 7 | 7 | 0 | 0 | 14,57 | 14,57 |
| `keen::SoundReverbPresetResource` | 0 | 0 | 7 | 7 | 7 | 7 | 0,00 | 0,00 |
| `keen::VoxelBlueprintConfig` | 7 | 7 | 7 | 7 | 0 | 0 | 0,00 | 0,00 |
| `keen::fishing::TrashLootTableResource` | 6 | 6 | 7 | 7 | 1 | 1 | 0,00 | 0,00 |
| `keen::BiomeMapResource` | 0 | 0 | 5 | 5 | 5 | 5 | 0,00 | 1,56 |
| `keen::FogVoxelMappingResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,01 | 0,01 |
| `keen::SceneRandomLootResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,78 | 0,94 |
| `keen::SceneResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,32 | 0,38 |
| `keen::SceneWhiteboxLabelResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,31 | 0,37 |
| `keen::VoxelTemperatureResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,62 | 10,59 |
| `keen::VoxelWorldFog3Resource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,00 | 0,00 |
| `keen::WaterWorldResource` | 5 | 5 | 5 | 5 | 0 | 0 | 0,00 | 0,00 |
| `keen::anim_graph::RetargetDataResource` | 6 | 6 | 3 | 3 | -3 | -3 | 0,00 | 0,00 |
| `keen::DetailScatterResource` | 2 | 2 | 2 | 2 | 0 | 0 | 0,24 | 0,18 |
| `keen::IblResource` | 2 | 2 | 2 | 2 | 0 | 0 | 0,00 | 0,00 |
| `keen::WaterSurfaceEffectMaterialResource` | 2 | 2 | 2 | 2 | 0 | 0 | 0,00 | 0,00 |
| `keen::anim_graph::tool_graph::FeetIKSettings` | 0 | 0 | 2 | 2 | 2 | 2 | 0,00 | 0,00 |
| `keen::AchievementDefinitionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,01 |
| `keen::ActorSequenceCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,03 | 0,03 |
| `keen::AmbientCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::AmbientEnvironmentMapResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::AmbientEventSoundsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::AmbientParameterResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::AmbientPostProcessingResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::AmbientSfxCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,23 | 0,13 |
| `keen::AmbientVfxCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,01 |
| `keen::AtmosphereResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::AttributeContainerResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,02 | 0,02 |
| `keen::BalancingTable` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::BuffTypeCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,14 |
| `keen::BuildingMaterialBlendingResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,31 | 0,76 |
| `keen::BuildingMaterialParametersResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,09 | 0,09 |
| `keen::CameraStatesManager` | 1 | 1 | 1 | 1 | 0 | 0 | 0,06 | 0,08 |
| `keen::CameraStatesOverrideConfig` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,01 |
| `keen::CharacterPresetCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0,03 | 0,03 |
| `keen::CharacterScenePostProcessingResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::CommerceProductCollection` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::DefaultLootLabelCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,01 |
| `keen::DevJournalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,15 | 0,18 |
| `keen::EmoteResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::FbUiBundle` | 1 | 1 | 1 | 1 | 0 | 0 | 0,20 | 0,23 |
| `keen::FishingEffectResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::Game38ClientResources` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::Game38ServerResources` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::GameKnowledgeQueryResourceDb` | 1 | 1 | 1 | 1 | 0 | 0 | 1,04 | 1,23 |
| `keen::GameKnowledgeQueryTriggerResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,06 | 0,06 |
| `keen::GameKnowledgeResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,52 | 0,87 |
| `keen::GameSettingsPresetsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::GiVoxelBuildingMaterialResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,04 | 0,08 |
| `keen::GuidRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::ImpactRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,01 |
| `keen::IngameTimeConfig` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::ItemKnowledgeResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,21 | 0,32 |
| `keen::ItemRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,19 | 0,24 |
| `keen::JournalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,15 | 0,18 |
| `keen::LocaTagCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::LogicSystemResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,01 |
| `keen::LootableItemsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,57 | 0,68 |
| `keen::MapMarkerRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::MaterialFeedbackCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 2,07 | 2,62 |
| `keen::MidiSongCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::NpcCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,01 |
| `keen::PerkCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,01 |
| `keen::Ps5ActivityCollection` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::RecipeRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,69 | 0,88 |
| `keen::RenderResources` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::RenderSettingsGpuPresetsResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,01 |
| `keen::RenderSystemResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::RenderWaterResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::RenderWeatherResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::SimpleWorldMaterialResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,07 | 0,12 |
| `keen::SkillTreeResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,07 | 0,07 |
| `keen::SlopeDefinition` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::SoundReverbCollectionResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::SoundReverbEarlyReflectionsResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::SoundReverbLateReflectionsResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::SoundReverbPresetsResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::SoundWaterCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::SoundWaterFlowScenariosResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::SoundWaterfallParametersResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::TerraformingEfficiencyRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,03 | 0,04 |
| `keen::TwitchDropCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::UiFontResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::UiMapResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::UiWorldEventTriggerCollection` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::VfxSystemResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::VoicePackDirectoryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::VolumetricFog3Resource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::VoxelBlueprintItemRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,01 |
| `keen::VoxelBlueprintMaterialPoolRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::VoxelMaterialVfxDataResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::WeatherSystemResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,03 | 0,03 |
| `keen::WorkshopRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,01 |
| `keen::WorldMaterialBlending2Resource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,29 | 0,57 |
| `keen::ZoneVisualizationResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,00 | 0,00 |
| `keen::anim_graph::BlendProfileResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::debug::EnemyDebugMenu` | 1 | 1 | 1 | 1 | 0 | 0 | 0,01 | 0,02 |
| `keen::ecs::EntityProxyModelSetResource` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,01 |
| `keen::ecs::TargetSelectionSettings` | 0 | 0 | 1 | 1 | 1 | 1 | 0,00 | 0,00 |
| `keen::ecs::TemplateCollectionResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,16 | 0,19 |
| `keen::enemy::DevEnemyArsenalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,15 | 0,21 |
| `keen::enemy::EnemyArsenalRegistryResource` | 1 | 1 | 1 | 1 | 0 | 0 | 0,14 | 0,21 |
| `UNKNOWN_47a54aff` | 534 | 534 | 0 | 0 | -534 | -534 | 0,02 | 0,00 |
| `UNKNOWN_8bea0afc` | 1 | 1 | 0 | 0 | -1 | -1 | 0,00 | 0,00 |
| `keen::BuffType` | 224 | 224 | 0 | 0 | -224 | -224 | 0,11 | 0,00 |

## Parserlauf und Einschränkungen

| Parserlauf | Eingabe | Ergebnis |
| --- | --- | --- |
| Alter CLI-Parser `f201f76` | KFC3-Sicherung | Der KFC3-Kopf und Index wurden eingelesen. Der vollständige Export brach beim ersten Ressourceninhalt ab. Das Programm protokollierte zunächst „unexpected end of data“ und danach eine Speicheranforderung von 60.363.262.464 Byte. Es wurden keine JSON-Deskriptoren exportiert. |
| Aktueller Parser `a404b15` | KFC4-Installation | Die KFC4-Datei, ihre Ressourcen-IDs, Größen, Chunks und Typ-Hashes wurden über die aktuelle KFC-Indeximplementierung ausgewertet. Dieser Bericht führt keinen vollständigen JSON-Export und keine Feld-für-Feld-Validierung jedes Deskriptors aus. |

Die 535 unbekannten KFC3-Ressourcen sind als Indexeinträge vorhanden. Die eingesetzte Reflection-Datenbank kennt ihre zwei Typ-Hashes nicht. Eine unbekannte Bezeichnung bedeutet daher nicht, dass die Ressource im KFC-Index fehlt.

## Quellen im Parser-Repository

Die untersuchten KFC-Dateien sind nicht Bestandteil dieses Repositories. Der KFC3-Stand stammt aus einer von ShroudForge angelegten Original-Sicherung. Der KFC4-Stand stammt aus der lokalen Spielinstallation am 15. Oktober 2026. Die SHA-256-Prüfsummen oben identifizieren genau diese beiden Eingabedateien.

Die Dokumentation zum [KFC-Dateiformat](../file.md) und zu [Reflection-Typen](../types.md) erklärt die Grundlagen. Dateikopf und Indizes sind in [`header.rs`](../../../../../crates/kfc-base/src/container/header.rs) definiert. Das Einlesen des KFC-Index steht in [`file.rs`](../../../../../crates/kfc-base/src/container/file.rs). Der CLI-Exportpfad steht in [`main.rs`](../../../../../crates/cli-legacy/src/main.rs). Die Typzuordnung erfolgt über [`registry.rs`](../../../../../crates/kfc-base/src/reflection/registry.rs).
