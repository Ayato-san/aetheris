# Notes de version bêta d’Aetheris

## PENDING

### Add

### Modify

### Fixes

- Réparation et mise à jour Windows : attente de la fermeture effective des
  programmes avant leur remplacement, pour éviter les erreurs d’accès refusé
  sur les exécutables. Une relance pendant l’installation ne garde plus le
  compagnon bloqué en arrière-plan ; les processus encore ouverts sont indiqués.

## 0.1.0-beta.6 — 2026-11-10T20:45:30Z

### Add

### Modify

### Fixes

- Correction de l’installation du service de capture lors d’une mise à jour
  automatique depuis une ancienne version ; son absence ou un échec de démarrage
  produit désormais un message précis.

## 0.1.0-beta.5 — 2026-11-10T20:01:19Z

### Add

- Service Windows de capture réseau, installé et configuré par l’assistant,
  permettant de capturer le trafic du jeu sans lancer Aetheris en administrateur.

### Modify

### Fixes

## 0.1.0-beta.4 — 2026-11-10T17:40:24Z

### Add

### Modify

### Fixes

- Correction du crash DirectX au démarrage des builds Windows debug
  installées sur une autre machine que celle de compilation.

## 0.1.0-beta.3 — 2026-11-10T16:05:03Z

### Add

- Installeur Windows avec raccourcis au choix et mises à jour sans nouvelle
  demande administrateur. Une mise à jour attend la fermeture d’Aetheris dans
  toutes les sessions Windows. Le ZIP reste disponible pour l’installation manuelle.
- Pour une installation via l’assistant, référence Engine.ini propre à chaque
  utilisateur, avec import facultatif depuis un ancien dossier ZIP. La
  désinstallation conserve les données personnelles et les réglages du jeu.

- Fenêtre de progression pendant l’installation des mises à jour, avec les
  étapes en cours et un redémarrage automatique. Tenter de relancer Aetheris
  pendant l’installation affiche cette fenêtre sans ouvrir une seconde instance.

### Modify

### Fixes

## 0.1.0-beta.2 — 2026-11-06T17:52:10Z

### Add

- Dans la page « Combat · Debug », réservée aux builds de développement,
  analyse des messages compressés avec accès à leur contenu, aux enveloppes
  qui les contiennent et aux paquets d’origine.
- Reconnaissance de 47 types de messages serveur dans cette page, avec leur
  nom et une recherche par code ou par nom. Le détail indique la source de
  l’identification et permet toujours de consulter les données non interprétées.
- Analyse automatique des captures commencées après la connexion au jeu,
  lorsque suffisamment de messages serveur permettent de retrouver leur
  découpage. Cette reconnaissance fonctionne pour les fichiers importés et
  la capture en direct ; les portions non reconnues restent consultables.

### Modify

- Rendu du contenu visible partagé entre les pages Accueil, Engine.ini et
  Design System, ainsi que les tables, les listes Combat et les options des Select.

- Défilement d’Engine.ini accéléré : seules les catégories visibles sont rendues,
  en conservant les brouillons et le focus des champs hors écran.

- Suppression du déclenchement différé après saisie dans les champs de recherche ;
  la recherche attend la validation du champ.

- Les listes Select se ferment au clic extérieur, y compris juste à côté du champ.

- Barres de défilement visibles au style Penpot dans les longues pages,
  les listes de sélection et les panneaux, avec déplacement à la souris.

- Les composants du design system reprennent les styles Penpot : couleurs,
  dimensions, focus, survol et police Montserrat intégrée. Leur apparence est
  centralisée dans un fichier de tokens JSON modifiable.

- Interface Engine.ini harmonisée : contrôles partagés, thème sombre neutre et
  focus visible au clavier. Les champs se valident avec Enter, Tab ou en quittant
  le champ ; attendre deux secondes ne suffit plus à appliquer une saisie.
- En cas d’échec de sauvegarde, les valeurs validées restent affichées et une
  action permet de réessayer. Escape annule le brouillon d’un champ.

- La page « Combat · Debug » distingue désormais les messages identifiés
  de ceux dont le contenu est effectivement décodé. Les anciennes estimations
  non confirmées d’entité, de points de vie et de dégâts, ainsi que le filtre
  par entité, ont été retirés. Aucun DPS ni dégât reçu confirmé n’est calculé.
- Les sessions utilisant une ancienne analyse invitent à cliquer sur
  « Réanalyser » pour bénéficier des améliorations, en conservant les paquets
  capturés d’origine.

### Fixes

## 0.1.0-beta.1 — 2026-11-02T13:53:25Z

### Add

- Première bêta d’Aetheris pour Windows, avec une page d’accueil et un accès
  à l’éditeur `Engine.ini` depuis la navigation latérale.
- Détection automatique du fichier `Engine.ini` dans les profils Windows
  accessibles, avec possibilité de le sélectionner manuellement s’il n’est
  pas trouvé.
- Éditeur regroupant les réglages en 13 catégories : DLSS/FSR, entrées,
  fluidité, affichage, ombres, visibilité des personnages, qualité d’image,
  distance d’affichage, latence, post-traitement, gestion de la mémoire,
  textures et shaders. Chaque réglage présente une explication, sa valeur
  actuelle et sa valeur conseillée.
- Application des réglages conseillés et retour à l’état d’origine, pour une
  catégorie ou pour l’ensemble des catégories. Les commentaires, les chemins
  du jeu et les réglages non concernés sont conservés.
- Enregistrement immédiat des choix et interrupteurs. Les valeurs numériques
  sont enregistrées après deux secondes sans saisie ; les valeurs liées sont
  vérifiées ensemble. Une saisie invalide est conservée pour correction et
  bloque la fermeture de la fenêtre.
- Saisie des délais du clavier en millisecondes, avec virgule ou point décimal.
  Les valeurs absentes ou non reconnues ne sont remplacées que sur choix
  explicite.
- Détection de la carte graphique et suggestion de résolution selon l’écran
  du jeu, avec repli sur l’écran principal. La carte graphique peut être
  corrigée manuellement et ce choix est mémorisé. La résolution conseillée
  n’est appliquée que sur demande.
- Affichage de l’état d’Aion 2 dans la barre de titre. L’édition et les
  sauvegardes sont bloquées pendant que le jeu tourne ou lorsque son état
  est inconnu ; le fichier reste consultable.
- Protection d’`Engine.ini` en lecture seule après chaque sauvegarde, avec
  conservation de son encodage et de ses fins de ligne.
- Icône Aetheris dans la fenêtre et dans Windows, avec commandes de réduction,
  d’agrandissement, de restauration et de fermeture dans la barre de titre.
  L’application Windows s’ouvre sans fenêtre de console.
- Compagnon de lancement qui reste disponible en cas de crash ou d’échec au
  démarrage. Il permet d’ouvrir le journal local, de relancer Aetheris ou de
  fermer le diagnostic. Les dix derniers journaux sont conservés localement.
  Un nouveau lancement active la fenêtre déjà ouverte.
- Vérification des mises à jour au lancement puis chaque heure, avec choix
  du canal stable ou préversion depuis le numéro de version en bas de la
  barre latérale. Revenir au canal stable ne réinstalle pas une version plus
  ancienne. Les mises à jour vérifient l’authenticité des fichiers et
  préservent les réglages, les journaux et les autres données personnelles.
- Vérification des fichiers de l’application au démarrage, y compris hors
  ligne, et proposition de réparation s’ils sont endommagés. Une mise à jour
  interrompue peut être restaurée ; un crash au premier lancement après
  mise à jour propose de revenir à la version précédente.

### Modify

### Fixes
