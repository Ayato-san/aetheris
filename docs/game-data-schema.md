# Décodage strict des tables — sous-issue 42

`game_data::schema::Schema` décrit un layout ordonné, les hashes de colonnes
attendus, la version interne de table et les couples version/région compatibles.
Un client absent de cette liste est refusé avant lecture des lignes. Le jeu
versionné `aion2-steam-26581260-global-v1` déclare désormais le client
`26581260 / global`, dans les limites de couverture décrites ci-dessous.
La réussite d’un `probe-table` reste une exploration sans cette déclaration.

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
pas la signification d’un champ sur un client réel. L'import USMAP et les variantes d'Item servent également au jeu de schémas
versionné décrit ci-dessous ; les commandes d'exploration ne l'autorisent pas
implicitement.

```sh
cargo test --locked --no-default-features --test game_data_schema --test game_data_usmap
```

## Ressources de cartes

`probe-map <mapping.usmap> <MapData|AionWorldMapExportInfo> <decrypted.bin> <output.json>`
lit le profil sans version de table : hashes de colonnes puis une structure
unique. La longueur de la chaîne de hashes inclut ses octets exacts, sans exiger
un terminateur. Le nombre de feuilles et la consommation complète sont vérifiés.
L’entrée est explicitement déjà déchiffrée ; aucun XOR automatique n’est tenté.
Le lecteur distinct de hiérarchie v1 contrôle ses tags et conserve le mot inconnu
de chaque item. Voir [#81](game-data-81.md) pour la projection et ses limites.

## Jeu de schémas versionné — intégration #42

`game_data::schema_set` fournit le même décodage aux outils et au futur worker
Aetheris. Le contrat initial est
[`steam-26581260-global.schemas.json`](../resources/game-data/steam-26581260-global.schemas.json).
Il ne contient ni mapping USMAP, ni clé, ni ligne ou texte du jeu : seulement
les chemins, structures, versions internes, hashes de colonnes et namespaces
nécessaires aux contrôles. `bundled(client)` sélectionne ce contrat avec un
couple version/région exact ; les mises à jour peuvent aussi être installées
avec les ressources signées de #45.

Le contrat lie :

- le SHA-256 exact du mapping USMAP ;
- la révision SHA-256 du code Rust effectivement compilé, calculée sur les
  sources du lecteur de schémas, USMAP, primitives, protection et préparation ;
- le profil `aion2-kiskemu-versioned-permutation-v1` et son contexte AppVersion
  `2.0.11`, distinct du build Steam ;
- les versions et hashes de colonnes attendus de chaque table, et la version
  et le namespace de chaque localisation ;
- la liste explicite des couples client compatibles.

Cette empreinte inclut les modifications locales du lecteur : remplacer ce
code impose une nouvelle validation et un nouveau contrat. Une révision amont
plus récente ou un nom de fichier USMAP ne suffit pas. Le mapping reste celui
de KiskEmu `8fa7095fe2566fd847fcdb81d745616135cd0a63`, avec l'empreinte indiquée
plus haut. CUE4Parse et son correctif C# ne sont pas exécutés : le lecteur Rust
borne la valeur absolue des longueurs signées et vérifie leur multiplication,
leur terminaison et les octets restants avant allocation/lecture.

### Identité et couverture retenues

Le build Steam **26581260** est celui du manifeste fourni, rattaché aux archives
par confirmation du propriétaire. Son SHA-256 est
`9c839fcc6817fca05f5fd465b5b0d0c53250d9206800212959610c7903907cc9`.
Le fichier extrait `Aion2/Config/DefaultBuildConfig.ini` porte le sélecteur
`NCCRAppIDKey=GLOBAL_LIVE_DIST` dans la section du build Aion2 ; son SHA-256 est
`d996fa646ca95da5b96e2cb8f11a26934811d72ffc8b675176611661377382bb`.
La valeur `global` du contrat désigne cette **distribution du client**,
pas la région d'un serveur ou d'un personnage. Les métadonnées cryptographiques,
la langue et la liste des autres sélecteurs possibles ne fournissent pas cette
identité. Les valeurs sans rapport avec cette identité ne sont pas recopiées.

Le périmètre déclaré est de **353 tables et 24 localisations**. Les dix refus
historiques de #75 restent hors contrat : `ActInfo`, `ClientSliderSettings`,
`DefaultSettings`, `FieldEventPlatformerDilemma`, `GlobalSetting`, `KiskData`,
`RS`, `Resources`, `ServerMessage`, `SpecialIdle`. Les sept tables de
`StartUpData` ne sont pas requises par la sélection du catalogue courant et
ne sont pas déclarées compatibles par ce contrat. Les ressources binaires de
cartes gardent le lecteur distinct de #81. Une entrée absente du contrat est
refusée par son décodeur ; elle n'est pas présentée comme décodée.

La déclaration concerne le format lu et la conservation fidèle de ses valeurs.
Elle ne certifie pas la signification de chaque champ ni ses unités : les champs
inconnus et paramètres sont gardés avec leurs noms source et valeurs exactes.
Les IDs des lignes restent distincts des hashes/IDs des colonnes. Les modes
ANSI court/UTF-8 et XOR UTF-16 des noms, `StrProperty` UTF-8, lignes sans tag de
struct, structures imbriquées et tableaux tagués ont leurs contrôles propres.
Les fixtures littérales de noms rejoignent les observations des sérialiseurs
KiskEmu ; leurs round-trips sont une preuve complémentaire, sans introduire
un éditeur ou un écrivain de patches dans Aetheris.

### Contrôle des étapes et exécution

Le parcours distingue explicitement conteneur, payload déchiffré et données
décodées. `SchemaSet::decode` exige `InputStage::Decrypted`. Il ne décide jamais
de déchiffrer sur la seule présence du nombre `13`, qui peut être une version
interne de table. Le rapport de préparation porte désormais `payload_stage`,
`protection` et `reader_revision`. Leur absence dans un ancien rapport reste
inconnue : ce rapport est refusé et doit être produit de nouveau, sans lui
inventer une provenance. Les octets des payloads n'ont pas changé.

```sh
# Observation : crée uniquement un candidat, sans client déclaré compatible.
aetheris-game-data draft-schema-set mapping.usmap prepared selection.json candidate.json
# Relecture du contrat approuvé, identité explicite {"version":"26581260","region":"global"}.
aetheris-game-data decode-schema-set resources/game-data/steam-26581260-global.schemas.json mapping.usmap prepared identity.json fresh-decoded
# Exploration : même contrat et mêmes contrôles, activation toujours interdite.
aetheris-game-data decode-schema-set candidate.json mapping.usmap prepared explore fresh-exploration
```

`selection.json` associe les chemins virtuels `.dat` du rapport de préparation
à `{"kind":"table","row_struct":"ItemTableItem"}` ou
`{"kind":"localization"}`. `draft-schema-set` ne remplit jamais
`compatible_clients`. Son résultat doit être revu avant distribution.
Les limites sont de 16 Mio par contrat/rapport, 4 096 entrées déclarées,
64 Mio par payload, un million d'éléments par collection et 16 millions de
valeurs par table, avec profondeur bornée. La sortie doit être un dossier neuf,
hors des sources. Les chemins et empreintes des payloads préparés sont vérifiés.

`decode_prepared` écrit un rapport `decoding.json`, un JSON par entrée réussie,
et les erreurs des entrées déclarées rejetées ou manquantes. Aucune table
partiellement lue n'est écrite comme valide. Le rapport reste `incomplete`
pendant le traitement ; progression et annulation coopérative sont disponibles
entre entrées. Une annulation laisse `cancelled`, une erreur d'entrée `partial`.
Les erreurs amont restent visibles, notamment `NavSignature.dat`, hors de ce
périmètre. Elles ne rendent pas compatibles les familles qu'elles concernent.

`Report::require_activation(client)` refuse une exploration, une identité
non déclarée, un résultat partiel ou une annulation. Ce contrôle autorise
seulement l'étape de normalisation : les relations, la cohérence entre familles
et la publication SQLite restent contrôlées séparément (#43/#44/#46).
Le normaliseur de développement `export-catalogue` conserve son mode d'analyse
sans déclaration automatique de compatibilité ; ce contrat ne transforme pas
ses anciennes sorties en catalogues de production.

Les bundles signés acceptent `kind: "schema_set"`. Avant installation, ils
contrôlent la compatibilité couverte par le manifeste signé, la présence du
mapping de même empreinte, son parsing et la compilation des layouts, ainsi
que la révision du lecteur. Un contrat seul ou un mapping simplement voisin
ne suffit pas.

### Validation de cette intégration

La relecture du contrat sur les payloads fraîchement préparés donne :

| Contrôle | Résultat |
| --- | ---: |
| Tables déclarées / rejetées | 353 / 0 |
| Lignes de tables | 966 104 |
| Localisations | 24 |
| Entrées de localisation | 2 452 022 |
| Empreintes de payloads préparés comparées à #41 | 2 387 identiques |

Les **966 104 lignes** et leurs structures complètes concordent exactement avec
les 353 probes conservés de #75. Les **24 localisations** concordent avec leurs
exports précédents. Les versions internes et hashes de colonnes sont également
comparés. Ce contrôle étend l'audit initial des trois tables comparées au lecteur
de référence ; il ne remplace pas cette preuve indépendante par un simple
round-trip de notre propre écrivain. Aucun écrivain de tables n'est ajouté.

Deux essais réels de la commande, avec le build `26581261` ou la région `other`,
sont refusés avant création d'une sortie. Les tests synthétiques couvrent aussi
le mapping incomplet ou différent, la révision de lecteur/protection différente,
le contexte AppVersion différent, les rapports historiques sans état établi,
les familles incohérentes, les chemins/dossiers de sortie, les longueurs UTF-16
extrêmes, les troncatures, les tables manquantes et l'annulation en cours.
L'installation signée refuse un mapping absent et une compatibilité non
couverte par la signature, sans activer une ressource défectueuse.

Validation du code : **355 tests Linux distincts réussis** (suite générale de
353 puis deux cas supplémentaires, couverts dans le dernier passage ciblé),
quatre tests dépendant d'artefacts locaux ignorés ; **46 tests Windows distincts
réussis** (44 puis les deux nouveaux cas), Clippy sans avertissement et formatage
validés. L'application et l'outil Windows compilés sont dans
`target/windows/debug/aetheris.exe` et `target/windows/debug/aetheris-game-data.exe`.

Les preuves locales sont sous `target/game-data-validation/` :
`schema-42-identity-evidence.json`, `schema-42-prepared/decryption.json`,
`schema-42-decoded/decoding.json` et `schema-42-validation.json`.
Les logs de tests et de compilation sont sous `target/schema-42-*.log`.
Les clés, mapping et lignes du jeu restent hors de Git. La base SQLite n'a pas
été régénérée : ses payloads source n'ont pas changé. Aucune activation de
catalogue métier n'est effectuée par cette validation de lecture.
