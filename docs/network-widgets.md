# Widgets réseau en jeu

L’issue #52 raccorde `overlay::network` au collecteur partagé
[`capture::statistics`](network-statistics.md). L’Overlay Editor permet d’activer
cinq widgets individuels et un regroupement indépendant, tous désactivés par
défaut. Les mêmes mesures peuvent apparaître simultanément dans les deux formes.
Le regroupement permet d’ajouter et de retirer chaque mesure puis de changer
l’ordre avec Monter et Descendre. Une composition vide ne crée aucun abonnement.

Les choix et l’ordre sont sauvegardés dans la section `network` de `overlay.json`,
ainsi que les six configurations communes `WidgetConfig` : position, dimensions,
zoom, opacité, fond, ancrage de croissance et confidentialité. Les réglages de
présentation apparaissent sous les widgets activés ; les dimensions sont conservées
dans la configuration. Les fichiers antérieurs sans section réseau restent
lisibles et laissent les nouveaux widgets désactivés. La configuration est validée
avant sauvegarde et acquisition ; les mesures répétées au sein du regroupement,
fenêtres hors limites et configurations de présentation invalides sont refusées.

## Valeurs et interprétation

- Ping et jitter : millisecondes, arrondies à l’unité. Le ping inclut la réponse
  TCP et un éventuel retard de lecture de la capture. Un long aller-retour est
  affichable lorsqu’il est finalement confirmé, sans devenir une durée de freeze.
- Débits : octets/s, Kio/s à partir de 1 024 o/s, Mio/s à partir de 1 024 Kio/s.
  Les octets capturés incluent les en-têtes réseau, sur tous les flux attribués
  au processus, indépendamment de la connexion retenue pour les mesures TCP.
- Retransmissions TCP : nombre de segments de données observés comme retransmis,
  suivi de leur pourcentage parmi les segments de données observés dans les deux
  sens. Sans segment, le compteur peut être nul et le pourcentage indisponible.
  Ce pourcentage n’est pas une mesure exacte des paquets perdus sur le trajet.

Les options absentes affichent « — », sans masquer le widget. Une capture inactive
invalide toutes ses valeurs affichées ; une capture fonctionnelle sans trafic
peut donner un débit nul. Aucune durée effective de fenêtre partielle, alerte ou
couleur de seuil n’est ajoutée. Les explications de l’éditeur précisent qu’un débit
seul ne qualifie pas la connexion et que les mesures ne suffisent pas à déterminer
la cause d’un freeze.

## Cycle de vie et réglages debug

Chaque widget actif possède un abonnement aux seuls calculs nécessaires, sur le
même collecteur. Une modification de l’ordre ou de la présentation conserve
l’abonnement. Une modification des calculs demandés acquiert le nouvel abonnement
avant de libérer l’ancien, pour préserver les historiques encore utilisés.
Au retrait d’un widget, son abonnement est libéré immédiatement, même si le thread
de sa fenêtre native détient encore une référence à sa source de texte. Au dernier
retrait, le collecteur efface l’historique ; les consommateurs indépendants ne sont
pas affectés. Le service est retenté automatiquement par le collecteur partagé.

Sous Windows, les fenêtres utilisent le rendu texte commun : clics traversants,
affichage uniquement avec le jeu au premier plan, position relative à sa fenêtre,
zoom, opacités, ancrage et protection des captures selon les mêmes règles de
validation que les autres widgets. L’éditeur lit les mêmes valeurs environ toutes
les 250 ms ; le collecteur les publie environ chaque seconde.

En debug, la section réseau permet de modifier les fenêtres communes de ping/jitter
et de débits (1 à 900 secondes) et des retransmissions (1 à 3 600 secondes), avec
retour aux valeurs par défaut (10, 10 et 900 secondes). Ces réglages sont sauvegardés
même sans widget actif et appliqués à la prochaine activation. Un changement
réinitialise uniquement les calculs concernés. En release, les commandes sont
absentes et les fenêtres par défaut sont utilisées ; les préférences debug restent
conservées dans le fichier pour un lancement debug ultérieur.

## Vérifications

Les tests couvrent unités, ordre, valeurs nulles et indisponibles, compte et
pourcentage distincts, compatibilité des anciens réglages, sauvegarde/relecture,
configuration commune, coexistence des widgets et partage d’acquisition. Ils
vérifient aussi la conservation des historiques pendant les changements de
composition, leur effacement après la dernière désactivation, et le rendu pendant
le silence, l’interruption et la récupération de capture. Les suites des mesures
TCP vérifient les longs aller-retour et les reconnexions.

La compilation Windows valide le raccordement au rendu natif et à GPUI. Une
validation manuelle en jeu reste nécessaire pour l’apparence réelle et les
scénarios de freeze/reconnexion dans l’environnement du joueur.
