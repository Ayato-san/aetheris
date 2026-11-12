# Premier jalon de l’overlay

La page Overlay Editor reste disponible en release. Elle conserve l’activation
de la boussole dans le dossier de données utilisateur, indépendamment de la
disponibilité du service. Une erreur d’écriture restaure le choix précédent et
s’affiche dans la page. Réessayer recrée la connexion sans modifier ce choix.

La fenêtre Windows est séparée de GPUI : layered, transparente aux clics, sans
activation et absente de la barre des tâches. Toutes ses ressources Win32/GDI
appartiennent à son thread. Elle suit les coordonnées client de la fenêtre
au premier plan, après vérification de son exécutable avec les noms déjà reconnus
par Aetheris. Une fenêtre réduite, non identifiable ou inaccessible masque
la bande. La vérification s’effectue toutes les 50 ms, indépendamment des appels
au service réseau. Le plein écran exclusif reste hors périmètre.

Le lecteur IPC partagé distribue des lots immuables avec des files bornées à
huit lots par abonné. Combat Debug conserve sa sauvegarde explicite ; l’overlay
consulte les lots en mémoire sans copier les corps ni écrire de capture.
Un abonné trop lent perd son abonnement avec une erreur explicite, sans
interrompre les autres. Le dernier abonnement libéré ferme la connexion après
le poll en cours. Une interface demandée explicitement doit correspondre à
celle du lecteur actif ; un abonnement automatique réutilise le lecteur actif.

Aucun opcode d’orientation n’est encore sélectionné ou décodé. Le traitement
commun de messages et les abonnements par opcode devront être raccordés lors
de la validation d’une source ; ce jalon partage l’acquisition seulement.
`Runtime::update_orientation` constitue l’entrée pour un futur décodeur avec
attribution au personnage local et calibration validées. Le modèle rejette les
angles non finis, normalise les angles finis et expire les échantillons au bout
de trois secondes. Le rendu utilise les graduations de ce modèle ; sans entrée
validée, il affiche « orientation indisponible ». Les tests de données contrôlées
ne valident pas le protocole du jeu. Voir [l’ADR 10](adr/0010-overlay-widgets-and-orientation.md).

## Vérification manuelle sous Windows

- Activer la boussole, fermer et rouvrir Aetheris : le choix reste activé.
- Vérifier l’attente du jeu, puis lancer Aion 2 en fenêtre et sans bordure.
- Vérifier la position de la bande après déplacement du jeu et sur un autre écran.
- Cliquer à travers la bande : le jeu reçoit les clics et garde le focus.
- Passer dans une autre application, réduire le jeu puis revenir : la bande se
  masque et réapparaît sans activer Aetheris.
- Arrêter le service : l’éditeur signale l’erreur et conserve l’activation.
  Relancer le service et utiliser Réessayer la connexion.
- Démarrer Combat Debug sur la même interface, désactiver la boussole et vérifier
  que l’enregistrement continue. Faire également l’essai dans l’ordre inverse.
- Vérifier qu’activer la boussole seule n’ajoute aucune session de capture en base.

La calibration et la validation de la source exigent ensuite les actions
contrôlées de [la note d’analyse passive](orientation-passive-research.md).
