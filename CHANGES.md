# Notes de version stables d’Aetheris

## PENDING

### Add

- Overlay Editor : widget de présence des cubes cachés détectés par le serveur,
  avec durée de détection continue et son facultatif. Désactivé par défaut ;
  activation séparée du guidage (distance, hauteur et repère du monde), avec
  exclusion des captures validée sur Windows 26200.9711 avec OBS 33.0.1 et le jeu
  sans bordure. Le guidage reste masqué sur les autres configurations.
  Un test avec cube fictif permet de vérifier l’affichage pendant 60 secondes.

- Historique expérimental des combats dans **Enregistrement → Historique** :
  analyse d’une capture importée, choix de la cible et du joueur local, puis
  enregistrement d’un bilan consultable même après suppression de la capture.
  Les dégâts, le DPS et les indicateurs par compétence sont regroupés dans un
  tableau unique, avec les indicateurs de coups affichés en pourcentage.

- Service Windows de capture réseau, installé et configuré par l’assistant,
  permettant de capturer le trafic du jeu sans lancer Aetheris en administrateur.

- Installeur Windows avec raccourcis au choix et mises à jour sans nouvelle
  demande administrateur. Une mise à jour attend la fermeture d’Aetheris dans
  toutes les sessions Windows. Le ZIP reste disponible pour l’installation manuelle.
- Pour une installation via l’assistant, référence Engine.ini propre à chaque
  utilisateur, avec import facultatif depuis un ancien dossier ZIP. La
  désinstallation conserve les données personnelles et les réglages du jeu.

- Fenêtre de progression pendant l’installation des mises à jour, avec les
  étapes en cours et un redémarrage automatique. Tenter de relancer Aetheris
  pendant l’installation affiche cette fenêtre sans ouvrir une seconde instance.

- Première version d’Aetheris pour Windows, avec un éditeur des réglages
  d’Aion 2 accessible depuis la navigation latérale.
- Détection automatique du fichier `Engine.ini`, avec sélection manuelle si
  nécessaire. Les réglages sont regroupés en 13 catégories et accompagnés
  d’explications, de leur valeur actuelle et de conseils de configuration.
- Application des réglages conseillés ou retour à l’état d’origine, par
  catégorie ou pour l’ensemble du fichier. Les commentaires et les réglages
  personnels non concernés sont conservés.
- Enregistrement automatique des réglages, avec vérification des valeurs
  saisies. Une saisie invalide reste disponible pour correction et empêche la
  fermeture de l’application tant qu’elle n’est pas corrigée.
- Conseils tenant compte de la carte graphique et de l’écran du jeu. La carte
  graphique peut être corrigée manuellement ; la résolution conseillée n’est
  appliquée que sur demande.
- Affichage de l’état d’Aion 2 dans la barre de titre. Les réglages restent
  consultables mais leur modification est bloquée pendant que le jeu tourne
  ou lorsque son état ne peut pas être déterminé. Après enregistrement,
  `Engine.ini` est protégé en lecture seule.
- Compagnon de lancement proposant un diagnostic en cas de crash ou d’échec
  au démarrage, avec accès au journal local et possibilité de relancer Aetheris.
  Un nouveau lancement active la fenêtre déjà ouverte.
- Mises à jour avec choix entre les canaux stable et préversion depuis le
  numéro de version affiché dans la barre latérale. Les fichiers téléchargés
  sont vérifiés avant installation et les données personnelles sont préservées.
- Réparation des fichiers de l’application endommagés et possibilité de revenir
  à la version précédente si une installation échoue ou si l’application
  plante au premier lancement après une mise à jour.

### Modify

- Rendu du contenu visible partagé entre les pages Accueil, Engine.ini et
  Design System, ainsi que les tables, les listes Combat et les options des Select.

- Défilement d’Engine.ini accéléré : seules les catégories visibles sont rendues,
  en conservant les brouillons et le focus des champs hors écran.

- Suppression du déclenchement différé après saisie dans les champs de recherche ;
  la recherche attend la validation du champ.

- Les listes Select se ferment au clic extérieur, y compris juste à côté du champ.

- Barres de défilement au style Penpot affichées automatiquement quand le
  contenu dépasse son conteneur, dans les pages, les listes, les tableaux et
  les panneaux, avec déplacement à la souris.

- Les composants du design system reprennent les styles Penpot : couleurs,
  dimensions, focus, survol et police Montserrat intégrée. Leur apparence est
  centralisée dans un fichier de tokens JSON modifiable.

- Interface Engine.ini harmonisée : contrôles partagés, thème sombre neutre et
  focus visible au clavier. Les champs se valident avec Enter, Tab ou en quittant
  le champ ; attendre deux secondes ne suffit plus à appliquer une saisie.
- En cas d’échec de sauvegarde, les valeurs validées restent affichées et une
  action permet de réessayer. Escape annule le brouillon d’un champ.

### Fixes

- Guidage des cubes : prise en compte des positions des effets appliqués par
  le personnage sur lui-même, pour le format réseau vérifié.

- Cubes cachés : les rapports répétés de la même zone ne suppriment plus les
  cubes détectés, leur durée ni l’identité du personnage.

- Guidage des cubes : prise en compte de la position au chargement de zone et
  des corrections réseau du personnage. La confirmation d’identité conserve
  les données du nouveau chargement. Le panneau privé distingue désormais
  une identité inconnue, une position absente et une position périmée.

- Cubes cachés : correction d’un faux « Flux réseau incomplet » lorsque le
  réassemblage TCP progresse à travers des trous successifs. Le diagnostic
  distingue maintenant les octets TCP manquants d’une trame serveur inachevée.

- Cubes cachés : remise à zéro du suivi au changement de personnage, y compris
  sans guidage ou lorsque l’identité locale est reçue pour la première fois
  après des annonces de cubes. Les anciens compteurs ne sont plus conservés.

- Guidage des cubes : correction du refus du style natif `14cf0000` relevé
  dans la configuration sans bordure validée sous Windows 26200.9711.
  Les vérifications OBS, DWM et d’exclusion des captures restent appliquées.

- Guidage des cubes : ajout des versions Windows/OBS et du résultat de
  qualification au panneau privé du test fictif, pour diagnostiquer un guidage
  réel absent malgré un test visible.

- Cubes cachés : les connexions TCP non reconnues comme flux du jeu ne
  réinitialisent plus le suivi lors d’un démarrage, d’un paquet incomplet ou
  d’une erreur de réassemblage.

- Cubes cachés : création des fenêtres au moment de leur affichage en jeu,
  avec le même chemin natif que le test. Le widget réel se place en haut à
  gauche et le test à côté. Le diagnostic indique la position native, la
  visibilité Windows et le masquage DWM.

- Cubes cachés : rafraîchissement explicite de la fenêtre réelle après son
  affichage, avec une copie stable des données pour la peinture native. Le
  diagnostic indique désormais le nombre de peintures effectuées.

- Overlay Editor : conservation du dernier résultat d’affichage en jeu lors du
  retour dans Aetheris et du dernier événement de suivi pour diagnostiquer un
  widget absent ou une annonce de cube rejetée.
- Cubes cachés : le message `21 36`, sans changement de zone établi, n’efface
  plus les cubes détectés.

- Reconnaissance de la fenêtre du jeu pour l’overlay sans ouvrir son processus,
  avec un état de réception et un diagnostic d’affichage dans l’éditeur.

- Capture réseau : maintien de la réception pendant une analyse longue pour
  éviter une coupure de la connexion au service. Les erreurs de cette liaison
  locale sont désormais identifiées dans le diagnostic.

- Capture réseau : détection du jeu lancé avec un autre compte administrateur
  dans la même session Windows, sans devoir élever Aetheris.

- Réparation et mise à jour Windows : attente de la fermeture effective des
  programmes avant leur remplacement, pour éviter les erreurs d’accès refusé
  sur les exécutables. Une relance pendant l’installation ne garde plus le
  compagnon bloqué en arrière-plan ; les processus encore ouverts sont indiqués.

- Correction de l’installation du service de capture lors d’une mise à jour
  automatique depuis une ancienne version ; son absence ou un échec de démarrage
  produit désormais un message précis.
