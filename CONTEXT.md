# Contexte du projet Aetheris

Ce document rassemble les décisions, contraintes et raisonnements établis
pendant l’initialisation du projet. Il doit être mis à jour lorsque l’une de
ces décisions change. Le `README.md` reste la documentation destinée aux
utilisateurs ; ce fichier sert surtout à reprendre le développement sans
perdre le contexte technique.

## Vision du produit

Aetheris est une application compagnon native pour Aion 2. L’objectif est de
regrouper dans une interface dédiée, puis éventuellement dans un overlay :

- un éditeur assisté du fichier `Engine.ini` ;
- des informations supplémentaires pendant le jeu ;
- un historique des donjons réalisés ;
- un combat meter ;
- une carte interactive ;
- des timers ;
- une liste des familiers reliée à la carte et aux lieux où les obtenir.

La première fonctionnalité développée est la gestion de `Engine.ini`. Les
intégrations futures avec le jeu devront respecter ses conditions
d’utilisation. La lecture de mémoire ou du trafic réseau n’est pas incluse
implicitement dans le périmètre du projet.

## Socle technique

- langage : Rust, édition 2024, version minimale 1.98 ;
- interface : GPUI depuis le dépôt de Zed ;
- licence : MIT ;
- `Cargo.lock` est versionné pour rendre les builds reproductibles ;
- la logique métier doit rester séparée de GPUI ;
- chaque page de l’application vit dans son propre fichier sous
  `src/ui/pages/` ;
- les composants partagés vivent sous `src/ui/components/`.

Le module `engine_ini` appartient à la bibliothèque et peut être testé sans
compiler toute l’interface grâce à :

```sh
cargo test --lib --no-default-features
```

## Développement sous WSL

Le développement principal se fait dans Debian sous WSLg. `DISPLAY` et
`WAYLAND_DISPLAY` sont fournis par WSLg, mais le pilote Vulkan choisi
automatiquement a provoqué des crashs. La commande validée utilise Lavapipe :

```sh
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json ./target/debug/aetheris
```

Deux scripts distincts sont volontairement conservés afin de ne pas recompiler
à chaque test visuel :

```sh
./build.sh
./run.sh
```

- `build.sh` compile avec le linker Debian `/usr/bin/gcc` ;
- `run.sh` lance le binaire déjà compilé avec le pilote Vulkan logiciel.

Ne pas imposer globalement `LD_LIBRARY_PATH` pour contourner un problème de
`libXau.so.6` : pendant les essais, cela a conduit à une segmentation fault.
L’installation ou la réinstallation de bibliothèques ne doit pas être proposée
sans avoir identifié précisément la dépendance manquante. Le lancement avec
Lavapipe est la solution qui a réellement fonctionné dans cet environnement.

## Build Windows depuis WSL

GPUI doit être compilé avec les outils Windows natifs pour générer correctement
ses shaders DirectX. `build-windows.sh` convertit le chemin WSL, lance
`powershell.exe`, puis utilise `cargo.exe` installé côté Windows :

```sh
./build-windows.sh
```

Le build est un release verrouillé (`--release --locked`) et produit :

```text
target/windows/release/aetheris.exe
```

Rust et le toolchain MSVC doivent donc être installés sous Windows. Le binaire
utilise `windows_subsystem = "windows"` afin de ne pas ouvrir de terminal au
lancement. Pour distribuer la version actuelle, l’exécutable release est
l’artefact principal ; il faudra vérifier les éventuelles DLL externes avec un
test sur une machine propre avant une vraie publication.

## Fenêtre et title bar

La fenêtre utilise une title bar personnalisée de 31 px. Les décisions prises
pour préserver le comportement natif sont les suivantes :

- sous Windows, les zones utilisent `WindowControlArea::Drag`, `Min`, `Max` et
  `Close` ;
- sous Linux/WSL, le déplacement et les actions passent par les callbacks
  GPUI (`start_window_move`, `minimize_window`, `zoom_window`,
  `remove_window`) ;
- l’icône affichée alterne entre maximiser et restaurer selon
  `window.is_maximized()` ;
- les icônes de la title bar sont blanches (`#FFFFFF`) et mesurent 14 px ;
- les boutons font 46 px de large ;
- le bouton de fermeture conserve un fond rouge au survol.

Le passage de `Window` jusqu’à `title_bar::render` est nécessaire pour connaître
l’état maximisé et choisir l’icône appropriée.

GPUI panique avec `hover style already set` si deux styles `hover` sont attachés
au même élément. Les variantes normale et dangereuse doivent donc rester
mutuellement exclusives avec `when`, ou être réunies dans un seul appel
`hover`, mais ne doivent pas être empilées.

## Système d’icônes GPUI

Les icônes ne dépendent d’aucun SVG au runtime. Les paths conçus dans les SVG
d’origine ont été traduits en appels `PathBuilder` GPUI, sans remplacer les
formes par des approximations. Chaque icône possède son propre fichier :

- `src/ui/components/icons/minimize.rs` ;
- `src/ui/components/icons/maximize.rs` ;
- `src/ui/components/icons/restore.rs` ;
- `src/ui/components/icons/close.rs`.

Le helper `ViewBox` :

- transpose les coordonnées originales dans les bounds GPUI ;
- conserve le ratio du SVG avec une mise à l’échelle uniforme ;
- centre le dessin si le ratio du composant diffère de celui du `viewBox` ;
- sait également mettre à l’échelle les rayons des arcs.

Les composants monocolores reçoivent `width`, `height` et `color`. Pour une
icône duotone, `new(width, height, color)` applique volontairement la même
couleur aux deux paths. `.with_second_color(color)` permet ensuite de modifier
uniquement le deuxième ton. Ainsi, fournir une seule couleur ne laisse jamais
un path avec une couleur implicite ou différente.

### Générateur SVG vers GPUI

`scripts/svg_to_gpui.py` génère un composant Rust formaté à partir d’un SVG :

```sh
./scripts/svg_to_gpui.py icon.svg \
  --name ExampleIcon \
  --output src/ui/components/icons/example.rs
```

Le script :

- lit le `viewBox` au lieu de supposer une taille fixe ;
- convertit `M`, `L`, `H`, `V`, `C`, `S`, `Q`, `T`, `A` et `Z` ;
- gère les commandes absolues et relatives ;
- conserve les contrôles Bézier et `fill-rule="evenodd"` ;
- produit automatiquement l’API monocolore ou multicolore adaptée au nombre
  de paths ;
- n’importe `px` que lorsqu’un arc en a réellement besoin ;
- passe le résultat dans `rustfmt` ;
- refuse explicitement les strokes et transformations qu’il ne sait pas
  traduire, plutôt que de produire silencieusement une icône fausse.

Après génération, le nouveau module doit encore être déclaré et réexporté dans
`src/ui/components/icons/mod.rs`. Les SVG peuvent servir de fichiers d’entrée
pendant la conception, mais ne doivent pas être chargés par l’application.

Le rendu d’une forme détaillée varie à très petite taille : un path dessiné dans
un `viewBox 52×52` et affiché en `14×14` transforme par exemple une bordure de
`4.875` unités en environ `1.31` pixel. Une différence visuelle à cette taille
doit d’abord être analysée comme un problème de rasterisation ou de taille,
avant de modifier les coordonnées originales.

## Fonctionnalité Engine.ini

Le chemin attendu est :

```text
%LOCALAPPDATA%\AION2\Saved_Steam\Config\Windows\Engine.ini
```

La découverte suit cet ordre :

1. le profil de l’utilisateur courant via `LOCALAPPDATA` ;
2. les profils du dossier `C:\Users` sous Windows ;
3. les profils de `/mnt/c/Users` sous WSL.

Les profils sont triés pour rendre la recherche déterministe. Une recherche
sur un autre compte Windows ne fonctionne que si le processus possède les
droits de lecture nécessaires ; les erreurs d’accès sont ignorées et la
recherche continue.

Dans l’interface, une pilule indique `Engine.ini · Trouvé` ou
`Engine.ini · Introuvable`. Lorsque le fichier est introuvable, un clic sur la
pilule ouvre le sélecteur de fichiers système. La sélection doit désigner un
fichier existant nommé `Engine.ini`, sans tenir compte de la casse. Les erreurs
sont affichées au-dessus de la pilule.

Le parseur `EngineIni` est volontairement conservateur :

- il conserve les commentaires, lignes vides, lignes inconnues et l’ordre ;
- les sections et propriétés sont recherchées sans tenir compte de la casse ;
- `get` renvoie la dernière affectation correspondante ;
- `set` met à jour la dernière affectation, ajoute la propriété à une section
  existante ou crée la section manquante.

L’éditeur implémente les 13 catégories de l’ADR 0001. Le catalogue et la
validation vivent dans `src/engine_ini/settings.rs`, les écritures atomiques
et la copie de référence dans `storage.rs`, la détection matérielle et la
préférence GPU dans `hardware.rs`. La bibliothèque reste indépendante de GPUI.
Les actions relisent le fichier avant de modifier les propriétés ciblées.

À la demande du joueur, deux actions globales complètent les actions par
catégorie : « Tout appliquer » applique le profil conseillé des 13 catégories,
et « Tout réinitialiser » rétablit leur état d’origine. L’ADR 0001 tient compte
de cette évolution par rapport à sa restriction initiale. Chaque
action globale effectue une seule écriture ; si une valeur conseillée ne peut
pas être déterminée (résolution native indisponible), aucune modification
partielle n’est enregistrée. Les brouillons sont remplacés après réussite et
conservés en cas d’échec.

Les champs numériques filtrent la saisie et le collage selon leur type
(entier/décimal, signé/positif). La préédition IME reste libre jusqu’à la fin de
la composition, puis la validation vérifie le résultat. Les séparateurs virgule et point
sont acceptés pour les décimales. Les valeurs incomplètes restent des brouillons
et passent la validation métier avant toute écriture.

Conformément à l’ADR 0005, Enter, Tab ou perte de focus valident un brouillon ;
aucun délai d’inactivité ne valide ou n’enregistre la saisie. L’acceptation
synchrone met à jour la référence de la page avant l’écriture. Une erreur de
persistance conserve la référence acceptée et permet de réessayer. Escape reprend
la dernière référence, y compris après une modification externe signalée comme
conflit. Une sauvegarde conserve les autres brouillons et les entités de saisie
pour maintenir le focus. Les limites FPS, la visibilité
et la résolution se valident ensemble. Une erreur dans un groupe ne bloque
pas la sauvegarde des groupes indépendants valides.

L’en-tête de l’éditeur reste fixe pendant le défilement des catégories afin
que les confirmations de sauvegarde et les erreurs restent visibles.

La fermeture native et le bouton de fermeture tentent immédiatement de vider
les modifications en attente. La demande native est différée jusqu’à la fin
du cycle d’événements en cours, puis la fenêtre est retirée par GPUI après
validation pour suivre son cycle de fermeture. Si l’écriture ou la validation échoue, la
fenêtre reste ouverte sur l’éditeur. Le hook de fin d’application effectue
également une dernière tentative. Les choix et interrupteurs écrivent
immédiatement. Aucun réglage conseillé n’est écrit à l’ouverture de la page.

Après une sauvegarde réussie, y compris sans changement de contenu, le fichier
est marqué en lecture seule. `storage::edit` retire temporairement cette
protection pour ses écritures atomiques et la rétablit ensuite. En cas d’échec,
les permissions précédentes sont restaurées. Cela concerne l’attribut Windows
et les permissions d’écriture Unix ; ce mécanisme est distinct du verrou de
l’éditeur pendant l’exécution du jeu décrit par l’ADR 0002.

`resources/Engine.baseline.ini` contient les 80 chemins relatifs de contenu
extraits de la configuration Aion 2 locale et les valeurs d’origine fournies
par le joueur : `r.ngx.dlss.enable=1` et `r.fidelityfx.fsr.enabled=0`. Ces chemins
ne servent qu’à documenter la référence ; ils ne sont jamais restaurés dans
le fichier du joueur. Les anciennes références avec DLSS à 0 ou sans chemins
sont migrées automatiquement. Seules les clés Paths de Core.System sont
reprises d’une référence existante ; toute autre personnalisation est ignorée.

Aucune compatibilité GPU/jeu n’est actuellement confirmée dans le catalogue :
le profil conseillé reste Off, même pour un GPU reconnu ou corrigé. Les choix
manuels DLSS et FSR restent disponibles. La sélection du constructeur est
persistée sous `%LOCALAPPDATA%/Aetheris/gpu-selection.txt` sous Windows et sous
`$XDG_CONFIG_HOME/aetheris` (sinon `~/.config/aetheris`) sous Linux.

Sous Windows, EnumWindows identifie la fenêtre des processus `AION2.exe` ou
`AION2-Win64-Shipping.exe`. MonitorFromRect choisit l’écran ayant la plus grande
intersection ; le repli est l’écran principal. Le nom GDI de cet écran est associé à un chemin
d’affichage CCD actif ; DisplayConfigGetDeviceInfo fournit son mode préféré,
avec inversion des dimensions pour une rotation de 90° ou de 270°. La
résolution actuelle du bureau ne sert pas de substitut à la résolution native. Sous Linux, xrandr fournit le mode préféré
de l’écran principal. La suggestion est rafraîchie toutes les trois secondes
et à l’ouverture de la page sans toucher aux brouillons ni au fichier.

Le champ de saisie est adapté de l’exemple GPUI sous Apache 2.0 ; sa licence
est conservée dans `LICENSES/GPUI-Apache-2.0.txt`.


## Qualité et Git

Les validations habituelles sont :

```sh
cargo fmt --all --check
cargo test --lib --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
bash -n build-windows.sh
```

Les commits suivent Conventional Commits et doivent rester atomiques. Toujours
inspecter `git diff` et `git status` avant de committer. Ne pas ajouter les
captures d’écran, fichiers `Zone.Identifier`, SVG de travail ou artefacts de
build sans décision explicite.

Historique structurant au moment de la rédaction :

- `0e4b33f feat: initialize Aetheris GPUI application` ;
- `e60e247 feat(engine-ini): add discovery and configuration page` ;
- `4adadd1 feat(ui): add GPUI title bar icons` ;
- `9a4b2c9 build(windows): add native release script` ;
- `12b5bb1 fix: harmonize the title bar size with window's default`.

Au moment de créer ce document, `src/ui/pages/engine_ini.rs` contient une
modification locale non commitée. Elle doit être préservée et examinée
séparément ; elle ne fait pas partie des commits d’icônes ou de build Windows.

## Compagnon et distribution (ADR 0004)

`src/companion/` contient la supervision indépendante de GPUI, le protocole IPC
loopback hérité et authentifié, le catalogue de releases GitHub, la vérification
Ed25519/SHA-256 et l’installation avec journal durable et sauvegardes SQLite.
`aetheris-companion` est le point d’entrée ; `aetheris` lui délègue un lancement
direct. `aetheris-update-helper` remplace les processus arrêtés depuis une copie
temporaire, et `aetheris-release` signe les ZIP dans le workflow de publication.
Les workflows stable et beta ciblent `main` et sérialisent leurs publications.
Le dépôt de distribution est `Ayato-san/Aetheris` ; la configuration initiale des
clés et les essais Windows avant publication figurent dans `docs/releases.md`.
L’intégrité est obligatoire en production, avec une exception debug seulement
quand aucun manifeste n’est présent. Les fichiers personnels à côté des
exécutables restent exclus du manifeste et des remplacements. Les bases SQLite
utilisent désormais l’historique incrémental `aetheris_migrations`.

## Design System GPUI

L’ADR 0005 est implémenté dans `src/ui/components/` et `src/ui/theme/`. Les
composants ne dépendent pas du métier ni de la persistance. Les pages conservent
les entités, acceptent synchroniquement les propositions et gèrent les effets.
La galerie `src/ui/pages/gallery.rs` est réservée aux builds de développement ;
Engine.ini est la page pilote. Le détail des API, des horloges suspendables et
des limites de vérification est dans `docs/design-system-implementation.md`.
Les tests GPUI sans interface native utilisent la feature `ui-tests`.

## Cubes cachés (ADR 0012)

L’Overlay Editor active le suivi des objets serveur `34 36` et le son facultatif,
avec des préférences désactivées par défaut. `capture::widgets` partage le lecteur
IPC avec Combat Debug et mutualise réassemblage TCP, tramage et LZ4 entre widgets ;
les consommateurs filtrent ensuite les opcodes et les entités suivies.
La présence est enregistrable et mesure uniquement la détection continue locale.
La fenêtre de guidage est distincte et n’est autorisée que sur une configuration
de capture Windows/OBS validée : Windows 26200.9711, OBS 33.0.1, jeu sans bordure,
selon les essais utilisateur. Les limites de décodage, les tests et la
matrice de validation restante sont documentés dans `docs/overlay.md`.

Le guidage possède désormais une activation persistée distincte et un rendu
(distance, dénivelé, repère fixe des axes du monde) sous la présence. Les actions
`02 38` ne fournissent une position que si leur cible est l’identité locale
connue. Un test d’affichage de 60 s utilise uniquement des données fictives,
avec les protections natives, pour permettre la validation de capture avant
l’affichage des données réelles. L’éditeur indique aussi la réception et les
éventuels échecs d’identification de fenêtre ; celle-ci utilise le PID issu d’un
instantané de processus, sans ouvrir le processus du jeu.
