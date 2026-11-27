# Monde, créatures, collecte et cartes — #81

La projection `world` étend le catalogue commun au **format SQLite 18**, sans
migration des formats de développement. Elle exploite les tables et ressources
du client fourni, rattaché par confirmation du propriétaire au build Steam
**26581260**. La région reste inconnue. La lecture structurelle n’est pas une
déclaration générale de compatibilité version/région.

## Sources et lecteurs

Les tables recherchées explicitement sont `Map`, `Subzone`, `Dungeon`,
`DungeonGroup`, `PartyDungeonLevel`, `WorldMapUIRegion`, `WorldMapMapList`,
`WorldMapFieldNamed`, `NpcData`, `NpcLoot`, `NpcAggroData`, `EnvObjData`,
`GatherSource`, `GatherSourceMaterial`, `GatherMastery`, `GatherSkill`,
`GatherSkillEffect` et `PeriodSpawn`. Les sources de récompenses déjà prises en
charge restent partagées avec les items et les définitions de butin.
`GlobalRegion` décrit les régions de service, pas les régions du monde.

L’export examine aussi `Map/`, `MapEvent/`, `MapDataHierarchy/` et `WorldMap/`
sous le même dossier `Data` que `Table`. Il contrôle les empreintes des fichiers
et la liste de ressources avant publication. Les entrées sont bornées et les
liens symboliques dans les dossiers parcourus sont refusés. Chaque refus garde
son chemin et son diagnostic ; une famille illisible n’est pas présentée comme
vide et valide.

- `MapData` et `AionWorldMapExportInfo` utilisent une chaîne de hashes de
  colonnes sans terminateur obligatoire, puis une seule structure USMAP.
  Ils ne portent ni version de table ni nombre de lignes. `probe-map` est
  distinct de `probe-table` et attend un payload déjà déchiffré.
- `MapDataHierarchy` v1 est un arbre binaire typé : carte, sous-zone, type de
  PNJ, PNJ, items. Le lecteur contrôle les tags, comptes, deux IDs de carte,
  tailles et consommation complète. Le dernier mot de chaque item, ignoré
  par le lecteur de référence, est conservé comme `UnknownWord`.
- Les événements sont du JSON, dont les conditions restent non interprétées.
  Les directives `StartSpawner` et `EndSpawner` gardent leur chemin source.
- Le déchiffreur applique désormais la branche XOR cartes à `WorldMap/*.dat`.
  Sur cette copie, les **161 fichiers WorldMap** sont déchiffrés. Le refus
  restant concerne `Map/Start/NavSignature.dat`, conservé dans le rapport.

Les structures et la branche XOR ont été confrontées aux lecteurs de
[KiskEmu/CUE4Parse au commit épinglé](https://github.com/Doxylamin/KiskEmu/tree/8fa7095fe2566fd847fcdb81d745616135cd0a63/third_party/CUE4Parse/CUE4Parse/GameTypes/Aion2/Objects),
notamment `FAion2MapDataFile` et `FAion2MapHierarchyFile`. Les tableaux complets,
éléments supprimés des maps/sets et champs non projetés restent dans
`catalogue_sources`, avec les empreintes, structures et builds. Les références,
textes, conditions et langues utilisent les référentiels communs.

## Projection

| Tables | Données |
| --- | --- |
| `world_maps`, `world_subzones`, `world_map_categories`, `world_ui_regions` | Identités, textes EN/localisés, carte de base, carte/sous-zone, race, catégories et indications UI. `ZoneId` reste une référence sans parent inventé. |
| `world_dungeon_groups`, `world_dungeons`, `world_dungeon_levels`, `world_dungeon_loot` | Groupes, types, difficultés, cartes, recommandations, limites, recharge brute, paliers d’équipement et lien vers les définitions de butin déjà suivies. Les instances ne sont pas toutes des donjons de groupe. |
| `world_npcs`, `world_npc_skills`, `world_npc_loot` | PNJ et créatures par ID, niveaux, classifications, relation/faction source, ressources, compétences explicites et lien vers `loot_tables`. |
| `world_environment_objects` | Objets interactifs, usage, catégorie, conditions et lien de collecte uniquement lorsque l’usage est `GatherSource`. |
| `world_gather_sources`, `world_gather_materials`, `world_gather_results` | Maîtrise, type/grade, tous les `GroupKey`, matériaux principaux, supplémentaires, d’échec et de quête, quantités et références. |
| `world_gather_mastery`, `world_gather_skills`, `world_gather_effects` | Maîtrise et expérience, compétences, niveaux, coûts et probabilités brutes des effets. Le reste demeure dans la source liée. |
| `world_periodic_spawns`, `world_field_named`, `world_periodic_spawner_links` | Carte, nom de spawner, cycle et ticks bruts, jours, références et correspondances exactes. |
| `world_map_assets` | Ressources identifiées par chemin relatif stable, empreinte, carte source quand explicite et états du repère/de la transformation. Une carte absente de `Map` n’est pas inventée. |
| `world_spawners`, `world_spawn_candidates`, `world_spawn_positions` | Chaque spawner, tous ses candidats PNJ/objets et toutes ses positions et rotations. Aucun candidat n’est choisi arbitrairement. |
| `world_routes`, `world_route_points` | Waypoints, identifiant et clé source, type, traversée, déplacement et points ordonnés. Aucun rattachement au spawner par proximité ou ressemblance de nom. |
| `world_subzone_volumes`, `world_volume_points` | Volumes, contours, hauteurs et référence explicite de sous-zone ; coordonnées originales. |
| `world_hierarchy_entries` | Appartenance à l’arbre carte/sous-zone/PNJ/item, type ordinal et mot inconnu ; les références absentes restent nulles à côté de la valeur source. |
| `world_event_spawner_commands`, `world_event_spawner_links` | Directives et liens exacts carte + nom ; les variantes de ressources restent plusieurs correspondances. |
| `world_map_layouts`, `world_map_preview_locations` | Secteurs, dimensions, origine, bornes et positions de libellés. Le rapprochement de `WorldMapDataTableKey` avec `Map.Name` reste candidat. |

Le nom d’une source de collecte peut reprendre celui du matériau seulement si
tous ses résultats principaux désignent le même item. Cette clé indirecte de
localisation participe au suivi des changements. Les alternatives ne sont pas
réduites à la première ligne. Les usages `UsageValue` d’autres types ne sont pas
interprétés comme de la collecte.

## Consultation, versions et limites

Database propose cartes, sous-zones, donjons/groupes, PNJ, objets du monde,
collecte, compétences de collecte, apparitions périodiques, ressources,
spawners et trajets. Les fiches et leurs relations restent bornées ; les
sources complètes sont ouvertes sur demande. Les identifiants source restent
entiers, les classifications utilisent les dictionnaires `SMALLINT` bornés.

Les définitions racines et ressources ont leur dernière version de modification
et le suivi commun du dernier changement, sans historique de valeurs. Le butin
est suivi sur sa définition : changer `NpcLoot` ne modifie pas la définition du
PNJ ou les items. Changer la difficulté d’un donjon peut modifier sa définition
et son contexte de distribution de butin. Les traductions de même langue sont
comparées ; changer de langue ne devient pas un changement du jeu.

Les positions et trajets sont des possibilités du client, **pas des présences
en temps réel**. Les horaires, conditions, quantités et poids ne deviennent pas
des probabilités ou des disponibilités garanties. Les HP, dégâts et défenses
ne sont jamais extrapolés depuis le niveau des PNJ ; aucune formule des builders
communautaires n’est importée.

### Compétences des créatures

`world_npc_skills` conserve les associations explicites de
`NpcData.ActiveSkillList`, avec référence source et clé étrangère vers `skills`.
Les **47 806 associations** de cette copie sont résolues, mais cette liste
ne prouve pas que toutes les attaques d’un boss y figurent. Pour Ludra
(`NpcData.ID = 2300290`), elle ne contient que `Skill.ID = 1803000`, une
transition de phase sans nom (`SkillString_Key = None`). La table `Skill`
contient aussi 63 lignes dont l’animation mentionne `Durbati_01_V01` ; ce
rapprochement par nom reste une piste et ne crée aucune association en base.
`NpcBehaviorCase` décrit des cas, paroles et actes, sans liste explicite de
compétences dans ses huit champs.

Une future indication de sort de boss pourra consulter une compétence par ID,
ses textes, son icône et ses effets lorsqu’ils existent. L’identification du
sort effectivement lancé exige un événement de combat et la validation de la
correspondance entre ID réseau et ID client ; elle n’est pas déduite de cette
liste statique. Les noms absents restent absents, sans texte inventé.

### Fonds graphiques récupérés

Le convertisseur hors ligne partagé avec les icônes propose `--world-maps`.
Il parcourt les niveaux `.umap` de `UI/Map`, charge les propriétés explicites
`SectorTexture` des acteurs et exporte les PNG dans un manifeste
`world-maps.json`. Les alias de fichier identiques sont dédupliqués ; un
conflit de contenu est refusé. Il conserve aussi les niveaux sans secteur et
les diagnostics, sans assimiler une absence de secteur à une erreur.
Il inspecte également tous les `.uasset` de `UI/Map/WorldMap`, charge leurs
objets `Texture2D` et récupère les textures indépendantes, masques et couches
de chemins. Une texture sans acteur reste une ressource sans lien de secteur ;
son dossier et son nom ne servent pas à lui fabriquer une association.

Le lecteur Rust importe le manifeste et les images dans le catalogue :

| Tables | Données |
| --- | --- |
| `world_map_levels` | Chemin et empreinte du niveau Unreal, dernier build et nombre de secteurs. |
| `world_map_textures` | Chemin d’objet exact, dimensions, format source et PNG via `catalogue_icons`. |
| `world_map_sectors` | Acteur, classe, indice explicite ou état hérité non résolu, référence de texture. |
| `world_map_level_candidates` | Configuration WorldMap et niveau dont le nom correspond exactement après le préfixe `WorldMap_` ou `SharedMap_` ; association candidate. |

Database ajoute « Niveaux graphiques des cartes » et « Textures des cartes ».
Chaque texture dispose de son image et les secteurs restent liés aux niveaux.
Les sources, textes, images et identités stables utilisent le catalogue commun.
Une modification est suivie sur le niveau ou la texture concernés, sans
inventer un changement des définitions de PNJ ou d’items.

L’import contrôle le mapping, les références et identités uniques, le chemin,
la signature PNG, les dimensions et l’empreinte ; il refuse les traversées de
chemin et liens symboliques. Le manifeste est borné à 32 Mio, les PNG à 16 Mio
et 4096×4096, les images à 1 Gio au total. Tous les fichiers sont revérifiés
avant publication. L’outil hors ligne vérifie aussi le décodage des PNG.
Les fichiers clients et images restent dans les dossiers ignorés par Git.

La transformation coordonnées monde → secteurs/texture reste **non interprétée**.
Les références acteur → texture sont explicites ; les rapprochements de noms
avec les configurations ne prouvent ni l’échelle, ni l’orientation, ni les
étages. Les indices de secteur non sérialisés restent inconnus : aucune
valeur par défaut n’est inventée depuis le seul nom du fichier. Une future
carte géographique avec marqueurs nécessitera cette validation supplémentaire.
La confirmation visuelle de l’utilisateur établit que les `World_L_A_0X`
sont bien des morceaux de carte lisibles. Elle ne valide pas la transformation
entre les coordonnées monde et les pixels.

L’extraction des cartes est également implémentée en Rust, sans lancement de
.NET : `game_data::map_textures::extract` monte les conteneurs IoStore v5,
lit les packages Zen d’Aion 2 (UE 5.3), leurs dépendances et exports publics,
les propriétés non versionnées avec le mapping USMAP, puis les champs du
Blueprint `BP_WorldMapPlane_C` depuis sa définition sérialisée. Les références
acteur → texture sont résolues par identifiants et empreintes d’exports.
Les fichiers patchs ont priorité sur les conteneurs de base.

Le décodeur prend en charge les trois formats présents : DXT1 (BC1), DXT5
(BC3) et BGRA8, avec transparence et plus grand mip disponible. Les données
peuvent être intégrées au package ou référencées dans un chunk bulk distinct.
Les autres formats, textures virtuelles et versions de packages non prises
en charge produisent un diagnostic explicite, sans image inventée. Un dossier
de sortie neuf, hors des archives du jeu, est exigé. Les PNG sont compressés
et restent compatibles avec le même importeur SQLite. Le convertisseur C#
sert de référence hors ligne ; il n’est pas appelé par cette extraction.
Cette capacité porte sur les cartes ; elle ne remplace pas encore l’extraction
C# des icônes et sprites.

La conversion complète sur le client fourni a inspecté **221 niveaux distincts**
et **1 901 packages graphiques**, sans erreur. Elle a récupéré **1 758 textures**,
soit **1 000 994 776 octets de PNG**. Les **492 références de secteurs**
désignent 492 textures distinctes ; les **1 266 autres textures** restent
indépendantes. Les 1 758 PNG ont été redécodés et leurs dimensions et SHA-256
vérifiés. Le rapport de conversion se trouve dans
`target/game-icons/world-maps-all/world-maps.json`.

Un échec ultérieur d’extraction de niveau ou de texture rend la comparaison
de builds impossible : une entrée manquante ne devient pas une suppression du
jeu. Cette règle est vérifiée par un test de régression conservant l’entrée
précédente comme présente, avec un état de comparaison explicite.

## Validation sur le client fourni

La première base de travail a été entièrement
régénérée au format 17 : **889 704 448 octets**, build **26581260**, langue
`fr-FR`. La copie précédente au format 16 est conservée sous
`data-before-world-81.sqlite`. La révision du lecteur enregistrée dans le
manifeste est
`9049939a72135eeeede2ca8ec0081a52f0c5f890370cbfd1ec9a53a0c257cb9e`.

| Projection | Nombre |
| --- | ---: |
| Cartes / sous-zones / donjons | 741 / 1 712 / 734 |
| PNJ et créatures / liens vers compétences | 13 536 / 47 806 |
| Objets du monde / sources de collecte / résultats | 5 511 / 119 / 850 |
| Ressources Map / MapEvent / hiérarchie / WorldMap | 746 / 1 085 / 1 / 161 |
| Spawners / candidats / positions | 77 350 / 79 365 / 135 118 |
| Trajets / points de trajet | 5 538 / 29 282 |
| Entrées de hiérarchie | 146 594 |

Toutes les références de compétences PNJ, de résultats de collecte et de
PNJ/items de la hiérarchie sont résolues. Les 142 liens d’objets vers une
source de collecte respectent l’usage typé. Les 4 216 spawners avec plusieurs
candidats PNJ et les 9 255 avec plusieurs positions conservent leurs variantes.

Les références manquantes sont signalées, sans création artificielle :
162 occurrences vers `Map.ID`, 112 vers `Map.Name`, 24 vers `Subzone.ID`.
Sur les 161 configurations WorldMap, seules 45 ont un candidat exact dans
`Map.Name` ; les 116 autres gardent une clé étrangère nulle. Les associations
de configuration restent candidates dans les deux cas.

L’export valide l’intégrité avant publication et le contrôle complémentaire
`foreign_key_check` ne rapporte aucune erreur. Le rapport local est
`target/game-data-validation/world-81-validation.json`. Les vérifications de
code passent : 259 tests de bibliothèque et 52 tests d’intégration, Clippy
Linux/Windows, formatage et compilation Windows. Deux tests nécessitant leur
environnement spécifique restent ignorés. L’exécutable de vérification se
trouve dans `target/windows-item-background-preview/debug/aetheris.exe`.

### Catalogue avec images de cartes — format 18

La base de travail `target/game-data-validation/data.sqlite` a ensuite été
entièrement régénérée au format **18** : **1 895 989 248 octets**, build
**26581260**, langue `fr-FR`. La base précédente est conservée sous
`data-before-world-map-images.sqlite`. La révision du lecteur est
`23c295e37dbb34e7c847a63decb1c44ffa47c9fb666ad0781e2cd741c1ed442f`.

Les 221 niveaux, 1 758 textures et 492 secteurs sont présents en SQLite avec
leurs images et références. Les 151 rapprochements de configurations restent
candidats ; dix configurations n’ont pas de niveau correspondant. Sur les
492 secteurs, 338 indices sont explicites et 154 restent non résolus.
L’intégrité et les clés étrangères passent avant publication ; des contrôles
supplémentaires vérifient les liens niveau/secteur/texture et trois empreintes
de PNG lus directement dans SQLite. Le rapport est
`target/game-data-validation/world-images-validation.json`.

Les vérifications finales passent : **263 tests de bibliothèque**, Clippy,
formatage et compilation Windows. Les 52 tests d’intégration des lecteurs
passent également. Deux tests nécessitant un environnement spécifique restent
ignorés. L’exécutable Windows actualisé se trouve dans
`target/windows/debug/aetheris.exe`.

### Validation de l’extracteur Rust

Sur le même client, l’extraction native retrouve **221 niveaux**, **1 901 packages
graphiques** et **1 758 textures**, sans erreur de niveau ni de texture. Les PNG
natifs occupent **970 139 081 octets**. Les **1 758 images RGBA sont identiques
pixel par pixel** à la référence C# ; dimensions, formats et références sont
également identiques. Les 221 descriptions de niveaux, leurs empreintes de
payload et les **492 liaisons de secteurs** correspondent exactement, y compris
les indices explicitement enregistrés ou absents.

Le résultat natif est conservé dans `target/game-icons/world-maps-rust` ; les
journaux locaux sont `target/native-world-maps-final.log` et
`target/native-world-maps-comparison.log`. Le test de comparaison est dans
`tests/game_data_world_images_native.rs` et exige deux extractions locales :

```sh
AETHERIS_MAP_REFERENCE=/reference AETHERIS_MAP_NATIVE=/extraction-rust cargo test --release --locked --no-default-features --test game_data_world_images_native -- --ignored --nocapture
```

Les tests sans fichiers de jeu couvrent les limites de packages et mips,
troncatures, vues relatives de dépendances, noms UTF-16, indices USMAP hérités
avec trous, transparence BC1, alpha interpolé BC3 et ordre des canaux BGRA.
Le test comparatif local est ignoré par défaut ; il ne nécessite pas .NET.

La base de travail `target/game-data-validation/data.sqlite` a été régénérée
avec l’extraction native : **1 867 083 776 octets**, format **18**, build
**26581260**, langue `fr-FR`. La révision du lecteur est
`2ba7cfa90437586a71227815d9233414db8683f090cb1cc528e4f1646e7512d9`.
La copie précédente est conservée dans
`data-before-native-world-map-images.sqlite`. L’export termine ses contrôles
d’intégrité et de clés étrangères avant publication ; le contrôle complémentaire
relit les **1 758 PNG stockés en SQLite** et vérifie dimensions et empreintes
contre le manifeste natif. Les rapports locaux sont
`target/game-data-validation/native-world-maps-validation.json` et
`target/game-data-validation/world-images-native-validation.json`.

Le passage général compte **342 tests réussis**, plus le test comparatif local
complet des images. Clippy, formatage et compilation Windows passent.
L’application et l’outil natif sont disponibles sous
`target/windows/debug/aetheris.exe` et
`target/windows/debug/aetheris-game-data.exe`.
