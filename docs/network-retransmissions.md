# Retransmissions TCP observées

Le module `capture::retransmissions` implémente l’issue #50 de l’épic #37 à partir
des [observations partagées](network-observations.md). Il prépare les instantanés
des futurs widgets de l’[ADR 0014](adr/0014-mesures-reseau-observees.md).
Il ne fournit pas un pourcentage exact de paquets perdus sur le chemin réseau.
Ce lot ne raccorde pas encore l’indicateur à un widget.

## Numérateur et dénominateur

Le calcul suit uniquement `Selection::Selected(connection)`, avec un historique
séparé pour les directions entrante et sortante. Il commence lorsque la connexion
est identifiable sans ambiguïté ; le segment qui permet cette sélection peut
être compté, sans attendre une fenêtre complète. Les paquets de candidats qui
n’étaient pas encore sélectionnés ne sont pas ajoutés rétroactivement.

Chaque segment ayant au moins un octet de données TCP contribue **une fois** au
dénominateur, y compris ses retransmissions. Il contribue également une fois au
numérateur si sa plage de données recoupe au moins un octet déjà observé dans
la même direction de cette connexion. Une retransmission partielle portant aussi
des octets nouveaux compte donc comme un segment retransmis, sans pondération
par le nombre d’octets répétés. Un segment couvrant plusieurs plages connues
compte toujours une seule fois. Les plages adjacentes ne se chevauchent pas.

Les ACK, SYN, FIN et RST sans données ne contribuent à aucun compteur. SYN
occupe un numéro avant les données éventuelles ; FIN occupe un numéro après
celles-ci, sans faire partie des plages de données. Les deux espaces de
séquences ne sont jamais comparés entre eux. Les changements d’identité de
connexion, notamment sa génération et son incarnation, effacent ces historiques.

## Cas ambigus et limites de classification

| Observation | Classification |
| --- | --- |
| Segment entièrement nouveau, même reçu sous une séquence inférieure au maximum déjà vu | Original observé : un réordonnancement seul ne suffit pas à conclure à une retransmission. |
| Segment qui remplit un trou sans recouper d’octet connu | Original observé. Le trou n’est ni ajouté au dénominateur ni compté comme une perte. |
| Doublon intégral, chevauchement partiel, ou segment mêlant octets déjà vus et nouveaux | Un segment retransmis observé. Les octets nouveaux complètent aussi l’historique. |
| Copie supplémentaire du même paquet dans la capture | Même règle de chevauchement : indiscernable ici d’une retransmission TCP. Un identifiant de paquet répété au lieu d’une nouvelle observation déclenche toutefois la discontinuité signalée par #48. |
| Premier segment observé d’une retransmission dont l’envoi initial a échappé à la capture | Original observé faute de preuve antérieure ; l’indicateur ne détecte pas toutes les retransmissions réelles. |
| Enroulement u32 avec déplacements interprétables dans un demi-espace de séquence | Dépliage autour de la frontière maximale connue ; même règle d’intersection avant et après l’enroulement. |
| Déplacement exactement égal à la moitié de l’espace u32, longueur hors demi-espace ou dépliage non représentable | Classification indisponible, abandon de l’historique de cette direction et nouvelle fenêtre complète de récupération. |

Les comparaisons cycliques suivent les principes de la
[RFC 9293, section 3.4](https://www.rfc-editor.org/rfc/rfc9293.html#section-3.4).
Le dépliage suppose que le déplacement entre une observation et la frontière
connue est interprétable à moins de 2³¹ octets. Une capture passive ne permet pas
de résoudre tous les cas où de vieux segments réapparaissent après un tour
complet de séquences, ni un long saut non signalé de la capture.

Le calcul utilise les plages de séquences, sans stocker ni comparer les corps de
paquets. Un chevauchement aux octets contradictoires suit donc la même règle,
sauf si l’analyseur partagé signale une interruption du flux, qui invalide alors
les compteurs. Il n’utilise ni le seul maximum des séquences pour déclarer une
retransmission, ni le nombre d’ACK dupliqués comme compteur de segments retransmis.

## Fenêtre et disponibilité

La fenêtre glissante vaut quinze minutes par défaut. Elle inclut les observations
dont l’âge de capture est strictement inférieur à sa durée. L’horodatage de
capture est converti sur l’horloge monotone ancrée au démarrage, comme pour
[la latence](network-latency.md). Un ancien paquet en file ne rajeunit pas les
compteurs. Sa plage peut néanmoins compléter l’historique et permettre de
reconnaître une retransmission ultérieure. Une plage connue ne disparaît pas
simplement parce que le segment original est sorti de la fenêtre des compteurs.

`Snapshot::counts`, lorsqu’il est disponible, contient :

- `retransmissions` : nombre de segments de données observés comme retransmis ;
- `data_segments` : tous les segments de données observés dans la fenêtre ;
- `percentage` : `100 × retransmissions / data_segments`, ou `None` si le
  dénominateur est nul. Une fenêtre exploitable vide a des compteurs nuls et
  un pourcentage indisponible ; un pourcentage de zéro avec des données est valide.

`counts = None` désigne l’absence de sélection unique ou la récupération après
une mesure compromise. Le futur widget devra afficher « — » pour l’indisponible,
sans afficher de durée de capture. `recovering` décrit cet état séparément.

Une interruption, un trou détecté par #48, une attribution incertaine, une
troncature, une erreur d’horodatage ou une limite de ressources effacent les
compteurs et imposent une nouvelle fenêtre complète exploitable. Les compteurs
se reconstruisent pendant cette période mais restent masqués. Un nouveau défaut
recommence l’attente ; une période sans connexion sélectionnable ne la satisfait
pas. L’historique de séquences connu de la même connexion reste conservé, sauf
lorsqu’il ne peut plus être déplié de manière interprétable. Les défauts propres
à une autre connexion n’invalident pas cet indicateur.

`set_window` accepte une durée strictement positive jusqu’à une heure, sans
modifier l’état pour une valeur refusée ou identique. Une modification efface
les compteurs et exclut les anciens paquets en file ; les plages de séquences
de la même connexion restent connues. Une récupération en cours repart avec la
nouvelle durée. La configuration et sa persistance seront raccordées avec les widgets.

## Capture et ressources

`Retransmissions::observe` consomme les événements ordonnés de #48. `snapshot`
expose l’état immédiatement ; `snapshot_if_due` publie le premier instantané
immédiatement puis au plus un par seconde. Il doit être appelé pendant les
silences pour expirer les compteurs. `capture::retransmissions::Source` fournit
l’abonnement à la capture partagée et cette cadence ; son propriétaire appelle
`poll` et peut consulter `snapshot` après une erreur pour lire l’état indisponible.
Le dernier propriétaire libéré abandonne cet abonnement et son historique ;
les autres consommateurs du hub continuent. Aucune collecte ne démarre à la
simple présence du module. L’intégration des widgets réunira leurs besoins de calcul.

La mémoire est bornée à 4 096 plages disjointes par direction et 262 144
observations datées dans la fenêtre, sans corps de paquet ni SQLite. Les plages
contiguës fusionnent : leur mémoire ne croît pas avec le nombre de segments d’un
flux continu. Une saturation masque la mesure et impose la récupération, sans
éliminer silencieusement des observations pour modifier le pourcentage. Une
saturation des plages reste signalée par l’indisponibilité tant que de nouveaux
segments ne permettent pas de fusionner les trous ou qu’une nouvelle connexion
ne remplace pas cet historique.

Les trous de capture et les pertes du collecteur n’entrent jamais dans le
numérateur ni le dénominateur. Le compteur de pertes collecteur reste celui
exposé séparément par #48, actuellement `None` sur ce chemin Winsock ; ce module
n’en invente aucun. Un saut de séquence encore susceptible d’être un simple
réordonnancement ne déclare pas à lui seul un trou de capture. Les interruptions
explicites détectées par la capture ou le décodeur sont traitées séparément.

## Validation

Les tests couvrent les deux directions, les chevauchements, doublons et nouvelles
fins de segments, le réordonnancement et le remplissage de trous, l’enroulement,
l’absence de données, l’expiration exacte, les reconnexions et ambiguïtés,
les interruptions, les horodatages invalides et les anciennes données en file.
Ils vérifient aussi les deux bornes de mémoire et le redémarrage de récupération
après changement de fenêtre. Un test passe par des octets IPv4/TCP, la sélection
par trois ticks, des doublons des deux directions et une troncature, tout en
vérifiant que les pertes du collecteur restent indisponibles. La qualification
sur une capture de jeu réelle reste à faire.
