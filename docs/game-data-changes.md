# Suivi des changements du catalogue — #76

Le suivi compare les extractions fiables d’une même famille et d’un même profil
de lecture. Il conserve seulement le dernier changement observé, sans historique
des anciennes valeurs ni détail des champs modifiés. L’identification du build
et le rattachement des fichiers au manifeste restent les prérequis décrits dans
[la cible d’extraction](game-data-target.md#versions-et-changements).

## Sources comparées

| Famille | Identité et dépendances comparées |
| --- | --- |
| Objets | `Item.ID`, projection métier et textes EN/localisés, lignes liées d’obtention et d’utilisation hors sources de loot, ensembles et progression. Les références portées par l’item participent ; les contenus et allocations des tables de loot ont leur propre suivi. |
| Compétences | `Skill.ID`, projection métier et textes, niveaux, coûts, armes, classes, spécialisations, conditions et effets liés ; lignes source des SE, niveaux, filtres, acquisition et charge, ainsi que les altérations candidates référencées et leurs ABE/niveaux. |
| Altérations | `SkillAbnormal.ID`, textes, ligne source, référence de texte et lignes des ABE et niveaux associés. Un changement d’ABE est suivi pour son altération, même sans compétence propriétaire. |
| Tables de loot | Table source + clé naturelle : `NpcLoot.NpcId`, `RewardData.ID`, `RewardItem.Name`, `RewardPackage.Name`, `RewardPackageItem.PackageName`, `BoxReward.Group`, `RewardOdEnergyCube.Name`, `DungeonRewardInfo.ID`, `RewardDecomposePackage.Name`, `RewardLevelScaleData.ID`. Pour `Dungeon.ID` Party/Raid, seuls les champs `Reward*`, le type et la difficulté participent. |

Un changement de poids ou de contenu est marqué sur la définition de la pool ;
un changement de chance d’obtenir une pool ou de nombre de tirages est marqué
sur sa définition d’attribution. Les items contenus ne sont pas marqués modifiés
pour cette seule raison. Une nouvelle définition de loot est un ajout, avec sa
propre identité. Les variantes de race ou de niveau partageant une même clé sont
regroupées : leur ordre dans le fichier n’est pas une identité. Les définitions
vides restent suivies et peuvent devenir non vides sans être de nouveaux objets.

Les identifiants de dictionnaires internes ne sont pas des identifiants du jeu.
Les empreintes utilisent les références naturelles de ces dictionnaires. La
position d’une ligne dans son fichier ne constitue pas son identité ; les
listes ordonnées à l’intérieur d’une ligne restent significatives. Les champs
non interprétés des lignes source liées participent sans recevoir de sens ou
d’unité supposés. Une relation candidate SE → altération reste candidate.

La copie réelle disponible correspond au build **26581260**. Les relations et
sources actuelles sont documentées dans [l’audit objets/compétences](game-data-77-78.md).
Leur lecture structurelle ne prouve pas la stabilité des identifiants dans
toutes les régions ou futures versions. Les autres familles n’ont pas encore
de projection autonome ni de suivi par entrée ; leur intégration au moteur
accompagnera les travaux #79 à #83. Leurs lignes déjà liées aux objets peuvent
néanmoins participer au suivi de ces objets.

## États conservés

`catalogue_entries` conserve les empreintes de comparaison, la présence connue,
les derniers builds fiables, le dernier changement (`added`, `modified`,
`deleted`) et son intervalle de comparaison. Une trace supprimée ne conserve pas
l’ancien contenu. `items.client_version_id` et `skills.client_version_id`
restent le dernier build où une modification des données a été observée, ou le
build initial lorsque rien n’a encore changé.

`loot_tables` conserve la paire `(source_name, source_key)` et un identifiant
interne stable entre extractions. Il permet à `catalogue_entries` de suivre les
définitions nommées comme celles à ID numérique. Il conserve uniquement cette
identité pour les suppressions, sans ancien contenu. Son `client_version_id`
pointe vers le dernier build des données comparables ; le type et le build de
suppression restent dans `catalogue_entries`. `reward_pools.loot_table_id`
permet de retrouver directement la définition d’une pool affichée. Un changement
d’une pool ne marque pas ses attributions parentes si leurs propres règles
n’ont pas changé ; les liens permettent de retrouver les contextes concernés.

`catalogue_comparisons` ajoute une référence par famille, y compris vide :

- `observed_build_id` : build de la dernière référence fiable ;
- `reference_build_id` : référence de la comparaison courante ;
- `comparison_profile` : empreinte du lecteur, des structures et des mappings ;
- `comparison_state` : `initial`, `comparable` ou `impossible` ;
- `interval_incomplete` : comparaison interrompue connue, ou reprise après une
  telle interruption. Il ne déduit pas les mises à jour manquantes depuis les
  valeurs numériques des IDs Steam.

Le build de l’extraction tentée reste dans `catalogue_manifest`. Avec une source
conservée, il peut être plus récent que la référence fiable. L’export refuse de
mélanger des sources conservées avec des payloads connus modifiés lorsque leur
cohérence ne peut plus être établie ; l’ancien catalogue reste alors actif.

| Situation | Résultat |
| --- | --- |
| Première extraction complète | État initial ; aucun ajout. Une famille vide reçoit aussi sa référence. |
| Entrée nouvelle après une référence fiable | Ajout, même si la famille était vide. |
| Valeur, texte de même langue ou relation modifiés | Modification des entrées concernées. |
| Même contenu, nouvel export ou ordre des lignes différent | Aucun nouveau changement ; le dernier changement connu reste conservé. |
| Entrée absente d’une lecture comparable | Suppression ; la trace persiste sans répéter la suppression aux extractions suivantes. |
| Réapparition après suppression | Ajout. |
| Autre langue sélectionnée | Les traductions des deux langues ne sont pas comparées ; l’anglais et les autres données restent comparables. |
| Nouveau lecteur, mapping ou profil de sources | Comparaison impossible ; nouvelle référence pour les entrées présentes. Aucune suppression n’est déduite des entrées anciennes devenues non comparables. |
| Source conservée après échec | Comparaison impossible ; conserver les empreintes et builds fiables précédents, sans compter les entrées comme inchangées. |
| Reprise après une extraction partielle | Comparer à la dernière référence fiable et signaler l’intervalle incomplet, sans attribuer le changement à une mise à jour précise. |

`catalogue_coverage` / `build_comparison` est partiel lorsqu’une entrée ou une
famille n’est pas comparable, y compris une famille vide. Une trace ancienne
absente dont le profil est incompatible reste non comparable tant qu’une
identité fiable n’est pas rétablie ; elle n’est pas transformée en suppression.

## Vérification

Les tests de `target_export_tests.rs` couvrent les cycles ajout/modification/
suppression/réapparition, les familles vides, les traductions, les profils
incompatibles, les sources conservées, les intervalles incomplets, les
réexports du même build, l’ordre des lignes et les dépendances objets/effets.
Les tests de `skill_database_tests.rs` vérifient aussi les niveaux d’effets et
la stabilité lorsque les IDs internes des dictionnaires changent.

Le scénario de reprise injecte les états aux mêmes frontières que le
coordinateur : build 100 fiable, tentative 200 partielle, build 300 fiable.
La modification observée en 300 a pour référence 100, avec
`interval_incomplete=true`. Une comparaison fiable suivante remet cet indicateur
à faux. Les builds 100/200/300 sont des fixtures, pas des versions réelles du jeu.
Un second client réel d’un autre build n’est pas disponible pour une comparaison
terrain ; aucun changement réel entre deux patches n’est revendiqué.

Validation du 25 novembre 2026 : **51 tests unitaires ciblés et 15 tests
d’intégration réussis**, Clippy sans avertissement sous Linux et Windows,
compilation Windows réussie. La base réelle régénérée dans
`target/game-data-validation/data.sqlite` contient 9 452 objets, 16 283
compétences, 6 708 altérations et 162 sources. Les 32 443 objets, compétences et
altérations et les 232 259 définitions de loot sont initiaux,
sans changement déclaré ; les quatre familles ont leur référence au build
26581260. Intégrité SQLite, clés étrangères et empreinte du lecteur courant
vérifiées. L’API de consultation lit cette base au format 9, expose la référence
de famille et conserve les valeurs de la compétence réelle 9504 / SE 95041
(aggro 200 au niveau 2).

```sh
cargo test --locked --no-default-features --lib game_data::
cargo test --locked --no-default-features --test game_data_items --test game_data_skills
cargo clippy --locked --no-default-features --all-targets -- -D warnings
```

Pour lire les références sans confondre IDs internes et builds Steam :

```sql
SELECT c.family, c.comparison_state, o.steam_build AS observed_build,
       r.steam_build AS reference_build, c.interval_incomplete
FROM catalogue_comparisons c
JOIN client_version o ON o.id = c.observed_build_id
LEFT JOIN client_version r ON r.id = c.reference_build_id;

SELECT e.family, e.source_id, e.present, e.comparison_state,
       e.change_type, c.steam_build AS change_build,
       r.steam_build AS change_reference_build
FROM catalogue_entries e
LEFT JOIN client_version c ON c.id = e.change_build_id
LEFT JOIN client_version r ON r.id = e.change_reference_build_id
WHERE e.change_type IS NOT NULL OR e.comparison_state = 'impossible';

-- Quelle définition de loot a changé ? Aucun ancien poids n'est stocké.
SELECT t.source_name, t.source_key, e.change_type,
       c.steam_build AS change_build, r.steam_build AS reference_build
FROM loot_tables t
JOIN catalogue_entries e ON e.family = 'loot_tables' AND e.source_id = t.id
LEFT JOIN client_version c ON c.id = e.change_build_id
LEFT JOIN client_version r ON r.id = e.change_reference_build_id
WHERE e.change_type IS NOT NULL;
```

Le format SQLite **9** conserve la référence par famille et ajoute le suivi
indépendant des définitions de loot. Les bases de
développement sont régénérées ; aucune migration n’est exécutée. Une lecture
principale illisible ou un export interrompu avant publication préserve la base
active, sans prétendre avoir observé les entrées du build en échec.
