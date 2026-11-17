# Catalogue des données du jeu — implémentation en cours

L’issue [38](https://github.com/Ayato-san/aetheris/issues/38) et ses sous-issues
ne sont pas toutes terminées. Les implémentations de #40 et #44 sont terminées ;
leur fermeture est référencée dans leurs commits. Le moteur est indépendant de GPUI et du réseau.
Les lecteurs d’archives sont maintenant éprouvés sur la copie fournie ; le catalogue métier complet reste à construire.

## Disponible

- Inventaire SHA-256 reproductible des `.pak`, `.utoc` et `.ucas`, chemins
  relatifs et tailles ; contrôle des compagnons IoStore manquants.
- Détection des bibliothèques Steam et des dossiers copiés, sans lanceur ;
  [preuves de version et limites](game-data-inventory.md).
- Vérification complète de l’inventaire avant et après la construction.
- Décodeur borné de tables binaires déchiffrées : version, hashes de colonnes,
  nombres, booléens, chaînes UTF-8/UTF-16 avec XOR optionnel, enums, structures
  tableaux avec ou sans tags, maps, sets et textes ; voir les
  [types pris en charge](game-data-schema.md). Les champs inconnus gardent leur nom et leur valeur.
  Les schémas doivent déclarer explicitement les couples version/région acceptés.
- Lecture PAK v11 et IoStore v5, AES local et codecs None/Oodle ; extraction
  ciblée avec provenance SHA-256, priorité des patchs et suppressions.
- Profils de déchiffrement des tables et localisations après extraction ;
  [formats et limites](game-data-protection.md).
- Construction SQLite des sept familles : objets, compétences, buffs,
  créatures, familiers, recettes et cartes. Les relations explicites peuvent
  conserver apparitions, coordonnées, quantités, conditions et indices sources.
  Cette représentation attend une normalisation revue ; elle ne déduit aucune
  relation à partir d’un nom ou d’un identifiant réseau.
- Validation des identifiants, lignes sources, relations et provenances.
  Chaque famille doit déclarer ses tables requises avant le décodage et chacune
  doit avoir un rapport ; une absence confirmée exige une
  justification et se distingue d’une erreur de lecture. Une traduction
  manquante n’empêche pas l’activation.
- Résolution des noms par langue choisie, anglais `en-US`, nom technique puis
  identifiant, avec statut du repli et du caractère périmé. Sélection du
  catalogue local correspondant, puis fourni correspondant, puis dernier
  catalogue disponible marqué périmé.
- Construction temporaire avec annulation coopérative, transaction SQLite,
  vérification d’intégrité et activation atomique d’un pointeur. Les bases
  précédentes restent présentes. Un échec ne remplace pas le pointeur actif.
  Le contenu normalisé est trié avant import ; son SHA-256 sert de clé de
  réutilisation. Une base existante est comparée à la reconstruction avant
  activation pour refuser un cache modifié. Les publications sont protégées par
  un verrou de processus, et des rapports locaux conservent les erreurs, les
  langues et le nombre de lignes par table. Le pointeur actif est le point de
  validation définitif ; un rapport `validated` sans activation peut correspondre
  à une interruption juste avant ce point.
- Cache intermédiaire borné indexé par les hashes des données et du schéma,
  le client et la révision du décodeur. La purge ne touche ni aux catalogues
  immuables ni aux rapports. Les temporaires abandonnés sont nettoyés après
  acquisition du verrou, lors de la prochaine tentative ou de la purge.

Le manifeste complet, les champs bruts, les traductions et leurs sources restent
dans la base. Ce stockage privilégie la traçabilité ; sa taille et ses limites
devront être évaluées sur les tables réelles avant intégration dans l’application.

## Outil de développement

```sh
cargo run --locked --no-default-features --bin aetheris-game-data -- extract-data /copie/Content/Paks /local/cle-archives.txt /travail/raw
cargo run --locked --no-default-features --bin aetheris-game-data -- probe-iostore /copie/Content/Paks/conteneur.utoc /local/cle-archives.txt
# probe-iostore accepte aussi un dernier argument SHA-256 attendu du TOC.
cargo run --locked --no-default-features --bin aetheris-game-data -- decrypt-data /travail/raw /local/materiel-manifeste.txt /travail/dechiffre
cargo run --locked --no-default-features --bin aetheris-game-data -- inspect /copie/client
cargo run --locked --no-default-features --bin aetheris-game-data -- discover /bibliotheque/steamapps/common /copies
cargo run --locked --no-default-features --bin aetheris-game-data -- decode schema.json identity.json table-dechiffree.bin
cargo run --locked --no-default-features --bin aetheris-game-data -- decode-cached schema.json identity.json table-dechiffree.bin /stockage/catalogue
cargo run --locked --no-default-features --bin aetheris-game-data -- import-reviewed build.json /copie/client /stockage/catalogue
cargo run --locked --no-default-features --bin aetheris-game-data -- status /stockage/catalogue
cargo run --locked --no-default-features --bin aetheris-game-data -- purge-cache /stockage/catalogue
```

`inspect` ne modifie pas la source et n’invente pas sa version ou sa région.
Le fichier optionnel `aetheris-client.json`, placé par l’opérateur dans la copie
de travail, contient une déclaration explicite :

```json
{"version":"version-verifiee-par-operateur","region":"region-verifiee-par-operateur"}
```

Ce fichier est une déclaration, pas une preuve de compatibilité. Les builds
Steam vérifiables et leurs preuves restent séparés dans les métadonnées ;
la région n’en est pas déduite. Sans déclaration, l’inventaire fonctionne mais une activation
est refusée. `identity.json` utilise le même format pour le décodeur.

`import-reviewed` accepte une description `catalog::Build` JSON avec inventaire,
compatibilité, tables requises par famille, rapports, enregistrements, traductions
et relations. C’est une
entrée réservée aux mainteneurs après revue des sources, pas une preuve qu’un
fichier a été extrait ou décodé par Aetheris. Elle conserve un rapport JSON local
dans `reports/` pour les constructions acceptées ou rejetées après lecture du
document. Les documents JSON illisibles échouent avant la création d’un rapport.

Le stockage contient `catalogues/<hash>.sqlite3`, `active.json`, `cache/` et
`reports/`. La purge retire `cache/` et les temporaires abandonnés ; elle ne
supprime pas le catalogue actif. Ne pas supprimer manuellement une base
référencée par `active.json`. Une annulation ou erreur conserve ce pointeur.

## Suite et validation réelle

La copie fournie est accessible sous
`/mnt/d/Programmes/Steam/steamapps/common/AION2/Aion2/Content`.
L’installation Steam Windows correspondante utilise
`D:\Programmes\Steam\steamapps\common\AION2\Aion2\Content` ;
une bibliothèque par défaut peut utiliser
`C:\Program Files (x86)\Steam\steamapps\common\AION2\Aion2\Content`.
Aucun numéro de build vérifiable ni exécutable n’est disponible dans cette copie.
Aucun schéma de production ou catalogue initial n’est déclaré compatible.

Validation locale : 273 PAK lus, 2 390 fichiers extraits sans erreur (2 389
`.dat` et un fichier de configuration), indices PAK contrôlés par SHA-1.
Les 274 TOC ont été lus avec leur premier chunk UCAS décompressé sans erreur ;
ce contrôle ne couvre pas tous les chunks ni la désérialisation des assets Unreal.
La clé d’archives fournie fonctionne ; elle n’est ni embarquée ni enregistrée
dans les rapports. Le codec Oodle utilise le décodeur Rust `oozextract` (MIT),
sans DLL propriétaire.

Les 1 085 fichiers MapEvent passent la branche XOR puis une lecture JSON.
Après adaptation de la [PR CUE4Parse #471](https://github.com/FabianFG/CUE4Parse/pull/471)
(commit `6d792d8b1e1bfde4a965c3e5c0c404f68da2a67b`), le manifeste se valide
avec le contexte cryptographique `AppVersion = 2.0.11`. Ce résultat ne constitue
pas une preuve de build Steam ou de compatibilité d’un schéma métier.
La dérivation versionnée de la clé et de la racine d’en-tête, puis la permutation
L10N, permettent la lecture des 24 localisations : huit langues dans trois
emplacements. Chaque localisation principale contient 153 008 entrées
(153 007 pour `ko-KR`) ; chacune des huit localisations de démarrage contient
487 entrées. Les chaînes sont exportées en `.json` à côté des payloads `.bin`,
notamment `decrypted/AION2/Content/L10N/Text/fr-FR/L10NString.json`.
Le lecteur contrôle également le suffixe optionnel de quatre octets nuls
observé après les entrées ; tout autre contenu final est rejeté.

Le dernier passage produit 2 226 payloads : 746 cartes binaires, une hiérarchie,
1 085 événements, 370 tables et 24 localisations. Il signale encore 162 erreurs :
161 fichiers WorldMap et un NavSignature de familles encore non prises en
charge. `Skill.dat`, `Item.dat` et `NpcData.dat` font partie des 370 tables dont la protection est retirée ; leur interprétation structurée et
leur validation avec un schéma restent à effectuer. Les cartes binaires restent
également à interpréter. Aucun catalogue métier issu de ces données partielles
n’est activé.

Le passage du 17 novembre après consultation des nouvelles analyses KiskEmu
valide également les contrôles renforcés sur les 274 TOC : bornes de tous les
blocs/partitions, empreinte SHA-256 et décompression du premier chunk de chaque
conteneur, sans erreur. Les signatures et les dépendances de packages restent
non vérifiées. Le résultat est dans `target/game-data-validation/iostore-bounded.json`.
Le rapport de déchiffrement actualisé est dans
`target/game-data-validation/decrypted-permuted/decryption.json` ; les nouveaux
payloads et localisations JSON sont dans le même dossier.
Les artefacts locaux sont sous
`target/game-data-validation/`, ignorés par Git.

Tous les tests versionnés utilisent des données synthétiques créées dans le
dépôt, sans octets du jeu ni clés de production.

| Sous-issue | Avancement et travail restant |
| --- | --- |
| #40 | Implémentée : inventaire, bibliothèques Steam, métadonnées vérifiables, partitions et dossiers copiés. |
| #41 | Lecteurs PAK/IoStore, AES, None/Oodle et extraction réelle validés dans les limites ci-dessus ; manifeste versionné et 24 localisations lus ; 370 tables déchiffrées après ajout de la permutation ; 161 WorldMap et un NavSignature restent non pris en charge. |
| #42 | Décodeur strict et types composés disponibles ; import `.usmap`, variantes de structures et schémas réels à compléter. |
| #43 | Stockage et résolution disponibles ; mappings métier des sept familles, traductions et relations concrètes à établir sur les sources réelles. |
| #44 | Implémentée : validation des familles et tables requises, publication atomique, conservation, contrôle de source, progression, annulation, rapports, cache et purge ; interruption réelle du processus testée sous Linux et Windows. |
| #45 | Installation et lecture hors ligne de ressources signées implémentées, avec contrôle d’intégrité et refus des retours à une ancienne révision ; catalogue initial, ressources réelles et canal de téléchargement à préparer. Voir [les ressources signées](game-data-resources.md). |
| #46 | Intégration GPUI, consentement, report, progression, priorité, sélection persistée par installation et reprise après mise à jour de schéma à réaliser. |
| #47 | Correspondances réseau/définitions, intégration du combat et parcours réel complet à valider. |

Références consultées : `LECTURE_FICHIERS_JEU.md` du workspace et copies locales
de `Aion2_Client_Decoding` (commit
`46fb22f36b8556c61f9a8c60ecef688222009552`) et `FModelMats` (commit
`effa42c50dbe7890d1ab47d9eb642c4dd1008876`). Le premier dépôt ne contient pas
de licence racine dans la copie consultée. Aucun script, mapping, clé de contenu, binaire
ou schéma de ces dépôts n’est distribué. Les dispositions binaires et une constante de dérivation d’en-tête sont implémentées dans le lecteur. Avant de distribuer des ressources
externes, établir leurs conditions de redistribution et leur compatibilité.

```sh
cargo test --locked --no-default-features --test game_data_inspect --test game_data_catalog --test game_data_schema --test game_data_protection --test game_data_archives --test game_data_resources
```

Les tests couvrent notamment les langues et replis, les sept familles, les
relations invalides, la reproductibilité logique, la corruption du cache,
l’annulation à chaque phase, les sources modifiées et chaque troncature des
petites tables synthétiques. Un sous-processus est tué pendant sa transaction
SQLite, puis le catalogue actif et la reprise sont vérifiés. Ces tests ne
remplacent pas une extraction réelle ni une simulation de panne matérielle.
