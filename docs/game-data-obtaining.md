# Sources d’obtention des objets

L’export commun indexe les indications d’obtention déclarées par le client et
les objets explicitement produits par les tables de récompenses, de fabrication,
de collecte et de boutiques. Il conserve leur provenance dans `data.sqlite`.
Une indication du client peut concerner du contenu désactivé, saisonnier ou de
test ; elle ne garantit pas que l’objet soit actuellement obtenable en jeu.

## Familles déclarées dans ItemGetRoute

Toutes les propriétés non vides sont indexées, y compris les booléens et les
routes sans identifiant. Les noms ci-dessous sont ceux du client, avec leurs
fautes éventuelles. Une propriété vide ne crée aucune source d’obtention.

| Famille | Propriétés source |
| --- | --- |
| Monstres | `MonsterGetRoutes` |
| Fabrication, conversion | `CraftGetRoutes`, `ConversionRecipeGetRoutes` |
| Collecte | `GatherGetRoutes` |
| Quêtes, ascension | `QuestGetRoutes`, `AscensionInfo` |
| Exploits | `AchievementInfo` |
| Donjons, raids, activités journalières | `PartyDungeonGetRoutes`, `TrialDungeonGetRoutes`, `RaidInfo`, `DailyInfo`, `SuppressionInfo`, `AwakenInfo` |
| Boutiques de PNJ, monnaies, boutique commerciale, boutique aléatoire | `NPCShopInfo`, `CoinShopInfo`, `BMShopInfo`, `RandomShopInfo` |
| Coffres et sélection de récompenses | `RewardBoxInfo`, `HiddenCubeInfo` |
| Décomposition | `DecomposeItemInfos` |
| Exploration | `ExplorationSealInfo`, `ExplorationGarrisionInfo` |
| Événements, défis de boss | `FieldEventInfo`, `BossChallengeInfo` |
| Monolithes | `MonolithInfo` |
| Pass, connexion, abonnement | `BattlePassInfo`, `MembershipPassInfo`, `AttendanceInfo`, `SubscribeInfo` |
| Classements, arène | `RankingInfo`, `bArena` |
| Collections périodiques, apparences, montures | `PeriodCollectionInfo`, `SkinInfo`, `bVehicle` |
| Autres indications | `OnboardingInfo`, `ExtraInfo` |

Les textes et catégories manuels de `ItemGetRouteManual` sont également
indexés. Leurs valeurs restent dans la ligne source plutôt que d’être
interprétées comme des identifiants.

## Tables de résultats explicites

| Sources | Résultats indexés |
| --- | --- |
| `NpcLoot` | Liste des objets et objet d’âme de monture |
| `CraftRecipe`, `ConversionRecipe` | Produit normal, produit de réussite critique et récompenses d’échec |
| `GatherSource`, `GatherSourceMaterial` | Matériau, objets supplémentaires, objets d’échec et matériau de quête |
| `ArcanaCreate` | Arcana créé ; matériaux consommés exclus |
| `GameShopGoods`, `RandomShopGoods`, `BMShopGoods` | Objet vendu ; monnaies et paiements exclus |
| `QuestReward`, `QuestClassReward` | Récompenses normales et au choix |
| `AchievementReward`, `AchievementGroupReward` | Récompenses d’exploits et de groupes |
| `BattlePass`, `Subscribe`, `PeriodItemCollectionList`, `VehicleCollection` | Récompenses explicites ; objets à remettre pour une collection exclus |
| `DungeonRewardInfo`, `RewardDungeon`, `RewardOdEnergyCube`, `RewardMentor` | Objets affichés ou attribués, dont récompenses de plafond |
| `RewardMonolith`, `AscensionReward`, `BossChallengeReward` | Récompenses propres à ces activités |
| `SeasonRankingRewardItem`, `RewardRaceRanking`, `RewardArena` | Récompenses de classement et d’arène |
| `BoxReward`, `BoxClassReward` | Objets directs et au choix |
| `RewardItem`, `RewardPackageItem` | Définitions de récompenses et contenu de packages ; leur seule présence ne prouve pas une source accessible |

Les tables auxiliaires de donjons, PNJ, cartes, zones, événements, saisons,
pass, exploits, boutiques et packages sont décodées et conservées pour retrouver
les conditions et poursuivre les chaînes de relations. Le fichier et la structure
USMAP peuvent porter des noms différents ; les alias sont explicites dans le
lecteur, après vérification du décodage.

## Représentation SQLite

- `item_obtaining_types` : dictionnaire des familles effectivement rencontrées,
  identifiants `SMALLINT` bornés à 1–32767.
- `item_obtaining_sources` : objet, famille, table source, numéro et chemin de
  la ligne, nature de la preuve (`client_route`, `manual`, `output`), quantité
  minimale/maximale et poids brut éventuel. Une occurrence n’est pas fusionnée
  avec une autre occurrence du même objet.
- `item_obtaining_routes` : références rencontrées dans les routes client,
  avec table cible, référence et état de résolution. Une cible inconnue reste
  `NULL`, plutôt que d’être devinée à partir d’un numéro ressemblant.
- `catalogue_sources` : lignes décodées complètes, compressées, contenant
  conditions, races, classes, niveaux, dates, coûts et contexte.
- `catalogue_coverage` : couverture et erreurs de lecture, distinctes des
  absences de données.

Les références sont contextuelles : `RandomShopInfo` cible `RandomShopGoods`,
`PeriodCollectionInfo` cible `PeriodItemCollectionList`, et
`AchievementInfo.TableId` peut cibler un exploit ou un groupe d’exploits.
Les maps Unreal ignorent leur liste `Removed`. Les booléens faux ne créent pas
de route. `BossChallengeInfo` n’est actif que si son booléen l’indique.

Lister toutes les occurrences pour un objet :

```sql
SELECT t.code, s.evidence, s.source_name, s.source_row, s.source_path,
       s.quantity_min, s.quantity_max, s.weight_raw
FROM item_obtaining_sources AS s
JOIN item_obtaining_types AS t ON t.id = s.type_id
WHERE s.item_id = :item_id
ORDER BY t.code, s.source_name, s.source_row, s.source_path;
```

Retrouver les objets associés à un PNJ :

```sql
SELECT DISTINCT item_id
FROM item_obtaining_routes
WHERE target_table = 'NpcData' AND target_reference = :npc_id;
```

Les nouvelles relations participent à la détection des changements d’un objet
entre builds, sans conserver un historique des anciennes valeurs.

## Limites de cette étape

Les résultats explicites et les routes client sont indexés ; les chaînes de
packages ne sont pas encore déroulées jusqu’à toutes leurs activités d’origine.
Une définition `RewardPackageItem` reste identifiable comme telle. Les conditions
restent consultables dans la source et ne sont pas évaluées pour un joueur.
La normalisation des pools et l’attribution des tirages des donjons Party/Raid
sont maintenant interprétées selon les règles décrites ci-dessous. Les autres
contextes restent non interprétés.
Les règles exclusivement serveur et les échanges entre joueurs ne peuvent pas
être déduits de ces tables client.

Sur le lot fourni, deux tables auxiliaires restent en erreur :
`FieldEventPlatformerDilemma` n’a pas de structure correspondante dans l’USMAP,
et `GlobalSetting` présente un champ de structure dupliqué. Les diagnostics
restent dans la couverture ; ces données ne sont pas déclarées extraites.

`ItemGetRoute` contient également 1 355 lignes dont l’ID objet est absent de
`Item.bin`. Elles restent dans la source complète et ne créent aucun objet
artificiel ni relation violant la clé étrangère vers `items`.

## Mesure sur le build Steam 26581260

L’index couvre **28 familles client actives**, contre les quatre familles
précédemment parcourues. L’export produit
**335 687 occurrences d’obtention pour 7 401 objets** :
8 859 indications client (la table manuelle est vide dans ce lot) et
326 828 occurrences de résultats
explicites, provenant de 30 tables de résultats. Toutes les références d’objets
de ces sorties sont résolues. Les indices de route comportent 382 911 références
vers des objets, PNJ, cartes, zones, activités et autres définitions ;
ces 382 911 références sont toutes résolues dans ce lot.

Ces nombres comptent des occurrences et des références techniques ; ils ne
représentent pas autant de lieux ou d’activités distincts. Les 162 tables source
décodées incluent les tables déjà utilisées pour les objets et compétences.

Par exemple, l’objet `110120003`, « Sabre d’extermination de Ludra », possède
une route de raid et une route de coffre, ainsi que des définitions de contenu
dans `BoxReward` et `RewardPackageItem`. L’index conserve ces preuves séparées ;
les taux conditionnels dans le pool sont distincts des règles d’attribution du
pool par le donjon.

## Affichage dans l’application de débogage

La sélection d’un objet affiche sa fiche et deux onglets : **Obtention** et
**Utilisations**. Les listes montrent le nom anglais ou localisé, la famille,
la quantité connue et les informations de taux pour les butins. Les objets
associés (produit de recette, coffre, contenu du coffre) sont cliquables.
Les doublons de présentation identiques sont regroupés ; les conditions et
occurrences originales restent conservées dans les sources. SQLite compte et
sélectionne au plus 50 relations par requête, en arrière-plan, pour le seul
onglet actif. La fiche s’affiche indépendamment de ces relations. Le défilement
ne reconstruit que les lignes visibles de la liste ; les pages donnent accès à
l’ensemble des résultats. La recherche attend 180 ms après la dernière frappe
pour éviter de lancer une lecture de fiche pour chaque caractère.

`item_relations` est une projection préparée lors de l’export. La fiche ne
décompresse aucune table supplémentaire pour lire ces relations. Les utilisations
proviennent de champs explicites : composants et remplacements de recettes,
matériaux d’arcana, coûts de progression, objets de collection, tickets d’entrée,
monnaies de paiement et contenus directs de coffres. Elles ne constituent pas
encore une interprétation de toutes les mécaniques d’usage du client.

Les taux génériques restent inconnus en dehors du domaine confirmé. Pour les
loots de donjon, les nouvelles tables de pools et d’attribution fournissent
les taux à l’interface, sans réécrire les poids des tables sources.
Les taux confirmés sont affichés en pourcentage, y compris zéro.

## Pools de butin des donjons

La règle d’attribution a été confirmée par l’utilisateur sur Ludra, et la
séquence correspond exactement aux valeurs du client. Elle est appliquée aux
entrées `DungeonType=Party/Raid` reliées explicitement à leurs récompenses,
avec difficulté, race et groupe source conservés. `Dungeon.bin` décrit aussi
des événements, quêtes et instances de carte : leur présence dans ce fichier
ne suffit pas à leur appliquer cette règle.

La chaîne est `Dungeon.RewardOdEnergyCube` → `RewardOdEnergyCube.RewardCommon`
(ou récompense de boss intermédiaire) → `RewardPackage.Name` →
`PackageItems[].Item` → `RewardPackageItem.PackageName`. Les références directes
`Dungeon.RewardLight/RewardDark` sont également suivies lorsqu’elles pointent
vers ces définitions. Les noms de boss proviennent des références explicites
et des localisations du client.

**À l’intérieur d’un pool**, pour un objet donné :

```text
taux conditionnel = somme des poids des occurrences de cet objet
                    / somme des poids de toutes les entrées du pool × 100
```

Les entrées non résolues restent dans le dénominateur. Les variantes d’un même
objet additionnent leurs poids ; leurs quantités ne multiplient pas le poids.
Les doublons introduits par les liens vers plusieurs coffres ne sont pas
comptés : le calcul utilise la liste source `RewardPackageItem.Items`.
Un pool de poids total zéro n’a aucun taux calculable.

**Attribution du pool**, avec `U = 100 000 000` :

```text
tirages garantis = RewardPackage.PackageItems[].Prob div U
taux d’un tirage supplémentaire = (Prob mod U) / U × 100
```

`Count` est conservé séparément des tirages. Les parents rencontrés dans les
données de donjons fournies ont tous `Count=1`. Les probabilités des différentes
pools ne sont pas normalisées entre elles.

Ludra, difficulté source `Normal` (`Conquest` / `Conquête` en jeu) :

| Pool | Code | Attribution |
| --- | --- | --- |
| 1 | `PKG_REFORGEDABYSS_RAID_EQUIP_E_BOSS` | 1 tirage à 5 % |
| 2 | `PKG_REFORGEDABYSS_RAID_EQUIP_U_BOSS` | 1 tirage garanti |
| 3 | `PKG_REFORGEDABYSS_RAID_EQUIP_L_BOSS` | 3 tirages garantis |
| 4 | `PKG_RAID_EQUIP_COMMONDROP_1S_01` | 4 tirages garantis + 1 à 50 % |
| 5 | `PKG_REFORGEDABYSS_RAID_MATERIAL_DRACONIC_KEY` | 1 tirage garanti |
| 6 | `PKG_PARTYDUNGEON_REMNANT_MATERIAL_DRACONIC` | 1 tirage garanti |
| 7 | `PKG_PARTYDUNGEON_MATERIAL_SMELTSTONE_45` | 4 tirages garantis + 1 à 80 % |
| 8 | `PKG_REFORGEDABYSS_RAID_ETC` | 1 tirage à 5 % |

Dans le pool 1, le sabre `110120003` pèse `6 250 000` sur `100 000 000` :
**6,25 % dans le pool**, lequel est attribué une fois avec **5 % de chance**.

Les `RewardItem.Items` du même groupe décrivent des attributions directes
indépendantes. Elles utilisent ici la même unité d’attribution ; leurs poids
ne sont pas additionnés pour créer un pool artificiel. Par exemple, une
valeur de `600 000 000` représente six tirages garantis, avec la quantité
minimale/maximale par attribution conservée séparément.

La base expose `reward_pools` (total et provenance), `reward_pool_entries`
(entrées, poids et taux conditionnels) et `dungeon_reward_rolls` (contexte,
tirages garantis, tirage supplémentaire et provenance). L’interface joint ces
tables pour présenter les deux niveaux, sans calculer un taux global d’au
moins une obtention sur plusieurs tirages ni supposer des tirages indépendants.
Un changement de la règle d’attribution du pool participe au suivi de version
de tous les objets qu’il contient.
