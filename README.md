# Aetheris

Aetheris est une application compagnon native écrite en Rust avec
[GPUI](https://www.gpui.rs/). Elle réunira des informations de jeu dans une
interface dédiée et, lorsque la plateforme le permet, dans un overlay.

## Feuille de route

- éditeur assisté du fichier `Engine.ini` ;
- overlay d'informations ;
- historique des donjons ;
- combat meter ;
- carte interactive ;
- timers ;
- liste des familiers reliée à la carte et à leurs lieux de farm.

Le premier module métier est `engine_ini`. Il sait parser, consulter, modifier
et restituer un fichier en conservant ses commentaires et ses lignes inconnues.
L'interface le détecte automatiquement dans les profils Windows accessibles et
permet de le sélectionner manuellement si nécessaire. L’éditeur regroupe les
réglages en 13 catégories : profil DLSS/FSR, entrées, fluidité, affichage,
ombres, visibilité des personnages, qualité d’image, distance/LOD/streaming,
latence, post-traitement, ramasse-miettes, textures/VRAM et shaders.

Le statut d’Aion 2 est affiché dans la barre de titre et actualisé chaque seconde.
Lorsque le client tourne, Engine.ini reste consultable mais ses commandes et champs
sont verrouillés. Un statut inconnu verrouille également les modifications. Chaque
écriture vérifie de nouveau les processus juste avant de remplacer le fichier.
La détection est disponible sous Windows ; sous Linux/WSL, le statut reste inconnu
et l’édition est verrouillée. Les saisies en attente sont conservées en mémoire
pendant le verrouillage, sans sauvegarde automatique au retour du jeu à l’arrêt.
Elles sont enregistrées lors d’une nouvelle validation, d’une navigation ou d’une
fermeture après déverrouillage, et perdues si Aetheris est fermé pendant le verrouillage.

Chaque réglage possède une explication, sa valeur actuelle et la valeur
conseillée. Les champs numériques refusent les collages incompatibles et distinguent le
brouillon de la valeur validée. Enter, Tab ou la perte de focus valident la saisie ;
une simple attente ne l’applique pas. La navigation et la tentative de fermeture
valident également les brouillons en attente. Les champs liés sont validés et écrits
ensemble ; une valeur invalide conserve son brouillon et empêche la fermeture.
Les choix et interrupteurs s’enregistrent immédiatement. Une erreur d’écriture
conserve les valeurs validées en mémoire et propose « Réessayer ». Escape annule
le brouillon du champ et reprend sa dernière valeur de référence. Les valeurs absentes ou non
reconnues restent intactes jusqu’à un choix explicite. Les délais des entrées
se saisissent en millisecondes, avec une virgule ou un point décimal.

Les boutons globaux « Tout appliquer » et « Tout réinitialiser » agissent sur
les 13 catégories en une seule sauvegarde. Ils remplacent les brouillons en
attente ; un échec conserve les brouillons et le fichier.

Chaque catégorie possède ses actions « Appliquer les réglages conseillés » et
« État d’origine ». Le retour à l’origine retire ses propriétés ajoutées et
rétablit DLSS à 1 et FSR à 0 pour cette catégorie. Les chemins
`[Core.System]`, les autres catégories, les clés inconnues et les commentaires
sont conservés. Une section devenue vide après une remise à l’état d’origine
est supprimée ; les commentaires et les réglages restants la conservent.
Les blocs écrits portent un commentaire
`; Aetheris — <numéro et nom de catégorie>`, avec une ligne vide entre les
catégories partageant la même section. L’encodage UTF-8/UTF-16 et les fins
de ligne sont conservés ; l’écriture remplace le fichier de manière atomique.
Après chaque sauvegarde, `Engine.ini` est protégé en lecture seule sur disque.
Aetheris retire temporairement la protection pour ses prochaines modifications,
puis la rétablit ; un échec d’écriture restaure les permissions précédentes.

La référence `Engine.baseline.ini` est créée dans `%LOCALAPPDATA%\Aetheris`
pour une installation gérée, et à côté de l’exécutable pour le ZIP. La carte
graphique corrigée manuellement est mémorisée dans le dossier de configuration
d’Aetheris. La résolution conseillée suit l’écran du jeu sous Windows, avec
repli sur l’écran principal ; elle n’est écrite que sur action explicite.


## Widgets d’overlay

**En jeu → Overlay Editor** permet d’activer la présence des cubes cachés
annoncés par le serveur, avec leur durée de détection continue et un son
facultatif à la première détection. Le widget est désactivé par défaut.
La présence peut être enregistrée. Le guidage privé reste masqué en attendant
la validation de son exclusion des captures Windows et OBS.

L’overlay d’orientation est temporairement retiré de l’interface : le suivi
réseau ne fournit pas encore de cap fiable pour le personnage. Son ancienne
activation sauvegardée ne démarre plus de capture ni de fenêtre d’overlay.
L’implémentation est conservée pour une reprise après validation de la source.
Voir [l’état de l’overlay](docs/overlay.md).

## Historique des combats

**Enregistrement → Historique** permet d’importer une capture PCAPNG ou de choisir
une capture déjà importée, puis de désigner une cible, le joueur local et une
fenêtre en secondes depuis le début de la capture, bornes incluses. Les entités
observées sont proposées ; leurs identifiants peuvent aussi être saisis. Un
identifiant réutilisé demande une fenêtre plus précise.

Le bilan présente les joueurs identifiés et un tableau unique de dégâts par
compétence avec le nombre de coups directs et les indicateurs en pourcentage.
Les dégâts des invocations attribuées et des ticks sont inclus dans les montants,
sans colonnes dédiées. Toutes les lignes du tableau sont affichées ; la page
assure le défilement. Le DPS utilise l’intervalle entre le premier et le dernier
dégât retenus pour chaque joueur ; il est indisponible si cet intervalle est nul.
Les sources incertaines et les messages à examiner sont exclus des totaux fiables ;
leurs détails sont affichés uniquement en mode debug. L’enregistrement exige un
dégât fiable du joueur local contre la cible, même lorsque l’analyse est incomplète.

Chaque enregistrement crée un bilan indépendant dans `combat-history.sqlite3`,
dans le dossier de données utilisateur, avec sa version du décodeur et ses limites.
Supprimer une capture ou mettre à jour le décodeur ne recalcule pas les bilans.
Ils peuvent être supprimés individuellement. Cette base dispose de migrations
et est sauvegardée avant les mises à jour, comme les autres données utilisateur.

Le décodage est expérimental : le combat Zikel reproduit les 4 807 725 dégâts,
482 coups directs, 347 ticks et 13 lignes du relevé Questlog. Le parcours
multijoueur est testé sur des données contrôlées ; une capture réelle de groupe
reste nécessaire pour étendre cette validation. Voir [l’ADR 9](docs/adr/0009-combat-history.md).

Les messages de référence nécessaires aux tests sont versionnés dans
`tests/fixtures`. Avec la capture privée présente dans `base-doc/first-analyze`,
la validation complète peut être lancée avec :

```sh
cargo test --no-default-features --test combat_history_reference -- --ignored
```

## Prérequis

- Rust 1.98.1 (version suivie par le fichier `rust-toolchain.toml`) ;
- les dépendances système requises par GPUI ;
- sous Linux : Vulkan, Wayland/X11, fontconfig et leurs en-têtes de
  développement.

Sur Debian 13, les dépendances minimales de cette application s'installent
avec :

```sh
sudo apt install pkg-config libfontconfig-dev libvulkan1 libwayland-dev \
  libx11-xcb-dev libxkbcommon-x11-dev
```

GPUI est consommé depuis le dépôt officiel de Zed. Le fichier `Cargo.lock` doit
être versionné afin de rendre les builds reproductibles.

## Développement

```sh
cargo build --bins
cargo run
cargo test --lib --no-default-features
cargo test --bin aetheris --features ui-tests
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`build.sh` et `build-windows.sh` chargent automatiquement le fichier `.env`
placé à la racine du projet. Ils lisent `AETHERIS_PUBLIC_KEY` et, si nécessaire,
`AETHERIS_REPOSITORY` ; les valeurs non vides du fichier remplacent celles de
l’environnement. Le script Windows transmet ces variables au compilateur natif
via WSL. La clé privée `AETHERIS_SIGNING_KEY` est réservée à la signature du ZIP.
Voir [la configuration de publication](docs/releases.md) et
[la rédaction des notes et l’installation du hook de commit](docs/changes.md).

Sous WSL, compilez avec le linker Debian :

```sh
./build.sh
```

Puis lancez le binaire avec le pilote Vulkan logiciel :

```sh
./run.sh
```

### Design System

La page **Design System**, disponible uniquement en développement, présente les
contrôles partagés, leurs états, les mises à jour programmatiques et les panneaux
imbriqués. Engine.ini utilise ces composants. Les couleurs et dimensions communes
se trouvent dans `src/ui/theme/` ; le compagnon reste indépendant de GPUI.
Voir [l’intégration et les vérifications](docs/design-system-implementation.md).

### Générer un composant d’icône depuis un SVG

Le générateur convertit les éléments `path` remplis d’un SVG en appels
`PathBuilder` GPUI. Le nom du composant est déduit du nom du SVG, ou peut être
fourni explicitement :

```sh
./scripts/svg_to_gpui.py square.svg \
  --name SquareIcon \
  --output src/ui/components/icons/square.rs
```

Pour convertir tous les SVG d’un dossier (sans parcourir les sous-dossiers) :

```sh
./scripts/svg_to_gpui.py ./mes-svgs \
  --output-folder src/ui/components/icons
```

Le dossier de sortie est créé si nécessaire. Chaque SVG produit un fichier
Rust en snake_case : `arrow-left.svg` devient `arrow_left.rs`, avec le composant
`ArrowLeftIcon`. Les fichiers cibles existants sont remplacés. Si un SVG est
invalide, les autres sont tout de même convertis et le script termine avec un
code d’erreur. `--name` et `--output` sont réservés au mode fichier unique.

Le composant généré expose sa largeur et sa hauteur. `new(width, height,
color)` applique une même couleur à tous les `path`. Sur une icône duotone,
`.with_second_color(color)` permet de surcharger le second ton. Pour une icône
ayant davantage de couleurs, `with_colors(...)` accepte le tableau complet.
Il faut ensuite déclarer son module et le réexporter dans
`src/ui/components/icons/mod.rs`. Le générateur prend en charge les commandes
`M`, `L`, `H`, `V`, `C`, `S`, `Q`, `T`, `A` et `Z`, y compris leurs variantes
relatives. Les SVG utilisant des contours (`stroke`) ou des transformations
sont refusés explicitement.

Pour produire un exécutable Windows depuis WSL, Rust et les outils de
compilation MSVC doivent être installés côté Windows. Le script lance le build
avec le toolchain Windows natif afin que GPUI puisse compiler ses shaders
DirectX :

```sh
./build-windows.sh
```

L'exécutable est généré dans `target/windows/release/aetheris.exe`.

### Capture et analyse du combat en debug

La page **Combat · Debug** est disponible dans les builds de développement et
exclue des builds de production. Depuis WSL, compilez sa version Windows avec :

~~~sh
./build-windows.sh --debug
~~~

L’exécutable se trouve alors dans target/windows/debug/aetheris.exe.
Le script désactive la compilation incrémentale, dont les verrous ne sont pas
pris en charge sur les chemins UNC du partage WSL.
La page démarre en mode fichier. Importez un PCAPNG, ouvrez sa session, puis
survolez une ligne pour consulter ses octets et les paquets source complets.
Les filtres, la pagination, la réanalyse et la suppression confirmée travaillent
sur SQLite. Les données sont conservées dans
%LOCALAPPDATA%\Aetheris\debug-captures.sqlite3 sous Windows, ou
$XDG_DATA_HOME/Aetheris (sinon ~/.local/share/Aetheris) sous Linux.
Une seule instance de la page debug peut utiliser cette base à la fois ; un
verrou système empêche une seconde instance d’interrompre la session active.

Le lecteur accepte Ethernet (avec VLAN) et IP brut, IPv4/IPv6, TCP/UDP, les
sections et interfaces PCAPNG multiples, ainsi que les résolutions et décalages
d’horodatage propres aux interfaces. Les octets des paquets et leurs horodatages
originaux sont conservés ; le fichier source complet n’est pas dupliqué.
Un fichier corrompu peut produire une session partielle. Les fragments IP et
les formes non prises en charge restent bruts. Les retransmissions identiques
ne produisent pas de messages supplémentaires ; les trous, contradictions,
troncatures et erreurs de longueur restent visibles.

Sans SYN capturé, l’analyseur cherche automatiquement trois messages serveur
`00 36` avec des horodatages croissants, reliés par des frontières de messages
valides. Il décode à partir du premier tick reconnu et indique que la
synchronisation et, si nécessaire, la direction sont déduites. Les octets
antérieurs restent bruts. La recherche initiale est bornée à 256 Kio ; les
trous TCP et les erreurs interrompent la branche sans reprise automatique.
Les connexions TLS identifiées sont exclues et les corps client sans SYN
restent bruts. Cette reconnaissance s’applique aux imports et au live.
Les deux captures fournies dans base-doc sont reconnues par ce mécanisme.
Réanalysez les sessions existantes pour en bénéficier.

Le bouton **Alignement expérimental** permet un repli offline explicite depuis
le premier octet reconstruit lorsqu’aucune synchronisation n’a été reconnue.
Cette option est mémorisée par session et ne s’applique jamais au live.

Le catalogue S2C contient les 47 opcodes nommés du registre communautaire
transmis. La table affiche leur code et leur nom ; le détail conserve la source
figée et le niveau de confiance documentaire. Le filtre Opcode accepte un code
hexadécimal ou un nom. Un nom reconnu signifie « identifié », pas que son corps
est décodé. Les opcodes inconnus et tous les corps restent consultables.
L’horodatage de `SERVER_TICK` est décodé selon sa structure documentée.
Les anciennes extractions positionnelles d’entité, HP et dégâts ont été retirées :
le catalogue ne fournit pas leur grammaire. La page ne calcule ni DPS ni dégâts
reçus confirmés.

Le mode réseau utilise Winsock natif, sans Npcap ni bibliothèque d’analyse
réseau. Il requiert Windows et le service **Aetheris Capture**, installé avec
l’assistant Windows. L’installation demande les droits administrateur une fois ;
l’application et son compagnon restent ensuite en utilisateur normal.
Le service démarre avec Windows et configure son autorisation TCP entrante lors
de l’installation. Les mises à jour gérées ajoutent aussi le service lorsqu’il
est absent. Pour réparer une ancienne mise à jour qui l’a omis, relancez
l’installeur en réparation. La distribution ZIP seule ne fournit pas ce service.
Sélectionnez une interface, puis utilisez **Start / Stop**. Le choix automatique
utilise les adresses locales des sockets du jeu. La capture peut commencer
après la connexion : le décodage attend alors les trois ticks de confirmation.
Démarrer avant la connexion conserve également le SYN et les premiers messages.
L’attribution au PID utilise des instantanés des tables de sockets :
elle ne garantit pas l’observation des connexions trop brèves pour y apparaître.
Les autres trafics ne sont pas conservés en base.
Le service vérifie le processus applicatif connecté et ne transmet que les
paquets du jeu appartenant au même utilisateur et à la même session Windows.
Les ports UDP partagés avec un autre processus sont exclus. Le service n’ouvre
aucune base personnelle ; l’application conserve l’analyse et l’enregistrement.
Le service reste disponible en arrière-plan ; cette page démarre et arrête encore
explicitement l’acquisition et l’enregistrement. Elle n’active pas de conservation
illimitée des paquets lorsque l’interface est fermée.

Cette version du live capture IPv4. IPv6 est analysable offline, mais les
sockets bruts Winsock ne rendent pas son en-tête original : la page indique cette
limite lorsqu’elle détecte des sockets IPv6 du jeu. L’arrêt ou la relance du
client ne ferme pas la session ; un processus unique est rattaché à nouveau.
Une ambiguïté sur les processus suspend l’attachement. Les sessions interrompues
par la fermeture inattendue de l’application sont signalées au lancement suivant.

Le réassemblage est limité à 32 Mio par direction TCP. L’acquisition continue
pendant les écritures et analyses dans une file de 32 Mio ; une saturation arrête
explicitement la capture après sauvegarde des paquets déjà reçus. La réanalyse
charge les paquets de la session en mémoire : privilégiez des sessions distinctes
pour les longues expérimentations. Aucune purge automatique n’est effectuée.

Les deux fichiers de référence étant des données locales non versionnées, leur
test de régression se lance explicitement lorsqu’ils sont présents :

~~~sh
cargo test --no-default-features --lib reference_captures -- --ignored
~~~

## Compagnon, diagnostics et mises à jour

La distribution Windows propose un [installeur pour tous les utilisateurs](https://github.com/Ayato-san/Aetheris/releases/latest/download/aetheris-windows-x86_64-setup.exe),
avec un service de mise à jour, et conserve le ZIP signé pour l’installation manuelle.
Les deux contiennent l’application, un compagnon indépendant de GPUI et son helper.
Le [fonctionnement de l’installeur](docs/windows-installer.md) précise les données
conservées et les vérifications Windows. Le compagnon
supervise le démarrage et les crashs, conserve dix journaux locaux et propose
la réparation ou la restauration de la version précédente. L’interface permet
de choisir le canal stable ou préversion de `Ayato-san/Aetheris`.

La configuration de la signature, les workflows manuels de publication et les
vérifications Windows sont décrits dans [docs/releases.md](docs/releases.md).
Une build debug sans manifeste reste utilisable pour le développement. Les
builds de production exigent un ZIP signé complet.

## Architecture

- [src/capture/](src/capture/) : acquisition native, service Windows et IPC local ;
- [src/combat_debug/](src/combat_debug/) : import PCAPNG, analyse conservatrice et SQLite en debug ;

- `src/main.rs` : démarrage de l'application ;
- `src/engine_ini.rs` : lecture, modification et découverte du fichier ;
- `src/ui/` : état, composition et composants partagés de l'interface GPUI ;
- `src/ui/pages/` : une interface distincte par page ;
- les futures fonctions seront ajoutées comme modules métier, puis exposées
  dans GPUI sans coupler leur logique au moteur de rendu.

La page de debug effectue une observation passive du trafic attribué au jeu.
Elle ne lit ni ne modifie sa mémoire et n’injecte aucun paquet.

## Code tiers

Le champ de saisie est adapté de l’exemple GPUI `examples/input.rs` de Zed
Industries, sous [licence Apache 2.0](LICENSES/GPUI-Apache-2.0.txt).
