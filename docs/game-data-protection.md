# Protection des données — sous-issue 41 en cours

Le module `game_data::protection` traite les payloads déjà sortis des archives :

- manifeste de clés version 2 : dérivation BLAKE3 du matériel fourni localement,
  AES-256-ECB et entrées de 48 octets ;
- tables version 2 : AES-256-ECB puis décompression d’un bloc LZ4 ;
- tables version 3 : flot AES avec compteur little-endian sur les huit derniers
  octets, initialisés à zéro ;
- localisation protégée version 2, dont l’en-tête diffère de celui des tables ;
- branche XOR des payloads de cartes, à sélectionner explicitement par famille.

Les tailles et versions sont contrôlées avant allocation. Les payloads sont
limités à 256 Mio. Une clé manquante, une troncature, une taille incohérente ou
une version inconnue provoquent une erreur. Les payloads déchiffrés ne sont pas
considérés comme des tables valides avant le décodage avec un schéma compatible.
Les fichiers protégés par ces formats ne comportent pas ici de contrôle
d’authenticité cryptographique : une décryption réussie seule n’est pas une
validation.

Aucune clé de contenu ni ressource du jeu n’est embarquée. Une constante
de dérivation d’en-tête identifie le profil de référence ; elle ne remplace
pas le matériel local du manifeste ni les clés de contenu. `KeyRing` reçoit le matériel de dérivation par l’appelant et ne
permet pas sa sérialisation dans un rapport. AES est fourni par RustCrypto,
BLAKE3 par la bibliothèque officielle et LZ4 par `lz4_flex`.

Les tests utilisent uniquement des fichiers synthétiques. Ils contrôlent les
deux familles compressées, les clés absentes, toutes les troncatures des petits
exemples et le compteur à l’aide de deux blocs AES calculés indépendamment avec
OpenSSL. Ils ne démontrent pas encore la compatibilité avec un client installé.

Les lecteurs `archives::Pak` (v11) et `archives::iostore::IoStore` (v5)
acceptent les blocs None/Oodle et les clés AES fournies localement. PAK exige
un index complet et des entrées compactes ; les entrées non compactes sont
refusées. Les indices PAK sont vérifiés par SHA-1 ; les signatures RSA des
conteneurs ne sont pas authentifiées. Les extractions conservent les hashes
SHA-256 et l’origine de chaque fichier, appliquent les patchs et suppressions,
et rendent explicite toute tentative incomplète. Aucun rapport de déchiffrement
ne constitue à lui seul une preuve de validité d’un catalogue.

Le profil supplémentaire `from_current_manifest` / `current_table` /
`current_localization` décrit les références d’octobre 2026 : préfixe opaque de
8 octets du manifeste, seeds dérivés des noms, en-têtes masqués BLAKE3, AES/LZ4
avec préfixe déchiffré de 32 octets ou flot AES avec nonce dérivé de la clé.
Le nom « current » désigne cette variante de référence, sans détection
automatique ni garantie sur les mises à jour du jeu.

`from_versioned_manifest` essaie le profil sans version, puis les versions
numériques `major.minor.patch` (major et minor de 0 à 9, patch de 0 à 999).
Il dérive le contexte BLAKE3 `AION2.TableKey.AppVersion` avec la version,
puis la clé du manifeste à partir du matériel local et de ce contexte. Chaque
candidat doit avoir les huit octets réservés nuls dans **tous** ses enregistrements.
Le profil sans version contrôle désormais aussi ces octets, évitant d’accepter
des clés aléatoires obtenues avec du matériel erroné. La version trouvée est
rapportée comme contexte cryptographique ; elle ne devient pas automatiquement
une identité d’installation validée. Les clés ne figurent jamais dans le rapport.

Le contexte versionné modifie la racine de dérivation des en-têtes. Les tables
compressées et L10N appliquent en plus la permutation des quatre champs obtenue
depuis cette racine : type, taille brute, taille compressée et taille alignée.
Les tables en flot conservent leur en-tête de deux champs et leur clé de masque
distincte, y compris lorsque le payload fait moins de huit octets.
Les limites de taille restent contrôlées avant AES et LZ4. Le contenu L10N est
ensuite lu par le parseur de chaînes, avec contrôle des doublons et du suffixe optionnel
de quatre octets nuls observé sur cette copie.

Cette adaptation de la [PR #471](https://github.com/FabianFG/CUE4Parse/pull/471)
(commit `6d792d8b1e1bfde4a965c3e5c0c404f68da2a67b`, non fusionnée lors de la lecture)
valide le manifeste de la copie avec `AppVersion = 2.0.11`, les 24 localisations
et, après intégration de la permutation documentée dans KiskEmu, le
déchiffrement des 370 tables, dont Skill, Item et NpcData. Les 161 WorldMap et
le fichier NavSignature restent refusés explicitement comme familles non prises
en charge. Le déchiffrement des tables ne vaut pas
validation de leur contenu métier. L’issue 41 reste ouverte ; voir
[les résultats réels](game-data.md#suite-et-validation-réelle).
Les cartes XOR ne sont pas considérées comme des tables métier décodées.

La variante de permutation est vérifiée contre la copie locale KiskEmu épinglée
à `8fa7095fe2566fd847fcdb81d745616135cd0a63`, fichiers
`third_party/CUE4Parse/CUE4Parse/GameTypes/Aion2/Encryption/Dat/`.
Elle partage la permutation entre tables compressées et localisations, sans
introduire le dictionnaire statique de cette référence. Les fixtures synthétiques
couvrent aussi A → B → A, les lectures concurrentes, une mauvaise permutation,
les fichiers renommés, le contexte introuvable et une queue réservée altérée
dans un enregistrement autre que le premier.

Références de format :
[CUE4Parse, protection DAT](https://github.com/FabianFG/CUE4Parse/tree/ec6595e/CUE4Parse/GameTypes/Aion2/Encryption/Dat),
[manifeste](https://github.com/FabianFG/CUE4Parse/blob/ec6595e/CUE4Parse/GameTypes/Aion2/Objects/FAion2KeyManifestFile.cs).
Le décodeur Oodle utilise [oozextract](https://github.com/lvlvllvlvllvlvl/oozextract)
(version verrouillée dans Cargo.lock, licence MIT). Les ressources externes
conservent leurs propres conditions de redistribution.

```sh
cargo test --locked --no-default-features --test game_data_protection
```
