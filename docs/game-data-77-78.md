# Objets et compétences : vérification de #77 et #78

Cet état distingue les projections exploitées dans SQLite, les sources
conservées et les interprétations encore à établir. Le client étudié est le
build Steam `26581260`, rattaché aux archives par confirmation de l’opérateur.

## #77 — Objets et moyens d’obtention

| Domaine | État |
| --- | --- |
| Identité, noms EN/localisés, classifications, race, niveau requis et niveau d’objet | Projections disponibles ; traductions et absence de libellé distinguées. |
| Permissions, statistiques, emplacements de pierres, prix et monnaie | Valeurs source conservées ; permissions vraies/fausses distinctes d’une absence de données. |
| Ensembles d’arcanes | Définitions, membres et bonus conservés ; au plus une référence d’ensemble par arcane. Les quatre lignes `ItemSetEffect` du lot n’ont pas de `OptStats` direct et référencent des altérations. |
| Progression | Groupes, coûts et valeurs source liés. Les statistiques `AdditionalStat` référencées par `ItemSurpass` participent désormais au suivi des modifications, comme celles de `ExceedEnchant`. |
| Obtention et utilisations | Toutes les propriétés non vides de `ItemGetRoute` et les résultats explicites des tables examinées sont indexés, avec contexte et provenance. Voir [l’inventaire détaillé](game-data-obtaining.md). |
| Butins Party/Raid | Poids normalisés dans leur pool ; attribution du pool séparée en tirages garantis et chance d’un tirage supplémentaire. |
| Autres probabilités et conditions | Poids et conditions conservés ; les échelles et les chaînes non vérifiées ne deviennent pas des taux finaux ni des conditions évaluées pour un joueur. |

La correction de dépendance `ItemSurpass → AdditionalStat` concerne **1 140
items**, avec **4 560 références** sur le lot étudié. Elle conserve les lignes
de progression parentes et n’ajoute aucune statistique calculée au jeu.

Deux sources auxiliaires restent en erreur : `FieldEventPlatformerDilemma`
(structure absente du mapping) et `GlobalSetting` (champ de structure
dupliqué). Les champs de progression non interprétés et les probabilités hors
du domaine Party/Raid confirmé restent à examiner. Ces limites ne sont pas
masquées par la présence de leurs tables sources.

## #78 — Compétences, effets et états

| Domaine | État |
| --- | --- |
| Identité, textes, icônes et classes | Définitions et traductions conservées ; classes reliées par les catalogues et spécialisations explicites. Aucun lien réseau supposé. |
| Niveaux, spécialisations, coûts, incantation, charge et recharge | Projections disponibles, avec surcharges de niveau et repli de base ; les autres champs restent consultables dans la source complète. |
| Effets et états | Toutes les lignes SE, ABE, filtres et niveaux sont conservées, indépendamment de leur présence dans une description. |
| Aggro | Valeurs absolues et ratios conservés par effet et niveau ; la fiche affiche désormais la surcharge sélectionnée, y compris zéro. |
| Conditions | Conditions d’activation, de cible et d’effet conservées. Les conditions SE sont désormais visibles dans les fiches d’effets, avec leurs expressions booléennes source. |
| Stagger | Valeur directe exposée pour un seul effet inconditionnel ; plusieurs effets ne sont pas additionnés en un total supposé. |
| Descriptions | Sélecteurs directs résolus avec le niveau sélectionné. Les formules client et contextes non validés restent visibles et signalés. |
| Rareté | Besoin retiré par le propriétaire du projet : les compétences n’ont pas de rareté métier. La colonne `skills.rarity_id`, la table `skill_rarities` et l’affichage sont retirés ; `SkillGrade` reste dans la source complète. |

La base conserve **26 538 paramètres de conditions d’effet**, pour **10 007
effets SE distincts**. Ces valeurs ne sont pas des conditions propres aux ABE
portant éventuellement les mêmes identifiants.

La clôture de #78 demande une vérification de complétude des champs et
relations de la cible, avec des exemples et limites documentés. Elle n’impose
pas d’interpréter chaque paramètre ou unité : les valeurs non interprétées
doivent être conservées et signalées. La résolution finale des descriptions
relève de l’interface ; leurs marqueurs, références et valeurs contextuelles
nécessaires doivent être conservés par l’extraction. La correspondance avec
les événements réseau relève de #47 et demeure non recherchée ici.

La suite sur les descriptions peut être ciblée à partir des modèles anglais
du lot : `se_dmg` apparaît **12 317 fois**, `se_abe_dmg` **497 fois** et
`se_abe` **283 fois**. Ces trois familles ne sont pas encore résolues ; les
formules et le contexte d’application doivent être établis avant substitution.
Ce sont des occurrences dans les modèles des variantes de compétences, pas
des formules ou des compétences uniques. Les sélecteurs directs `se`, `abe`
et `sef` restent traités séparément.

## Vérification finale du périmètre d’extraction

| Critère | Preuve et représentation |
| --- | --- |
| Champs d’objets et classification | `Item` vers `items` et ses dictionnaires ; `PermitLevelMin` vers `item_details`. `ItemTier`, listes de classes et propriétés spécifiques non projetées restent dans la ligne Item complète, sans conversion supposée. |
| Permissions et statistiques | `Can*` vers les permissions vraies/fausses ; listes et indices de classes conservés séparément. `MainStats`, statistiques d’enchantement et `AdditionalStat` conservés avec leurs références source. |
| Progression et coûts | Groupes d’enchantement, dépassement, succession et ajout d’âme vers `item_progression_links` ; coûts, quantités et probabilités brutes dans les lignes parentes. Zéro référence AdditionalStat non retrouvée dans le lot. |
| Ensembles et coffres | Définitions, membres et quatre bonus d’ensembles présents ; `UseEffect=RewardBox` relié à ses récompenses. Choix, quantités et poids conservés ; références non résolues maintenues. |
| Obtention | 28 familles actives, 335 687 occurrences d’obtention ; 382 911 références de routes résolues dans ce lot. 1 671 allocations Party/Raid avec poids et attribution distincts. |
| Compétences et progression | `Skill`, `SkillLv`, `SkillAcquireData`, `SpecializedSkill*`, `SkillCharge` vers les projections de niveaux, coûts, spécialisations et temps. Les autres champs de temps, portée et contexte restent accessibles dans leurs lignes complètes. |
| Effets, états et conditions | Sources SE/ABE, niveaux et filtres intégralement projetés ; paramètres ordonnés, aggro, cibles et conditions conservés. Durées, cumul, remplacement et classification des états gardent leur provenance et les unités non établies. |
| Textes et références | Anglais et français conservés, replis identifiables, modèles et marqueurs originaux préservés. Les 375 occurrences de références de compétences non retrouvées sont détaillées dans `game-data-skills.md` ; aucune cause n’est devinée et aucune ligne source n’est supprimée pour les masquer. |

Les deux sources auxiliaires en erreur ne sont utilisées par aucune projection
de champ métier de ces deux familles. Leur lecture incomplète reste un manque
de couverture déclaré, à poursuivre dans le décodage et l’inventaire global
(#42/#75), ainsi que dans les activités et le monde (#80/#81). Aucune règle
globale, condition de mini-jeu ou disponibilité en jeu n’est déduite de ces
sources illisibles. Leur absence ne devient pas une valeur nulle supposée
valide ni un taux de drop inventé.

Sur la base régénérée, les comptes source et projection concordent :

| Famille | Lignes source et lignes projetées |
| --- | ---: |
| Item | 9 452 |
| Skill | 16 283 |
| SkillAbnormal | 6 708 |
| SkillEffect | 31 631 |
| SkillEffectLv | 78 564 |
| SkillAbnormalEffect | 12 804 |
| SkillAbnormalEffectLv | 16 751 |
| SkillEffectFilter | 10 339 |

Les critères de #77 et #78 sont satisfaits sur le périmètre d’extraction du
build examiné : inventaire, conservation avec provenance, états de couverture,
exemples et échecs documentés. Cela n’établit pas l’interprétation de toutes les
valeurs, la disponibilité de chaque contenu en jeu, la compatibilité de tous
les clients ni l’intégration de production. Ces limites restent explicites.

## Contrôles de cette étape

Les 40 tests unitaires du périmètre et 15 tests d’intégration objets/compétences
passent. Les tests ciblés vérifient les dépendances de statistiques de dépassement, le
signalement des références non retrouvées, la déduplication des dépendances
partagées, les modifications limitées aux objets concernés, les surcharges
d’aggro à zéro, la séparation des conditions SE/ABE et l’absence de rareté
métier des compétences. Les sources du jeu et
les rapports de validation locaux restent hors de Git.

Une lecture de la base réelle vérifie notamment la compétence `9504`, effet
SE `95041` : l’aggro absolue passe de `0` au niveau de base à `200` au niveau 2,
et la fiche restitue bien `200` pour ce niveau.
