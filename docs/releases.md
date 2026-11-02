# Distribution Windows et compagnon (ADR 0004)

Le point d’entrée distribué est `aetheris-companion.exe`. Un lancement direct de
`aetheris.exe` délègue au compagnon. Les trois programmes sont distribués ensemble,
avec `release.json` et sa signature Ed25519 brute `release.sig`. Le manifeste signé
lie la version SemVer, la plateforme et le SHA-256 de chaque programme. Le ZIP
public est toujours nommé `aetheris-windows-x86_64.zip`.

Le dépôt par défaut est `Ayato-san/Aetheris`. `AETHERIS_REPOSITORY`, fourni à la
compilation, permet de le remplacer. `AETHERIS_PUBLIC_KEY` est une clé publique
Ed25519 de 32 octets encodée en hexadécimal, intégrée à chaque exécutable lors de
la compilation. Une build de production sans clé ou sans manifeste authentique
refuse de démarrer. Aucun secret privé ne doit être intégré aux exécutables.
Le binaire `aetheris-release` est un outil de publication ; il n’est pas distribué.

## Première configuration GitHub

Dans un environnement GitHub Actions nommé `release`, configurer :

- la variable `AETHERIS_PUBLIC_KEY`, contenant la clé publique hexadécimale ;
- le secret `AETHERIS_SIGNING_KEY`, contenant la graine privée hexadécimale de
  32 octets correspondant à cette clé publique ;
- les protections de publication adaptées au dépôt et l’autorisation pour
  `GITHUB_TOKEN` de pousser sur `main`, créer des tags et publier des releases.

Générer une paire de clés sur une machine de confiance, avec un chemin **hors du
dépôt**, puis conserver la clé privée dans un gestionnaire de secrets et une
sauvegarde protégée :

```sh
cargo run --no-default-features --bin aetheris-release -- keygen /chemin/prive/aetheris.key
```

Le fichier `.pub` adjacent contient la variable publique. Le générateur refuse
d’écraser un fichier existant et crée la clé avec les permissions `0600` sous
Unix. Ne jamais ajouter la clé privée à Git ou la copier dans une description de
release. La commande `package` vérifie que le secret correspond à la clé publique
intégrée avant de signer.

Depuis la branche `main`, déclencher **Publish stable Windows release** ou
**Publish beta Windows release**, en fournissant `major.minor.patch`. Les deux
workflows partagent un verrou de publication. Le compteur beta est calculé à
partir des tags existants (`v1.2.3-beta.1`, `v1.2.3-beta.2`, etc.). Les versions
stable et beta utilisent le même format SemVer et le même format de signature.

Le workflow prépare un commit Conventional Commit avec `Cargo.toml` et
`Cargo.lock`, exécute formatage, tests et Clippy, compile, signe, puis pousse
atomiquement le commit et son tag. Une release reste en brouillon jusqu’au
chargement complet du ZIP. Un échec avant publication ne rend aucun ZIP partiel
accessible comme release publique. Un échec de publication après la création du
tag peut laisser un tag ou un brouillon à reprendre manuellement.

## Fonctionnement local

Les paramètres de canal, journaux, téléchargements et copies temporaires du
helper vivent dans `%LOCALAPPDATA%/Aetheris`. Les dix derniers journaux de
lancement sont conservés dans `logs`. Ils contiennent les étapes de démarrage,
les sorties standard/erreur (dont les panics), la version Windows disponible et
le statut de sortie. Le diagnostic propose « Ouvrir le journal » pour ouvrir
directement le fichier dans l’éditeur associé à son extension, tout en laissant
le diagnostic ouvert et sans relancer Aetheris. Aucun journal n’est envoyé au
réseau. Leur partage reste une action manuelle de l’utilisateur depuis le chemin
affiché au diagnostic.

Le compagnon vérifie d’abord l’intégrité locale, utilisable hors ligne. La
vérification des nouvelles versions n’empêche pas l’ouverture de la fenêtre.
Seul un résultat initial reçu dans les trois secondes peut déclencher un
téléchargement automatique. Une interaction annule la fermeture automatique ;
le bouton de mise à jour reste disponible. Une fermeture pendant le
téléchargement l’annule. Les vérifications suivantes ont lieu chaque heure et
n’entraînent un téléchargement que sur demande. Le canal préversion inclut les
versions stables plus récentes. Revenir au canal stable ne rétrograde pas.

La version installée est affichée tout en bas de la sidebar (`v0.1.0`, par
exemple, ou `v1.2.3-beta-2` pour une bêta). Un clic ouvre le sélecteur stable /
préversion au-dessus de cette valeur, avec le canal courant coché. Une sélection,
un clic à l’extérieur ou Échap referme le panneau. Ce contrôle est temporaire en
attendant la future page de paramètres ; les tags et les comparaisons conservent
le format SemVer `1.2.3-beta.2`.

Le helper est exécuté depuis une copie temporaire authentifiée, hors du dossier
d’installation, après l’arrêt des processus. Il conserve les programmes
précédents et les bases affectées dans `.aetheris-transaction`, écrit un journal
avant toute modification, garde les transactions SQLite ouvertes pendant le
remplacement des programmes, puis les valide avant la relance. Un arrêt avant
la validation durable de l’opération déclenche une restauration groupée au
prochain lancement. Les sauvegardes utilisent l’API SQLite, y compris pour les
données présentes dans le WAL. Une erreur de permissions est signalée sans
demander automatiquement les droits administrateur.

L’historique SQLite vit dans `src/companion/install.rs::database_history` et les
scripts associés. Ne jamais supprimer une migration nécessaire aux anciennes
versions. `aetheris_migrations` enregistre chaque migration appliquée. La base de
capture debug existante adopte également cet historique ; les installations
neuves créent le schéma actuel dans une transaction. Toute nouvelle base affectée
par une migration doit être ajoutée à la liste de bases du helper.

La sauvegarde précédente reste dans `.aetheris-previous`. Un marqueur y identifie
le premier lancement non encore confirmé par dix secondes de fenêtre ouverte.
Un crash pendant cet intervalle propose la restauration. Une fermeture normale
ne déclenche pas cette proposition. Avant une restauration, les bases après le
crash sont sauvegardées à part, puis les anciennes sauvegardes redeviennent
actives. La version refusée est exclue des réinstallations automatiques ; un
nouvel essai explicite et les versions suivantes restent possibles.

Les fichiers personnels adjacents, dont `Engine.baseline.ini`, ne sont pas
remplacés par le ZIP, ni inclus dans le contrôle des programmes. Une information
de vérification absente ou invalide bloque le lancement et propose une
réparation. Celle-ci privilégie la version signée installée ; un changement de
version demande une confirmation explicite. Sans accès réseau et sans package
signé disponible, le compagnon affiche l’échec et permet de réessayer.

## Fichier `.env` pour les builds locaux

`./build.sh` et `./build-windows.sh` lisent automatiquement le fichier `.env` à la
racine du projet, quel que soit le dossier depuis lequel le script est lancé :

```dotenv
AETHERIS_PUBLIC_KEY=<clé publique hexadécimale de 64 caractères>
AETHERIS_REPOSITORY=Ayato-san/Aetheris
```

Les valeurs non vides du fichier remplacent les variables d’environnement
correspondantes. Les guillemets simples ou doubles, les commentaires, le préfixe
`export` et les fins de ligne Windows sont acceptés. Les valeurs sont lues comme
des données ; les substitutions de commandes ne sont pas exécutées. Sans `.env`,
les scripts utilisent l’environnement existant et les valeurs par défaut.
Le fichier doit rester exclu de Git.

Le script WSL transmet la clé publique et le dépôt au compilateur Windows via
`WSLENV`. Une valeur mal formée interrompt le build sans afficher les clés.
`AETHERIS_SIGNING_KEY`, même présent dans le fichier ou l’environnement, n’est
pas transmis à Cargo pendant la compilation : cette clé est utilisée par la
commande de signature `aetheris-release package`. Les scripts de build compilent
les exécutables ; la génération du ZIP signé reste une étape de publication.

## Développement et validation

```sh
cargo build --bins
cargo run
cargo test --locked --no-default-features
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
./build-windows.sh --debug
```

Une build debug sans manifeste permet le développement sans signature et sans
service de releases. Cette exception n’existe pas en production. Les deux
exécutables auxiliaires doivent être construits avant le premier `cargo run`.
La cible initiale de distribution est Windows x86-64 MSVC ; les modules métier et
le protocole restent portables, mais les packages Linux ne sont pas publiés.

Avant la première publication réelle, effectuer sur Windows un essai graphique
avec deux ZIP signés : démarrage direct et via compagnon, lancement répété,
arrêt forcé, relance explicite, réparation d’un fichier modifié, fonctionnement
hors ligne, saisie invalide empêchant la fermeture, arrêt pendant téléchargement,
mise à jour du compagnon lui-même, interruption de l’installation et restauration
après crash initial. Vérifier aussi le lancement du ZIP sur une machine propre
pour identifier toute DLL système supplémentaire nécessaire à GPUI.

Une ancienne build non signée nécessite une migration manuelle unique : extraire
le premier ZIP signé complet en préservant les fichiers personnels. La
vérification ne peut pas prouver la fiabilité d’un compagnon déjà compromis ;
la confiance initiale dépend du ZIP signé obtenu par un canal fiable.
