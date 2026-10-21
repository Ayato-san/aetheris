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
permet à l'utilisateur de le sélectionner manuellement si nécessaire.

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
