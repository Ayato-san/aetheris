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

Le premier module métier prévu concerne la configuration du fichier
`Engine.ini`.

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

## Architecture

- `src/main.rs` : démarrage de l'application ;
- `src/ui/` : état, composition et composants partagés de l'interface GPUI ;
- `src/ui/pages/` : une interface distincte par page ;
- les futures fonctions seront ajoutées comme modules métier, puis exposées
  dans GPUI sans coupler leur logique au moteur de rendu.

La lecture de la mémoire ou du trafic du jeu n'est pas incluse dans ce socle.
Chaque intégration devra respecter les conditions d'utilisation du jeu et ne
collecter que les données explicitement autorisées.
