# Inventaire des familles du client — #75

L'inventaire identifie les sources des familles de la [cible](game-data-target.md),
contrôle leurs clés et documente ce qui manque avant les projections #79 à #83.
Il n'ajoute aucune table métier et ne modifie pas `data.sqlite`.

## Copie étudiée et méthode

- Build Steam **26581260**, manifeste `base-doc/data/appmanifest_3393110.acf` ;
  rattachement de la copie confirmé par le propriétaire. Région non établie.
- Tables : `target/game-data-validation/decrypted-permuted/AION2/Content/Data/Table`.
  L'inventaire porte sur les `.bin` directement dans ce dossier ; les ressources
  de cartes et les textes sont inventoriés séparément.
- Mapping fourni : `AION2-5.3.2.0-2.0.6.0.usmap`, SHA-256
  `019f06284f923aa5f12660c5debbefcb99f3a424a16b4d62b8d2fdfd4997ea44`.
  Son nom ne constitue pas une déclaration de compatibilité avec tous les builds.
- Textes : exports `en-US/L10NString.json` et `fr-FR/L10NString.json` de cette copie.
  Les descriptions gardent leurs marqueurs ; aucun chiffre n'est substitué par
  l'inventaire. Les clés absentes restent signalées, sans traduction inventée.
- Exécution reproductible : [outil et commandes](../tools/game-data/README.md),
  [règles explicites](../tools/game-data/catalogue-sources.json). Chaque table
  est relue par le lecteur Rust strict, avec empreintes du contenu et du mapping.
  Le rapport local est `target/game-data-validation/source-inventory/inventory.json` ;
  les lignes complètes restent dans `probes/`, hors du dépôt.

Une lecture structurelle réussie ne valide pas une interprétation métier.
Les contrôles ci-dessous portent sur des champs et clés explicites. Une liaison
candidate reste une hypothèse même si toutes ses valeurs correspondent. Les
valeurs nulles et `None` ne sont pas transformées en relations. Les groupes
acceptent plusieurs lignes uniquement quand la règle le déclare.

## Couverture et refus de lecture

**363 fichiers de tables, 353 décodés et 10 refusés** sur cette copie. Ce périmètre
ne doit pas être confondu avec le nombre de tables projetées dans SQLite, ni avec
l'ensemble des fichiers déchiffrés. Les familles sont des regroupements de
candidats, éventuellement partagés ; leurs nombres ne s'additionnent pas.
Le profil exécute **74 contrôles de relations**, plus le contrôle des noms de
statistiques Daevanion et des cartes des événements. Les règles candidates ou
sans référence ne sont pas comptées comme des relations métier établies.
Résultat des règles de tables : 56 contrôles de clés complets, 3 partiels,
3 candidats et 12 sans référence non nulle dans cette copie.

| Source refusée | Diagnostic / conséquence |
| --- | --- |
| `ActInfo` | Valeur flottante non finie, ligne 80, offset 89165 ; pas de remplacement arbitraire. |
| `ClientSliderSettings`, `FieldEventPlatformerDilemma`, `KiskData`, `RS`, `ServerMessage`, `SpecialIdle` | Structure requise absente du mapping ; aucun layout deviné. |
| `DefaultSettings`, `GlobalSetting` | Champs dupliqués dans le layout ; refus du lecteur strict. |
| `Resources` | Dépasse la limite de 64 Mio du lecteur ; cela ne prouve ni corruption ni incompatibilité. L'indirection de ressources reste partielle. |

Les diagnostics exacts, noms de structures candidates et empreintes sont dans le
rapport. Les alias tels que `VehicleList → VehicleDataTableItem`,
`PeriodItemCollection → PeriodCollectionTableItem` et
`RewardData → RewardTableItem` sont déclarés dans le profil puis contrôlés par
Rust ; ils ne sont pas des renommages métier. Les refus relèvent du travail sur
les schémas et conteneurs (#41/#42), sans élargissement silencieux du lecteur.

## Objets, compétences, états et obtention déjà projetés

Les [audits #77/#78](game-data-77-78.md), les documents
[objets](game-data-items.md), [compétences](game-data-skills.md) et
[changements #76](game-data-changes.md) restent les références de la projection.
La copie contient 9 452 lignes `Item`, 16 283 `Skill` et 6 708 `SkillAbnormal`.
Les effets directs, états, paramètres et évolutions restent des sources distinctes.
Une correspondance d'ID n'établit aucune correspondance avec le réseau.

Contrôles transversaux supplémentaires : 3 076 références
`Item.$variant.ExtractionSkins → Skin.Name` et 40 références d'ensemble
`Item.$variant.SetNames → ItemSet.SetName` aboutissent. Les ensembles sont des
groupes, pas des clés uniques par ligne. Les récompenses et attributions
(`Reward*`, `BoxReward`, `DungeonRewardInfo`, `NpcLoot`, etc.) restent reliées à
leurs définitions de loot ; leurs modifications ne marquent pas les items
contenus comme modifiés. Les poids sans échelle prouvée restent des poids.

## Recettes et quêtes — #79

| Source | Lignes | Champs et identités utiles |
| --- | ---: | --- |
| `CraftRecipe` | 1 120 | `ID`, `Name`, catégories, maîtrise, race, `Product`, quantité, `ComponentList`, frais et monnaie, `ComboRecipe`, `ComboProbability`, `CraftRateGroup`, conditions et limites. |
| `CraftRecipeRate` | 1 090 | `Name`, liste de couples `Gap` / `Rate`, valeurs brutes. |
| `CraftMastery` | 100 | Niveau de maîtrise, grade, expérience requise et points bonus. |
| `ConversionRecipe` | 296 | `ID`, `Name`, produit, composants, conditions d'enregistrement, race, coûts, succès et résultat critique. |
| `CraftComponentReplacement` / `ConversionComponentReplacement` | 126 / 15 | Nom de liste et alternatives `ReplacementItemList`, avec items et quantités. |
| `Quest` | 1 429 | `ID`, `Name`, `QuestText`, `Part`, `NextQuestName`, type/grade, race, niveaux, conditions, répétition, cartes d'acquisition et de complétion. |
| `QuestString` / `QuestPart` | 1 490 / 34 | Clé technique des textes, titres et descriptions localisées ; groupes de quêtes. |
| `QuestStep` | 2 553 | `QuestName`, `Order`, objectifs typés et leurs valeurs, cartes, dialogues et branche d'échec. |
| `QuestReward` | 1 321 | Groupe, récompenses normales, au choix et aléatoires ; règles brutes à conserver. |

Exemple : recette `111013001`,
`Recipe_Blacksmithing_L_wp_Sword_Unique_001`, produit
`equip_wp_Sword_L_u_craftdraconic_001a_s`, maîtrise 55 et valeur de combo **2500**.
Cette valeur n'est pas présentée comme un pourcentage. Les 1 120 produits et
2 318 composants principaux aboutissent à `Item.Name` ; 280 combos aboutissent
à `CraftRecipe.Name` et 606 groupes à `CraftRecipeRate.Name`.
`ComponentList.ReplacementItem` désigne une **liste de substitution**, et non
directement un item ; les composants de cette liste renvoient ensuite à `Item`.
Les recettes de conversion suivent le même principe.

Exemple : quête `1100010` / `HQ1100010`, texte `STR_HQ1100010`, partie
`hero_poeta_00`, suite `HQ1101010`. Les 1 429 clés de texte, 249 parties,
237 suites et 2 553 références d'étapes aboutissent. La chaîne numérique
`QuestReward.Group` correspond à `Quest.ID` sur 1 321 lignes ; son rôle précis
reste **candidat**, explicitement distinct d'une clé de nom.

Limites : lieu/station des recettes non établi ; ne pas confondre maîtrise,
niveau de personnage et qualification. Les valeurs d'objectif `GoalList.Value`
nécessitent une interprétation par type, sans règle universelle fondée sur leur
position. Les cartes manquantes sont listées plus bas. Les objets nécessaires,
résultats et récompenses donnent les dépendances principales vers le catalogue
items et les définitions de loot.

## Exploits, titres, pass et ravitaillement — #80

| Source | Lignes | Champs et relations |
| --- | ---: | --- |
| `Achievement` | 6 434 | `AchievementId`, groupes, objectifs typés, grade, cycle, prérequis, icône. |
| `AchievementStep` / `AchievementReward` | 17 342 / 13 743 | ID d'exploit, palier et compte ; items et points de récompense. |
| `AchievementGroup` / `AchievementGroupSet` / `AchievementString` | 473 / 167 / 7 661 | Catégories, regroupements et textes. |
| `Title` / `TitleCategory` | 1 517 / 8 | `ID`, `Name`, descriptions, race, grade, catégorie, statistiques de possession et d'équipement, périodes. |
| `BattlePass` / `BattlePassGroup` / `SeasonSchedule` | 88 / 44 / 67 | Faction, groupes, titres, dates brutes, conditions/coûts de déverrouillage, expérience, récompenses et saison candidate. |
| `PeriodItemCollection` / `PeriodItemCollectionList` | 356 / 1 122 | Groupes, onglets, race, niveau minimal ; items et compte demandés, récompenses, textes. |

Les 17 342 paliers et 13 743 récompenses retrouvent leur exploit ; les
19 627 références d'items des récompenses aboutissent. Les 1 517 catégories de
titre aboutissent. Exemple : titre `11010001`, `L_Quest_Hero_Vert_001`, avec
`StatList.Accuracy = 5` et `EquipStatList.FixingDamage = 5` : les effets de
possession et d'équipement doivent rester distincts.

Les 88 pass retrouvent leur groupe et leur exploit d'expérience ; 40 références
d'items de récompense et 66 d'items de déverrouillage aboutissent. Les 44
`BattlePassGroup.SeasonId` correspondent à `SeasonSchedule.ID`, liaison encore
candidate quant au rôle temporel. Les dates en ticks ne sont pas converties par
l'inventaire. Un nom suffixé `Pay` peut avoir `bIsPremium = false` tout en ayant
un coût de 1 000 `GameCash` : ni le nom ni ce booléen isolé ne suffisent à
construire les pistes gratuites et payantes. Aucun filtre sur la période
courante ne doit supprimer les anciens pass du client.

Le ravitaillement est confirmé par les textes de `PeriodItemCollection` :
**Urgentes**, **Hebdomadaires**, **Standard : Boss** et **Standard : fabrication**
pour les clés Light étudiées. Les clés Dark correspondantes manquent dans
l'export français avec leur casse exacte ; ne pas les inventer. Les 1 122
groupes, 5 364 items cibles et 2 024 récompenses aboutissent. Les listes
`TargetItems` peuvent contenir `Count = 0` alors que `TargetCount = 3` : la règle
de quantité et d'alternatives doit être conservée avant de produire l'affichage.
Le candidat `SupplyRequest.bin` n'a pas été trouvé sous ce nom, ce qui n'empêche
pas d'identifier la famille par ces tables et leurs textes.

## Monde, PNJ, collecte et cartes — #81

| Source | Lignes | Champs et relations |
| --- | ---: | --- |
| `Map` | 741 | `ID`, nom/description, race, type, fichier, répertoire et carte de base. |
| `Subzone` | 1 712 | `ID`, `MapId`, nom localisé, zone et références de carte/UI. |
| `Dungeon` | 734 | Identité, type/contexte, difficulté, `MapID`, limites, accès et références de récompenses. Toutes les lignes ne sont pas des donjons de groupe. |
| `WorldMapUIRegion` / `WorldMapMapList` | 227 / 4 | Liens vers cartes et sous-zones, présentation du monde. |
| `NpcData` / `NpcLoot` | 13 536 / 1 782 | Identité, type, niveau, ressources, compétences ; listes de butin liées aux IDs PNJ. |
| `PeriodSpawn` | 23 | Carte, nom de spawner, cycles, heures et conditions brutes. |
| `EnvObjData` | 5 511 | Catégorie, usage typé et valeurs associées, ressource, description et interaction. |
| `GatherSource` / `GatherSourceMaterial` | 119 / 96 | Identité, type/grade, maîtrise et groupe de matériaux ; résultat, quantité, jauge, résultats supplémentaires/échecs. |

`GlobalRegion` (4 lignes) décrit **les régions de service et codes pays**, par
exemple `US → NAW, NAE`. Ce n'est pas la hiérarchie géographique du monde.
Cette dernière s'appuie notamment sur `Map`, `Subzone` et les tables `WorldMap*` ;
les niveaux de hiérarchie non établis restent non résolus.

Les 280 références de carte de base, 734 de donjons, 1 712 de sous-zones,
227 cartes de régions UI et 23 cartes de spawners aboutissent. Les 47 806
références de compétences PNJ aboutissent à `Skill.ID` ; les 1 782 IDs de
`NpcLoot` et ses 120 016 références d'items aboutissent. Le PNJ `2000000`
(`SandbagMonster`) est un exemple de présence de données de test dans le client :
ne pas confondre présence dans les tables et disponibilité pour le joueur.
Aucune statistique n'est calculée depuis le niveau.

Les 119 groupes de collecte aboutissent ; les 96 matériaux principaux et
112 références supplémentaires aboutissent à `Item.Name`. Exemple :
`GatherSource` ID `1080001`, `Gather_Od_lv1_Common_001_f`. Le lien entre objet
d'environnement, définition de collecte et emplacement n'est pas encore établi
par un champ typé vérifié ; un `UsageValue` arbitraire ne suffit pas. Les tables
`GatherMastery` et `Gather*` supplémentaires sont dans le rapport comme candidats.

### Ressources hors tables

- `Data/Map/` : **746 fichiers binaires imbriqués**, dont
  `Test_QA/MapData.bin`. Identifiés et empreintés, pas décodés sémantiquement.
- `Data/MapDataHierarchy/MapDataHierarchy.bin` : **1 380 584 octets**, binaire,
  non JSON. L'échec de lecture JSON est signalé ; il ne prouve pas une corruption.
- `Data/MapEvent/` : **1 085 fichiers JSON `.bin`** lus. Champs `MapID`, `Name`,
  conditions, étapes et directives comme `StartSpawner`, `EndSpawner` ou
  `HpRemaining`. Les références de spawners restent opaques sans leur définition.
  **928 des 1 085 références `MapID`** aboutissent à `Map.ID` ; les 157 autres,
  dont `600064`, `600066` et `600067`, restent non résolues.
- Le rapport de déchiffrement contient **161 échecs WorldMap et un NavSignature**,
  famille de protection non prise en charge dans ce passage. Ils limitent la
  couverture ; ce n'est pas une preuve d'absence de fonds de carte ou de navigation.

Les systèmes de coordonnées, unités, transformations, fonds exploitables et
trajets restent **non interprétés ou non recherchés**. Les événements ne donnent
pas à eux seuls des positions ; aucun état de présence en temps réel n'est déduit.
Les 23 apparitions périodiques ne constituent pas un inventaire exhaustif des
spawns. Cette étape dépend des conteneurs et schémas (#41/#42), puis du lien
explicite spawner/ressource/définition.

## Ailes, apparences et familiers — #82

| Source | Lignes | Champs et relations |
| --- | ---: | --- |
| `Wing` | 86 | Identité, race, grade, icône, costume, groupes d'effets/progression, statistiques et visibilité. |
| `WingEnchant` / `WingEnchantEffect` | 30 / 309 | Groupe/niveau, items et coûts, probabilités brutes ; groupe/niveau/statistiques. |
| `Skin` / `SkinSet` | 1 579 / 128 | Identité, catégorie, grade, textes et icônes selon race/sexe, ressources, statistiques ; membres d'ensemble. |
| `VehicleList` | 213 | Identité, `CreatureType`, ressources, item et item d'âme, groupe de statistiques et progression. |
| `VehicleCreatureSlot` / `VehicleCreatureLevel` | 5 / 50 | Famille, niveaux d'emplacements, groupes d'options, coûts et textes ; expérience et probabilités de grade. |
| `VehicleCreatureOption` / `VehicleStat` | 2 062 / 15 | Options par groupe, grade, poids, statistique et bornes ; statistiques par groupe/niveau. |
| `VehicleLevel` / `VehicleCollection` | 3 / 51 | Coûts en âmes et expérience ; collections et récompenses. |

Les 86 groupes d'effets d'ailes aboutissent, ainsi que 90 références d'items des
coûts. En revanche, `Wing.EnchantGroup` vaut `None` sur toutes les lignes :
l'existence de `WingEnchant` ne prouve pas sa liaison à chaque aile. Les
848 membres de `SkinSet` aboutissent à `Skin.Name`. Exemple : aile `30100300`,
`Wing_L_WingA_003`, icône `Icon_WingA_003` ; apparence `10001101`,
`WP_0001_T03_GS`. Les conditions d'affichage et variantes ne prouvent aucune
possession par un personnage.

Le rattachement aux familiers s'appuie sur les types et textes explicites de
`VehicleCreatureSlot`, notamment `UI_BUFFGROUP_PET_*`, plutôt que sur le seul
nom `VehicleList`. Répartition des définitions : `Intellect` 41, `Feral` 65,
`Nature` 48, `Trans` 46, `Special` 13. Les titres de bonus français contiennent
**Intellia, Bestia, Natura, Varius, Singulia** dans cet export. Ce sont des textes
de bonus, pas la preuve d'un libellé de famille autonome : ne pas remplacer
automatiquement les noms de la cible (Cogni, Fera, etc.) par ces extraits.

La [projection #82](game-data-82.md) retrouve désormais les libellés autonomes
par `EnumString.ECreatureType` et les clés `UI_MAINTARGET_CREATURETYPE_*` :
Cogni/Fera/Natura/Varian/Special en anglais, Intellia/Bestia/Natura/Varius/Spéciaux
en français. Ces références explicites remplacent l'hypothèse fondée sur les
seuls textes de bonus ; ces derniers restent conservés séparément.

Les 213 items associés, 200 items d'âme, 200 groupes de statistiques et
45 groupes d'options d'emplacements aboutissent. Exemple : `1001`,
`BlackDragon_01`, type `Special`. Le candidat `Pet.bin` n'a pas été trouvé sous
ce nom ; les tables réellement présentes ci-dessus fournissent les pistes
vérifiées. L'obtention transite par les items et collections ; les règles de
probabilité et liens manquants restent à interpréter.

## Daevanion — #83

`DaevanionBoard` contient **54 plateaux** : ID, classe, niveau requis, type de
points, devise de réinitialisation, titre, icônes et textes de bonus. Exemple :
plateau `11`, Gladiator, `STR_Daevanion_Board_Title_01`, icône `Weapon_Sword`.

`DaevanionNode` contient **12 150 lignes** : ID, plateau, ligne/colonne, type,
grade, niveau, condition et valeur associée, points/coûts, titre/image et
`Value01..10`. Les 12 150 références de plateau aboutissent. Répartition :
5 712 `None`, 5 592 `Stat`, 792 `SkillLevel`, 54 `Start`. Il ne faut donc pas
présenter 12 150 cases comme des bonus utilisables.

Les 792 valeurs `Value01` des cases `SkillLevel` correspondent à `Skill.ID`,
après conversion numérique explicitement déclarée. Exemple : case `110037`,
plateau 11, valeur `11720000`, `Value02 = "1"`. La fonction de cette seconde
valeur reste à confirmer avant de l'appliquer à une compétence ; le lien de clé
ne valide pas à lui seul un effet de progression.

Pour les cases `Stat`, les noms sont recherchés via
`String_StatName_{Value01}_body`, pas par une jointure inventée vers `EnumString`.
Les 5 592 références ont un texte anglais et français dans cette copie.
Exemple : `CombatSpeed`, valeur brute `150`. L'existence du texte n'établit pas
l'unité de la statistique. Les connexions entre cases n'ont pas été trouvées
comme champs explicites dans ces deux tables : ligne/colonne et condition ne
prouvent pas une arête entre cases voisines. Le graphe et les règles d'activation
restent à rechercher avant la projection finale.

## Références incomplètes, textes et images

Trois relations de tables présentent des références non résolues :

| Relation | Résolues / références | Exemples manquants |
| --- | ---: | --- |
| `Quest.CompleteMapId → Map.ID` | 1 161 / 1 213 | 52 occurrences de la carte 21. |
| `QuestStep.GoalList.MapId → Map.ID` | 2 664 / 2 748 | 84 occurrences de la carte 21. |
| `WorldMapUIRegion.SubzoneId → Subzone.ID` | 215 / 227 | 12 références, dont 5019, 5020, 5021. |

Ces absences ne sont ni corrigées en choisissant un ID proche, ni attribuées au
debug, à une suppression ou à un défaut de mapping sans preuve. Les liaisons
vides (`Skin.SkillAbnormal`, certains groupes de pass, `Wing.EnchantGroup`, etc.)
sont signalées sans être annoncées comme vérifiées.

Les textes FText et les classifications `EnumString` sont inspectés avec leurs
clés et couvertures EN/FR. Les clés d'une map de statistiques ne sont pas des
clés de localisation. Le rapport conserve les textes absents et tronque les
exemples longs en le signalant ; les sondes gardent les valeurs complètes.
La liste de classes/types localisés n'est pas reconstituée à partir de noms
anglais si l'énumération correspondante est absente.

Quelques couvertures de clés FText distinctes (disponibles EN et FR / recherchées) :
`Title` 3 034/3 034, `BattlePass` 68/68, `PeriodItemCollection` 8/16,
`NpcData` 5 668/5 685, `Wing` 86/86, `Skin` 3 158/3 158 et
`VehicleList` 213/213. Ce sont des clés, pas des nombres d'entrées nommées.
`Quest` référence `QuestString` par une clé technique et n'a pas de FText
direct ; les recettes et matériaux réutilisent notamment les textes de leurs
items. Zéro FText direct ne signifie donc pas absence de traduction de la famille.

Les champs `Icon`, `IconImg`, `Image`, références de costume et de ressource
sont repérés par table comme **candidats**. Le rapport d'extraction UI existant
contient **2 447 sprites et 836 alias**, avec le même mapping et un échec connu
(`UT_FWindow_Enchant_NextSlot_Sprite.uasset`, texture cuite absente). Cela ne
prouve pas la résolution de chaque icône de ces nouvelles familles. Le montage
des packages et l'extraction native en production restent dans #46 ; les
référentiels existants sont réutilisables sans introduire .NET côté joueur.

## Ordre proposé après objets et compétences

1. **#79 : recettes puis quêtes.** Les composants/résultats réutilisent les
   items ; les étapes et récompenses établissent ensuite les liens communs.
   Préserver dès cette étape les références de cartes non résolues.
2. **#80 : exploits, titres, pass et ravitaillement.** Réutiliser items, quêtes,
   objectifs et récompenses, puis valider les périodes et pistes des pass.
3. **#82 : ailes, apparences et familiers.** Sources tabulaires déjà lisibles,
   dépendances principales items/statistiques/icônes ; pas de dépendance aux
   coordonnées de cartes. Confirmer les règles de progression et groupes vides.
4. **#83 : Daevanion.** Dépend des compétences, statistiques et coûts communs ;
   rechercher les connexions avant d'exposer un graphe navigable.
5. **#81 : monde, PNJ, collecte et cartes, en deux étapes.** Identités et relations
   tabulaires peuvent avancer dès #79 ; finaliser apparitions et cartes après
   résolution des ressources binaires/protégées et transformations de coordonnées.

Le butin et ses définitions restent transversaux. #76 fournit le suivi du dernier
changement, pas un historique de valeurs ; les comparaisons impossibles et les
intervalles incomplets restent signalés. L'essai de comparaison sur deux builds
réels attend une seconde copie fiable. Aucun nouveau domaine n'est ajouté à la
cible sur la seule base des autres tables découvertes.
