# Recettes et quêtes — #79

La projection utilise le même export Rust `export-catalogue` que les objets et
compétences. Les sources candidates et leurs vérifications initiales sont dans
[l'inventaire #75](game-data-sources.md). Python et .NET ne sont pas nécessaires
pour construire cette projection sur le PC du joueur.

## Sources et provenance

Les tables explicitement recherchées sont `CraftRecipe`, `CraftRecipeRate`,
`CraftMastery`, `CraftRecipeTabUI`, `CraftComponentReplacement`,
`ComponentCombine`, `ConversionRecipe`, `ConversionComponentReplacement`,
`ConversionComponentEquipment`, `ConversionTabUI`, `Quest`, `QuestPart`,
`QuestStep`, `QuestString`, `QuestReward`, `QuestClassReward`, `QuestType`,
`DistrictQuestGroup` et `DutyQuestGroup`.

Chaque table conserve sa définition complète compressée dans `catalogue_sources`,
avec empreintes, structure, build et état de lecture. Les tables déjà incluses
par l'extraction des routes d'obtention ne sont pas relues une seconde fois.
Les champs supplémentaires ne sont pas jetés lorsqu'ils n'ont pas encore de
projection métier. `CraftMastery` et les groupes de districts/vocations restent
des sources complètes à interpréter pour leurs usages ultérieurs.

`catalogue_entity_sources` rattache une recette ou une quête à ses définitions
et dépendances effectivement utilisées. `catalogue_references` décrit chaque
liaison projetée : origine, chemin, table/champ cible, référence, résultat
(`resolved`, `missing`, `ambiguous`, `not_researched`) et caractère vérifié ou
candidat. Une cible manquante conserve sa référence et une FK nulle. Aucun ID
voisin n'est substitué. Les IDs numériques et noms textuels restent distincts ;
les rares conversions numériques sont explicites dans le lecteur.

Les énumérations utilisent `catalogue_codes`, avec identifiants `SMALLINT`
bornés à 1–32767 et libellés EN/localisés issus de `EnumString`. Les races
réutilisent `races`. Les IDs source ne sont pas limités à deux octets. Les
booléens manquants restent nuls ; un booléen absent n'est pas forcé à faux.

## Recettes

| Table | Contenu |
| --- | --- |
| `recipes` | Identité `(source_name, id)`, clé source, noms du produit, classification, race, maîtrise requise, visibilité, fabrication de guilde, combo et groupe de taux. |
| `recipe_results` | Produit principal, résultat critique et résultat d'échec : référence/item, quantité, enchantement et probabilité brute lorsqu'ils sont présents. |
| `recipe_components` | Composants ordonnés : référence/item, quantité, enchantement et références de remplacement/combinaison. |
| `recipe_component_options` | Membres des listes de remplacement, d'équipement et de combinaison ; leurs quantités et enchantements propres. |
| `recipe_costs` | Frais normaux et distants, montant brut et devise explicitement référencée. |
| `recipe_rates` | Groupe, ordre, écart de maîtrise `Gap` et valeur `Rate` originale. |

Les IDs de fabrication et conversion sont dans des espaces distincts, même si
la copie actuelle n'a pas de collision. Les combos restent des liens vers une
autre recette de la même source. Une modification de la définition du combo est
portée par cette recette ; elle n'est pas propagée à tous ses parents.

`ComponentList.ReplacementItem` référence `CraftComponentReplacement.Name` ou
`ConversionComponentReplacement.Name` selon la source, **pas `Item.Name`**.
`CombineName` référence `ComponentCombine`, et `MultiSelectComponent` référence
`ConversionComponentEquipment`. Les quantités des alternatives ne sont ni
remplacées par la quantité du composant principal, ni multipliées selon une
règle supposée. Le référentiel de relations d'utilisation des items utilise
désormais ces listes plutôt qu'une recherche directe du nom de groupe dans les
items. Les ingrédients ne deviennent pas des sources d'obtention.

Les conditions d'apprentissage/enregistrement, qualifications de guilde, limites
de compte/personnage/serveur, dates en ticks et compteurs restent accessibles
dans `catalogue_conditions`, avec leur chemin et encodage original. Les autres
champs sont conservés dans la source complète. Les quantités et coûts nuls sont
conservés lorsqu'ils sont explicitement présents.

Exemple client : `CraftRecipe` **111013001**, produit
`equip_wp_Sword_L_u_craftdraconic_001a_s`, maîtrise 55, combo brut **2500**.
Les valeurs de `CraftRecipeRate.Rate`, par exemple **74011698**, restent des
entiers bruts. Ni ces valeurs ni `SuccessProp` / `CriticalSuccessProp` ne sont
présentés comme des pourcentages sans échelle validée.

La devise de `CraftingFee` vient de `CraftingFeeType`. `RemoteCraftingFee` n'a
pas de champ devise distinct vérifié : la projection garde ce montant avec
devise inconnue. Aucune station ni position de fabrication n'est déduite d'une
catégorie ou d'un niveau de maîtrise. La maîtrise requise n'est pas le niveau
requis du personnage.

Les noms et icônes de recettes proviennent du produit résolu dans le catalogue
items. Il s'agit d'une présentation dérivée, pas d'un titre propre extrait de
la recette. `ProductString`, s'il existe, est également conservé comme texte
source, sans remplacer automatiquement le nom du produit.

## Quêtes

| Table | Contenu |
| --- | --- |
| `quests` | ID, clé, titre EN/localisé, résumé EN/localisé, type/grade/race, niveaux requis et recommandé, répétition, partie, quête suivante et permission d'abandon. |
| `quest_parts` | Identité, ordre et titre localisé des parties. |
| `quest_texts` | Tous les textes référencés de `QuestString`, avec leur chemin et clé, notamment descriptions d'étapes et objectifs. |
| `quest_steps` | ID, quête et ordre, condition d'échec, ordre/étape cible d'échec, objectifs unifiés et score brut. |
| `quest_objectives` | Objectifs ordonnés, type, valeurs source, ID de carte, indicateurs facultatif/masqué/marqueur. |
| `quest_reward_groups` | Groupe source, association candidate à une quête, classe éventuelle, expérience et référence de récompense aléatoire. |
| `quest_rewards` | Items, quantités et enchantements, séparés selon `ItemRewards`, `ItemNormalRewards` et `ItemSelectRewards`. |

`Quest.QuestText → QuestString.Name` est un lien de nom, pas un ID numérique.
Le titre est `Title`, et le résumé affichable dans `quests.description_*` est
`ShortSummary`. Les autres rôles restent séparés dans `quest_texts` ; aucune
fusion de descriptions ni substitution de marqueurs n'a lieu à l'extraction.
`catalogue_texts` indique les traductions disponibles/manquantes et le repli
anglais. Les colonnes localisées originales restent nulles si le texte manque.

Exemple : **1100010** / `HQ1100010`, partie `hero_poeta_00`, texte
`STR_HQ1100010`, suite `HQ1101010`. Les prérequis `ClearQuest` /
`IncompleteQuest` gardent leurs références de noms ; `ClearQuestStep` conserve sa
valeur originale tout en contrôlant explicitement sa conversion en ID d'étape.
Les types d'acquisition/complétion, valeurs associées, conditions de sous-zone,
maîtrise ou objets d'environnement restent typés dans `catalogue_conditions`.
Leurs liens supplémentaires sont marqués candidats lorsque leur rôle métier
demande encore une vérification en jeu.

Les branches d'échec recherchent `FailGoToStep` comme **ordre dans la même quête**,
pas comme ID global. Une cible absente ou multiple reste non résolue avec
diagnostic. `GoalList.Value` reste une liste JSON ordonnée et typée ; aucune
position n'est arbitrairement interprétée comme nombre d'ennemis, ID d'item ou
condition universelle. Les autres paramètres de chaque objectif, dont les
références de déplacements/téléportation, restent dans les conditions sources.

`QuestReward.Group` et `QuestClassReward.Group` correspondent aux ID numériques
de quêtes sur la copie étudiée. Cette association reste **candidate** et ne
déclare pas l'éligibilité du joueur. Les récompenses par classe sont des
variantes distinctes : elles ne sont pas additionnées comme si un personnage
pouvait toutes les recevoir. `RandomReward` renvoie à **`RewardPackage.Name`**
sur cette copie, pas à `RewardItem` ni directement à `RewardPackageItem`.
La définition de loot garde son propre suivi de changement et ses poids ; aucun
pourcentage de quête n'est fabriqué à partir de cette seule référence.

## Builds, publication et limites

Les trois familles `craft_recipes`, `conversion_recipes` et `quests` utilisent le
[suivi du dernier changement](game-data-changes.md). Les empreintes incluent
les définitions et dépendances utilisées : listes de substitution, taux,
onglets, textes, parties, types de quêtes, étapes et récompenses. L'ordre des
lignes d'un fichier ne définit pas l'identité d'une entrée. Un changement de
langue ne devient pas une modification du jeu. Aucun historique des anciennes
valeurs n'est créé.

Les sources illisibles peuvent conserver leur dernier contenu fiable avec le
build et diagnostic d'origine. Le contrôle de cohérence existant refuse une
publication mêlant des sources conservées à d'autres définitions modifiées
dont les relations ne seraient plus garanties. Les comparaisons impossibles et
intervalles incomplets restent signalés. Une erreur d'intégrité empêche le
remplacement de la base active.

Le format SQLite passe à **10**. Les anciens formats de développement doivent
être régénérés ; aucun code de migration n'est ajouté. Le lecteur de la page de
debug accepte le nouveau format, sans ajouter de page de recettes/quêtes dans
cette issue. Les ressources d'images existantes restent réutilisées.

Limites connues : coordonnées/stations non établies, échelles de taux non
validées, sémantique de certains paramètres et conditions encore inconnue,
cartes 21 manquantes dans `Map`. Ces limites restent des données partielles,
pas des valeurs par défaut ni une compatibilité générale version/région.

## Vérification sur la copie fournie

Build **26581260**, langues `en-US` et `fr-FR`, mapping et rattachement décrits
dans #75. La base régénérée `target/game-data-validation/data.sqlite` est au
format 10 et contient **170 sources**, 9 452 items, 16 283 compétences et
6 708 altérations, avec les ajouts suivants :

| Projection | Nombre |
| --- | ---: |
| Recettes de fabrication / conversion | 1 120 / 296 |
| Composants / options de composants | 2 708 / 2 214 |
| Résultats de recettes / valeurs de taux | 1 416 / 66 284 |
| Quêtes / étapes / objectifs | 1 429 / 2 553 / 2 992 |
| Groupes de récompenses de quête, dont variantes par classe | 1 433, dont 112 variantes |
| Items de récompense de quête | 2 295 |
| Textes de quêtes référencés, tous présents en EN/FR | 12 671 |

Les 1 416 noms de recettes et 1 429 titres de quêtes existent en EN/FR.
Deux quêtes n'ont pas de `ShortSummary` référencé : les descriptions
correspondantes restent nulles. Exemple de produit de la recette 111013001 :
**True Dragon Lord Longsword / Épée du Roi dragon de vérité**. La quête
1100010 est **Poeta / Poéta**, avec la suite 1101010.

Les 24 346 références projetées comprennent **136 références non résolues**,
toutes sur la carte 21 : 52 cartes de complétion et 84 cartes d'objectifs. Les
41 branches d'échec non nulles retrouvent leur étape cible. Les 340 références
de récompense aléatoire retrouvent `RewardPackage.Name` ; leur association
reste candidate quant à l'application des règles en jeu. Les 1 321 groupes
standards et 112 variantes par classe retrouvent une quête candidate.

La première extraction établit **267 547 entrées initiales**, sans ajout,
modification ni suppression déclaré. Les sept familles disposent de leur
référence initiale. La base fait **661 237 760 octets** ; les 2 447 sprites UI
restent présents. Le contrôle d'intégrité et des clés étrangères a réussi dans
l'export. L'empreinte du lecteur a été comparée aux fichiers courants :
`0652f5a3f84d5a2930420170f44b407a19340e466bf0c9aee899903f125d8ae4`.
La base précédente est sauvegardée dans
`target/game-data-validation/data-before-79.sqlite`.

Validation automatisée : **57 tests unitaires ciblés et 35 tests d'intégration**,
Clippy sans avertissement. Les cas ajoutés couvrent collisions d'IDs entre
sources, substitutions et quantités, taux bruts, noms ambigus, références
manquantes, branches locales, récompenses par classe, marqueurs et repli de
traduction, changements des taux/étapes, suppression et réapparition de recette.
Les comparaisons multi-build restent testées sur des fixtures ; une seconde
copie réelle fiable est toujours nécessaire pour une comparaison terrain.
