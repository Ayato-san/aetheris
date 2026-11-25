# Données du jeu : cible d’extraction et inventaire

Ce document traduit l’analyse de `base-doc/data/data-description.md` et les décisions prises avec le propriétaire du projet. Il définit les données recherchées, pas la preuve qu’elles existent toutes dans le client. La consultation, la carte interactive et le DPS meter sont hors périmètre ; les données doivent toutefois pouvoir alimenter ces fonctions.

## Règles communes

- Rechercher d’abord les données dans les fichiers du client. Une recherche infructueuse ne prouve pas leur absence ; décider du recours éventuel à d’autres sources après investigation.
- Conserver les identifiants et références source, les relations non résolues et les valeurs brutes ou champs dont le sens reste inconnu, avec leur provenance et leur niveau d’interprétation. Ne pas inventer d’unités, de probabilités, de statistiques ou de relations.
- Distinguer ce qui est vérifié, repéré mais non interprété, recherché sans résultat et non recherché ; distinguer aussi valeur absente confirmée, non applicable et inconnue ou illisible. Signaler les erreurs avec build, fichier ou table et diagnostic technique disponibles.
- Conserver noms et textes anglais et dans la langue choisie (français initialement), avec repli sur l’anglais signalé si la traduction manque. Un changement de langue ou de couverture d’extraction n’est pas une modification du jeu. Les descriptions originales et leurs marqueurs sont conservés ; la résolution du texte final relève de l’interface, avec les relations et valeurs contextuelles nécessaires fournies par les données.
- Conserver la classification du jeu, sa hiérarchie et ses libellés localisés lorsque connus. Les parents non établis restent non résolus. Dédupliquer les notions partagées seulement lorsque leur identité est établie. Les identifiants internes des énumérations sont déclarés `SMALLINT` et bornés explicitement à 1–32767 ; SQLite ne fait pas respecter cette borne du seul fait du type déclaré. Les identifiants source du jeu ne sont pas soumis à cette limite.
- Extraire les icônes disponibles dans un référentiel partagé associant identifiant, référence et image. Conserver les relations d’obtention dans les deux sens : source, résultat, quantité, conditions et valeur de probabilité ou poids source. Une valeur d’échelle inconnue n’est pas un pourcentage.
- Un catalogue partiel peut être utilisé avec sa couverture et ses erreurs ; conserver les dernières données fiables d’une famille en échec et signaler leur build. Ne mélanger les familles que si leurs relations restent fiables ; sinon conserver le catalogue valide précédent. Une copie sans build fiable peut être explorée, sans comparaison versionnée fiable.

## Versions et changements

Le manifeste Steam peut identifier le build installé ; son rattachement à une copie de fichiers doit être établi. Comparer les entrées entre extractions fiables par identifiant source lorsque sa stabilité est vérifiée. La première extraction représente l’état initial, sans marquer les entrées comme ajoutées.

Pour chaque entrée, conserver seulement le dernier type de changement observé (ajout, modification ou suppression), le build observé et le build de référence de la comparaison. Conserver une trace minimale des entrées supprimées ; une réapparition est un ajout. Les relations, tables de butin et textes de même langue entrent dans la comparaison. Une donnée non comparable donne un résultat « comparaison impossible », pas une suppression ou une absence de changement. Si un build intermédiaire est illisible, comparer à la dernière version fiable de la famille et indiquer l’intervalle, sans attribuer le changement à une mise à jour précise. Les anciens contenus et le détail des champs modifiés ne sont pas exigés.

Les changements de contenu, poids, quantités ou attribution du butin sont portés
par la définition de loot concernée, identifiable par sa table source et sa clé.
Ils ne marquent pas les items contenus comme modifiés. Les liens entre pools et
attributions permettent de retrouver les contextes concernés sans propager le
marquage à toutes les entrées liées.

## Cible par famille

Pour toutes les familles : identifiant source, nom anglais et localisé, description et icône si disponibles, classification du jeu, provenance, relations et changement observé. Rechercher aussi les champs supplémentaires utiles, même absents de la liste initiale.

| Famille | Données et relations recherchées |
| --- | --- |
| Objets : armes, armures, accessoires, équipements, arcanes, ressources, divers, Panthéon et consommables | Rareté, tier, niveau d’objet, niveau requis, race, classes, statistiques et progression, emplacements de pierres, permissions `can`, prix et monnaie de revente ; propriétés spécifiques à chaque catégorie. Ensembles et bonus des arcanes : au plus un ensemble par arcane ; extraire les ensembles du jeu. Coffres et objets ouvrables : contenu, quantités, choix, tirages et conditions. |
| Recettes | Métier, lieu ou station, niveau et maîtrise requis, race, coûts et conditions, ingrédients et quantités, résultats et quantités, chance et résultat de combo. |
| Quêtes | Catégorie, lieu, niveaux requis et recommandés, race et autres conditions, répétition, étapes ordonnées et objectifs, prérequis, branches et quêtes liées, récompenses garanties, au choix ou conditionnelles. |
| Exploits et missions de saison | Catégorie, objectifs, paliers éventuels et leurs récompenses, prérequis, entités liées, saison, période et progression disponibles. |
| Régions et donjons | Hiérarchie régions/zones, factions et cartes ; donjons, type, accès, niveaux, groupe, difficultés, limites, boss et récompenses. Les comptages sont calculés depuis les relations connues et leur couverture. |
| Requêtes de ravitaillement | Type, objets et quantités demandés, récompenses, conditions et renouvellement. |
| Titres | Rareté, race, effets de possession et d’équipement, conditions et progression. |
| Pass Daeva | Faction, périodes, coûts, niveaux, pistes gratuites et premium, récompenses et quantités, y compris pass anciens présents dans le client. |
| PNJ et créatures | Type, niveau, classification, faction, statistiques et compétences disponibles, variantes par identifiant, apparitions et butins ; pas de statistiques calculées depuis le niveau. |
| Ailes et apparences | Identité et catégorie, rareté, race, effets de possession et d’équipement et progression des ailes ; liens vers objets et autres sources d’obtention. |
| Familiers | Familles, statistiques et effets par niveau, progression et obtention. Cogni, Fera, Natura, Variant et Special sont les familles actuellement connues, à extraire et localiser si possible. |
| Collecte | Points et matériaux distincts, types, classes et niveaux de collecte, emplacements, résultats et conditions. |
| Compétences | Classe, type, type de dégâts, niveaux, spécialisations, coûts, progression, arme, portée, recharge, incantation, charge, stagger, aggro, cibles et effets disponibles. Les compétences n’ont pas de rareté métier ; `SkillGrade` reste un champ technique de la source complète. Préserver les identifiants et références nécessaires au futur DPS meter sans supposer leur correspondance avec le réseau. |
| Altérations d’état | Classification bénéfique/néfaste si connue, durée, niveaux, conditions, cumul et effets, distincts des effets directs d’une compétence. |
| Plateaux et cases Daevanion | Classe, niveau et description des plateaux ; cases, positions, connexions, prérequis, rareté, effets, références et coûts selon le type de case. La possession dépend du personnage et ne découle pas de l’extraction de sa définition. |
| Cartes et apparitions | Fonds ou éléments graphiques, systèmes de coordonnées et transformation si disponibles ; points, zones, groupes et trajets possibles de ressources, créatures et boss, avec leurs conditions. Ce ne sont pas des présences en temps réel. |

## Inventaire initial des sources et état de vérification

L'[inventaire vérifié de #75](game-data-sources.md) complète ce point de départ :
363 fichiers de tables examinés, 353 décodés, 10 refus documentés, 74 contrôles
de relations et ressources de cartes inventoriées sur le build 26581260.
Un nom de table ne garantit ni son sens métier ni sa compatibilité générale.

| Domaine | Sources et état actuel | Suite |
| --- | --- | --- |
| Identification du client | `base-doc/data/appmanifest_3393110.acf` indique le build Steam installé 26581260 ; copie rattachée par confirmation du propriétaire. | Garder région et build distincts ; région non établie. |
| Objets | L'[audit #77](game-data-77-78.md) décrit la projection et ses images ; ensembles, coffres, progressions et routes sont conservés avec leurs limites. | Poursuivre les interprétations documentées sans étendre la preuve aux champs inconnus. |
| Compétences et états | `docs/game-data-skills.md` documente 41 tables et leurs relations sur une copie rattachée au build 26581260 ; les relations et paramètres ne sont pas tous interprétés. `src/game_data/skills.sql` et `skill_projection.rs` contiennent la projection actuelle. | Compléter les champs, les effets et les liens vers autres familles ; vérifier les marqueurs et les correspondances réseau séparément. |
| Créatures, apparitions, collecte, recettes et cartes | Sources client et clés contrôlées dans [#75](game-data-sources.md), indépendamment des builders communautaires. `GlobalRegion` décrit les régions de service ; `Map` et `Subzone` décrivent le monde. | Interpréter les ressources binaires de cartes, coordonnées et liens de spawners ; préserver les références manquantes. |
| Quêtes, exploits, ravitaillement, titres, pass, ailes, apparences, familiers et Daevanion | Sources, exemples et relations documentés dans [#75](game-data-sources.md). Le ravitaillement vient de `PeriodItemCollection*`, les familiers de `Vehicle*`. | Projections #79–83 ; confirmer périodes, règles de progression et connexions Daevanion. |

La copie locale déchiffrée sous `target/game-data-validation/decrypted-permuted/` contient notamment les noms de tables candidats suivants. Leur présence et leur nom ne valident ni leur signification ni leurs relations :

| Besoin | Noms candidats à examiner |
| --- | --- |
| Objets et obtention | `Item`, `ItemSet`, `ItemSetEffect`, `ItemGetRoute`, `ItemGetRouteManual`, `BoxReward`, `RewardPackageItem` |
| Recettes | `CraftRecipe`, `CraftRecipeRate`, `CraftMastery`, `ConversionRecipe` |
| Quêtes et exploits | `Quest`, `QuestStep`, `QuestPart`, `QuestReward`, `QuestString`, `Achievement`, `AchievementStep`, `AchievementReward`, `Journal` |
| Monde et butin | `GlobalRegion`, `Subzone`, `Map`, `Dungeon`, `DungeonRewardInfo`, `PeriodSpawn`, `NpcData`, `NpcLoot`, `RewardData`, `RewardItem` |
| Collecte | `GatherSource`, `GatherSourceMaterial`, `GatherMastery` |
| Pass, titres et apparences | `BattlePass`, `BattlePassGroup`, `SeasonSchedule`, `Title`, `Wing`, `WingEnchant`, `Skin`, `SkinSet` |
| Familiers | `VehicleList`, `VehicleCreatureLevel`, `VehicleCreatureOption`, `VehicleStat` ; rattachement confirmé par les types et textes `UI_BUFFGROUP_PET_*` de `VehicleCreatureSlot`. |
| Daevanion et ressources communes | `DaevanionBoard`, `DaevanionNode`, `EnumString`, `L10NString`, `UIIcon` |

L’[export commun](game-data-export.md) conserve désormais le niveau requis minimal des items, permissions explicitement fausses, définitions et bonus des
ensembles, sources des progressions, coffres et références d’obtention. Les
compétences Item/System et tous les effets décodés sont conservés. Les IDs
internes sont bornés à 32767. Les sources décodées complètes, couverture et
comparaisons de builds sont séparées des tables métier ; les migrations de
schéma de développement ont été retirées. Les images Unreal, certaines unités,
relations, probabilités et correspondances de rareté restent à interpréter ou
à extraire selon la famille et sont signalées comme telles. La réussite structurelle ne vaut
pas déclaration générale de compatibilité version/région.

Après objets et compétences, l'ordre proposé par [#75](game-data-sources.md) est
#79 (recettes/quêtes), #80 (exploits/titres/pass/ravitaillement), #82
(ailes/apparences/familiers), puis #83 (Daevanion). Les identités et relations de
#81 peuvent avancer dès #79 ; la finalisation des cartes/apparitions dépend des
schémas et ressources encore illisibles. Les catégories supplémentaires
découvertes sont proposées pour décision, sans extension automatique de la cible.

L’[état détaillé de #77 et #78](game-data-77-78.md) complète cet inventaire initial
avec les projections et vérifications actuelles, ainsi que les interprétations
encore manquantes pour les objets et compétences.

La [projection #79](game-data-79.md) ajoute les recettes et quêtes dans l'export
Rust et SQLite : composants et substitutions, résultats, coûts, combos, étapes,
conditions et récompenses par classe. Les probabilités brutes et références non
résolues restent distinguées des interprétations vérifiées.

La [projection #80](game-data-80.md) ajoute exploits et paliers, collections,
titres et bonus de possession/équipement, missions, saisons, Pass Daeva,
ravitaillements et définitions de journaux. Les objectifs opaques, périodes
brutes et règles d'éligibilité non établies restent explicitement distingués.
