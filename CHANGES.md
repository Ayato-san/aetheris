# Notes de version stables d’Aetheris

## PENDING

### Add

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

### Fixes
