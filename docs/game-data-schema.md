# Décodage strict des tables — sous-issue 42 en cours

`game_data::schema::Schema` décrit un layout ordonné, les hashes de colonnes
attendus, la version interne de table et les couples version/région compatibles.
Un client absent de cette liste est refusé avant lecture des lignes. Aucune
version réelle d’Aion 2 n’est déclarée compatible dans le dépôt.

Les primitives comprennent les entiers de 8 à 64 bits, flottants finis,
booléens stricts, chaînes UTF-8 et UTF-16, enums, structures, tableaux simples,
tableaux de structures avec tags Unreal, maps, sets et `FText` (historiques
`None` et `Base`). Le XOR des noms UTF-16 laisse le terminateur brut. Les maps
et sets conservent leurs éléments supprimés dans `Removed` et leurs données
dans `Entries` ou `Values`, pour éviter de perdre une valeur inconnue.

Les tags de tableau sont contrôlés par nom de champ, type, nom de structure,
taille et drapeau de GUID. Les noms de champ `Unknown…` restent inconnus. Les
guides de normalisation doivent continuer à traiter ces valeurs comme telles.
Les flags de `FText` inconnus et ses autres historiques sont refusés.

Chaque décodage exige une correspondance des hashes et du nombre de feuilles
du schéma avec l’en-tête. Le payload doit être intégralement consommé. Les
tailles, comptes, allocations et profondeurs sont bornés. Une erreur indique la
ligne et l’offset ; elle ne renvoie pas une table partiellement considérée valide.

## Analyse des tables avec un mapping USMAP

```sh
cargo run --locked --no-default-features --bin aetheris-game-data -- probe-table /local/mapping.usmap SkillTableItem /local/Skill.bin /travail/skill-probe.json
```

`probe-table` lit un mapping USMAP non compressé de version 0 à 4. Il contrôle
les tailles, références de noms, plages et chevauchements de propriétés,
profondeurs, cycles et budgets de complexité. Les métadonnées de version de
package sont bornées ; les mappings compressés et les en-têtes USMAP v5 sont
refusés. Les propriétés héritées précèdent celles du type dérivé ; les
propriétés locales suivent leur index de sérialisation. Les tableaux fixes
conservent chaque élément dans un champ `Nom[index]`.

Les types pris en charge sont convertis vers le lecteur strict des tables.
Un type non pris en charge dans le layout choisi provoque une erreur. Les
structures homonymes identiques sont réunies ; les définitions contradictoires
restent signalées dans `mapping_ambiguous_structures` et leur utilisation est
refusée, même indirectement. Leur présence hors du layout choisi ne bloque
pas l'analyse de la table.

Le profil `ItemTableItem` exige la version de table 13. Il sélectionne les
champs supplémentaires à partir de `ItemType` : Currency, Equip, Usable ou
Misc. Chaque ligne conserve ces champs sous `$variant`. Le nombre de colonnes
de l'en-tête compte les champs communs et les quatre variantes, alors qu'une
ligne ne contient que sa variante. Une valeur inconnue est refusée.

Le rapport JSON contient les lignes, la version interne, les hashes de colonnes,
les empreintes SHA-256 du mapping et du payload, et
`compatibility_verified: false`. Le nombre de feuilles doit correspondre à
l'en-tête et tous les octets doivent être consommés. Les hashes de colonnes
sont observés dans le fichier, sans correspondance sémantique établie avec les
noms du mapping. Cette commande explore un layout sans déclarer une région
ou un client compatible, sans produire de schéma de production et sans
activer un catalogue. Les entrées sont limitées à 64 Mio ; le décodage est
limité à 16 millions de valeurs, avec un million d'éléments par collection.
L'export remplace atomiquement le fichier de sortie après une lecture réussie.

### Validation sur la copie fournie — 18 novembre 2026

Le mapping local KiskEmu `AION2-5.3.2.0-2.0.6.0.usmap`, au commit
`8fa7095fe2566fd847fcdb81d745616135cd0a63`, a pour SHA-256
`019f06284f923aa5f12660c5debbefcb99f3a424a16b4d62b8d2fdfd4997ea44`.
Son nom de fichier ne prouve pas sa compatibilité avec un autre build.
Le mapping ancien `AION2-5.3.2.0.usmap` ne décrit pas le bon nombre de colonnes
pour ces trois tables et n'a pas été retenu pour leur décodage.

| Table | Version interne | Colonnes | Lignes intégralement lues |
| --- | --- | --- | --- |
| Item | 13 | 125 | 9 452 |
| Skill | 13 | 105 | 16 283 |
| NpcData | 13 | 81 | 13 536 |

Les trois empreintes de payload correspondent au rapport de déchiffrement.
Une comparaison locale avec `aion2dat.py` de la référence Aion2_Client_Decoding,
alimenté par le lecteur USMAP v4 KiskEmu, retrouve toutes les valeurs des
39 271 lignes. Pour comparer les représentations JSON, les variantes sont
aplaties et les maps/sets sont ramenés à leurs entrées ; aucun élément supprimé
n'était présent dans ces collections. Le lecteur Rust conserve ces éléments
quand ils existent. Cette concordance structurelle ne valide pas les unités,
la signification de chaque champ ni les liens entre tables.

`ItemType` distingue 3 557 équipements, 5 311 utilisables, 528 divers et
56 monnaies. `NpcType` distingue 8 298 Monster, 180 Bot, 1 214 Summon,
71 Etc et 3 773 Citizen ; toutes les lignes NpcData ne sont donc pas des
monstres. Aucune statistique n'est calculée depuis un niveau.

Les exports locaux sont `target/game-data-validation/item-probe.json`,
`skill-probe.json` et `npc-probe.json`. Le rapport récapitulatif est
`target/game-data-validation/interpretation-validation.json`. Les mappings et
les lignes du jeu restent hors de Git.

La lecture est également étendue aux 41 tables du profil de compétences,
soit 202 230 lignes comparées au lecteur de référence. La commande
`probe-skills` ajoute le diagnostic des relations, dont les limites et les
écarts sont détaillés dans [la lecture des compétences](game-data-skills.md).

Les tests synthétiques couvrent les types composés, Unicode, GUID, données
tronquées à chaque octet, types invalides et incompatibilités. Ils n’établissent
pas la signification d’un champ sur un client réel. L'import USMAP et les
variantes d'Item sont disponibles pour l'analyse ; les autres profils de
structures et la préparation des schémas de production restent à réaliser
avant de terminer l’issue 42.

```sh
cargo test --locked --no-default-features --test game_data_schema --test game_data_usmap
```
