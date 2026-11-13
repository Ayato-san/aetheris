# Widgets d’overlay

L’**Overlay Editor**, dans **En jeu**, propose le widget de cubes cachés.
Son activation et son son sont désactivés par défaut et sauvegardés dans
`overlay.json`. Une erreur d’écriture restaure le choix précédent ; Réessayer
la connexion conserve les préférences. La boussole reste retirée : son ancien
réglage n’active ni capture ni fenêtre.

## Présence et détection continue

Sous Windows, une fenêtre native transparente aux clics suit le haut de la
zone client du jeu au premier plan. La présence est enregistrable : elle donne
le nombre d’instances détectées et leur durée de détection continue par Aetheris.
Elle reste masquée pendant l’attente normale et en l’absence de cube. Une perte
de capture est signalée et invalide toutes les données suivies.

Les objets sont reconnus par le format `34 36` et la liste de types de la note
locale `base-doc/widgets/TRACKING_CUBES_CACHES.md`. Le décodeur vérifie les champs,
le masque, la longueur complète et les coordonnées finies et bornées. Il ne
charge pas de catalogue d’emplacements possibles. La variante d’apparition
`41 36`, encore incertaine, est exclue.

Une réannonce de la même instance met à jour sa position sans remettre sa durée
à zéro ni rejouer le son. Le son facultatif utilise le mécanisme commun de
notifications des widgets ; il ne contient aucune direction ou distance.
Les retraits `35 36`, `3a 36` et `42 36` libèrent le suivi, sans annoncer une
collecte. Les frontières `21 36`/`23 36`, reconnexions et interruptions annulent
les cubes, les durées, la position et la cible. Aucune donnée n’est partagée
entre utilisateurs ; le champ serveur d’ancienneté n’est pas utilisé.

## Traitement partagé

`capture::widgets` utilise le lecteur IPC de `capture::shared`, également utilisé
par Combat Debug. Les widgets partagent un réassembleur incrémental, le tramage
et la décompression, puis déclarent leurs besoins par opcode et entité.
Les annonces de découverte restent analysées sans filtre d’entité ; les retraits
et mouvements ne sont distribués que pour les identifiants suivis. Les filtres
sont réévalués entre deux messages, même dans une enveloppe compressée.

La direction serveur est attribuée à l’adresse locale connue de la capture,
sans deviner les ports. Le flux doit être synchronisé par trois ticks serveur
cohérents. Une seule connexion synchronisée alimente le monde suivi. Les corps
client chiffrés restent ignorés ; aucun filtrage ne prétend y lire les opcodes.
Combat Debug garde son analyse et sa sauvegarde explicites des preuves brutes.

Les buffers sont bornés à huit flux, 1 Mio de tampon et d’historique par flux,
128 segments en attente et 8 Mio de décompression par alimentation, sur quatre
niveaux au maximum. Un trou TCP de trois secondes, un chevauchement contradictoire,
un paquet tronqué ou dix secondes de silence du flux synchronisé invalident le
suivi. Le modèle accepte au plus 256 cubes. Le dernier désabonnement libère le
pipeline et sa souscription IPC, sans interrompre un enregistrement Combat Debug.

## Guidage privé

Le modèle choisit le cube le plus proche en distance 3D, avec une hystérésis :
un concurrent doit être au moins 10 % et 2 m plus proche pour changer la cible.
Il calcule la distance horizontale et le dénivelé, en mètres depuis les
coordonnées réseau en centimètres. Il ne produit aucune flèche déduite du déplacement.
Les coordonnées initiales du personnage peuvent venir du message `23 36`
avec tag d’entité zéro ; les mises à jour `1a 37` ne sont acceptées que pour
l’identité locale reçue dans `33 36`. Les autres personnages, le loot et les
recherches de coordonnées en fin de paquet ne servent pas de substituts.
Les horodatages de capture sont rattachés à une horloge monotone afin que le
traitement d’un lot en retard ne rajeunisse pas la position.
Après quatre secondes sans position fiable, le guidage disparaît et la présence
reste suivie. Cette fraîcheur conservatrice exige une validation en jeu.

Le guidage dispose d’une fenêtre distincte de la présence. L’affichage exige
`WDA_EXCLUDEFROMCAPTURE`, sa relecture réussie, la composition DWM active et une
configuration de capture validée. La présence ne reçoit aucun message expliquant
l’indisponibilité du guidage. Microsoft limite la protection à certaines API de
capture et aux versions compatibles ; le succès de l’appel ne prouve pas à lui
seul la compatibilité OBS ([documentation Win32](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity)).

**Aucune configuration de capture n’est encore validée.**
`guidance_visibility::configuration_validated()` reste donc à `false` et le
guidage reste entièrement masqué. La matrice ci-dessous doit être exécutée sur
Windows avant de définir comment identifier et autoriser une configuration.
Il n’existe pas de garantie universelle ni d’option utilisateur pour contourner
cette condition.

## Vérification sous Windows

- Activer le widget, redémarrer Aetheris : activation et son sont conservés.
  Vérifier également une ancienne configuration avec seulement la boussole active.
- Attendre le jeu puis jouer en fenêtre et sans bordure : rien ne s’affiche tant
  qu’aucun cube n’est annoncé. Vérifier les durées de plusieurs instances et
  l’absence de son répété lors des réannonces.
- Déplacer le jeu sur un autre écran, cliquer à travers le panneau, changer
  d’application et réduire le jeu : position, focus et masquage restent corrects.
- Changer de zone et reconnecter le jeu : durées et cible repartent à zéro.
  Arrêter le service : l’interruption s’affiche et les cubes sont invalidés.
- Activer Combat Debug sur la même interface, puis désactiver le widget :
  l’enregistrement continue. Répéter dans l’ordre inverse. Le widget seul
  n’ajoute aucune capture en base.
- Valider les sources de position lors de marche, saut, dash, ailes et immobilité.
  Attendre quatre secondes sans nouvelle position : la présence doit subsister.
- Sur une version instrumentée permettant la validation de la fenêtre privée,
  vérifier la présence visible et le guidage absent dans chaque capture :

| Configuration | Présence enregistrable | Guidage exclu | État |
| --- | --- | --- | --- |
| Capture Windows / Impr. écran | À vérifier | À vérifier | Non validée |
| Outil Capture d’écran Windows | À vérifier | À vérifier | Non validée |
| OBS · capture d’écran | À vérifier | À vérifier | Non validée |
| OBS · capture de fenêtre (chaque méthode disponible) | À vérifier | À vérifier | Non validée |
| OBS · capture de jeu | À vérifier | À vérifier | Non validée |

Vérifier aussi l’échec de l’affinité, une version Windows antérieure à la prise en
charge de `WDA_EXCLUDEFROMCAPTURE`, et la perte de composition : la fenêtre privée
et ses éventuels messages restent masqués. Consigner version Windows, version
OBS, méthode de capture et mode d’affichage du jeu pour chaque résultat.

Les tests automatisés couvrent le vecteur d’objet réel documenté par a2kit
(`28137 / 1200039`, `−3998, 89851, 22067`), ses découpages TCP, le désordre,
les retransmissions, LZ4, le filtrage, les resets, les durées et la fraîcheur.
Ils ne remplacent pas les essais en jeu ni la matrice de capture.
