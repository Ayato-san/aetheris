# Ping et jitter TCP observés

Le module `capture::latency` implémente les calculs de l’issue #49, à partir des
[observations de la capture partagée](network-observations.md) introduites par #48.
Il expose les instantanés utilisés par les [widgets réseau](network-widgets.md)
de l’[ADR 0014](adr/0014-mesures-reseau-observees.md).

## Appariement données / ACK

Le calcul suit uniquement la connexion désignée par `Selection::Selected`.
Les segments de données sortants sont retenus par leur frontière de séquence et
leur horodatage de capture. Un ACK serveur qui avance peut fournir un échantillon
lorsqu’il couvre entièrement un segment dont l’envoi a été observé.
Les ACK sans données serveur sont exploitables. Un acquittement partiel attend
la couverture complète ; les ACK dupliqués ou rétrogrades ne créent aucun RTT.
SYN et FIN occupent chacun un numéro de séquence mais ne fournissent pas
à eux seuls d’échantillon de latence.

Un ACK cumulatif fournit **au plus un RTT**, associé au dernier segment de données
entièrement acquitté parmi ceux encore en attente. Les segments antérieurs
acquittés par le même ACK sont retirés sans créer d’autres échantillons. Ce choix
évite de multiplier des échantillons corrélés pour un même acquittement ; il ne
prétend pas identifier le paquet ayant déclenché la décision d’acquitter du serveur.
Le RTT inclut donc notamment le délai des ACK différés.

Une répétition ou un chevauchement des séquences déjà observées rend l’appariement
ambigu. Le calcul exclut tous les ACK jusqu’à celui couvrant la frontière des
données en vol au moment du chevauchement, cet ACK compris. Une retransmission
partielle avec une nouvelle fin est également exclue. Une retransmission d’un
préfixe déjà acquitté peut déclencher un ACK cumulatif : s’il reste des données en
attente, celles-ci sont aussi protégées par cette exclusion. Sans donnée en attente,
un ancien doublon acquitté ne recrée aucun envoi exploitable. Les nouvelles données
acquittées après la frontière peuvent de nouveau fournir des RTT. Une retransmission
seule n’est pas assimilée à une interruption du collecteur.

Cette exclusion conservatrice applique le principe de Karn de la
[RFC 6298, section 3](https://www.rfc-editor.org/rfc/rfc6298.html#section-3).
Les comparaisons de séquences et d’ACK utilisent l’espace cyclique u32, selon la
[RFC 9293, section 3.4](https://www.rfc-editor.org/rfc/rfc9293.html#section-3.4).
Un écart exactement égal à la moitié de cet espace est indécidable et invalide le
suivi. Un saut des séquences sortantes ou un ACK couvrant des octets jamais observés
signale une continuité compromise ; aucun RTT n’en est déduit. Une barrière d’ACK
permet de dépasser les octets inconnus avant de reprendre les appariements.

## Fenêtre et formules

Pour chaque échantillon, `RTT = timestamp_capture_ACK − timestamp_capture_envoi`.
La différence doit être strictement positive, représentable et fondée sur des
horodatages non manquants, non rétrogrades et non futurs à l’instant de traitement.
Les timestamps TCP et les dates des ticks serveur ne remplacent jamais ces heures.
Les limites de l’horodatage Winsock pris après `recv` restent celles de #48.

La fenêtre glissante vaut dix secondes par défaut. Un RTT appartient à la fenêtre
selon **l’heure de capture de son ACK**, convertie sur une horloge monotone ancrée
au démarrage. Les ACK en attente dans une file ne rajeunissent donc pas les mesures.
Un échantillon dont l’âge atteint la durée de fenêtre est retiré.

Avec les `n` RTT de la fenêtre, dans l’ordre des ACK :

- ping = `somme(RTT_i) / n`, disponible dès le premier échantillon ;
- jitter = `somme(abs(RTT_i − RTT_(i−1))) / (n−1)`, disponible à partir de deux
  échantillons. Les deux membres de chaque paire doivent encore être dans la fenêtre.

Les moyennes utilisent une somme entière en nanosecondes et un arrondi inférieur
à la nanoseconde. `None` signifie indisponible et devra être affiché « — ».
Un jitter nul entre RTT égaux est valide ; aucun RTT nul n’est fabriqué à partir
de deux horodatages identiques. Une fenêtre vide n’efface pas les envois en attente :
un ACK fiable reçu douze secondes après son envoi peut fournir un RTT de douze
secondes dans la nouvelle fenêtre. Cette valeur n’est pas une durée de freeze.

## Réinitialisation et récupération

Changer de connexion, perdre sa sélection ou entrer en ambiguïté efface les
échantillons et les appariements. Le jitter ne relie jamais deux connexions.
Une connexion qui démarre normalement n’attend pas une fenêtre pleine.

Une interruption de capture, un trou détecté, des temps incohérents ou une limite
de ressources effacent le calcul et imposent une fenêtre complète exploitable
avant publication. Les frontières de séquence et d’ACK de la même connexion
restent connues, afin qu’une retransmission observée avant le défaut ne puisse
pas devenir un nouvel envoi exploitable après la remise à zéro. Les échantillons
fiables recueillis pendant cette période
alimentent déjà la prochaine fenêtre, mais restent masqués. Un nouveau défaut
recommence l’attente. Si la connexion est absente ou ambiguë, le temps écoulé ne
satisfait pas cette attente : elle repart lors d’une sélection unique. Des paquets
capturés avant une réinitialisation ne peuvent pas reconstruire la fenêtre par
simple livraison tardive.

`Latency::set_window` accepte une durée strictement positive jusqu’à quinze minutes.
Une nouvelle durée efface les échantillons et les envois en attente ; si une
récupération est en cours, elle redémarre avec cette durée. La persistance et les
commandes debug sont intégrées à la configuration des [widgets réseau](network-widgets.md).

## Collecte et ressources

`Latency::observe` consomme les événements de #48. `snapshot` permet une lecture
immédiate ; `snapshot_if_due` produit le premier instantané immédiatement puis au
plus un par seconde. Il faut continuer à l’appeler pendant les silences pour
faire expirer les données. Les abonnements, la cadence et le cycle de vie des
widgets passent désormais par le [collecteur partagé](network-statistics.md),
`capture::statistics::subscribe`. L’ancien propriétaire autonome `latency::Source`
a été remplacé pour éviter des calculs dupliqués entre widgets.

Le calcul retient au plus 4 096 envois en attente et 4 096 RTT dans la fenêtre,
sans corps de paquet ni base SQLite. Dépasser une limite rend les mesures
indisponibles et impose une nouvelle fenêtre exploitable ; aucun échantillon
n’est éliminé silencieusement au prix d’une moyenne biaisée. Aucun délai maximal
arbitraire ne retire un envoi en attente : les bornes en nombre, les reconnexions
et les interruptions suffisent à borner la mémoire.

Une capture passive peut manquer des retransmissions ou connexions entières.
Ces calculs excluent les ambiguïtés **observables** ; ils ne corrigent pas une
perte du collecteur non signalée et ne qualifient pas la précision réelle de
l’horloge de lecture Windows. La qualification sur un trafic de jeu réel reste
nécessaire avant toute promesse de précision.

## Vérification

Les tests couvrent moyennes et jitter, ACK cumulés/partiels/dupliqués/rétrogrades,
retransmissions complètes et partielles, bouclage u32, SYN/FIN, temps absents ou
incohérents, ACK tardif après expiration, vieillissement d’une file, reconnexion,
ambiguïté, interruption longue et changement de fenêtre. Un test passe par de
vrais octets IPv4/TCP, la reconnaissance par trois ticks et un ACK après douze
secondes de silence. Les limites de mémoire et la cadence sont testées sans
attente réelle. Les consommateurs métier existants conservent leurs tests
antérieurs et leur expiration de silence.

## Dernière mesure, maximum et attente TCP

Le widget conserve la moyenne définie plus haut et lui adjoint quatre diagnostics :

- `last_rtt` : le dernier RTT fiable. Un seul échantillon supplémentaire est
  conservé, même après expiration de la fenêtre, accompagné de son âge.
- `max_rtt` : le plus grand RTT fiable encore présent dans la fenêtre configurée.
  Il expire avec les échantillons ; il ne s’agit pas d’un maximum de session.
- `last_sample_age` : temps monotone écoulé depuis l’heure de capture de l’ACK
  du dernier échantillon fiable, mis à jour à chaque instantané.
- `awaiting_ack` : temps écoulé depuis l’envoi observé le plus ancien dont les
  données n’ont pas encore été entièrement acquittées. Ce délai peut croître
  sans produire de RTT et peut porter sur un envoi devenu ambigu après
  retransmission. Ce n’est ni un RTT confirmé ni une mesure de pertes.

Le dernier RTT et son âge sont effacés au changement de connexion ou de fenêtre,
ou à l’invalidation de la capture. Les diagnostics sont indisponibles pendant
la récupération ; ils ne réintroduisent aucun appariement rejeté. Une attente
TCP ne modifie ni la moyenne ni le jitter, et aucun seuil d’alerte n’est ajouté.
L’actualisation reste celle du collecteur partagé, environ chaque seconde.

Ces informations répondent au constat des captures `ping-1.png`, `ping-2.png` et
`ping-3.png` : 18/16/19 ms affichés par Aetheris contre 239/329/287 ms par Questlog.
Les captures ne permettent pas d’attribuer cet écart à une cause précise. La
moyenne peut néanmoins masquer un pic : 99 RTT de 15 ms et un RTT de 1 000 ms
produisent une moyenne de 24,85 ms. Le dernier RTT et le maximum montrent alors
1 000 ms. Les tests reproduisent cette situation et distinguent l’attente d’un
ACK de son RTT finalement confirmé. Ils vérifient aussi l’expiration, les
réinitialisations et l’exclusion des retransmissions ambiguës.

Cet ajout améliore la lisibilité ; il ne qualifie pas les horodatages Windows,
ne démontre pas l’équivalence avec le ping Questlog et ne permet pas de déduire
des pertes à partir d’un freeze. Une capture réseau contemporaine de l’incident
reste nécessaire pour examiner l’écart des deux mesures.
