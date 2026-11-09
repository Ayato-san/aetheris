# Installeur Windows

Statut : implémentation ajoutée le 9 novembre 2026 ; recette complète de l’installeur
et des mises à jour entre sessions Windows à effectuer avant diffusion.

## Besoin exprimé

Proposer un installeur Windows avec une présentation plus soignée que la
distribution ZIP actuelle. Le dossier proposé par défaut est
`C:\Program Files\Aetheris`. L’installation doit permettre d’ajouter Aetheris
au menu Démarrer.

## Décisions confirmées

- Installation pour tous les utilisateurs, avec demande de droits administrateur.
- Dossier proposé : `C:\Program Files\Aetheris`, modifiable dans l’assistant.
- Assistant graphique aux couleurs d’Aetheris, dans l’esprit d’Inno Setup :
  accueil, dossier, options, progression et possibilité de lancer l’application
  à la fin.
- Ajout au menu Démarrer coché par défaut ; raccourci Bureau facultatif,
  décoché par défaut.
- Installeur comme téléchargement principal ; ZIP conservé pour l’installation
  manuelle et le mécanisme actuel de mise à jour.
- Inno Setup appelé directement depuis une étape PowerShell du workflow GitHub
  Actions, avec le compilateur présent sur le runner Windows officiel et sans
  Action tierce dédiée.
- Service Windows installé avec les droits administrateur initiaux pour
  appliquer les mises à jour authentifiées sans nouvelle demande d’élévation.
  Le service est supprimé à la désinstallation ; l’application reste exécutée
  sans droits administrateur.
- Données personnelles propres à chaque compte Windows dans
  `%LOCALAPPDATA%\Aetheris`, y compris la copie de référence `Engine.baseline.ini`.
- Désinstallation des programmes et raccourcis en conservant les données
  personnelles et les réglages appliqués au jeu ; aucune modification de
  `Engine.ini` lors de la désinstallation.
- Migration facultative au premier lancement : sélection de l’ancien dossier
  ZIP pour récupérer la copie de référence, sans écraser les données personnelles
  déjà présentes ni supprimer le dossier source. Il n’existe actuellement qu’un
  déploiement bêta personnel ; aucune exigence supplémentaire de compatibilité
  avec un parc historique n’a été exprimée.
- Une seule installation, avec le sélecteur stable/préversion existant et sans
  rétrogradation automatique lors du retour au canal stable.
- Si Aetheris est utilisé dans une autre session Windows, la mise à jour attend
  la fermeture de toutes les instances, avec un message explicatif et sans
  arrêt forcé.
- Installeur autonome contenant tous les fichiers nécessaires à la version
  distribuée ; installation possible hors ligne, puis recherche des mises à
  jour lorsqu’une connexion est disponible.
- Canal stable/préversion commun à toute l’installation gérée, avec une
  indication de cette portée dans le sélecteur.
- Installation manuelle depuis le ZIP indépendante du service : aucun service
  installé, mises à jour directes si le dossier est accessible en écriture,
  message d’erreur dans le cas contraire. Le service ne gère que l’installation
  créée par l’installeur.
- Relancer l’installeur de la même version permet une réparation ; une version
  plus récente effectue une mise à niveau ; une version plus ancienne est
  bloquée avec une explication. Les données personnelles sont conservées et
  la restauration après une mise à jour défaillante reste possible.

## Contraintes d’implémentation

Le service applique uniquement des versions Aetheris authentifiées à son
installation enregistrée. La vérification ne doit pas reposer sur la seule
validation effectuée par le processus utilisateur. Les permissions du dossier
de programmes restent protégées ; le service ne constitue pas une interface
générale d’exécution privilégiée ou d’écriture vers des chemins arbitraires.

Les données personnelles et les opérations de migration restent rattachées au
compte Windows qui utilise Aetheris, même si l’installation initiale est
autorisée avec les identifiants d’un autre administrateur. Le lancement final
depuis l’assistant doit ouvrir le compagnon sans lui transmettre les privilèges
administrateur de l’installeur.

Les garanties existantes de vérification des signatures, de sauvegarde,
de migration SQLite, de récupération après interruption et de restauration
après échec restent applicables. Les opérations de l’installeur, du service et
du compagnon doivent être coordonnées sur la même installation.

## Vérifications attendues

- Installation hors ligne sur une machine Windows propre, dans le dossier par
  défaut et dans un dossier personnalisé ; présence des raccourcis choisis et
  de l’entrée de désinstallation Windows.
- Lancement depuis le menu Démarrer et depuis l’assistant sans élévation de
  l’application ; données créées dans le profil du bon utilisateur.
- Mise à jour via le service sans nouvelle demande administrateur ; refus des
  paquets non authentiques et des destinations étrangères à l’installation.
- Attente d’une autre session active, sans arrêt forcé ; absence de concurrence
  entre mise à jour, réparation et désinstallation.
- Réparation à version identique, mise à niveau et refus d’un ancien installeur ;
  récupération cohérente après interruption ou échec de mise à jour.
- Migration facultative sans écrasement et fonctionnement indépendant du ZIP.
- Désinstallation retirant service, programmes et raccourcis tout en conservant
  captures, préférences, journaux, copie de référence et fichier du jeu.
- Publication du ZIP et de l’installeur pour stable et bêta ; aucun artefact
  incomplet rendu public si la génération de l’un des deux échoue.

## Vérifications de l’outillage

La [liste officielle des logiciels du runner Windows Server 2025](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md)
inclut Inno Setup. Son [compilateur en ligne de commande officiel](https://jrsoftware.org/ishelp/topic_compilercmdline.htm)
permet de générer l’installeur depuis une étape PowerShell du workflow existant,
sans Action tierce. Aucune Action dédiée maintenue par l’éditeur d’Inno Setup
n’a été identifiée lors de la vérification du 3 novembre 2026.

## Contrainte existante

L’[ADR 0007](adr/0007-windows-installer-and-update-service.md) fait évoluer les
choix des ADR 0001 et 0004 pour les installations gérées par l’installeur :
programmes communs protégés, données par utilisateur et mises à jour par service.

L’application écrit actuellement sa copie de référence `Engine.baseline.ini`
à côté de l’exécutable. Les autres données utilisateur résident déjà dans
`%LOCALAPPDATA%\Aetheris`. Le raccourci devra lancer le compagnon, point d’entrée
normal d’Aetheris.

## Implémentation

`installer/aetheris.iss` produit `aetheris-windows-x86_64-setup.exe` après la
signature du ZIP. Le workflow compile les deux distributions avant de pousser
le tag ; le script de publication charge les deux fichiers dans un brouillon
avant de le rendre public.

Le service `AetherisUpdate` utilise une copie du helper dans
`{app}\service\bootstrap.exe`. Cette copie est remplacée par une réparation ou
une mise à niveau de l’installeur ; les mises à jour du service délèguent
l’application du paquet au nouveau helper signé. Le bootstrap, son dossier de
travail et les programmes ne sont modifiables que par SYSTEM et les
administrateurs. Un emplacement personnalisé doit avoir des parents protégés ;
les jonctions et les chemins remplaçables par un utilisateur sont refusés.

Les utilisateurs authentifiés disposent uniquement des droits de consultation
et des commandes SCM prédéfinies : canal stable/préversion, mise à jour,
restauration, récupération, réparation et confirmation d’un lancement réussi.
Aucune commande ne transporte de chemin, d’URL ou de programme à exécuter.
Le service télécharge dans son propre dossier, vérifie les signatures et refuse
les rétrogradations automatiques. La restauration explicite utilise la sauvegarde
signée. Les états d’attente et les erreurs se lisent dans `{app}\service\status`.

Les verrous de fichiers protégés `service/instances.lock` (lecteurs multiples)
et `service/update.lock` coordonnent les processus entre sessions Windows.
L’installeur et le désinstalleur arrêtent le service sans le tuer, puis prennent
ces verrous en exclusivité. Aucun processus applicatif n’est arrêté de force.
L’installeur dépose d’abord les programmes signés dans un dossier de préparation,
puis utilise le journal de remplacement du helper, y compris pour une réparation.

Le service n’ouvre aucune base personnelle avec ses droits SYSTEM. Au premier
accès par son propriétaire, une migration SQLite crée une sauvegarde de son
ancien schéma. Après une restauration des programmes, le propriétaire restaure
le schéma correspondant et conserve une copie des données postérieures à la
mise à jour dans un fichier `post-rollback-*.sqlite3`. Les migrations échouées
restent transactionnelles. Ces sauvegardes sont conservées à la désinstallation.

Au premier lancement géré sans référence personnelle, un sélecteur propose
l’ancien dossier ZIP ; Annuler utilise la référence fournie avec l’application.
Seul `Engine.baseline.ini` est importé, sans écrasement ni suppression du fichier
source. Les raccourcis et le lancement final ciblent le compagnon, ce dernier
utilisant les droits de l’utilisateur d’origine via `runasoriginaluser`.

### Recette restant nécessaire

La compilation et les tests ne remplacent pas les vérifications listées plus
haut : exécuter l’installeur hors ligne dans une VM Windows, avec un compte
standard et des identifiants administrateur distincts, puis tester deux sessions,
une coupure pendant le remplacement, les options de raccourcis et la
désinstallation. Le compilateur Inno Setup est requis pour la génération locale ;
le workflow utilise directement le compilateur officiel du runner Windows.
