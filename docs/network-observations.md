# Observations réseau pour les widgets

Le socle de l’issue #48 prépare les mesures de l’[ADR 0014](adr/0014-mesures-reseau-observees.md).
Il ne calcule encore ni RTT, ni jitter, ni taux de retransmissions et n’active
aucun widget. Son point d’entrée est `capture::observations::Source::subscribe()`.
Le propriétaire appelle `poll` et traite les événements synchrones ; sa destruction
libère son abonnement. La capture reste partagée avec les widgets existants et
Combat Debug, sans accès à SQLite ni conservation des observations par le socle.

## Contrat des événements

- `Traffic` couvre tous les paquets attribués, analysables et orientables, y compris
  UDP et les connexions non sélectionnées. Le volume est `captured_len`, en octets,
  avec les en-têtes fournis par la liaison. En live Windows, il s’agit des octets
  IPv4, en-têtes IP et transport compris, sans en-tête Ethernet. Les paquets
  malformés ou tronqués déclenchent un événement de fiabilité ; leur absence des
  volumes exploitables ne doit pas être interprétée comme un débit nul fiable.
- `Tcp` expose direction, séquence, ACK si son drapeau est présent, longueur des
  données TCP, huit drapeaux et option timestamp éventuelle. Les ACK sans données
  sont conservés. Aucune comparaison de séquence ne doit ignorer le bouclage u32
  ni les octets de séquence occupés par SYN et FIN.
- `SelectionChanged` distingue absence, connexion unique reconnue et ambiguïté.
  La clé inclut génération du processus, interface, endpoints local/distant et
  incarnation. Un SYN différent sur le même tuple crée une nouvelle incarnation ;
  un SYN identique retransmis conserve l’identité. FIN/RST dans les deux directions,
  changement de génération et interruption invalident la sélection.
- `Reliability` signale séparément interruption d’acquisition ou d’attribution,
  discontinuité de numérotation des paquets, troncature, erreur d’analyse,
  direction absente, horodatage absent/incohérent, interruption du flux reconstruit
  et limite de capacité. Un consommateur doit invalider ses agrégats concernés
  avant de traiter les observations suivantes, même si la connexion se
  resynchronise pendant le même appel.

Les événements TCP incluent aussi les connexions non sélectionnées ; seul
`Selected(connection)` autorise les futurs calculs propres au jeu. Les octets
client chiffrés, un port ou une adresse privée ne suffisent pas à choisir le jeu.
La direction repose exclusivement sur `local_address` fourni par la capture.
Deux endpoints partageant cette adresse ne sont pas orientables ici.

Chaque candidat réutilise `capture::widgets::Decoder` avec un consommateur qui
n’accepte aucun message métier. La reconnaissance exige les trois ticks serveur
croissants sur des frontières de trames valides déjà utilisés par les widgets.
Deux candidats reconnus simultanément donnent `Ambiguous`, sans privilégier le
plus récent. Le décodeur conserve ses règles existantes : expiration des trous
bloqués après trois secondes, resynchronisation démontrée par trois ticks,
exclusion TLS et expiration du trafic serveur après dix secondes.

Le suivi est borné à huit connexions candidates, chacune avec les limites du
décodeur partagé : buffers de 1 Mio, au plus 128 segments en attente et budget
de décompression de 8 Mio par livraison. Une neuvième connexion n’évince pas
silencieusement un candidat : elle signale la limite et interdit une sélection
unique pendant dix secondes après sa dernière observation rejetée. Son volume
reste livré via `Traffic`. Aucun événement ni paquet original n’est mis en file
par ce module ; la file partagée existante reste limitée à huit lots.

## Horodatages : étude du chemin de capture actuel

Constats tirés de `capture/windows.rs`, `capture/packet.rs`, du lecteur PCAPNG
et de la reconnaissance des ticks, vérifiés par lecture du code et tests
synthétiques. Ils ne constituent pas une qualification sur une capture réelle
Windows ; aucune précision effective de l’horloge n’a été mesurée dans ce lot.

| Source | Origine et résolution représentée | Limites pour une mesure aller-retour |
| --- | --- | --- |
| Live Windows | `SystemTime::now()` après chaque `recv`, epoch Unix, stockage en nanosecondes (`resolution = 9`) | Heure de lecture utilisateur, pas heure d’arrivée noyau. La précision réelle n’est pas garantie par le stockage en ns. Tampons Winsock, ordonnancement, lecture séquentielle des interfaces et attente d’attribution peuvent retarder ou réordonner la livraison. |
| PCAPNG | Ticks d’origine, résolution base 10 ou base 2 et décalage propres à l’interface ; `timestamp_ns` normalisé lorsqu’il est représentable | Origine et précision dépendent du producteur. Des blocs sans horodatage restent sans horodatage. Une différence entre deux interfaces n’est pas validée par la seule normalisation. |
| Option TCP TS (kind 8) | `TSval` et `TSecr`, deux compteurs u32 de l’hôte TCP | Fréquence de l’horloge distante inconnue, bouclage possible, règles d’écho et négociation à prendre en compte. Les compteurs ne sont ni une date Unix ni des durées en millisecondes garanties. |
| `SERVER_TICK` (`00 36`) | Valeur serveur interprétée comme date en millisecondes, utilisée pour reconnaître trois ticks croissants | Pas de requête locale appariée ni de synchronisation des horloges démontrées. Ce champ n’est pas une mesure de RTT. |

La [RFC 7323, sections 3 et 4](https://www.rfc-editor.org/rfc/rfc7323.html#section-3)
définit l’option TCP timestamp et son écho. Le socle les expose sans convertir
leurs valeurs en RTT. Les futurs appariements données/ACK devront exclure les
retransmissions ambiguës, conformément au principe de Karn décrit dans la
[RFC 6298, section 3](https://www.rfc-editor.org/rfc/rfc6298.html#section-3).

Un horodatage absent reste `None`. Une valeur négative, non représentable sur
l’horloge monotone ancrée au démarrage, future à cet instant de traitement ou
antérieure au dernier horodatage valide de cette connexion est signalée et
retirée de l’observation exploitable. L’absence ne remet pas à zéro la dernière
valeur valide. Des timestamps égaux restent des observations à résolution finie ;
ils ne prouvent pas une durée aller-retour nulle. Aucun timestamp de traitement
ne remplace un timestamp de capture manquant dans `TcpObservation`.

## Continuité et limites Windows

`CaptureBatch::interruption` transporte une suspension non terminale, y compris
sans paquet : processus absent, attribution suspendue (ambiguïté ou échec de
lecture des processus) ou absence d’interface de capture. Ce champ traverse
l’IPC et le hub partagé. Le protocole IPC passe en version 2 : application et
service doivent être mis à jour ensemble ; une version incompatible est refusée
au handshake avec le message de réparation existant.

Une erreur terminale ou la fermeture de l’abonnement invalide également le suivi,
après livraison des paquets complets déjà reçus. Un abonné trop lent est déconnecté
par le hub sans arrêter les autres consommateurs. Le suivi compare les numéros
de paquets, mais une discontinuité indique uniquement des observations manquantes
ou réordonnées, sans en attribuer la cause. Ces numéros sont attribués après le
filtrage au processus : ils ne révèlent pas les pertes antérieures à ce filtrage.

`collector_drops()` renvoie `None` : ce chemin Winsock ne fournit aucun compteur
fiable de pertes du collecteur. Ni un trou de séquence TCP, ni une retransmission,
ni une saturation locale explicite ne doit transformer ce compteur indisponible
en zéro ou en pourcentage de pertes réseau. Les trous détectés par le décodeur
concernent le flux serveur reconstruit ; le socle ne prétend pas détecter tous
les trous, notamment dans la direction client.

Le live reste IPv4. Les instantanés de sockets peuvent manquer des connexions
brèves ; un changement de socket entre deux instantanés peut échapper au suivi.
Une reconnexion sans SYN, sans FIN/RST et réutilisant exactement le même tuple
sans interruption visible ne peut pas être identifiée avec certitude. Une heure
murale ajustée ou des paquets livrés hors ordre peuvent invalider des observations
sans prouver un incident réseau. Ces limites doivent rester visibles dans
l’interprétation des futures métriques.

## Validation

Les tests de `capture::observations` couvrent ACK seuls, séquences, drapeaux et
options TCP, direction inconnue, reconnaissance et ambiguïté, reconnexion et
SYN dupliqué, changement de génération, FIN/RST, limites de capacité, paquets
malformés/tronqués, timestamps manquants/rétrogrades/futurs, trous TCP,
discontinuités de numérotation et interruptions sans paquet. Les tests IPC
vérifient aussi le transport du signal de suspension. Les suites existantes
vérifient toujours le partage de capture, le décodeur des widgets et l’analyse
Combat Debug. Une capture réelle Windows reste nécessaire pour mesurer le biais
de lecture et vérifier les options TCP effectivement négociées avec le jeu.
