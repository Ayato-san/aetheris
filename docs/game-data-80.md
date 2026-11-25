# Exploits, journaux, missions, titres et pass — #80

L'export Rust `export-catalogue` ajoute les familles de #80 à la même base que
les objets, compétences et [recettes/quêtes](game-data-79.md). Il utilise les
sources du client recensées dans [#75](game-data-sources.md), sans déduire la
possession, la progression ou l'éligibilité d'un personnage.

## Sources, provenance et langues

Les sources explicitement recherchées sont `Achievement`, `AchievementStep`,
`AchievementReward`, `AchievementString`, `AchievementGroup`,
`AchievementGroupSet`, `AchievementGroupReward`, `Journal`, `Title`,
`TitleCategory`, `BattlePass`, `BattlePassGroup`, `SeasonSchedule`,
`SeasonScheduleGroup`, `SeasonContents`, `SeasonPickupContents`, `EventMission`,
`PeriodItemCollection` et `PeriodItemCollectionList`.

Chaque source conserve ses lignes complètes et typées dans `catalogue_sources`,
avec build, structure, empreintes et état de lecture. Les champs sans projection
métier restent dans cette preuve complète ; aucune interprétation de leur nom
ou de leur valeur n'est inventée. Les tables supplémentaires déjà découvertes
dans #75 restent des pistes pour les autres familles de l'epic.

Les parties communes de #79 sont partagées dans `catalogue_projection.rs` et
`catalogue_projection.sql` : références respectant le type de clé, provenance,
conditions, textes, dictionnaires et empreintes. Les codes internes utilisent
des IDs `SMALLINT` bornés à 1–32767 ; les IDs source restent dans leur espace
d'origine. Les dictionnaires ne deviennent pas des identités de jeu par ordre
de ligne. Les races et statistiques réutilisent les référentiels existants.

`catalogue_references` distingue cible unique résolue, groupe explicitement
résolu à plusieurs lignes (`group`), absence, ambiguïté et cible non recherchée.
Une correspondance de clés ne valide ni une formule, ni une présence en jeu,
ni l'application d'une condition. Les hypothèses restent marquées candidates.
Les références non résolues ne sont pas remplacées par un ID voisin ou un zéro.

Les textes anglais et localisés sont conservés séparément, avec clé et état de
repli dans `catalogue_texts`. `catalogue_text_values` garde les textes FText
supplémentaires de ces familles, notamment les titres, descriptions et lieux
des collections. Une traduction manquante laisse la valeur localisée nulle ;
le repli ne réécrit pas l'original. Les marqueurs des descriptions sont conservés.
Une classification sans libellé client vérifié garde son code et des libellés
nuls, sans assimiler arbitrairement les grades des titres à ceux des objets.

## Exploits, collections et paliers

| Table | Contenu |
| --- | --- |
| `achievement_groups` | ID et clé, catégorie/type, collection majeure, textes, race, prérequis et partage de compte. |
| `achievement_group_sets` / `achievement_group_members` | Regroupements nommés et membres ordonnés, avec références manquantes conservées. |
| `achievements` | `AchievementId`, groupe et sous-groupe, objectif typé et valeurs, grade, cycle de remise à zéro, exploit requis, types de collection et textes. |
| `achievement_steps` | `DataId`, exploit, numéro de palier, seuil brut et textes propres au palier. |
| `achievement_rewards` | ID source, exploit et palier, liaison à l'étape correspondante, points bruts. |

Un palier est identifié par son `DataId`, et relié à son exploit par
`AchievementId`. Les récompenses sont rattachées au couple **exploit + numéro de
palier**, sans déduire un ID d'étape à partir de leur ID. Les récompenses de
groupe gardent leur seuil `CompleteCount`, distinct du compte d'un objectif.
Les récompenses en items sont décrites dans `catalogue_item_links`, avec
quantité/enchantement, rôle et origine exacte ; les quantités explicitement
nulles ou égales à zéro ne sont pas converties en un item par défaut.

Le titre de collection explicite est prioritaire pour nommer un exploit. Sinon,
le titre de son premier palier ordonné sert de nom dérivé. Les titres et textes
de tous les autres paliers restent séparés. Une définition sans texte disponible
n'est pas supprimée.

`ObjType` et `ObjValues` restent distincts ; les valeurs d'objectif sont gardées
comme liste JSON typée. Une valeur ne devient pas un identifiant de PNJ, une
quantité, une durée ou une formule sur la seule base de sa position. Les cycles
et prérequis connus sont conservés, sans calcul de progression du joueur.

## Titres

`titles` conserve identité, noms/descriptions, catégorie, grade, race, cible
d'application, catégorie d'équipement, ordre et indicateurs de visibilité.
`title_categories` conserve les catégories et libellés source.

`title_stats` sépare les effets de **possession** (`StatList`) de ceux du titre
**équipé** (`EquipStatList`). Les valeurs restent brutes, reliées à `stats` ;
elles ne sont ni additionnées ni converties en pourcentages sans règle validée.
Les périodes de disponibilité restent en ticks dans `catalogue_periods`.

Les items avec `EItemUseEffect::GetTitle` sont recherchés comme moyens
d'acquisition candidats : leurs valeurs numériques sont contrôlées contre
`Title.ID`. Les items correspondants sont indexés dans `catalogue_item_links`
avec le rôle `granting_item`. Une référence inconnue reste dans les diagnostics ;
un nom technique contenant « Quest » ne suffit pas à attribuer un titre à une
quête. Une modification de prix d'un item donnant un titre ne modifie pas, par
propagation, la définition du titre.

Les décorations, fonds et VFX restent des références de ressource dans la source
complète. Ils ne sont pas automatiquement assimilés à des icônes. Les icônes
explicitement référencées par les exploits et groupes utilisent le référentiel
d'images existant ; l'absence d'une image extraite est signalée.

## Saisons et missions

`season_schedule_groups`, `seasons` et `season_contents` séparent les groupes de
calendrier, les périodes numérotées et leurs contenus typés. La correspondance
**nom de groupe de calendrier + numéro de saison** est conservée dans
`season_content_schedules`, sans choisir une saison depuis la date courante.
Les périodes de saison et de récompense restent distinctes.

Pour `ESeasonContentsType::SeasonMission`, les listes Light/Dark de
`SeasonPickupContents` mélangent noms de groupes d'exploits et chaînes d'IDs
numériques d'exploits. `mission_group_sets` garde l'ordre, la race, la référence
originale et la cible candidate de chaque forme. Les autres types de contenu ne
reçoivent pas arbitrairement cette interprétation. Les missions d'événement de
`EventMission` gardent leurs groupes par faction, saison et période d'activation.

Les rattachements de calendrier et listes de missions restent des associations
de définitions, pas une preuve de disponibilité actuelle. Les autres règles de
classement, affixes et contenus du client restent dans leurs sources complètes
pour leur traitement propre, notamment #81.

## Pass Daeva

`battle_pass_groups` conserve les groupes, race, saison candidate, limites
d'achat et d'expérience, item d'achat d'expérience et ordre d'onglet.
`battle_passes` conserve chaque piste, son groupe, son exploit d'expérience,
le booléen source `bIsPremium`, niveau requis, quête de déverrouillage et
altération éventuelle.

Les coûts sont dans `catalogue_costs` : devise, montant brut et chemin source,
avec position pour `ExpCosts`. Cette position n'est pas transformée d'office en
numéro de niveau. La progression est accessible par l'exploit d'expérience et
ses paliers/récompenses. Les items de déverrouillage et récompenses directes du
pass restent des rôles distincts dans `catalogue_item_links`. Les conditions de
puissance, niveau d'item et abonnement restent typées dans `catalogue_conditions`.

**`bIsPremium = false` ne signifie pas gratuit** : certaines pistes du client
portent ce booléen avec un coût de déverrouillage en `GameCash`. Le coût, les
conditions et le booléen restent séparés ; aucun type de piste n'est déduit
uniquement d'un suffixe `Pay` ou `Free`.

Les périodes d'activation et de souscription sont conservées séparément, en
ticks. Aucun ancien pass ou calendrier n'est supprimé parce qu'une date est
passée. La conversion de l'époque, du fuseau et des sentinelles n'est pas
supposée ; un zéro explicitement présent reste un zéro source.

## Ravitaillement et journaux

`supply_request_groups` conserve chaque ligne de `PeriodItemCollection`, avec
race, niveau minimal, type d'onglet, textes et indicateur de date.
`supply_requests` conserve les tâches de `PeriodItemCollectionList`, textes,
`TargetCount` et expérience de récompense. `supply_request_variants` relie une
tâche à **toutes** les variantes partageant le nom de groupe ; le moteur ne
sélectionne pas un niveau ou une race à la place du joueur.

Les items demandés (`requested_item`) et récompenses (`reward`) sont séparés
dans `catalogue_item_links`. Dans le client, `TargetItems.Count` peut valoir zéro
alors que `TargetCount` vaut trois : les deux valeurs sont gardées, sans les
écraser ni déclarer tous les items simultanément requis. Le renouvellement
n'est pas inventé depuis les suffixes des noms techniques ; les périodes ou
conditions non établies restent inconnues.

`journals` conserve les définitions de carte de `Journal`, leur race, ordre,
référence de carte et position/zoom de vue par défaut. Le nom de carte est un
libellé dérivé. Ces coordonnées ne sont pas présentées comme des coordonnées
mondiales ou des points de collecte. Les collections sont également conservées
dans les champs `Collection*` des exploits ; aucun lien collection/journal
géographique n'est inventé.

## Publication, versions et validation

Le format SQLite passe à **11**, sans migration des formats de développement.
Les parties communes de #79 sont déplacées, pas dupliquées. Les 12 nouvelles
familles ayant un identifiant source participent au suivi du dernier changement
de #76 ; les catégories nommées restent des référentiels partagés.

Les empreintes des exploits incluent leurs paliers, textes et récompenses. Les
pass incluent la définition de leur progression explicite. Les relations
d'acquisition de titres et les libellés de statistiques partagés sont suivis
sans incorporer des propriétés sans rapport de l'item donneur. Les
changements de langue et d'ordre des lignes ne deviennent pas des modifications
du jeu. Les limites de comparaison, rétentions de sources et publications
atomiques restent celles de l'export commun.

Les tests ciblés couvrent les correspondances entre paliers/récompenses,
objectifs opaques, bonus de possession/équipement, titres accordés par items,
coûts des pistes avec `bIsPremium = false`, périodes anciennes et zéros sources,
variantes de ravitaillement, listes mixtes de missions et provenance, coordonnées
brutes, dépendances des empreintes et repli de traduction. Une comparaison sur
deux builds réels attend toujours une seconde copie fiable.

### Vérification sur le client fourni

Export effectué sur la copie rattachée au build Steam **26581260**, en anglais
et français (`fr-FR`). La région reste inconnue ; ce résultat ne certifie pas
le mapping pour d'autres versions ou régions.

| Projection | Lignes |
| --- | ---: |
| Exploits | 6 434 |
| Paliers | 17 342 |
| Récompenses de palier | 13 743 |
| Groupes / ensembles de groupes d'exploits | 473 / 167 |
| Titres / catégories | 1 517 / 8 |
| Bonus de possession / d'équipement | 846 / 1 414 |
| Pass / groupes de pass | 88 / 44 |
| Saisons / groupes de calendrier | 67 / 37 |
| Contenus saisonniers / missions d'événement | 116 / 1 |
| Requêtes de ravitaillement / variantes de groupe | 1 122 / 356 |
| Journaux | 13 |

Les 19 sources recherchées pour #80 sont toutes décodées. Les 17 342 paliers et
13 743 récompenses retrouvent leur exploit et, pour chaque récompense, son
palier. Les 116 contenus retrouvent leur calendrier par groupe et numéro de
saison. Les 22 références de groupes/IDs des listes de missions sont résolues
comme candidates. Les 28 828 liens d'items de ces familles retrouvent leur item,
dont 1 477 liens d'acquisition de titres. Les variantes de ravitaillement
produisent 57 900 associations : ce sont des correspondances de groupe, pas
57 900 tâches ni des conditions d'éligibilité validées.

Exemples et limites observés :

- `battlepass_Light_S1_1_Pay` (100111) conserve un coût de 1 000
  `ECurrencyCategory::GameCash` et `bIsPremium = false`. Cette contradiction
  apparente ne devient pas un pass gratuit dans la projection.
- 106 références de paliers vers `AchievementString.Name` sont absentes ;
  elles correspondent aussi à 82 références au niveau du nom dérivé d'exploit.
  Exemple : exploit 234240500, clé `STR_Group_Season_4_L_Thema_234240501`.
  Les définitions et références originales restent présentes.
- Les 356 variantes de groupe de ravitaillement référencent des descriptions
  courtes et longues absentes des deux dictionnaires : 712 états de texte
  manquant. Les noms de ces groupes et les textes des 1 122 tâches sont disponibles.
- Le journal 2001 référence `Map.ID = 21`, absent de la source `Map`. Aucune
  carte de remplacement n'est choisie. Les 13 définitions restent présentes.
- Les dates restent en ticks et les valeurs d'objectifs restent typées sans
  interprétation de leurs paramètres. Les missions et pass anciens sont conservés.
  La couverture métier reste partielle, même quand toutes les références
  structurelles d'une famille sont résolues.

L'export complet conserve également les 9 452 items, 16 283 compétences,
6 708 altérations, 1 416 recettes et 1 429 quêtes. Il contient 175 sources
complètes et 277 815 entrées suivies dans 19 familles. Cette reconstruction
établit une référence initiale ; elle ne prétend pas observer une modification
de patch. Les contrôles d'intégrité et de clés étrangères de l'export passent.

Base de validation : `target/game-data-validation/data.sqlite`, format 11,
708 599 808 octets. La base précédente est conservée sous
`target/game-data-validation/data-before-80.sqlite`. Empreinte du lecteur :
`69b5e7cd2e0764e040bc4fefca16a9de84146adee0caac0d43b17e3f7b45c857`.
Le résumé reproductible de cet export est dans
`target/game-data-validation/catalogue-80-summary.json`.

Validation du code : 63 tests unitaires `game_data`, 35 tests d'intégration
(catalogue, items, compétences et USMAP), Clippy sans avertissement et
compilation Windows de l'application réussis. Le test enfant de processus
du catalogue reste ignoré dans l'exécution globale par conception.
Exécutable : `target/windows-scroll-preview/debug/aetheris.exe`.
Cette étape ajoute les projections SQLite ; elle n'ajoute pas de nouvelle
page de consultation dans l'application.
