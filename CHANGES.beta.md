# Notes de version bêta d’Aetheris

## PENDING

### Add

### Modify

### Fixes

## 0.1.0-beta.19 — 2026-11-14T13:25:15Z

### Add

### Modify

### Fixes

- Guidage des cubes : prise en compte de la position au chargement de zone et
  des corrections réseau du personnage. La confirmation d’identité conserve
  les données du nouveau chargement. Le panneau privé distingue désormais
  une identité inconnue, une position absente et une position périmée.

## 0.1.0-beta.18 — 2026-11-14T12:47:05Z

### Add

### Modify

### Fixes

- Cubes cachés : correction d’un faux « Flux réseau incomplet » lorsque le
  réassemblage TCP progresse à travers des trous successifs. Le diagnostic
  distingue maintenant les octets TCP manquants d’une trame serveur inachevée.

- Cubes cachés : remise à zéro du suivi au changement de personnage, y compris
  sans guidage ou lorsque l’identité locale est reçue pour la première fois
  après des annonces de cubes. Les anciens compteurs ne sont plus conservés.

## 0.1.0-beta.17 — 2026-11-14T11:50:28Z

### Add

### Modify

### Fixes

- Guidage des cubes : correction du refus du style natif `14cf0000` relevé
  dans la configuration sans bordure validée sous Windows 26200.9711.
  Les vérifications OBS, DWM et d’exclusion des captures restent appliquées.

## 0.1.0-beta.16 — 2026-11-14T11:33:27Z

### Add

### Modify

### Fixes

- Guidage des cubes : ajout des versions Windows/OBS et du résultat de
  qualification au panneau privé du test fictif, pour diagnostiquer un guidage
  réel absent malgré un test visible.

- Cubes cachés : les connexions TCP non reconnues comme flux du jeu ne
  réinitialisent plus le suivi lors d’un démarrage, d’un paquet incomplet ou
  d’une erreur de réassemblage.

## 0.1.0-beta.15 — 2026-11-14T11:01:36Z

### Add

### Modify

### Fixes

- Cubes cachés : création des fenêtres au moment de leur affichage en jeu,
  avec le même chemin natif que le test. Le widget réel se place en haut à
  gauche et le test à côté. Le diagnostic indique la position native, la
  visibilité Windows et le masquage DWM.

## 0.1.0-beta.14 — 2026-11-14T09:40:37Z

### Add

### Modify

### Fixes

- Cubes cachés : rafraîchissement explicite de la fenêtre réelle après son
  affichage, avec une copie stable des données pour la peinture native. Le
  diagnostic indique désormais le nombre de peintures effectuées.

## 0.1.0-beta.13 — 2026-11-13T22:45:25Z

Cette version ne comporte aucun changement visible pour le joueur.

### Add

### Modify

### Fixes

## 0.1.0-beta.12 — 2026-11-13T22:03:35Z

### Add

### Modify

### Fixes

- Overlay Editor : conservation du dernier résultat d’affichage en jeu lors du
  retour dans Aetheris et du dernier événement de suivi pour diagnostiquer un
  widget absent ou une annonce de cube rejetée.
- Cubes cachés : le message `21 36`, sans changement de zone établi, n’efface
  plus les cubes détectés.

## 0.1.0-beta.11 — 2026-11-13T21:36:27Z

Cette version ne comporte aucun changement visible pour le joueur.

### Add

### Modify

### Fixes

## 0.1.0-beta.10 — 2026-11-13T18:40:01Z

### Add

- Overlay Editor : widget de présence des cubes cachés détectés par le serveur,
  avec durée de détection continue et son facultatif. Désactivé par défaut ;
  activation séparée du guidage (distance, hauteur et repère du monde), avec
  exclusion des captures validée sur Windows 26200.9711 avec OBS 33.0.1 et le jeu
  sans bordure. Le guidage reste masqué sur les autres configurations.
  Un test avec cube fictif permet de vérifier l’affichage pendant 60 secondes.

### Modify

### Fixes

- Reconnaissance de la fenêtre du jeu pour l’overlay sans ouvrir son processus,
  avec un état de réception et un diagnostic d’affichage dans l’éditeur.

## 0.1.0-beta.9 — 2026-11-12T16:01:29Z

### Add

- Historique expérimental des combats dans **Enregistrement → Historique** :
  analyse d’une capture importée, choix de la cible et du joueur local, puis
  enregistrement d’un bilan consultable même après suppression de la capture.
  Les dégâts, le DPS et les indicateurs par compétence sont regroupés dans un
  tableau unique, avec les indicateurs de coups affichés en pourcentage.

### Modify

- Affichage automatique des barres de défilement quand le contenu dépasse
  son conteneur, partagé entre les pages, les listes, les tableaux et les panneaux.

### Fixes

- Suppression des captures importées accélérée, sans affecter les bilans enregistrés.

## 0.1.0-beta.8 — 2026-11-11T12:00:27Z

### Add

### Modify

### Fixes

- Capture réseau : maintien de la réception pendant une analyse longue pour
  éviter une coupure de la connexion au service. Les erreurs de cette liaison
  locale sont désormais identifiées dans le diagnostic.

## 0.1.0-beta.7 — 2026-11-11T11:25:16Z

### Add

### Modify

### Fixes

- Capture réseau : détection du jeu lancé avec un autre compte administrateur
  dans la même session Windows, sans devoir élever Aetheris.

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
