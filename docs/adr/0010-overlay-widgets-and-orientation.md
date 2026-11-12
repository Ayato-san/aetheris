# Overlay composé de widgets et première boussole

Statut : accepté

Aetheris proposera un overlay en jeu composé de widgets activables depuis une page **Overlay Editor**, placée sous Accueil sans section. Chaque widget déclare les opcodes dont il a besoin ; les abonnements des widgets actifs partagent la capture et le traitement commun, puis seuls les messages demandés sont décodés et distribués. Désactiver un widget ne doit pas interrompre les autres abonnés. L’overlay utilise l’infrastructure de capture de Combat Debug et ne déclenche pas d’enregistrement permanent automatique ; une capture destinée à l’analyse reste une action explicite.

Ce découpage permet d’ajouter des widgets sans leur donner chacun une capture indépendante. Pour le trafic client chiffré, le filtrage ne peut pas précéder entièrement le traitement cryptographique : les opcodes sont dans les corps chiffrés et l’état RC4 est continu par connexion. Le flux doit donc rester synchronisé avant que les messages destinés aux abonnés puissent être identifiés. Combat Debug ne déchiffre pas actuellement ces corps.

## Premier jalon

Le premier widget est une boussole d’orientation du personnage, distincte de l’orientation de la caméra. Son activation est conservée au redémarrage et démarre la capture nécessaire. Sous Windows, en mode fenêtré ou fenêtré sans bordure, la bande apparaît en haut au centre du jeu et laisse passer les clics. Elle est masquée lorsque le jeu n’est pas au premier plan ou que sa fenêtre ne peut pas être identifiée de façon sûre. L’éditeur signale l’attente du jeu ou l’indisponibilité du service de capture ; un service indisponible ne désactive pas le choix du joueur et la connexion peut être retentée.

La boussole indique « orientation indisponible » en l’absence de donnée fiable, et après trois secondes sans mise à jour fiable ; ce délai est provisoire et sera vérifié en jeu. Aucune direction ni calibration n’est inventée. La première implémentation prépare l’interface et la chaîne de données même si la boussole ne peut pas encore fournir un cap réel. La configuration des widgets (position, taille, couleur), le déplacement et la sauvegarde ou le chargement de configurations sont reportés.

## Source et validation

La source de l’orientation doit d’abord être recherchée par capture passive, sans lecture de la mémoire du jeu. Les messages serveur liés au mouvement sont une piste non confirmée ; le champ d’orientation documenté côté client est lui-même une interprétation à valider et nécessite l’accès au trafic déchiffré. La seule capture ne fournit pas actuellement la clé nécessaire. Si la piste passive échoue, l’accès aux données fera l’objet d’une nouvelle décision avant d’élargir les accès au jeu. Les éléments et limites de cette recherche figurent dans [la note d’analyse](../orientation-passive-research.md).

Une future capture accompagnée d’actions contrôlées en jeu servira à vérifier la source, les changements d’orientation à l’arrêt, la différence avec la caméra, le recul et le déplacement latéral. La calibration des directions cardinales et la qualification de la boussole comme fonctionnelle sont reportées jusqu’à ces essais. Les tests sur données contrôlées peuvent vérifier la chaîne et l’affichage, sans remplacer cette validation.
