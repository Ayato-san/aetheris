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
limités à 64 Mio. Une clé manquante, une troncature, une taille incohérente ou
une version inconnue provoquent une erreur. Les payloads déchiffrés ne sont pas
considérés comme des tables valides avant le décodage avec un schéma compatible.
Les fichiers protégés par ces formats ne comportent pas ici de contrôle
d’authenticité cryptographique : une décryption réussie seule n’est pas une
validation.

Aucune clé, constante de dérivation propre au client ou ressource du jeu n’est
embarquée. `KeyRing` reçoit le matériel de dérivation par l’appelant et ne
permet pas sa sérialisation dans un rapport. AES est fourni par RustCrypto,
BLAKE3 par la bibliothèque officielle et LZ4 par `lz4_flex`.

Les tests utilisent uniquement des fichiers synthétiques. Ils contrôlent les
deux familles compressées, les clés absentes, toutes les troncatures des petits
exemples et le compteur à l’aide de deux blocs AES calculés indépendamment avec
OpenSSL. Ils ne démontrent pas encore la compatibilité avec un client installé.

Les lecteurs `.pak`/IoStore, codecs d’archives, anciennes localisations et
conditions de redistribution des ressources restent à établir. L’issue 41
reste donc ouverte.

```sh
cargo test --locked --no-default-features --test game_data_protection
```
