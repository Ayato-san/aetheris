# Intégration du Design System GPUI

L’[ADR 0005](adr/0005-centralized-gpui-design-system.md) et ses
[contrats](design-system.md) sont implémentés dans l’application principale.
Le compagnon n’ajoute aucune dépendance GPUI.

## Organisation et utilisation

- `resources/design-tokens.json` contient les couleurs, dimensions et poids ;
  `theme/mod.rs` expose leurs rôles sémantiques. Le build compile les tokens en
  constantes Rust. Le remplacement exige une recompilation.
- `src/ui/components/mod.rs` est l’interface commune d’import. `Button` expose
  un élément GPUI auquel la page attache son action. `NavigationButton` suit
  la navigation acceptée par `AetherisApp::navigate`. `Card::element` compose
  un titre et un corps libre ; une action principale s’ajoute comme Button
  distinct pour ne pas intercepter les contrôles enfants.
- `Switch`, `Slider`, `Selector`, `ShortcutSelector`, `TextInput`, `SearchBar`,
  `Timer` et `Overlay` utilisent des entités conservées par la page.
  `NumberInput::create` construit un TextInput numérique configuré.
- `SelectorKind::{Segment, Multi, Select}` expose les trois présentations avec
  le même modèle `Selection`. Options et sélection sont identifiées par ID ;
  `configure` remplace leurs contraintes et leurs valeurs atomiquement.
- Les callbacks `on_change_requested` rendent une décision synchrone. Les
  événements typés `ValueChanged`, `SwitchChanged`, `SliderChanged`,
  `SelectionChanged` et `ShortcutChanged` suivent seulement une valeur acceptée
  différente. `ContentChanged` observe le brouillon ; `Validated` est un événement
  interne de validation, utilisé notamment par SearchBar sans dupliquer les recherches.
- Les mises à jour programmatiques n’émettent pas ces changements utilisateur.
  Les opérations pouvant échouer rendent `Validation<T>`. Une configuration
  refusée ne modifie ni les contraintes ni la valeur précédente.
- `TextInput::set_content` synchronise la référence et préserve un brouillon
  concurrent avec indication de conflit. `reset` abandonne explicitement le
  brouillon, l’erreur et le conflit. Les valeurs numériques respectent leurs
  contraintes aussi lors des mises à jour programmatiques.
- `Table` compose colonnes, en-tête triable et cellules libres. La page traite
  `SortRequest`, conserve ses données et les entités des cellules. `ordinary`
  accepte les hauteurs variables et un emplacement de statut ; `virtualized`
  utilise `VirtualRows` avec une enveloppe de hauteur uniforme.
  `fit_content()` affiche toutes les lignes ordinaires à leur hauteur naturelle,
  sans défilement vertical interne ; le layout de page assure le défilement.
  Le bilan des combats utilise ce mode pour son tableau unique de dégâts et
  d’indicateurs. Les dégâts des invocations et des ticks sont inclus dans les
  montants affichés, sans colonnes dédiées.
- `OverlayKind::Popover` et `OverlayKind::Popup { modal: true }` fournissent les
  panneaux composables. Le Select utilise le même positionnement ancré et différé.
  `Overlay::trigger` mesure le déclencheur et gère son ouverture ; les dimensions,
  le placement et le clic extérieur sont configurables. Les demandes de fermeture
  indiquent leur origine et peuvent être refusées par la page. Les sentinelles
  de focus maintiennent Tab et Shift-Tab à l’intérieur d’une Popup modale.

## Engine.ini

L’éditeur conserve les contrôles dans `AetherisApp`. Les valeurs non reconnues
restent dans le document jusqu’à une action explicite. Les brouillons numériques
sont observés sans lancer de sauvegarde. Enter, Tab et perte de focus passent par
la validation synchrone de la page ; les réglages liés sont acceptés ensemble.
Une sauvegarde échouée conserve le document accepté, distinct du fichier enregistré,
et expose « Réessayer ». Une modification extérieure du fichier est détectée
avant écriture ; une action explicite permet de recharger le fichier en abandonnant
les modifications locales. La navigation et la fermeture continuent de vérifier les
brouillons et les écritures en attente, en respectant le verrouillage par le jeu.
Les actions explicites d’origine et de recommandation remplacent les brouillons
concernés après réussite de l’écriture.

## Horloges

Un Timer de durée utilise `CLOCK_BOOTTIME` sous Linux et `GetTickCount64` sous
Windows : ces compteurs incluent la veille sans dépendre de l’heure civile.
Un Timer calendaire suit `SystemTime`. Le rendu calcule le restant depuis
l’échéance, arrondit les secondes au supérieur et conserve l’expiration unique
par décompte dans l’entité. La page doit garder cette entité si elle souhaite
conserver cette identité au retour sur la page ; `restart` ouvre un nouveau décompte.
Le callback d’expiration ne s’exécute que lors du rendu du composant.

## Vérifications

La galerie **Design System** est disponible uniquement en développement. Elle
présente les seize composants, tailles de boutons, refus, désactivation,
conflits, réinitialisation, programmation sans événement, table de 10 000 lignes
et Select dans une Popup. Les contrôles de cellules restent détenus par la page.

```sh
cargo test --lib --no-default-features
cargo test --bin aetheris --features ui-tests
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

La feature `ui-tests` active le support de test de GPUI pour exercer les événements
sans serveur graphique. Les contrats purs vérifient aussi les contraintes
numériques, la synchronisation des brouillons, les sélections, les pas irréguliers
et le format et les horloges du Timer.

Les composants exposent les rôles, libellés, valeurs et états disponibles dans
la révision verrouillée de GPUI. Les indisponibilités sont décrites en texte ;
les contrôles désactivés quittent le parcours Tab. Cela ne remplace pas une
vérification avec NVDA sous Windows. Le rendu visuel, l’IME natif et la reprise
après veille réelle nécessitent également une validation sur les plateformes cibles.

## Référence Penpot et tokens modifiables

Les composants reprennent `base-doc/design-system-penpot-export.html` et les
valeurs de `base-doc/tokens.json`. Leur source versionnée et embarquée est
[`resources/design-tokens.json`](../resources/design-tokens.json). Le dossier
`base-doc`, ignoré par Git, sert de référence et n’est pas requis pour compiler.
`theme/controls.rs` expose les rôles des boutons et segments depuis ce JSON.

| Élément | Dimensions et typographie | Couleurs de l’export |
| --- | --- | --- |
| Button primary | 40 px de haut, 53 px minimum, padding horizontal 12 px, rayon 8 px, bordure 1 px, Montserrat 500 en 14/20 px | fond et bordure `#65d1b2`, survol `#4db89a`, texte blanc |
| Button secondary | mêmes dimensions et typographie | fond `#484848`, bordure et survol `#a3a3a3`, texte blanc |
| SegmentGroup | largeur de référence 600 px, hauteur 44 px, padding 4 px, segments égaux de 36 px sans espacement, Montserrat 700 en 14/20 px | fond `#242424`, sélection `#83dec4` avec texte `#4db89a`, survol `#303030` avec texte `#eeeeee`, texte désactivé `#a3a3a3` |

Le groupe s’adapte à la largeur disponible et défile si les minima des segments
ne tiennent plus. Les pilules n’héritent plus du style Button : aucune bordure
ou coche n’est ajoutée à leur état normal. `SelectorKind::Segment` conserve
l’acceptation, les identifiants, le focus et le clavier ; `SegmentGroup` fournit
les éléments visuels dans son propre module.

L’export décrit la taille medium des boutons. Les variantes small (32 px) et
large (48 px), le focus et la désactivation restent disponibles en complément ;
l’état appuyé reprend la couleur de survol.

Les fontes latines de l’export sont intégrées en TrueType dans `resources/fonts/`
et chargées au démarrage. Elles utilisent une famille interne privée pour que
leur sélection soit cohérente sous Linux et Windows. Leurs contours et métriques
sont conservés ; voir les [informations des fontes](../resources/fonts/README.md).
La galerie présente les composants et leurs interactions, avec des champs à la
largeur de référence de 300 px. Les icônes restent à fournir séparément.

### Modifier le style

Modifier les `$value` dans `resources/design-tokens.json`, puis relancer
`cargo run` (ou reconstruire la version distribuée). Aucune modification Rust
n’est nécessaire pour changer une couleur ou une dimension existante.

- `Global` conserve les tokens Penpot, dont `accent`, les gris, `size-base`,
  `sm`, les rayons et les familles. Les références comme `{accent}` sont suivies
  automatiquement ; les calculs tels que `{size-base}/4` gardent leur priorité
  arithmétique.
- `Components` ajoute les rôles et métriques relevés dans le HTML : `button`,
  `segment`, `field`, `select`, `switch`, `slider`, `navigation`, `shortcut`,
  `card`, `table`, ainsi que les dimensions partagées et les panneaux.
- Pour recolorer les boutons primaires et les barres actives, modifier
  `Global.accent`, `Global.accent-dark` et `Global.accent-light`. Pour agrandir
  les champs, modifier `Components.field.height` et `paddingX`. Pour changer
  la taille du texte courant, modifier `Global.sm` ou sa référence `size-base`.
- Le build résout les alias et les expressions en pixels, puis écrit les
  constantes dans `OUT_DIR`. Les références inconnues, cycles et expressions
  invalides interrompent la compilation avec une erreur. Les composites de
  typographie Penpot sont conservés dans le JSON ; les composants consomment
  les tailles et poids scalaires de leurs rôles.

Les champs de texte, nombre, recherche et sélection partagent le fond `gray-850`,
la bordure `gray-400`, le rayon de 8 px et la hauteur de 40 px. Le focus utilise
une bordure de 2 px `accent-dark` ; une erreur garde la priorité en rouge.
Les listes utilisent des lignes de 28 px, du texte normal en Montserrat 400 et
une sélection grise en Montserrat 700. Les switches ont une piste de 40 × 22 px
et une poignée de 18 px ; les sliders une piste de 4 px et une poignée de 8 px.
Les cartes et tables reprennent leurs fonds, bordures et rayons Penpot tout en
conservant une composition flexible pour les contenus applicatifs.

`cargo test --test design_tokens --locked --offline` vérifie la propagation des
changements de palette et d’échelle, ainsi que le rejet des tokens invalides.
Les sections de la galerie sont mesurées à l’initialisation ; ses sections visibles
sont ensuite rendues par le même composant que les autres pages.

### Défilement visible

[`ScrollContainer`](../src/ui/components/layout.rs) est le layout commun pour
un contenu de hauteur libre dans un conteneur contraint. Il gère son état de
défilement et affiche automatiquement la scrollbar lorsque le contenu dépasse
la hauteur disponible. Les pages n’ont pas de `ScrollHandle` ou de `ScrollArea`
à conserver, ni de dépassement à mesurer.

```rust
ScrollContainer::new("page-content", content).size_full()
// Pour un panneau ou une liste de hauteur limitée :
ScrollContainer::new("panel-content", content).max_h(px(300.))
```

Le contenu conserve sa hauteur naturelle ; les contraintes de taille s’appliquent
au layout. L’identifiant doit être stable et unique parmi ses frères : la position
de défilement est conservée entre deux rendus. Le layout réagit aux changements
de contenu et de taille et ramène la position dans les limites valides lorsque le
contenu diminue. Historique, les tables et les panneaux Popover/Popup utilisent
cette interface. `VirtualList` et `VirtualRows` lui confient également leur
scrollbar en lui transmettant leur état de liste. Ils ne conservent que l’état
nécessaire à la virtualisation, au focus et à la navigation vers une ligne.
La création de la scrollbar, son affichage et son déplacement sont gérés par
le layout ; les pages et les contrôles ne l’enveloppent plus manuellement.

En interne, `ScrollArea` conserve un `ScrollHandle` et l’état du déplacement du curseur.
Il enveloppe le contenu défilant et peint la barre dans un élément frère fixe :
la piste ne défile pas avec le contenu. Il est utilisé dans Engine.ini, la galerie,
les listes de sélection et les panneaux Popover/Popup. La barre apparaît quand
le contenu dépasse la hauteur disponible ; la molette reste disponible.

`Components.scrollbar` configure la largeur (4 px), le retrait (8 px), la taille
minimale du curseur (32 px), la piste (`gray-900`) et le curseur (`gray-400`).
La taille du curseur suit le rapport entre contenu et hauteur visible. Un clic
sur la piste positionne le curseur ; le glisser continue hors de la barre et se
termine au relâchement du bouton. Les tests vérifient ces interactions et la
disparition de la barre lorsque le contenu ne dépasse plus.

### Performance d’Engine.ini

La page utilise une `ListState` GPUI à hauteurs variables pour les commandes
globales et les 13 catégories. Les hauteurs sont mesurées initialement pour
conserver une scrollbar correspondant à toute la page ; pendant le défilement,
seules les catégories visibles et une marge de 200 px sont reconstruites.
Les entités des champs restent détenues par `AetherisApp`. Les groupes de focus
sont enregistrés dans la liste pour continuer à rendre un champ actif sorti de
l’écran, sans perdre sa saisie. `ScrollArea::for_list` utilise le déplacement et
les limites de la liste ; les dimensions sont stabilisées pendant un glisser.

Le profilage local en test GPUI sans serveur graphique (build debug sous Linux,
30 images de défilement, profil conseillé complet) passe de 13,83 s à 0,26 s,
soit environ 461 ms à 9 ms par image. Il mesure la construction et le rendu
GPUI de test, pas les performances GPU ou Windows. Le benchmark manuel reste
disponible pour comparer des modifications ultérieures :

```sh
XDG_DATA_HOME=/tmp/aetheris-profile-data \
XDG_CONFIG_HOME=/tmp/aetheris-profile-config \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/gcc \
cargo test --bin aetheris --features ui-tests --locked --offline \
  engine_frame_timing -- --ignored --nocapture
```

Le test ordinaire de la liste virtualisée vérifie le nombre limité d’éléments
rendus, la conservation d’un brouillon et du focus hors écran, et le déplacement
de la scrollbar jusqu’à la dernière ligne.

### Rendu partagé entre les pages

[`VirtualList`](../src/ui/components/virtual_list.rs) centralise les sections à
hauteur variable : état GPUI, groupes de focus, mesure initiale, marge de rendu
et scrollbar Penpot. Accueil, Engine.ini et la galerie utilisent ce composant.
La marge autour de la zone visible se règle dans
`Components.layout.virtualOverscan` (200 px par défaut).

Chaque page conserve une instance de `VirtualList` et ses entités de contrôle.
Son `element(id, renderer)` reçoit une fonction qui construit une section à
partir de son index. Cette fonction est appelée à la demande ; les sections
ne doivent pas être construites dans un `Vec<AnyElement>` avant l’appel.
`set_count` met à jour le nombre de sections en conservant les groupes de focus
du préfixe et `gap` définit leur espacement. Les hauteurs sont mesurées au
premier affichage et lorsque la largeur change pour que la scrollbar puisse
représenter toute la page. Pendant le défilement, seuls les éléments visibles,
la marge et un éventuel élément conservant le focus sont reconstruits.

`VirtualRows` centralise les listes de lignes à hauteur fixe : le propriétaire
conserve l’état de liste et le layout automatique gère la scrollbar. Le callback
reçoit seulement la plage de lignes à construire. Les tables virtualisées,
les résultats et détails Combat, les Select et les sélections multiples
l’utilisent. Les petits tableaux statiques conservent `Table::ordinary` pour
les cellules de hauteur libre ; les grands ensembles utilisent `virtualized`.

Les nouvelles pages composées de sections défilantes doivent utiliser
`VirtualList`, et les longues listes de lignes régulières `VirtualRows`, pour
reprendre ce comportement sans ajouter une implémentation propre à chaque page.
Les tests couvrent les brouillons et le focus hors écran, une table de 10 000
lignes, et le choix par clavier puis clic de la dernière option d’un Select
contenant 1 000 entrées.
