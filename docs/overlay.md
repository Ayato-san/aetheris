# Widgets d’overlay

L’**Overlay Editor**, dans **En jeu**, propose le widget de cubes cachés.
Son activation, son guidage privé et son son sont désactivés par défaut et sauvegardés dans
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
collecte. Ils retirent uniquement l’instance identifiée, pas tous les cubes.
Les frontières `21 36` / `23 36`, changements d’identité locale, reconnexions et
interruptions annulent les cubes, les durées, la position et la cible. Aucune donnée n’est partagée
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
Une nouvelle connexion TCP reste candidate jusqu’à cette synchronisation : son
SYN, ses erreurs, sa fermeture ou ses trous de réassemblage n’effacent pas le
monde de la connexion du jeu déjà reconnue. Après remplacement par une nouvelle
connexion reconnue, les paquets de l’ancienne connexion ne reprennent pas le suivi.

Les buffers sont bornés à huit flux, 1 Mio de tampon et d’historique par flux,
128 segments en attente et 8 Mio de décompression par alimentation, sur quatre
niveaux au maximum. Trois secondes sans progression du réassemblage en attente,
un chevauchement contradictoire, un paquet tronqué ou dix secondes de silence
du flux synchronisé invalident le suivi. Le modèle accepte au plus 256 cubes.
Le dernier désabonnement libère le
pipeline et sa souscription IPC, sans interrompre un enregistrement Combat Debug.
Des trous TCP successifs résorbés ne constituent pas une interruption tant que
la séquence contiguë avance. Les doublons et les segments reçus au-delà d’un trou
non résolu ne prolongent pas son délai. Le diagnostic distingue un trou TCP
d’une trame serveur encore inachevée.

## Guidage privé

Le modèle choisit le cube le plus proche en distance 3D, avec une hystérésis :
un concurrent doit être au moins 10 % et 2 m plus proche pour changer la cible.
Il calcule la distance horizontale et le dénivelé, en mètres depuis les
coordonnées réseau en centimètres. Il ne produit aucune flèche déduite du déplacement.
Les coordonnées initiales du personnage viennent du chargement `21 36`
(zone u32 à l’offset 4 du payload, XYZ aux offsets 16/20/24), ou de `23 36`
avec tag d’entité zéro. Les corrections `3c 37` exigent le marqueur `03` et
l’identité locale confirmée ; les mises à jour `1a 37`, `1b 37` et `1c 37` ne sont
acceptées que pour l’identité locale reçue dans `33 36`. Le message d’action
`02 38` fournit aussi une position, après validation complète du format, si
sa **cible** est le personnage local. Un sort du personnage visant quelqu’un
d’autre ne rafraîchit pas sa position. Les effets `2a 38` / `2b 38` peuvent aussi
fournir XYZ dans le format structuré vérifié (suffixe de métadonnées `01 00`,
puis exactement douze octets de coordonnées), uniquement lorsque cible **et**
lanceur sont le personnage local. Aucun balayage des derniers octets ni
attribution entre deux personnages différents n’est effectué. Les autres
personnages, le loot et les
recherches de coordonnées en fin de paquet ne servent pas de substituts.
Les horodatages de capture sont rattachés à une horloge monotone afin que le
traitement d’un lot en retard ne rajeunisse pas la position.
Après quatre secondes sans position fiable, le guidage disparaît et la présence
reste suivie. Les projets de carte examinés ne démontrent aucun flux exact
continu de la position locale pendant les déplacements : certains utilisent
des estimations. Ces estimations ne sont pas utilisées par le guidage 3D.
Voir la [recherche et ses sources](research/player-position-sources.md).

L’option « Guidage privé vers le cube le plus proche » est persistée séparément.
Sa désactivation libère les abonnements de position, sans effacer les cubes
encore détectés ni leurs durées. L’identité `33 36` reste suivie pour invalider
les cubes au changement de personnage, même sans guidage. Une identité déjà
reçue reste connue lors d’un changement de réglage ; les positions et la cible
sont effacées. Si la capture commence au milieu d’une connexion, la première
identité confirmée invalide les cubes antérieurs non attribués, puis seules de
nouvelles annonces les ajoutent au suivi. Une réannonce de la même identité
connue conserve le suivi. `21 36` mémorise la zone : sa première observation
établit une référence et ses répétitions dans cette zone conservent le suivi
et l’identité. Seul un identifiant de zone différent efface l’ancien monde ;
la confirmation `33 36` conserve alors les données reçues depuis cette frontière.
Aucune identité n’est inventée en attendant `33 36`.

Les cubes réels se placent en haut à gauche du jeu. Les fenêtres natives ne
sont créées qu’avec des données affichables et le jeu au premier plan. Présence
réelle et présence fictive partagent la même classe native ; il en va de même
pour les fenêtres de guidage. Elles sont déclarées au premier plan dès leur
création, sans prendre le focus. Le panneau privé se place sous la présence pour éviter leur superposition. Il
donne le même numéro de cube que la liste de présence, sa distance horizontale
et son dénivelé. Son petit repère utilise **−Y en haut et +X à droite** dans les
coordonnées du monde, sans supposer le nord ni l’orientation de la caméra. Une
cible à la verticale ne reçoit aucune direction horizontale arbitraire. Une
position absente ou périmée remplace les mesures par une attente de position,
uniquement dans la fenêtre privée protégée.

Le guidage réel dispose d’une fenêtre distincte de la présence. L’affichage exige
`WDA_EXCLUDEFROMCAPTURE`, sa relecture réussie, la composition DWM active et une
configuration de capture validée. La présence ne reçoit aucun message expliquant
l’indisponibilité du guidage. Microsoft limite la protection à certaines API de
capture et aux versions compatibles ; le succès de l’appel ne prouve pas à lui
seul la compatibilité OBS ([documentation Win32](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity)).

**Essais de capture rapportés par l’utilisateur le 13 novembre 2026 :** screenshot
et OBS en capture d’écran, de fenêtre et de jeu excluent le guidage fictif visible
à l’écran. En capture d’écran OBS, le panneau de présence fictif est bien
conservé ; en capture de fenêtre/jeu, les deux fenêtres d’overlay sont absentes,
puisque ces modes capturent le jeu. Le mode du jeu testé est sans bordure.
La machine testée utilise **Windows 11 25H2, build 26200.9711 et OBS 33.0.1**,
versions confirmées par l’utilisateur. Ce profil est inscrit dans
`guidance_visibility::VALIDATED` et autorise le guidage réel en mode sans bordure,
sous réserve des vérifications natives de protection.
Le 14 novembre, l’utilisateur a confirmé conserver ce mode et relevé le style
natif `14cf0000` dans le test protégé. Ce style est admis pour ce profil précis :
la présence du bit `WS_CAPTION` ne suffit pas à déduire le mode de présentation
du jeu. Il ne s’agit pas d’une validation générale de tous les modes fenêtrés.

La qualification compare le build Windows et sa révision ainsi que les versions
des processus OBS actifs aux profils ayant effectué tous ces essais. Un autre
build, une autre version d’OBS, un échec de lecture du contexte ou un autre style
portant `WS_CAPTION` ne reprend pas la validation. La lecture du registre et des
processus est seule utilisée ; les scènes OBS ne sont pas modifiées et leur
méthode de capture ne peut pas être identifiée par ce mécanisme. C’est pourquoi
un profil admis doit couvrir les trois modes OBS, et non un mode supposé.
L’environnement est réévalué toutes les 250 ms, et les vérifications natives
d’affinité et de composition restent appliquées à chaque affichage et peinture.
Ces vérifications ne sont pas une garantie universelle pour d’autres logiciels
ou méthodes de capture. Il n’existe aucune option pour forcer la qualification.

## Diagnostic et test d’affichage

L’éditeur indique l’état de la fenêtre de présence, la synchronisation du flux,
le nombre d’annonces d’objets reçues et le nombre de cubes suivis. Ces informations
ne contiennent aucune position, cible ou explication d’indisponibilité du guidage.
La fenêtre du jeu est identifiée par son PID dans un instantané de processus,
sans ouvrir le processus ni lire son chemin ; cela évite une dépendance aux
permissions d’interrogation du jeu. Les erreurs de coordonnées ou d’identification
restent consultables dans l’éditeur après le retour au premier plan d’Aetheris.
Le dernier affichage en jeu et les limites de la fenêtre native sont conservés
après un changement de fenêtre pour distinguer un affichage confirmé d’une
attente sans aucun affichage préalable.
Le compteur mesure les passages de peinture, sans prouver à lui seul la visibilité
finale à l’écran. Le diagnostic ajoute la position native relue, `IsWindowVisible`
et le masquage DWM (`DWMWA_CLOAKED`, zéro si non masqué, « indisponible » si la
lecture échoue ; [documentation Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)). Chaque fenêtre
copie ses données avant les appels Win32 ; `WM_PAINT` dessine cette copie sans
verrouiller le suivi réseau. Une invalidation et une peinture synchrone suivent
le positionnement de la fenêtre. Le traitement des messages est borné par tour
pour ne pas empêcher les mises à jour du contenu et de la visibilité. Une copie
de guidage conserve l’échéance de sa position : même une peinture retardée
remplace les mesures périmées par l’attente protégée.
Le dernier événement de présence est également conservé : annonce ajoutée,
format d’annonce rejeté, retrait réseau, changement de zone ou resynchronisation.
Il n’affiche aucune coordonnée de cube ni information de guidage. Le message
`21 36` est désormais reconnu comme chargement de zone après vérification des
implémentations Daevas et Nyerk ; seule une différence avec la zone précédemment
observée réinitialise le suivi. Un rapport répété ne constitue pas une frontière. Pour `23 36`, le tag local
zéro et des XYZ complets valides sont exigés avant toute invalidation.
Le panneau privé distingue identité inconnue, aucune position reçue et position
périmée (âge en secondes). Ces explications restent exclues des captures.

« Tester l’affichage pendant 60 s » ouvre deux fenêtres de test lorsque le jeu
est au premier plan, à côté des widgets réels : une présence marquée **TEST · Données fictives** et un
panneau de guidage fictif (5 m horizontalement, 12 m au-dessus). Le bouton permet
aussi d’arrêter le test. Le test n’ouvre aucune capture, ne consulte aucun cube
réel et ne modifie pas les préférences ou la validation du guidage réel.
Le panneau privé de test ajoute les versions Windows/OBS observées, le style de
la fenêtre au premier plan et le résultat de la qualification. Ce diagnostic
ne lit aucune donnée de suivi et reste exclu des captures ; il faut relever
son texte à l’écran. Il permet de distinguer un test fictif fonctionnel d’une
configuration refusée pour le guidage réel.

La fenêtre privée de test utilise les mêmes styles natifs, le même rendu et
les mêmes vérifications d’affinité/DWM que le guidage. Seule la qualification
préalable de configuration est omise pour ces **données fixes fictives**, afin
de permettre de remplir la matrice de capture avant d’autoriser les données du
jeu. Si l’affinité ou la composition ne peuvent pas être vérifiées, même ce
panneau fictif reste masqué. Un test absent à l’écran ne valide pas son exclusion
des captures.

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
- Utiliser le test d’affichage : vérifier d’abord que la présence fictive et le
  guidage fictif sont tous deux visibles à l’écran, puis vérifier la présence
  visible et le guidage absent dans chaque capture :

| Configuration | Présence enregistrable | Guidage exclu | État |
| --- | --- | --- | --- |
| Screenshot utilisateur (outil non précisé) | Rapportée visible | Oui | Essai utilisateur réussi |
| Outil Capture d’écran Windows (autres méthodes) | À vérifier | À vérifier | Non vérifiées séparément |
| OBS · capture d’écran | Oui, panneau fictif présent | Oui | Essai utilisateur réussi |
| OBS · capture de fenêtre | Absente : seul le jeu est capturé | Oui | Essai utilisateur réussi en sans bordure |
| OBS · capture de jeu | Absente : seul le jeu est capturé | Oui | Essai utilisateur réussi en sans bordure |

Vérifier aussi l’échec de l’affinité, une version Windows antérieure à la prise en
charge de `WDA_EXCLUDEFROMCAPTURE`, et la perte de composition : la fenêtre privée
et ses éventuels messages restent masqués. Consigner version Windows, version
OBS, méthode de capture et mode d’affichage du jeu pour chaque résultat.

Les tests automatisés couvrent le vecteur d’objet réel documenté par a2kit
(`28137 / 1200039`, `−3998, 89851, 22067`), ses découpages TCP, le désordre,
les retransmissions, LZ4, le filtrage, les resets, les durées et la fraîcheur.
Ils ne remplacent pas les essais en jeu ni la matrice de capture.

Un test graphique Windows vérifie aussi le rendu de présence réel : fenêtre
absente sans cube ou sans jeu au premier plan, création à l’arrivée d’une détection, pixels de texte et de fond sur la
surface native, masquage au retrait puis réapparition et redimensionnement,
pendant qu’un panneau fictif protégé est peint sur un autre thread. Il nécessite une session Windows
interactive et s’exécute explicitement avec
`cargo test --lib --no-default-features real_presence_is_painted -- --ignored`.
Il utilise uniquement le cube de référence et ne valide pas le rendu sur le
poste du joueur ni l’exclusion des captures.
