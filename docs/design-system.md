# Design System d’Aetheris

Statut : conception acceptée à l’issue du grilling, formalisée dans
l’[ADR 0005](adr/0005-centralized-gpui-design-system.md). Ce document rassemble le
périmètre initial et les contrats confirmés. L’implémentation, la migration
Engine.ini et leurs vérifications sont décrites dans le
[guide d’intégration](design-system-implementation.md).

## Périmètre et organisation

Le Design System centralise les éléments visuels et interactifs réutilisables de
l’application principale, construite avec Rust et GPUI. Les composants sont
composables et indépendants des règles métier. Les pages assurent leur intégration
avec le métier, la navigation, la persistance et les traitements asynchrones.

Le compagnon est hors périmètre pour l’instant. Son interface et ses dépendances
restent indépendantes de GPUI, conformément à l’ADR 0004.

- Composants partagés : `src/ui/components/`, avec une interface d’import commune.
- Thème et tokens : `src/ui/theme/`, dont `palette.rs` pour les couleurs.
- Overlays : module interne partagé pour leur infrastructure.
- Aucune bibliothèque séparée dans la première version.

La liste fonctionnelle initiale contient seize composants. Elle n’impose pas seize
primitives techniques : Icon, Label, Checkbox, Badge, Divider et conteneurs de
layout peuvent servir de fondations communes. Le panneau initialement nommé Menu
correspond à Popover ; Menu est réservé à une éventuelle liste de commandes.

## Thème, tokens et variantes

Le thème utilise des gris neutres sombres, des bordures discrètes et l’accent
turquoise commun `#65d1b2`. Les autres couleurs sont réservées aux usages
explicitement définis, notamment les sélections, les erreurs et le Timer.

La palette possède son propre fichier. Elle définit les couleurs disponibles ;
les tokens sémantiques leur attribuent des rôles : surface, bordure, texte, accent,
danger, etc. Les composants consomment ces rôles. Espacements, typographies,
rayons, épaisseurs de bordure et dimensions communes sont également centralisés.
Aucune couleur ou dimension propre au thème n’est dispersée arbitrairement dans
les implémentations.

La première version permet de remplacer la palette dans le code puis de
reconstruire l’application. Elle ne propose pas de sélection de thème à l’exécution.
Les valeurs exactes des gris, dimensions et combinaisons d’états seront validées
visuellement dans la galerie de développement.

Les contrôles utilisent des tailles nommées, par exemple `small`, `medium` et
`large`, avec `medium` par défaut. Les conteneurs conservent des contraintes
configurables de largeur et de hauteur. Les exceptions visuelles passent par des
variantes explicites plutôt que par des styles arbitraires sur chaque instance.

Les états communs sont `default`, `hover`, `focus`, `pressed`,
`active/selected` et `disabled`, selon le contrôle. Le focus reste visible
indépendamment de la sélection et de l’erreur. Un état ne doit pas être
compréhensible uniquement par sa couleur. `disabled` supprime les réactions
hover et pressed.

## Interface Rust et propriété de l’état

L’interface suit les conventions GPUI : constructeurs typés, propriétés configurées
par méthodes chaînées et enums pour les variantes. Chaque instance interactive
possède un identifiant stable. Les contrôles avec état transitoire utilisent des
entités GPUI créées une fois et conservées par la page, sans recréation à chaque rendu.

La page détient la valeur de référence. Les composants conservent les états
transitoires nécessaires : focus, brouillon, ouverture ou capture. La page peut
refuser une modification demandée.

### Événements et acceptation

| Callback | Contrat |
| --- | --- |
| `on_input` | Observe le brouillon saisi par l’utilisateur. |
| `on_change_requested` | Soumet une valeur candidate à l’acceptation synchrone de la page. |
| `on_value_change` | Signale une valeur utilisateur acceptée et différente de la précédente. |
| `on_click` | Déclenche une action. |
| `on_interaction_end` | Signale notamment la fin d’une interaction de Slider. |
| `on_expire` | Signale l’expiration d’un décompte de Timer. |
| `on_search` | Fournit une recherche à la validation de SearchBar. |

Ces noms et significations sont communs partout où ils s’appliquent ; les valeurs
transportées sont typées. `on_value_change` correspond au `valueChange` du
périmètre initial.

La page accepte ou refuse immédiatement une proposition. Après acceptation,
`on_value_change` n’est émis que si la valeur a changé. En cas de refus, un champ
conserve son brouillon avec une erreur ; un Switch ou un sélecteur garde sa valeur
précédente. Une validation sans changement n’émet pas de changement supplémentaire :
`Enter` suivi d’une perte de focus ne produit pas deux événements identiques.

Les effets asynchrones sont traités ensuite par la page. Une erreur d’enregistrement
est distincte d’une erreur de format ou de validation.

### Mises à jour programmatiques

Tous les contrôles peuvent être modifiés par programmation, notamment les inputs,
Switch, Slider, sélecteurs et raccourcis. Les mises à jour mettent l’affichage en
accord avec la valeur de référence sans émettre les événements de changement
utilisateur. Un événement de cycle de vie tel que l’expiration d’un Timer reste
soumis à son propre contrat.

Les contraintes s’appliquent aussi aux valeurs programmatiques. Une valeur hors
limites, un identifiant inconnu ou une configuration incohérente sont refusés et
signalés au code appelant, sans correction silencieuse ni modification de l’état
précédent. Configuration et valeur peuvent être mises à jour atomiquement, notamment
pour les bornes d’un Slider ou les options et la sélection d’un Select. L’ensemble
est accepté ou refusé sans état intermédiaire invalide.

Une nouvelle valeur de référence synchronise immédiatement un champ sans brouillon.
Avec un brouillon en cours, celui-ci est préservé et le conflit est signalé.
`Escape` restaure alors la nouvelle valeur de référence. Une nouvelle validation
peut soumettre le brouillon au métier.

Une opération explicite de réinitialisation remplace la valeur et abandonne le
brouillon, l’erreur et le conflit, sans événement utilisateur.

### Disponibilité et interruption

`disabled` empêche l’interaction utilisateur et retire le contrôle du parcours
`Tab`, tout en le laissant compréhensible pour les technologies d’assistance.
`read_only` permet le focus et la copie sans modification, pour les contrôles où
cela a un sens. Ces états n’interdisent pas les mises à jour programmatiques valides ;
une option désactivée peut ainsi être sélectionnée par le code.

Si un champ devient désactivé ou en lecture seule pendant la saisie, son brouillon
est préservé sans validation automatique. Une capture ou un glissement devenu
indisponible s’arrête sans nouvelle modification ; les valeurs déjà acceptées
pendant un glissement restent acquises.

## Focus, clavier et overlays

Toutes les interactions doivent être utilisables au clavier avec un focus visible.
Les boutons à icône seule disposent d’un libellé accessible.

### Infrastructure partagée

Popover, Select et Popup partagent superposition, positionnement et restitution du
focus, tout en gardant leurs politiques d’ouverture et de fermeture.

Les interactions imbriquées sont traitées de l’intérieur vers l’extérieur. Dans
une Popup contenant un Select ouvert, le premier `Escape` ferme le Select et un
second peut fermer la Popup. ShortcutSelector en capture consomme `Escape` pour
supprimer le raccourci. Un clic dans un panneau enfant est intérieur à son parent.

Un champ ne consomme `Escape` que lorsqu’il annule réellement un brouillon, une
erreur ou un conflit, ou si l’IME le traite. Sinon, le parent peut traiter l’événement.

Le focus revient au déclencheur seulement s’il existe encore et peut le recevoir.

### Règles des champs

TextInput et NumberInput distinguent brouillon et valeur validée. `Enter`,
la perte de focus et `Tab` déclenchent la validation. `Tab` poursuit le parcours,
même si la valeur est invalide. Une erreur conserve le brouillon et l’ancienne
valeur validée ; une valeur hors limites n’est pas corrigée silencieusement.

`Escape` restaure la dernière valeur de référence, efface erreur et conflit,
et conserve le focus. Pendant une composition IME, l’IME traite d’abord les
événements ; `Enter` confirmant la composition ne valide pas simultanément le champ.

### Règles des sélecteurs

Les options de SegmentSelector, MultiSelector et Select ont un identifiant stable,
distinct du libellé, et peuvent être désactivées individuellement. Des libellés
identiques sont autorisés ; un réordonnancement préserve les sélections.
Les déplacements clavier ignorent les options désactivées.

MultiSelector et Select partagent `min_selected` et `max_selected` :
le minimum vaut 0 par défaut, `0 ≤ min_selected ≤ max_selected` et
`max_selected ≥ 1`. En mode simple, le maximum vaut 1. Une désélection doit
respecter le minimum ; un minimum de 1 empêche de retirer la dernière sélection.

À la limite maximale, les options sélectionnées restent désélectionnables dans le
respect du minimum. Les autres restent visibles avec une indication expliquant
pourquoi elles ne peuvent plus être ajoutées. Si une option sélectionnée disparaît,
la page fournit ensemble les nouvelles options et une sélection compatible ;
le composant ne choisit pas arbitrairement un remplacement.

| Contrôle | Clavier |
| --- | --- |
| SegmentSelector | Un arrêt dans le parcours Tab ; les flèches changent directement la sélection. |
| MultiSelector à choix unique | Flèches pour le focus ; Espace sélectionne ou désélectionne selon le minimum. |
| MultiSelector multiple, Select multiple | Flèches pour le focus ; Espace bascule l’option, sans fermer le panneau du Select. |
| Select simple | Flèches pour le focus ; Enter sélectionne et ferme. |
| Sélecteurs | Home et End atteignent la première et la dernière option disponibles. |

Select et Popover se ferment par défaut au clic extérieur, avec `Escape` ou par un
nouveau clic sur le déclencheur. Les interactions internes ne ferment pas le panneau,
sauf la sélection dans Select simple. `Tab` poursuit le parcours et ferme Select
lorsqu’on en sort. Le clic extérieur est configurable pour Popover.

## Contrats des composants

### 1. NavigationButton

Bouton exclusivement destiné à la sidebar, avec libellé et icône optionnelle.
États : default, hover, focus, pressed, active et disabled.

Toute navigation passe par le mécanisme de navigation de l’application. `active`
suit la page effectivement affichée après acceptation de la navigation.

### 2. Button

Bouton d’action avec variantes `primary` et `secondary`, contenant du texte,
une icône ou les deux. États : default, hover, focus, pressed et disabled.
Il peut exécuter un callback, ouvrir une Popup ou demander une navigation.
Une navigation utilise le même mécanisme applicatif que NavigationButton.

### 3. Card

Conteneur à titre et icône optionnelle, avec un corps entièrement composable.
Une Card statique n’est pas focalisable par elle-même. Une Card interactive possède
les états default, hover, focus, pressed et disabled. Une flèche à droite indique
que son action principale est une navigation.

L’action principale possède une cible distincte des contrôles internes :
le fond ou le titre peut la déclencher ; un Switch interne ne déclenche que lui-même.
Au clavier, l’action principale et les contrôles internes ont des arrêts de focus
distincts.

`disabled` concerne seulement l’action principale. Les enfants gardent leur propre
disponibilité ; désactiver l’ensemble relève d’une configuration explicite des enfants.

### 4. Slider

Libellé descriptif, barre horizontale avec partie remplie accent et partie vide grisée,
poignée circulaire déplaçable et valeur actuelle affichée sur le côté.
`min`, `max`, `value`, `steps` et `snap` sont configurables ;
`snap` vaut true par défaut.

`steps` est une liste explicite, éventuellement irrégulière, strictement croissante
et contenant les deux bornes. Avec snap, seules ces valeurs sont proposées.
Le geste choisit le step numériquement le plus proche ; une égalité est départagée
vers le haut. Une valeur programmée hors steps reste refusée avec snap.
Sans snap, les steps sont des repères et les valeurs intermédiaires sont permises.
Sans steps, la sélection reste continue même si snap vaut true.

Les changements sont proposés pendant le déplacement, sans répétition inutile
d’une même valeur. `on_interaction_end` permet à la page de différer un effet coûteux.
Avec snap, les flèches passent au step précédent ou suivant ; sans snap, un incrément
clavier configurable vaut 1 par défaut, indépendamment des repères visuels.
Home et End atteignent les bornes. Les déplacements restent dans l’intervalle.

### 5. SegmentSelector

Titre optionnel ; au moins deux segments, sans limite fixe. Une seule rangée dans
un conteneur grisé, segments de largeur égale indépendamment du texte, fond accent
sur la sélection. Exactement une option est sélectionnée.

Une largeur minimale commune est appliquée ; le conteneur défile horizontalement
si nécessaire. La navigation clavier rend le segment focalisé visible.

### 6. Switch

Le Switch standard est horizontal et entièrement arrondi. Actif : fond accent,
poignée blanche à droite. Inactif : fond gris, poignée à gauche. Un libellé peut
être placé à gauche. Un clic inverse la valeur selon le protocole d’acceptation.

La variante Card contient une icône et du texte, possède son propre fond et une
bordure grise inactive ou accent active. Les Cards sont espacées, sans conteneur
visuel de sélection partagé. Toute la surface et Espace activent ce contrôle booléen.
Les Cards restent indépendantes ; plusieurs peuvent être actives.
Valeur et événements sont identiques au Switch standard. Une Card contenant
plusieurs contrôles relève du composant Card général.

### 7. ShortcutSelector

Déclencheur rectangulaire arrondi sur fond gris très foncé, combinaison en blanc
et gras, libellé optionnel à gauche. Sans raccourci : texte « Non défini » et bordure
blanche. Une icône X supprime le raccourci. Un état visuel distingue la capture.

Clic, Enter ou Espace démarrent la capture. Une seule capture est active à la fois,
locale aux événements reçus par la fenêtre Aetheris. Les raccourcis ordinaires de
l’application sont suspendus pendant la capture. L’événement d’ouverture est ignoré.

Une combinaison comporte une touche principale et au moins un modificateur,
sauf F1 à F12 qui peuvent être utilisées seules. Modificateurs seuls et répétitions
sont ignorés. Escape ne peut être enregistré ; pendant la capture, il supprime
le raccourci. Un clic extérieur ou une perte de focus annule la capture et conserve
l’ancienne valeur.

La page peut refuser une combinaison, vérifie les conflits et gère l’enregistrement
effectif des raccourcis. Le composant n’installe ni n’active de raccourci global.

### 8. TextInput

Champ de texte libre avec titre optionnel, focus et désactivation cohérents.
La chaîne vide est une valeur valide. Il applique les règles communes de brouillon,
validation, annulation, IME et synchronisation programmée.

### 9. NumberInput

Présentation et validation de TextInput, avec bornes configurables et prise en
charge configurable des entiers, décimaux et valeurs négatives.
La première version utilise un format décimal simple, sans notation scientifique
ni séparateurs de milliers. Virgule et point sont acceptés comme séparateurs
décimaux si les décimaux sont autorisés ; l’affichage est normalisé selon la langue.

Les états intermédiaires comme un signe seul ou `1,` sont permis pendant la saisie.
À la validation, `1,` et `1.` peuvent être normalisés en `1` ; un signe seul
reste invalide. Un collage incompatible, tel que `12abc`, est refusé intégralement,
sans extraction silencieuse de chiffres. Une composition IME peut se terminer
avant vérification.

`allow_empty` vaut false par défaut. S’il est activé, la valeur validée est
optionnelle et une absence de valeur est distincte de zéro. Effacer le champ reste
toujours possible pendant la saisie, même si sa validation à vide est interdite.

### 10. MultiSelector

Options verticales avec espacement régulier, coins légèrement arrondis, libellé
et indicateur circulaire à gauche. Sélection : fond bleu-gris foncé, indicateur
bleu avec coche. Hors sélection : fond transparent et cercle gris vide.

Un clic sélectionne ou désélectionne dans les limites configurées. Un maximum de 1
limite à un choix, sans imposer une sélection non vide : cette obligation dépend
du minimum. Un maximum supérieur à 1 autorise plusieurs choix. Les contraintes
communes de sélection et d’identification des options s’appliquent.

### 11. Timer

Affichage horizontal compact, sans fond propre. Nombres jaune clair en gras,
unités jaune plus foncé et plus petites. À expiration, le texte configurable
remplace le décompte et une pastille verte apparaît à gauche.

La source est une durée initiale ou une date d’échéance. La durée représente un
temps écoulé : une correction de l’horloge système ne la modifie pas. Une échéance
calendaire suit l’horloge système. Le temps passé en veille est compté dans les
deux cas. Le mécanisme d’horloge doit être vérifié sur les plateformes visées.

Une durée établit une échéance de temps écoulé une seule fois au démarrage.
Un rendu ne redémarre pas le décompte ; une nouvelle durée ou échéance explicite
ouvre un nouveau décompte. Le rafraîchissement intervient chaque seconde, mais le
temps restant est calculé depuis l’échéance, sans soustractions successives.
Après un retard ou une suspension, l’affichage rejoint le temps restant réel.

Le format dépend de la plus grande unité restante :

| Plus grande unité | Format |
| --- | --- |
| Jour | d h m |
| Heure | h m s |
| Minute | m s |
| Seconde | s |

Les unités intermédiaires à zéro restent affichées : `1d 0h 5m`, `1h 0m 5s`,
`2m 0s`. Le maximum d’unités, trois par défaut, conserve les premières :
deux unités donnent `1d 0h` ou `1h 0m`. Les secondes restantes sont arrondies au
supérieur ; l’expiration intervient à l’échéance réelle, arrête le décompte et
peut émettre `on_expire` au plus une fois par décompte.

Le callback est un événement d’interface, émis seulement pendant la présence du
composant. Une action métier survivant à un changement de page relève de
l’application. Un Timer apparaissant avec une échéance passée affiche immédiatement
l’expiration et émet l’événement une fois pour ce décompte, sans le répéter s’il
avait déjà été émis. L’identité et l’état du décompte doivent être conservés en conséquence.

### 12. Select

Déclencheur rectangulaire arrondi sur fond gris très foncé, bordure grise/accent,
valeur ou placeholder et chevron d’ouverture. Panneau sombre, bordure grise,
coins arrondis et options verticales. Le survol ou la sélection utilise un fond
bleu-gris foncé.

Le mode simple sélectionne une option et ferme le panneau. Le mode multiple utilise
des cases à cocher, un maximum configurable, un résumé dans le déclencheur et un
badge de nombre optionnel. Options, valeur initiale et placeholder sont configurables.
Dans l’interface contrôlée, la valeur initiale est fournie par la page comme valeur
de référence ; elle n’introduit pas une seconde source de vérité.

Hauteur maximale configurable et défilement vertical. Les règles communes des
sélecteurs, du clavier et des overlays s’appliquent.

### 13. Table

Fond gris très foncé, bordure discrète, coins arrondis, en-tête en petit texte gris
clair, séparateurs horizontaux fins et hauteur de ligne cohérente.
Espacements, bordures et états visuels sont personnalisables par les tokens et variantes.

La page possède lignes, données, chargement, pagination et tri. La Table fournit
la présentation, la composition des cellules et les demandes de tri optionnelles
par colonne. Ajouts, suppressions et modifications sont fournis par la page.
Lignes et colonnes ont des identifiants stables. Les états indépendants des contrôles
restent associés aux lignes après tri, insertion ou sortie de la zone visible.

Les cellules sont entièrement composables : textes, titres, descriptions, icônes,
Timer, Select, Switch, Button, Badge et combinaisons. Les éléments peuvent être
masqués ou désactivés indépendamment.

Colonnes fixes ou flexibles avec minimum et alignement gauche, centre ou droite.
Les colonnes masquées sont retirées de l’affichage et du parcours clavier.
Le corps défile horizontalement plutôt que d’écraser les minima, et verticalement
avec une hauteur maximale. L’en-tête fixe est optionnel et garde l’alignement.

Deux modes explicites : ordinaire, qui permet les hauteurs variables, et virtualisé,
à hauteur uniforme configurable. La page conserve les états des contrôles virtualisés.

Des emplacements composables présentent chargement, absence de données et erreur ;
les lignes précédentes peuvent rester visibles pendant un rechargement.
La page fournit état et contenu ; aucun chargement ni retry n’est déclenché par Table.

### 14. Popover, nommé Menu dans le périmètre initial

Panneau flottant entièrement composable. Déclencheur sombre à bordure discrète
et coins arrondis, texte blanc, icône optionnelle à gauche et chevron optionnel à droite.

Panneau sombre à bordure grise, coins arrondis et dimensions configurables.
Placement préféré en haut, bas, gauche ou droite ; défaut sous le déclencheur,
aligné à gauche. Basculement automatique si l’espace manque, dimensions limitées
à la fenêtre et défilement vertical si nécessaire. La disparition du déclencheur
ferme le panneau.

Contenu libre : textes, descriptions, Button, Switch, Slider, Select, SegmentSelector,
Checkbox, ShortcutSelector, Divider et conteneurs horizontaux/verticaux.
Un en-tête avec titre et icône est possible. La navigation interne utilise Tab ;
ce panneau ne suit pas les conventions d’une liste de commandes.

### 15. Popup

Fenêtre superposée ouverte par une action, avec assombrissement de l’interface
derrière elle, contenu entièrement composable et dimensions configurables.

La variante par défaut est modale : interactions arrière bloquées, parcours clavier
maintenu à l’intérieur, Escape demande la fermeture et le focus est restitué au
déclencheur disponible. Le clic sur le fond ne ferme pas par défaut.
Les règles sont configurables ; une variante non modale est explicite.

La page peut refuser une demande de fermeture. Son origine est précisée : Escape,
clic extérieur, bouton de fermeture ou appel programmatique ordinaire.
Le Design System n’enregistre ni n’abandonne automatiquement les brouillons ;
la page décide. La destruction de la page est une opération distincte.

### 16. SearchBar

Base visuelle et états de TextInput, avec une loupe à gauche dans le champ,
zone de texte à droite et placeholder configurable. `on_search` fournit la recherche.

La recherche est déclenchée à la validation (Enter, Tab ou perte de focus).
Aucun délai après la saisie ne déclenche de callback. Les validations répétées
d’une même recherche sont dédupliquées. Une mise à jour programmée ne lance
pas de recherche.

## Persistance et migration

Brouillon, valeur validée et valeur enregistrée sont distincts. Si une écriture
échoue après acceptation, la page conserve la nouvelle valeur avec un état non
enregistré et une erreur, et propose une nouvelle tentative. Elle décide des
restrictions de navigation ou de fermeture. Aucun composant ne gère la persistance.

Dans l’éditeur Engine.ini, l’attente de deux secondes ne valide plus une saisie et
ne suffit plus à l’enregistrer. Enter, Tab et perte de focus déclenchent la validation ;
l’enregistrement automatique peut suivre une valeur validée.
Les politiques métier restent indépendantes du Design System.

L’adoption est progressive : une galerie de développement présente tous les
composants, variantes et états ; l’éditeur Engine.ini est la première page
entièrement convertie pour éprouver l’intégration avant généralisation.
Les anciens éléments peuvent coexister temporairement pendant la migration.

## Vérification et critères de validation

La galerie permet de valider visuellement les valeurs des tokens et les combinaisons
d’états. La page pilote valide l’usage réel, sans dépendance métier dans les composants.

Les vérifications doivent couvrir :

- clavier, focus visible, libellés accessibles et états non fondés uniquement sur la couleur ;
- saisie, IME, Tab/Enter/blur, annulation, erreurs et absence d’événements en double ;
- mises à jour programmatiques, réinitialisation, conflits et configurations atomiques ;
- acceptation/refus et distinction validation/enregistrement ;
- limites de sélection, changements d’options, tri et états des contrôles virtualisés ;
- overlays imbriqués, Escape consommé une seule fois, fermeture refusée et retour du focus ;
- capture des raccourcis sans déclenchement des actions ordinaires de l’application ;
- Timer sans dérive, veille, horloges distinctes et expiration unique ;
- absence de recherche temporisée, validation et déduplication après Enter.

Les sources GPUI locales à la révision verrouillée
`1f32ec0ceafddd6a368db8aef9d169a6569a1c23` exposent des API AccessKit de rôles,
libellés et états, ainsi qu’une intégration Windows. La traversée du focus et
les politiques d’overlay nécessitent une intégration explicite dans Aetheris.
Ces API ne prouvent pas l’utilisabilité réelle avec un lecteur d’écran :
elle reste à vérifier avant de fixer les engagements correspondants.

Les décisions architecturales sont formalisées dans
l’[ADR 0005](adr/0005-centralized-gpui-design-system.md). Les valeurs visuelles
exactes et les noms de types et signatures Rust détaillés seront concrétisés lors
de l’implémentation dans le cadre des contrats ci-dessus.
