# Plateaux et cases Daevanion — #83

La projection Rust `daevanion` ajoute les définitions tabulaires à
[l'export commun](game-data-export.md). **Le graphe de connexions reste à
établir** : la projection ne fabrique aucune arête depuis la grille. Elle
n'infère ni possession, ni activation, ni bonus appliqué au personnage.

## Sources, provenance et langues

Les sources recherchées explicitement sont `DaevanionBoard`, `DaevanionNode`,
`DaevanionMaxCapacity` et `Exp`. Les structures candidates sont leurs
`<nom>TableItem`, contrôlées par le lecteur USMAP strict. Les usages Daevanion
de la source `Item`, déjà conservée, sont également examinés.

Les sources complètes et typées restent dans `catalogue_sources` avec build,
mapping, structure, empreintes et état de lecture. Elles incluent les champs
non projetés, zéros, valeurs vides, paramètres et entrées supprimées des maps.
Les preuves utilisent les tables communes de #79/#80 :
`catalogue_entity_sources`, `catalogue_references`, `catalogue_texts`,
`catalogue_text_values` et `catalogue_conditions`.

Les textes anglais et localisés, clés et états de repli restent distincts.
Une traduction absente laisse l'original localisé nul. Les classes partagent
`skill_classes`, en reprenant littéralement le code `ECharacterClass` ; aucun
ordre de classe n'est déduit des IDs. Grades, types de case, conditions et
monnaies utilisent les codes source du référentiel commun et leurs libellés
explicites lorsqu'ils existent. Les IDs de dictionnaires sont des `SMALLINT`
bornés à 1–32767 ; les IDs des plateaux et cases restent les IDs source.

## Projection SQLite

| Table | Contenu |
| --- | --- |
| `daevanion_boards` | ID, dernière version de modification, nom, titre et description du bonus, classe, niveau requis, type de points, monnaie de réinitialisation et ordre. |
| `daevanion_nodes` | ID, référence originale et ID nullable du plateau, ligne/colonne, type, grade, niveau requis, coût en points et de réinitialisation, textes, condition et booléen `bAutoLearn`. |
| `daevanion_node_parameters` | Positions 1–10 et chaînes originales de `Value01..10`, y compris valeurs vides ou nulles présentes. Un champ absent ne crée pas de ligne. |
| `daevanion_stat_parameters` | Paires candidates de référence de statistique/valeur, position originale, statistique partagée nullable, chaîne et entier brut si analysable. |
| `daevanion_skill_parameters` | Référence originale de compétence, ID numérique si analysable, compétence nullable, `Value02` source et entier brut si analysable ; association candidate. |
| `daevanion_point_types` | Code source, nom technique, plafond brut et catégorie de monnaie. Les types sans plateau associé restent présents. |
| `daevanion_level_points` | Niveau du personnage, position de l'entrée de map, type de points et valeur brute de `Exp.DaevanionPointMap`. |
| `daevanion_point_items` | Item, usage source, nom de type de points, cible nullable et second paramètre brut ; association candidate. |

Les icônes principales et de bonus des plateaux restent séparées dans
`catalogue_entity_icons`, familles `daevanion_boards` et
`daevanion_board_buffs`. Les images des cases utilisent `daevanion_nodes`.
Elles partagent le cache commun et son état de disponibilité ; une référence
sans image extraite n'est pas remplacée par une image voisine.
Pour `DaevanionNode.Image`, la liaison cherche d'abord la clé exacte de
`UI_ImageTable_Composite` dans `ui_sprite_aliases` et réutilise l'image du
sprite. L'absence d'alias conserve le chemin habituel de recherche d'image,
sans sélectionner une variante dont le nom est seulement similaire.

Les positions et IDs ne sont ni reconstruits ni renumérotés selon une grille
supposée. Les cases `None`, `Start`, `Stat` et `SkillLevel` restent toutes
présentes, avec leurs types. Une case vide n'est pas présentée comme un bonus
utilisable ; `bAutoLearn` ne signifie pas qu'un personnage possède la case.

Les coûts en points des cases se rattachent au type de points du plateau ; les
coûts de réinitialisation à sa monnaie explicite. Ces domaines restent
distincts. Aucun coût n'est calculé depuis le grade et aucun zéro n'est
remplacé par une valeur par défaut.

## Paramètres, effets et prérequis

Pour les cases `Stat`, les paires `Value01/02`, `Value03/04`, jusqu'à
`Value09/10` sont conservées comme **candidates**. Les libellés sont recherchés
par la clé exacte `String_StatName_{référence}_body` ; les références canoniques
partagent `stats` sans remapper une statistique vers un autre code. Une
référence inconnue reste présente avec ID nul. Les valeurs gardent leur chaîne
source même lorsqu'un entier est analysable ; aucune conversion d'unité ou de
pourcentage n'est appliquée. La présence d'une deuxième paire n'est pas ignorée.

Pour `SkillLevel`, seule la conversion numérique explicitement déclarée de
`Value01 → Skill.ID` est effectuée. Une clé résolue ne démontre pas que
`Value02 = "1"` signifie « ajouter un niveau ». La projection ne modifie pas
`skills`, `skill_levels` ou les descriptions résolues. Les autres paramètres
restent disponibles sans rôle supposé.

Pour `Start`, la clé de texte de `Value01` est conservée avec son état et ses
textes disponibles. Elle ne devient pas une référence de statistique ou de
compétence. Les types futurs gardent leurs paramètres et leur source complète.

`NeedConditionType` et `NeedConditionValue` restent ensemble et typés dans
`catalogue_conditions`, avec la chaîne d'origine dans `daevanion_nodes`.
Une valeur numérique n'est pas automatiquement un ID de case ni un prérequis
de connexion. Sur la copie examinée, toutes ces conditions sont `None` et
toutes les valeurs associées sont vides : ce constat ne prouve pas l'absence
de règles d'activation dans le client.

## Points et progression

Les cinq définitions de `DaevanionMaxCapacity` sont conservées, y compris les
types saisonniers sans plateau associé. Les codes et noms techniques restent
distincts des libellés localisés ; aucun nom français n'est inventé depuis
`DaevanionCrystal` ou `ConquestCrystal`. Une capacité dupliquée pour un même
code est refusée, au lieu de choisir arbitrairement la dernière ligne.

`Exp.DaevanionPointMap` est la source des valeurs par niveau. Elles ne sont
ni appelées des gains par niveau, ni additionnées, ni calculées avec une
formule de progression. Les entrées supprimées restent dans la source
complète, séparées des entrées actives projetées.

Les usages `GetDaevanionPoint` et `Fix_DaevanionPoint` des items gardent leur
type d'usage, premier paramètre comme nom candidat de type de points, et
second paramètre source. Ils ne sont pas confondus et aucune opération
d'ajout, de remplacement ou de plafond n'est exécutée à leur lecture.

## Connexions et couverture manquante

La recherche sur la copie déchiffrée et l'inventaire d'extraction retrouve les
trois tables Daevanion nommées ci-dessus. Leurs layouts et lignes ne portent
aucun champ de connexion explicite identifié. Les positions de cases,
conditions, IDs voisins ou cases de départ ne suffisent pas à certifier un
graphe ou une règle d'accessibilité.

Deux messages des dictionnaires anglais/français confirment une notion de
case adjacente : `Message_MSG_GETDAEVANIONNODE_CANNOT_LEARN_PRENODE_body`
refuse l'acquisition sans case adjacente acquise, et
`Message_MSG_DAEVANIONNODE_CANNOT_RESET_PRENODE_body` demande de rembourser
d'abord une case adjacente acquise. Ces textes ne définissent ni les directions
autorisées, ni le rôle des diagonales ou des cases vides, ni un graphe orienté.
Ils ne sont donc pas utilisés pour créer des arêtes ou simuler les activations.

- `daevanion.connections.tables` signale `not_found` pour les layouts
  examinés. Si de futurs champs comportent des noms candidats de connexion,
  ils sont signalés `uninterpreted`, jamais convertis automatiquement en arêtes.
- `daevanion.connections.client_logic` reste `not_researched` : aucune règle
  d'activation ou d'adjacence vérifiée n'a été extraite de l'exécutable ou de
  la logique Unreal. L'inspection des seules tables ne couvre pas ces ressources.
- `daevanion.parameter_roles` garde les rôles et unités non établis.
- `daevanion.level_points` garde la sémantique des valeurs de progression non
  interprétée. Une correspondance de clés ne certifie pas une formule.

La projection tabulaire est disponible ; **la fermeture complète de #83
nécessite encore de vérifier les connexions et les règles d'activation**, ou
de recadrer explicitement cette exigence. Aucune absence générale de graphe
dans le jeu n'est affirmée.

## Versions et validation

Le format SQLite passe à **13**, sans migration des formats de développement.
Les deux familles avec IDs source (`daevanion_boards`, `daevanion_nodes`)
participent au suivi de la dernière modification. Les types de points sont
des références partagées et non des identifiants de jeu inventés.

Une modification de case est suivie sur la case et son plateau. Les coûts,
conditions et contextes explicites de plateau et plafond sont suivis. Les
statistiques et textes indirects des paramètres participent dans leur langue.
Seule la map Daevanion de `Exp` est utilisée : modifier `NeedExp` ne change
pas artificiellement les plateaux. Les entrées de points supprimées restent
prises en compte dans le suivi. Une réorganisation des lignes ou un changement
de langue ne devient pas une modification du jeu. Les valeurs précédentes
ne sont pas conservées comme historique.

Les tests couvrent les IDs non déduits des positions, cases vides/départ,
paires multiples, zéros, valeurs non entières, références manquantes,
conditions opaques, paramètres de compétences non appliqués, points et
monnaies, nouvelles colonnes de connexion non interprétées, réorganisation,
dépendances des empreintes et changement de langue.

### Vérification sur le client fourni

Export complet effectué sur la copie rattachée au build Steam **26581260**, en
anglais et français (`fr-FR`). Les quatre sources recherchées sont décodées ;
la région reste inconnue et le mapping n'est pas certifié pour d'autres copies.

| Projection | Lignes |
| --- | ---: |
| Plateaux | 54 |
| Cases, toutes catégories comprises | 12 150 |
| Cases `None` / `Start` | 5 712 / 54 |
| Cases `Stat` / `SkillLevel` | 5 592 / 792 |
| Paramètres source de cases | 121 500 |
| Paires candidates de statistiques | 6 024 |
| Références candidates de compétences | 792 |
| Types de points et plafonds | 5 |
| Entrées explicites de points par niveau | 89 |
| Items d'usage Daevanion | 10 |

Les 12 150 cases retrouvent leur plateau. Les 6 024 paires de statistiques
retrouvent le référentiel partagé ; toutes leurs clés de libellé sont
disponibles en anglais/français. Les 792 références `SkillLevel.Value01`
retrouvent `Skill.ID`. Les références de types de points des plateaux, valeurs
par niveau et items retrouvent aussi leur cible. Aucune référence examinée
pour ces familles n'est manquante ou ambiguë dans cet export.

Exemples et limites contrôlés :

- Le plateau 11 conserve `Gladiator`, le niveau requis 12, le type
  `EDaevanionPointType::DaevanionCrystal` et la monnaie de réinitialisation
  `ECurrencyCategory::GoldCombined`.
- La case 110033 conserve la position source (3, 3), le coût de quatre points,
  le coût de réinitialisation 500 et les paramètres `CombatSpeed` / `150`,
  sans conversion en pourcentage ni assimilation de ces deux coûts.
- La case 150003 contient `PvEAddDamage` / `3` et `PvEDamageDefense` / `30`.
  Elle fait partie des 432 cases ayant une deuxième paire ; les deux sont gardées.
- La case 110037 référence la compétence 11720000 et garde `Value02 = "1"`
  comme paramètre candidat, sans appliquer un niveau supplémentaire.
- Les 89 valeurs de `Exp` concernent `DaevanionCrystal`, niveaux 12–100.
  Leurs valeurs ne sont pas calculées ; aucune progression pour les autres
  types n'est inventée. Les plafonds sont 840 pour `DaevanionCrystal`, 232
  pour chacun des quatre autres types conservés.
- Les huit usages `GetDaevanionPoint` et deux `Fix_DaevanionPoint` restent
  distincts. Le second paramètre d'usage reste une valeur source.
- Les 6 438 cases avec référence d'image utilisent les cinq sprites UI
  correspondants, tous disponibles. Les icônes principales et de bonus des
  54 plateaux ont une référence sans image retrouvée dans le cache fourni.
  Cela ne démontre pas leur absence des archives.
- Aucun graphe ni prérequis de voisinage n'est publié. Les conditions source
  de cette copie sont toutes `None` et vides ; les messages d'adjacence ne
  suffisent pas à déterminer la topologie ou une règle d'activation complète.

Les tests synthétiques vérifient aussi les cas absents de cette copie :
plateau/compétence manquant, statistique inconnue, valeur non entière, condition
opaque et futur champ de connexion non interprété. Ils vérifient le repli de
traduction et les effets des changements de paramètres sur les empreintes.

La base complète conserve les familles précédentes, contient 197 sources et
294 205 entrées suivies dans 30 familles. Cette reconstruction établit une
référence initiale ; elle ne prétend pas observer un changement de patch réel.
Les contrôles d'intégrité et de clés étrangères de l'export passent.

Base de validation : `target/game-data-validation/data.sqlite`, format 13,
736 022 528 octets. La précédente base est sauvegardée sous
`target/game-data-validation/data-before-83.sqlite`. Résumé de l'export final :
`target/game-data-validation/catalogue-83-final-summary.json`.
Empreinte du lecteur :
`23646d2a52d48997d432acde6174ea9adcafa49d24f2fe98b18869096b3be262`.

Validation du code : 77 tests unitaires `game_data`, 35 tests d'intégration
(catalogue, items, compétences et USMAP), Clippy sans avertissement et
compilation Windows réussis. Un premier passage parallèle du test de cache
`cached_decoding_is_hash_addressed_and_purge_preserves_active_catalogues` a
échoué sur un verrou (« Une autre instance utilise cette installation. ») ;
la relance séquentielle des 35 tests d'intégration passe. La cause de cet
échec intermittent n'est pas établie par cette tâche. Le test enfant de
processus reste ignoré dans l'exécution globale par conception.
Exécutable : `target/windows-scroll-preview/debug/aetheris.exe`.
Cette étape ajoute les projections SQLite ; aucune nouvelle page de
consultation n'est ajoutée à l'application.
