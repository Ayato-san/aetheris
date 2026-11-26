# Plateaux et cases Daevanion — #83

La projection Rust `daevanion` ajoute les définitions au [catalogue commun](game-data-export.md).
Les connexions orthogonales, les règles de voisinage et de remboursement et
le bonus de niveau des cases `SkillLevel` suivent les **précisions données par
l’utilisateur le 26 novembre 2026**. Ces règles sont distinctes des champs lus
dans les fichiers : aucune analyse de l’exécutable ne les a établies.
La possession des cases reste une donnée du personnage, fournie par l’appelant.

## Sources, provenance et langues

Les sources recherchées explicitement sont `DaevanionBoard`, `DaevanionNode`,
`DaevanionMaxCapacity` et `Exp`, décodées avec leurs structures USMAP
`<nom>TableItem`. Les usages Daevanion de `Item` sont également examinés.
Les sources complètes et typées restent dans `catalogue_sources` avec build,
mapping, structure, empreintes et état de lecture. Elles conservent les champs
non projetés et les lignes `None`, sans les présenter comme des cases utilisables.
Les relations et textes utilisent `catalogue_entity_sources`,
`catalogue_references`, `catalogue_texts`, `catalogue_text_values` et
`catalogue_conditions`.

Les textes anglais et localisés restent distincts, avec leurs clés et états de
repli. Une traduction absente ne devient pas une traduction inventée. Les classes
partagent `skill_classes` ; types, grades, conditions et monnaies utilisent
`catalogue_codes`. Les dictionnaires ont des IDs `SMALLINT` bornés à 1–32767 ;
les plateaux et cases conservent leurs IDs source.

## Projection SQLite

| Table | Contenu |
| --- | --- |
| `daevanion_boards` | ID, dernière version de modification, nom, titre et description du bonus, classe, niveau requis, type de points, monnaie de réinitialisation et ordre. |
| `daevanion_nodes` | Cases autres que `None` : ID, plateau nullable et référence source, ligne/colonne, type, grade, niveaux, coûts, textes, condition et `auto_learn`. |
| `daevanion_node_connections` | Connexions non orientées, chaque paire stockée une fois (`node_id < neighbour_id`), avec deux clés étrangères vers les cases. |
| `daevanion_node_parameters` | Chaînes source de `Value01..10` présentes, y compris zéros, chaînes vides et valeurs nulles. |
| `daevanion_stat_parameters` | Paires candidates de référence de statistique/valeur, position originale, statistique nullable, chaîne et entier brut si analysable. Unités non converties. |
| `daevanion_skill_parameters` | Référence de compétence, compétence nullable, `value_source` et `level_bonus` entier : `Value02 = "1"` signifie **+1 niveau**. |
| `daevanion_point_types` | Codes, noms techniques, plafonds bruts et catégories de monnaie. |
| `daevanion_level_points` | Niveau du personnage, position de l’entrée de map, type de points et valeur brute de `Exp.DaevanionPointMap`. |
| `daevanion_point_items` | Usage source, référence de type de points et second paramètre brut ; association candidate. |

Les lignes `None` ne créent ni case métier, ni paramètres projetés, ni icône ou
entrée suivie de case. Elles ne transmettent aucune connexion. Leur changement
de métadonnées n’actualise pas les cases ou le plateau. Si une ligne devient
une vraie case, elle rejoint la projection et le suivi.
Les types futurs conservent leurs paramètres ; ils ne reçoivent pas un effet
inventé. Les positions ne sont ni compactées ni renumérotées.

Les icônes de plateaux et de leurs bonus restent séparées dans
`catalogue_entity_icons`. Les images des cases utilisent d’abord l’alias exact
de `UI_ImageTable_Composite`, puis la recherche habituelle de textures.
Une image manquante n’est pas remplacée par une référence ressemblante.

## Connexions, activation et remboursement

La règle confirmée est la suivante : deux cases du **même plateau** sont voisines
si leurs positions diffèrent d’une case dans une seule direction : haut, droite,
bas ou gauche. Les diagonales et les emplacements `None` ne comptent pas. Aucun
saut au-dessus d’un emplacement vide n’est créé. Un plateau ou une position
manquante ne produit pas de connexion supposée.

`game_data::daevanion_graph::Board` charge ce graphe et sa case `Start`.
Le départ sert d’ancrage pour les contrôles de connexion ; il peut être omis de
l’ensemble de cases acquises fourni par l’appelant. Il n’est pas remboursable
par cette API. Une case peut être activée si elle touche le départ ou une case
acquise, et si l’ensemble acquis est déjà relié au départ.

Un remboursement est autorisé uniquement si **toutes les cases acquises
restantes restent reliées au départ**. Le contrôle parcourt le graphe restant :
une case qui coupe une branche ne peut pas être retirée, tandis qu’un chemin
alternatif autour d’un cycle permet son remboursement. Les cases inconnues,
les ensembles acquis déconnectés et les plateaux sans départ unique sont refusés.

Ces méthodes contrôlent uniquement la connectivité. Les niveaux requis, points,
monnaies et conditions explicites restent des contrôles séparés. Aucun état de
possession ou bonus de personnage n’est calculé depuis `auto_learn` ou depuis
les seules définitions du client.

## Paramètres et limites restantes

Pour `SkillLevel`, `Value01` référence `Skill.ID` et `Value02` donne le bonus de
niveau additif confirmé par l’utilisateur. `level_bonus` vaut donc `1` pour
`"1"`. Une valeur non entière ou absente reste nulle, avec sa source conservée ;
elle ne devient pas implicitement +1. La projection ne modifie ni les niveaux
source de `skills`, ni leurs descriptions.

Pour `Stat`, les paires `Value01/02`, `Value03/04`, jusqu’à `Value09/10` restent
candidates. Les libellés viennent de `String_StatName_{référence}_body` et les
statistiques connues partagent `stats`. Une référence inconnue garde un ID nul.
La vérification de `StatCorrectionNumber.IndicationStatWindow` distingue désormais
les 36 statistiques utilisées : 16 en pourcentage et 20 en entier. Le format
SQLite 16 importe cette indication dans `stats.expression_format`, avec `NULL`
pour les indications inconnues. Les paramètres `value_source` et
`value_integer_raw` restent les valeurs originales ; la vue d’inspection brute
ne les remplace pas par une valeur affichée. Voir le
[format d’expression](game-data-export.md#format-dexpression-des-statistiques).

`Start.Value01` conserve sa clé et ses textes. `NeedConditionType` et
`NeedConditionValue` restent ensemble et typés, sans convertir une valeur
numérique en ID de case. Les valeurs de `Exp.DaevanionPointMap` restent brutes,
sans les appeler gains par niveau ou dériver une formule. Les usages d’items
`GetDaevanionPoint` et `Fix_DaevanionPoint` restent distincts et non appliqués.

## Couverture et suivi des changements

- `daevanion.connections.rules` rapporte la règle confirmée par l’utilisateur
  et les éventuels plateaux sans départ unique, positions absentes ou cases
  inaccessibles. Un graphe incomplet reste signalé `partial`.
- `daevanion.connections.tables` reste `not_found` quand aucun champ d’arête
  explicite n’est trouvé ; des champs futurs candidats restent `uninterpreted`.
- `daevanion.connections.client_logic` reste `not_researched` : l’algorithme
  n’a pas été extrait de l’exécutable.
- `daevanion.skill_level_bonus` distingue le rôle confirmé des valeurs absentes
  ou non entières qui ne peuvent pas être projetées en bonus entier.
- `daevanion.parameter_roles` et `daevanion.level_points` gardent les limites
  d’interprétation des autres paramètres et valeurs de progression.

Le format SQLite passe à **14**, sans migration de l’ancien format de développement.
Une reconstruction complète est nécessaire. Une case modifiée actualise son
suivi et celui du plateau. La liste triée des voisins participe aussi à
l’empreinte d’une case : déplacer ou retirer un voisin actualise les deux
extrémités concernées. Les valeurs précédentes ne sont pas conservées comme
historique. La réorganisation des sources et les changements de langue restent
traités comme dans les autres projections.

## Validation

Sur le build Steam **26581260**, les 12 150 lignes source comprennent 5 712
emplacements `None`, conservés uniquement comme source complète. La projection
contient **6 438 cases** : 54 `Start`, 5 592 `Stat` et 792 `SkillLevel`, sur
54 plateaux. La règle orthogonale produit **7 624 connexions non orientées** ;
toutes les cases sont accessibles depuis le départ de leur plateau.

Les 6 024 paires de statistiques retrouvent le référentiel et leurs libellés
anglais/français. Leurs valeurs restent brutes ; le format 16 ajoute leur format
d’expression dans le référentiel `stats`. Les 792 références de
compétences retrouvent `Skill.ID` et leur bonus est +1. Les cinq plafonds,
89 entrées explicites de points par niveau et dix items d’usage restent conservés.
Les 6 438 cases avec image utilisent les sprites UI extraits. Les icônes des
54 plateaux et de leurs bonus n’ont toujours pas d’image retrouvée dans le cache.

Exemples : la case 110033 garde sa position (3, 3), ses quatre points de coût,
son coût de réinitialisation de 500 et la paire `CombatSpeed` / `150` brute.
La case 150003 conserve ses deux paires `PvEAddDamage` / `3` et
`PvEDamageDefense` / `30`. La case 110037 référence la compétence 11720000
avec un bonus explicite de **+1 niveau**.

Les tests couvrent les branches, cycles, feuilles remboursables, ponts qui
créeraient des orphelins, diagonales, cases vides, frontières de plateaux,
départs absents, références manquantes, paramètres non entiers, zéros,
conditions opaques et empreintes des voisins. La possession reste fournie par
l’appelant, sans écriture d’état de personnage dans le catalogue.

Base de validation régénérée : `target/game-data-validation/data.sqlite`,
format 14, **730 718 208 octets**, 197 sources et 288 493 entrées suivies.
L’ancien format 13 est conservé dans
`target/game-data-validation/data-before-83-rules.sqlite`. Cette reconstruction
établit une nouvelle référence initiale ; elle ne représente pas un changement
observé du jeu. Résumé : `target/game-data-validation/catalogue-83-rules-summary.json`.
Empreinte du lecteur :
`09fcea44e3849b088691216278d8fa01d451868c3593bffcaca02beea545b31f`.

Validation : 83 tests unitaires `game_data`, 35 tests d’intégration catalogue,
items, compétences et USMAP (un test enfant ignoré par conception), Clippy
sans avertissement et compilation Windows. Les contrôles d’intégrité et de
clés étrangères passent sur le fichier généré. Le build séparé
`target/windows-daevanion-preview/debug/aetheris.exe` permet de consulter le
nouveau catalogue sans remplacer l’exécutable actuellement utilisé.
