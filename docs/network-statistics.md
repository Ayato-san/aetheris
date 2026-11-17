# Débits du processus et collecte réseau partagée

Le module `capture::statistics` implémente l’issue #51 de l’épic #37, selon les
[ADR 0011](adr/0011-widget-subscriptions.md) et [0014](adr/0014-mesures-reseau-observees.md).
Il réunit les [observations TCP](network-observations.md), [ping et jitter](network-latency.md),
[retransmissions observées](network-retransmissions.md) et le nouveau calcul de
débit `capture::throughput`. Ce lot prépare le raccordement des widgets et ne
modifie pas encore leur interface.

## Octets, directions et unités

Les débits couvrent tous les événements `Traffic` analysables et orientables de
la capture attribuée à Aion 2 : TCP et UDP, y compris les connexions non retenues
comme connexion de jeu, les ACK sans données et les segments retransmis.
La direction repose sur l’adresse locale fournie par la capture, jamais sur
un port supposé ou une plage privée. La reconnaissance de la connexion de jeu
n’est pas nécessaire pour mesurer les débits du processus.

Le volume compté est `captured_len`, avec les en-têtes fournis par la liaison.
En live Windows il comprend les en-têtes IPv4 et de transport, sans en-tête
Ethernet. Ce n’est donc ni le seul volume applicatif ni un débit physique du lien.
Un paquet tronqué, une entrée malformée ou une direction inconnue compromettent
la mesure au lieu de contribuer à un faux débit fiable. Les formats non analysables
ne sont pas inventés ; leurs signaux de fiabilité imposent la récupération.

Les deux valeurs publiques sont des **octets par seconde** :
`incoming_bytes_per_second` et `outgoing_bytes_per_second`. Pour une présentation
en Kio/s, le widget divisera par 1 024 ; pour des octets/s, aucune conversion
bits/octets n’est nécessaire.

## Fenêtre glissante et démarrage

La fenêtre vaut dix secondes par défaut. Chaque événement de trafic porte son
horodatage de capture normalisé ; le calcul l’ancre sur l’horloge monotone et
conserve les volumes par date de capture, dans l’ordre des dates. Une attribution
retardée peut livrer un paquet plus ancien après un autre flux sans fausser
l’expiration. Un paquet dont l’âge atteint la durée de fenêtre n’alimente plus
le volume. Une livraison tardive ne rajeunit pas les octets.

Pour chaque direction, le débit est la somme des octets de la fenêtre divisée
par `min(durée depuis le début de la période exploitable, durée de fenêtre)`.
Au démarrage, il utilise donc la durée partielle disponible ; après dix secondes,
le dénominateur est de dix secondes. Le premier lot peut contenir des paquets
capturés avant son traitement : le début de période inclut alors cette partie
initiale observée. Si des octets sont présents mais que la durée est encore
strictement nulle, le débit reste indisponible jusqu’à ce qu’une durée soit
mesurable. Aucun débit infini n’est produit.

Une capture active sans trafic donne zéro, y compris au démarrage lorsqu’aucun
octet n’a encore été observé. Une capture non confirmée ou indisponible donne
`rates = None`, à présenter « — ». Le widget n’a pas à afficher la durée de
capture ni à attendre une fenêtre pleine au démarrage normal.

## Disponibilité et récupération

`observations::Event::CaptureState` décrit la disponibilité de l’acquisition,
indépendamment de `SelectionChanged`, qui ne décrit que la connexion de jeu.
Un lot sain, même vide, peut confirmer une capture active. Une suspension
non terminale d’attribution, un arrêt, une erreur ou une fermeture d’abonnement
invalident la disponibilité. Le collecteur surveille aussi les lots : après deux
secondes sans lot reçu, il déclare l’acquisition indisponible. Cette limite peut
masquer une capture très retardée ; elle évite de transformer une acquisition
bloquée en débit nul. Un silence TCP normal continue à produire des lots vides.

Après interruption ou trou détecté compromettant le volume, les données sont
effacées et le débit reste indisponible jusqu’à une fenêtre complète exploitable.
Une période pendant laquelle la capture reste indisponible ne compte pas ;
l’attente repart à sa reprise. Un nouveau défaut recommence l’attente. Les
anciens paquets en file antérieurs à cette reprise ne peuvent pas reconstituer
artificiellement la période. Les horodatages absents, négatifs ou futurs sont
également indisponibles.

Une sélection de jeu absente ou ambiguë n’interrompt pas les débits du processus.
La saturation des candidats du suivi de connexion n’invalide pas non plus leurs
volumes : `Traffic` est livré avant cette limite. En revanche, un trou signalé
par le décodeur ou une erreur de capture peut compromettre les octets observés
et invalide la fenêtre de débit. Aucun nombre de pertes n’est déduit de ces
événements. `Snapshot::collector_drops` conserve la valeur séparée de #48,
actuellement `None` sur le chemin Winsock, sans la transformer en zéro.

## Abonnements des widgets

Le point d’entrée commun est `capture::statistics::subscribe(interests)`.
`Interests::LATENCY`, `TRAFFIC`, `RETRANSMISSIONS`, `ALL` ou une combinaison de
leurs champs déclarent les besoins de chaque widget. Un ensemble vide est refusé
sans lancer de collecte. Les widgets ping et jitter partagent le calcul de
latence ; les deux débits partagent celui du trafic.

Le premier abonnement démarre un seul worker. Celui-ci possède un abonnement au
hub de capture existant, un suivi des observations et au plus un exemplaire de
chaque calcul demandé. Ajouter un widget à une mesure déjà utilisée ne redémarre
ni son acquisition ni ses agrégats. Lorsqu’une mesure perd son dernier utilisateur,
son calcul et son historique sont libérés sans affecter les mesures restantes.
Avec les seuls débits actifs, le suivi n’effectue pas la reconnaissance et le
réassemblage du protocole de jeu.

Chaque abonné lit `Subscription::snapshot()`, un `Arc` vers l’unique instantané
immuable courant, sans file de paquets ni recalcul par widget. Le worker publie
environ chaque seconde, et plus tôt pour une disponibilité, une erreur ou une
configuration modifiée. Il continue à actualiser pendant les périodes sans
trafic. Une erreur de connexion au service reste visible dans `Snapshot::error`
et laisse les mesures indisponibles ; le worker réessaie environ chaque seconde
tant qu’il reste un abonné.

`Subscription` est clonable ; chaque copie compte comme abonné. Le départ du
dernier ferme le collecteur, retire immédiatement les résultats publiés et
réveille son attente. Le worker libère ensuite ses historiques privés et son
abonnement brut au retour du poll en cours (attente de lot de 100 ms ; une
connexion IPC en cours reste soumise à ses délais existants). Un nouvel abonné
obtient un nouvel historique. Les consommateurs indépendants du hub, notamment
Combat Debug et les widgets métier existants, continuent à recevoir leurs lots.

Les anciens propriétaires autonomes `latency::Source` et
`retransmissions::Source` ont été remplacés par ce point d’entrée partagé. Les
calculateurs purs restent disponibles pour les tests et la composition interne.

## Configuration et ressources

`Subscription::windows()` lit les durées communes demandées. `set_windows`
valide les trois durées avant toute mutation, puis les transmet atomiquement
au worker ; `Snapshot::windows` identifie la configuration effectivement appliquée.
Seuls les calculs dont la durée change sont remis à zéro. Une récupération en
cours repart avec la nouvelle durée. Les réglages sont conservés en mémoire entre
deux périodes sans abonnement ; la persistance sur disque et les commandes debug
seront raccordées avec la configuration des widgets.

Le calcul des débits retient au plus 262 144 dates distinctes, avec deux volumes
u64 par date, sans corps de paquet. Des paquets de même date sont agrégés. Un
dépassement de cette limite ou de la capacité des volumes rend la mesure
indisponible et impose la récupération ; aucun volume n’est abandonné silencieusement
pour publier une valeur biaisée. Les limites de #48, #49 et #50 restent appliquées
aux autres calculs. Il n’existe ni base SQLite, ni conservation illimitée, ni
file par widget dans ce collecteur ; les références d’abonnements sont suivies
par trois compteurs. Aucune collecte ne démarre à la simple présence du module.

## Validation

Les tests couvrent directions, octets/s et Kio/s implicites, fenêtre partielle,
expiration exacte, zéro pendant le trafic nul, disponibilité sans connexion de
jeu, troncatures, horodatages et attribution réordonnée, saturation et récupération.
Les tests du worker vérifient l’acquisition unique, plusieurs abonnés et clones,
le partage d’un compteur sans double comptage, la libération au dernier départ,
la conservation des autres consommateurs, le redémarrage sans historique, la
configuration atomique, l’absence de lots et la reprise après erreur terminale.
Les suites existantes restent utilisées pour les métriques TCP et les widgets
métier. Une qualification sur une capture réelle Windows reste nécessaire.
