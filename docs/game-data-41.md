# Lecture intégrée des archives et protections — #41

Le moteur commun prépare les fichiers nécessaires au catalogue depuis le client
fourni. Il reste indépendant de GPUI, du réseau et de la publication SQLite.
Les outils des mainteneurs et le futur worker de l’application appellent la même
API `game_data::reader::read_client`.

## Parcours intégré

```sh
cargo run --release --locked --no-default-features --bin aetheris-game-data -- read-client /client/Aion2/Content/Paks /local/archive-key.txt /local/manifest-material.hex /travail/lecture
```

Le dossier de travail doit être neuf, son parent doit exister, et il doit se
trouver hors des archives. Aucun fichier du jeu ni catalogue actif n’est modifié.
Les deux fichiers de matériel cryptographique remplissent des rôles distincts :
AES des archives et dérivation des clés du manifeste. Le moteur ne remplace pas
l’un par l’autre lorsqu’une étape échoue. Leurs valeurs ne sont ni embarquées,
ni ajoutées aux rapports.

1. Monter les PAK v11 et IoStore v5, en contrôlant index, plages, partitions et
   formats nécessaires. Les patchs remplacent la version précédente ; les
   tombstones PAK suppriment les entrées sélectionnées, y compris depuis IoStore.
2. Extraire les `.dat` et `DefaultBuildConfig.ini` sélectionnés sous `raw/`.
   Les identités de fichiers sont comparées sans tenir compte de la casse,
   mais **les chemins et noms originaux sont conservés** : ils interviennent
   dans la dérivation des clés de contenu.
3. Lire `key_manifest.dat`, essayer le profil sans contexte puis la recherche
   numérique AppVersion bornée à 100 000 candidats. La variante prise en charge
   pour ce client est celle de KiskEmu, avec racines et permutation versionnées.
4. Déchiffrer/décompresser tables, localisations et familles de cartes prises
   en charge sous `decoded/`. Les localisations ont leur payload `.bin` et un
   export `.json` contrôlé. Le retrait de la protection n’est pas une validation
   du schéma ou des relations métier.

Les API historiques `extract_data`, `decrypt_extraction` et
`from_versioned_manifest` réutilisent ces implémentations. Leurs variantes
`*_with_control` acceptent le même contrôle de progression et d’annulation.
Les autres variantes de protection gardent leurs lecteurs explicites et leurs
fixtures ; aucun profil FModelMats n’est sélectionné silencieusement en repli.

## Progression, annulation et rapports

`Control` reçoit un `AtomicBool` et un callback `Progress` local à l’appel.
Les phases sont `mount`, `extract`, `key_search`, `decrypt`, `complete`.
La recherche annonce sa borne puis sa longueur effective lorsqu’elle aboutit.
Chaque candidat vérifie l’annulation ; les notifications sont espacées pour
éviter d’envoyer 100 000 messages au consommateur.

Les contrôles d’annulation interviennent avant et après les notifications,
entre conteneurs et fichiers, et après leur lecture. Un appel individuel de
lecture ou de décompression n’est pas interrompu en son milieu. Le moteur
n’instancie pas de thread : son appelant choisit le worker, la priorité et le
transport des notifications, conformément à #46. Clés et contexte restent
propres à chaque traitement ; aucune session statique n’est partagée.

- `raw/extraction.json` conserve origine, taille et SHA-256 des fichiers,
  conteneurs montés et empreintes des TOC. Les signatures d’archives restent
  explicitement non authentifiées.
- `decoded/decryption.json` conserve famille, empreintes d’entrée/sortie,
  taille, contexte cryptographique et erreurs par chemin.
- `reader.json` indique `incomplete`, `complete`, `partial`, `failed` ou
  `cancelled`. Le marqueur incomplet précède le traitement ; une annulation ou
  un échec ne laisse pas un ancien résultat présenté comme terminé.

Une erreur d’asset garde les autres résultats, mais produit `partial` et un
code de sortie non nul. Une erreur de montage empêche le déchiffrement d’une
extraction incomplète. Un rapport d’entrée est borné à 16 Mio et 100 000 fichiers,
les doublons de chemins sont refusés et les empreintes sont revérifiées avant
lecture. Les sorties refusent les chemins vers les sources et les traversées
par `..`, avant de créer des descendants. Les liens qui feraient sortir les
écritures du dossier de travail sont refusés.

L’extraction des textures de cartes et leur lecture de dépendances Zen sont
également natives, dans [#81](game-data-81.md). Cette commande prépare les DAT ;
elle ne remplace pas la commande `extract-world-maps` pour les images. Les
icônes et sprites utilisent encore leur convertisseur hors ligne, à intégrer
sans .NET dans le cadre de #46.

## Ressources et redistribution

Le lecteur s’appuie sur les versions verrouillées de bibliothèques Rust,
sans téléchargement de codec pendant le montage :

| Composant | Version | Licence déclarée par le paquet |
| --- | --- | --- |
| `oozextract` | 0.5.7 | MIT |
| RustCrypto `aes` | 0.8.4 | MIT ou Apache-2.0 |
| `blake3` | 1.8.7 | CC0-1.0, Apache-2.0 ou Apache-2.0 avec exception LLVM |
| `lz4_flex` | 0.11.6 | MIT |
| RustCrypto `sha1` / `sha2` | 0.10.7 / 0.10.9 | MIT ou Apache-2.0 |

Les informations proviennent des manifests Cargo des paquets installés et de
`Cargo.lock`. Le paquet `oozextract` correspond au commit
[`8acaeb2bc375dd56c413a83aae4e87156136fdac`](https://github.com/lvlvllvlvllvlvl/oozextract/blob/8acaeb2bc375dd56c413a83aae4e87156136fdac/Cargo.toml),
qui déclare MIT ; aucun fichier LICENSE séparé n’est présent à cette révision
ni dans le paquet consulté. Conserver les déclarations et notices applicables
lors de la préparation de la distribution, suivie dans #45.

Aucune DLL propriétaire Oodle, aucun binaire CUE4Parse, aucun module ACL/RTM et
aucun programme .NET n’est requis pour cette lecture. Les formats ont été
confrontés aux références épinglées dans [la protection](game-data-protection.md),
sans intégrer leurs scripts ou leurs ressources au binaire distribué. Les clés,
mappings et données du jeu restent des entrées locales : cette validation ne
les déclare pas redistribuables et ne les ajoute pas à un bundle de ressources.

## Validation

Les fixtures redistribuables ne contiennent que des données synthétiques et des
clés de test. Elles couvrent les index PAK, AES, Oodle, les partitions et bornes
IoStore, TOC altéré, chunks trop grands, codecs inconnus, troncatures, mauvaises
clés, permutations et contextes absents. Les tests supplémentaires du parcours
intégré vérifient :

- PAK et IoStore dans le même traitement, remplacements base/patch et
  suppression d’un DAT IoStore par un tombstone PAK ;
- conservation du nom `Synthetic.dat`, déchiffrement depuis un chunk nommé,
  et clés distinctes d’archives, de manifeste et de contenu ;
- fichiers de sortie et rapports, étapes de progression et refus de réutiliser
  un dossier de travail existant ;
- annulation de chaque phase et pendant la recherche AppVersion, sans
  contamination du contexte d’une autre session ;
- fichier opaque conservé comme erreur explicite, sans empêcher les fichiers
  pris en charge ni transformer le résultat partiel en réussite complète.

```sh
cargo test --locked --no-default-features --test game_data_archives --test game_data_protection
```

Les résultats sur le client réel et les contrôles des empreintes sont indiqués
ci-dessous ; ils ne déclarent ni compatibilité de schéma de production (#42),
ni activation dans l’application (#46), ni correspondance réseau (#47).

### Résultat sur la copie fournie

Le parcours `read-client` a été exécuté dans un dossier neuf, sur la même copie
rattachée par le propriétaire au build Steam **26581260** :

| Résultat | Nombre |
| --- | ---: |
| PAK v11 montés | 273 |
| TOC IoStore v5 montés | 274 |
| Fichiers extraits | 2 390 |
| Payloads préparés | 2 387 |
| Tables / localisations | 370 / 24 |
| Cartes / événements / hiérarchie | 746 / 1 085 / 1 |
| Ressources WorldMap | 161 |

Le contexte cryptographique retrouvé est **2.0.11**. La région restait inconnue
à cette étape ; l'analyse ultérieure du sélecteur `GLOBAL_LIVE_DIST` dans #42
qualifie la distribution `global` (voir [les preuves](game-data-schema.md)).
Le contexte cryptographique ne déclare pas à lui seul la compatibilité d'un schéma métier.
`Map/Start/NavSignature.dat` reste explicitement non pris en charge. Le résultat
est donc `partial`, avec code de sortie non nul, même si toutes les familles
requises par le catalogue courant sont disponibles.

Les **2 390 empreintes de fichiers bruts** correspondent à la précédente
extraction validée. Les **2 387 empreintes de payloads** correspondent au passage
utilisé pour #81 ; leurs fichiers ont tous été relus et contrôlés après écriture,
ainsi que les 24 exports JSON de localisation. Les **274 TOC et premiers chunks**
ont aussi été relus avec leur empreinte de TOC attendue ; résumés et tailles
correspondent exactement au précédent contrôle borné. Ce dernier contrôle ne
prétend pas décoder tous les chunks graphiques du jeu ; la validation complète
des textures de cartes est documentée dans #81.

Un essai avec le matériel d’archives à la place de celui du manifeste est refusé
après la recherche bornée : état `failed`, 2 390 fichiers extraits et aucun
payload déchiffré. Aucun repli silencieux ni activation de base n’a lieu.

Rapports locaux, ignorés par Git :

- `target/game-data-validation/reader-41-validated/reader.json` et rapports des
  deux étapes, avec données préparées sous `raw/` et `decoded/` ;
- `target/game-data-validation/reader-41-validation.json` pour les comparaisons
  et contrôles d’empreintes ;
- `target/game-data-validation/iostore-41-validation.json` pour les TOC/chunks.

Validation du code : **346 tests Linux réussis**, quatre tests dépendant de leur
contexte laissés ignorés, **20 tests ciblés Windows réussis**, Clippy sans
avertissement, formatage et compilation Windows réussis. Un premier passage
parallèle a rencontré un refus de verrou dans le test de cache ; sa relance
isolée puis le passage complet avec `--test-threads=1` passent. La logique des
verrous du catalogue n’a pas été modifiée pour contourner ce résultat.
L’application et l’outil compilés sont dans `target/windows/debug/`.

Aucun schéma ou contenu de catalogue n’a changé : les payloads du nouveau
parcours sont identiques à ceux de la base précédemment générée. Cette étape
valide la lecture des sources ; elle ne reconstruit ni n’active SQLite.
