# Familiers, ailes et apparences — #82

La projection Rust `companions` complète [l'export commun](game-data-export.md)
dans `data.sqlite`. Les définitions gardent leurs IDs source, distincts des IDs
d'items associés. Elle n'infère ni possession ni progression du personnage et
n'ajoute pas de page de consultation dans l'application.

## Sources et couverture

Les 20 sources recherchées explicitement sont :

- `VehicleList`, `VehicleCreatureSlot`, `VehicleCreatureLevel`,
  `VehicleCreatureOption`, `VehicleStat`, `VehicleLevel`, `VehicleCollection` ;
- `Wing`, `WingEnchant`, `WingEnchantEffect` ;
- `Skin`, `SkinSet`, `SkinCollection`, `SkinCombine`, `SkinCombineCost`,
  `SkinGacha`, `SkinGachaGroup`, `SkinMaterial`, `SkinPattern`, `SkinMotion`.

L'alias de structure de `VehicleList` est `VehicleDataTableItem`. Les sources
complètes et typées restent dans `catalogue_sources`, avec build, mapping,
structure, empreintes et état de lecture. Les ressources, paramètres de
déplacement et champs sans projection ne sont pas supprimés de ces preuves.
Le décodage structurel ne certifie pas une compatibilité générale version/région.

`catalogue_entity_sources`, `catalogue_references`, `catalogue_texts` et
`catalogue_text_values` réutilisent les preuves communes de #79/#80 : origine
exacte, références uniques/groupées/manquantes/ambiguës/non recherchées, langues
et état du repli. Une relation candidate reste candidate après résolution de
ses clés. Une traduction absente laisse la valeur localisée nulle.

Les dictionnaires de familles, groupes, grades et catégories utilisent des IDs
`SMALLINT` bornés à 1–32767. Ils ne remplacent pas les IDs source. Les races,
statistiques et icônes réutilisent les référentiels communs. Les grades
`EItemGrade`, `ESkinGrade` et `ECreatureOptionGrade` gardent leurs propres codes
et libellés explicites, sans correspondance inventée entre ces espaces.

## Familiers

| Tables | Contenu |
| --- | --- |
| `pets` | ID et nom source, textes, famille, item d'acquisition et d'âme, groupe de statistiques, âmes requises, type de cristal, référence de polymorphie, ressource, ordre et booléens source. |
| `pet_families` | Code `ECreatureType`, nom anglais/localisé, titres/descriptions des bonus séparés, devises et types de cristal des coûts. |
| `pet_family_slots` | Positions source, niveaux d'ouverture et groupes d'options. |
| `pet_family_lock_costs` | Positions et coûts de verrouillage en âmes et monnaie. |
| `pet_family_levels` / `pet_grade_weights` | IDs source, famille, niveau, expérience requise/gagnée et tous les champs `Prob_*` bruts, y compris zéro. |
| `pet_option_groups` / `pet_options` | Groupes, IDs d'option, grade, statistique, bornes et poids brut. |
| `pet_stat_groups` / `pet_stat_levels` / `pet_stat_values` | Groupes et IDs de progression, niveaux et statistiques ordonnées. |
| `pet_levels` | Niveau source, coût en âmes et expérience de famille gagnée. |
| `pet_collections` | ID, seuil de collection, récompenses en item/quantité et référence candidate de familier. |

Les noms de famille proviennent des références explicites
`EnumString.ECreatureType → L10NString`, pas d'un mot extrait des descriptions
de bonus. Les libellés anglais du client sont Cogni, Fera, Natura, Varian et
Special ; les libellés français sont Intellia, Bestia, Natura, Varius et
Spéciaux. `BuffTitle` et `BuffDesc` restent des textes distincts.

Les tableaux parallèles d'emplacements et de coûts sont conservés jusqu'à leur
longueur maximale : une entrée manquante d'un côté reste nulle, sans tronquer
l'autre tableau. La position n'est pas transformée en niveau ou nombre de
verrous par une formule supposée. Les poids de grade et d'option ne deviennent
pas des pourcentages tant que leurs domaines et règles de tirage ne sont pas
établis. Les bonus `EquipStatList` sont dans `appearance_stats` avec le rôle
`equipped`. Les références de polymorphie restent des valeurs source, sans
équivalence avec une créature du monde supposée.

## Ailes

`wings` conserve les textes, grade, race, costume, ordre et visibilité, avec
références de groupes d'effets et d'enchantement séparées. Les icônes `IconImg`
sont reliées au cache d'images commun. Les statistiques d'équipement sont
séparées des statistiques de progression.

`wing_effect_groups`, `wing_effect_levels` et `wing_effect_stats` conservent le
groupe, niveau et entrées actives de la map `Stats`. Les entrées supprimées de
la map restent dans la source complète. Un bonus brut de zéro reste zéro.

`wing_enchant_groups`, `wing_enchant_levels` et `wing_enchant_costs` conservent
les groupes, niveaux, `BasicProb`, `AddProb`, `MaxProb`, items, quantités et
quantités d'ajout de probabilité. Ce sont des valeurs brutes. Le client fourni
a des tables de coûts mais **aucune aile avec `EnchantGroup` renseigné** :
elles restent disponibles séparément, sans rattachement par grade, nom ou groupe
d'effets. Cette absence n'est pas une preuve que l'enchantement est impossible
en jeu.

## Apparences et ensembles

`skins` garde identité, catégorie, grade, style, référence d'altération,
date de sortie source, ordre, visibilité et booléens de décoration/possession
multiple. `skin_variants` sépare les noms par race et les icônes/ressources par
race et sexe. Les valeurs explicites sont Light/Dark (races 2/3), Male/Female
(indices locaux de sexe 1/2) ; aucun lien entre ces indices et un enum externe
n'est supposé. Les images sont dédupliquées dans `catalogue_icons`, avec leur
état d'extraction. Les références de fonds, VFX et autres ressources ne sont
pas automatiquement traitées comme des icônes.

`appearance_stats` conserve les valeurs de `Skin.StatList` sous le rôle
`source_stat_list` ; cette liste seule ne démontre pas si son application
requiert possession, équipement ou autre condition. Les statistiques des
collections restent dans `skin_collection_stats`, avec catégorie, seuil de
points, position et valeur brute. Elles ne sont pas ajoutées aux bonus du joueur.

`skin_sets` conserve les noms par race, description, grade/catégorie, dates,
ordre et booléens de visibilité et remplacement d'apparence. `skin_set_members`
garde les membres ordonnés avec clé source et ID résolu nullable. Un membre
manquant n'est pas remplacé par une apparence voisine.

Les dates `ShowDate` restent en ticks dans `catalogue_periods` ; les chaînes
`ReleaseDate` restent telles quelles. Aucune définition n'est retirée parce
que sa date paraît ancienne ou son indicateur de visibilité vaut faux.

## Obtention et personnalisation

Les items d'acquisition et d'âme explicitement référencés par les familiers
restent reliés à `items`. Le moteur examine également les usages
`GetVehicle`, `GetVehicleSoul`, `GetWing`, `GetSkin`, `GetSkinSet` et les
listes `ExtractionSkins` des items. Seul le premier paramètre d'usage est traité
comme cible ; les suivants ne deviennent pas d'autres IDs, notamment le
paramètre supplémentaire des items d'âme. Les cibles numériques sont converties
seulement pour les usages qui référencent explicitement des IDs.

`catalogue_item_links` garde les rôles `granting_item`, `soul_item`,
`extraction_item`, `reward` et `cost`, avec provenance et quantités disponibles.
Plusieurs preuves peuvent relier le même item et la même définition. Les
sources d'obtention de l'item peuvent ensuite être consultées via ses relations
existantes, sans supposer que toute route est actuellement disponible.

`skin_combine_results` conserve les résultats de fusion, grade et groupe de
résultat, poids brut, cible et position. `WeaponSkinIds` garde ses positions
originales ; aucune correspondance de position vers une classe ou arme n'est
inventée. Les IDs zéro n'ajoutent pas de cible ; une cible non nulle absente
reste visible avec ID résolu nul. `skin_combine_costs` garde grade, monnaie,
item et coût, sans confondre grade de coût et grade du résultat.

`skin_gacha_results` et `skin_gacha_groups` gardent résultats, groupes et
poids de chaque étage séparément. Les lignes dont le nom contient `TEST` restent
des définitions du client ; elles ne constituent pas une preuve de boutique
active ou de moyen d'obtention accessible.

Les tables `SkinMaterial`, `SkinPattern` et `SkinMotion` sont conservées comme
sources complètes, signalées non interprétées pour leur application métier :
ressources, calques, gestes, coûts et règles de personnalisation restent à
traiter lorsqu'un usage le nécessitera.

## Versions et validation

Le format SQLite passe à **12**, sans migration des formats de développement.
Les neuf familles avec IDs source participent au suivi du dernier changement :
familiers, niveaux de famille, options, niveaux de statistiques, collections de
familiers, ailes, apparences, ensembles et résultats gacha. Les positions de
règles sans ID ne deviennent pas des identifiants stables du jeu.

Les empreintes des familiers incluent les règles de famille, options et niveaux
associés. Les ailes incluent leur progression explicitement liée. Les
apparences incluent leurs résultats de fusion/gacha et collections de même
catégorie. Les liens d'items utilisent leurs références naturelles et rôles,
sans propager une modification de prix de l'item donneur. Les noms de
statistiques partagés sont suivis dans leur langue. Les changements de langue
ou d'ordre des lignes ne deviennent pas des changements du client.

Les règles d'enchantement sans aile associée et de coût de fusion restent
consultables et conservées dans leurs sources complètes ; elles ne sont pas
attribuées artificiellement à une définition pour lui donner un changement.
Une comparaison entre deux builds réels attend toujours une seconde copie
fiable du client.

Les tests couvrent notamment les identités distinctes malgré des IDs numériques
égaux, les variantes et traductions manquantes, tableaux parallèles de tailles
différentes, zéros explicites, groupes d'enchantement absents, cibles manquantes,
paramètres d'items d'âme, dépendances de progression et changements de langue.

### Vérification sur le client fourni

Export complet effectué sur la copie rattachée au build Steam **26581260**, en
anglais et français (`fr-FR`). Les 20 sources recherchées sont décodées et les
références examinées pour ces familles, usages d'items et extractions compris,
ne présentent aucune cible manquante ou ambiguë sur cette copie. La région
reste inconnue.

| Projection | Lignes |
| --- | ---: |
| Familiers / familles | 213 / 5 |
| Emplacements / niveaux de famille | 45 / 50 |
| Options de familier | 2 062 |
| Niveaux de statistiques / valeurs | 15 / 24 |
| Niveaux de familier / collections | 3 / 51 |
| Ailes | 86 |
| Niveaux d'effets d'aile / valeurs | 309 / 1 133 |
| Niveaux de coûts d'enchantement / items de coût | 30 / 90 |
| Apparences / variantes race-sexe | 1 579 / 6 316 |
| Ensembles / membres | 128 / 848 |
| Statistiques de collections d'apparences | 10 000 issues de 9 000 paliers source |
| Résultats de fusion, variantes d'arme comprises | 2 396 issus de 1 588 lignes source |
| Coûts de fusion | 5 |
| Résultats / groupes gacha | 2 / 2 |

Exemples et limites contrôlés :

- Le familier 1001 (`BlackDragon_01`) conserve son identité, sa famille
  `ECreatureType::Special`, son item associé et ses cinq âmes requises. Aucun
  groupe de statistiques n'est fabriqué lorsque `StatGroupName` vaut `None`.
- L'aile 30100300 (`Wing_L_WingA_003`) retrouve son groupe d'effets et son icône.
  Les 86 ailes ont un `EnchantGroup` vide : les 30 niveaux de coûts restent
  distincts et aucune association par rareté n'est créée.
- L'apparence 10001101 (`WP_0001_T03_GS`) garde ses variantes Light/Dark et
  Male/Female. Les 848 membres d'ensembles retrouvent tous leur apparence.
- Les 213 items donnant un familier, 200 items d'âme, 102 items donnant une
  aile, 1 623 donnant une apparence et 127 donnant un ensemble retrouvent tous
  leur cible. Ces nombres comptent les définitions d'items, pas des joueurs ni
  des moyens d'obtention actuellement accessibles.
- Les résultats `TEST_Gacha_11` et `TEST_Gacha_1` retrouvent leurs apparences.
  Ils restent des définitions de test potentielles, sans déclaration de
  disponibilité en jeu ni transformation des poids en taux de drop.
- Les 86 icônes d'ailes sont disponibles dans le cache fourni. Parmi les
  6 316 variantes d'apparence, 6 110 ont une image disponible et 206 ont une
  référence sans image retrouvée. Les cinq familles ont aussi une référence
  d'icône de bonus sans image retrouvée. Il s'agit d'un état du cache d'images,
  pas d'une preuve que ces ressources sont absentes des archives du jeu.

La base complète conserve les familles précédentes et contient 193 sources,
282 001 entrées suivies dans 28 familles. Cette reconstruction initialise le
suivi ; elle ne prétend pas observer un changement de patch. Les contrôles
d'intégrité et de clés étrangères de l'export passent.

Base de validation : `target/game-data-validation/data.sqlite`, format 12,
720 396 288 octets ; précédente base sauvegardée sous
`target/game-data-validation/data-before-82.sqlite`. Résumé de l'export :
`target/game-data-validation/catalogue-82-summary.json`.
Empreinte du lecteur :
`7f0e1e7b499469bcf4e551d0c32ab5ce4a5ca0292f261719b5f25124a7f5b94e`.

Validation du code : 69 tests unitaires `game_data`, 35 tests d'intégration
(catalogue, items, compétences et USMAP), Clippy sans avertissement et
compilation Windows réussis. Le test enfant de processus du catalogue est
ignoré dans l'exécution globale par conception. Exécutable :
`target/windows-scroll-preview/debug/aetheris.exe`.
