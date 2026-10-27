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

Chaque réglage possède une explication, sa valeur actuelle et la valeur
conseillée. Les champs numériques refusent les caractères non numériques et
s’enregistrent automatiquement après deux secondes sans saisie, ainsi qu’à la
tentative de fermeture de la fenêtre. Les champs liés sont validés et écrits
ensemble ; une valeur invalide conserve son brouillon et empêche la fermeture.
Les choix et interrupteurs s’enregistrent immédiatement. Les valeurs absentes ou non
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

La référence `Engine.baseline.ini` est créée à côté de l’exécutable. La carte
graphique corrigée manuellement est mémorisée dans le dossier de configuration
d’Aetheris. La résolution conseillée suit l’écran du jeu sous Windows, avec
repli sur l’écran principal ; elle n’est écrite que sur action explicite.


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
cargo run
cargo test --lib --no-default-features
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Sous WSL, compilez avec le linker Debian :

```sh
./build.sh
```

Puis lancez le binaire avec le pilote Vulkan logiciel :

```sh
./run.sh
```

### Générer un composant d’icône depuis un SVG

Le générateur convertit les éléments `path` remplis d’un SVG en appels
`PathBuilder` GPUI. Le nom du composant est déduit du nom du SVG, ou peut être
fourni explicitement :

```sh
./scripts/svg_to_gpui.py square.svg \
  --name SquareIcon \
  --output src/ui/components/icons/square.rs
```

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

## Architecture

- `src/main.rs` : démarrage de l'application ;
- `src/engine_ini.rs` : lecture, modification et découverte du fichier ;
- `src/ui/` : état, composition et composants partagés de l'interface GPUI ;
- `src/ui/pages/` : une interface distincte par page ;
- les futures fonctions seront ajoutées comme modules métier, puis exposées
  dans GPUI sans coupler leur logique au moteur de rendu.

La lecture de la mémoire ou du trafic du jeu n'est pas incluse dans ce socle.
Chaque intégration devra respecter les conditions d'utilisation du jeu et ne
collecter que les données explicitement autorisées.

## Code tiers

Le champ de saisie est adapté de l’exemple GPUI `examples/input.rs` de Zed
Industries, sous [licence Apache 2.0](LICENSES/GPUI-Apache-2.0.txt).
