# Source locale des Twitch Drops

Le module `twitch_drops` implémente la source commune de l’issue #71, première
étape de #32. Il est indépendant de GPUI et ne crée encore ni calendrier, ni
widget, ni rappel. Le processus principal initialise `service::shared()` après
avoir acquis le verrou de l’application. Un seul worker possède la connexion
SQLite et le client HTTP ; les consommateurs lisent `Service::snapshot()`.
Les modifications de `UserState` passent par une file avec accusé de sauvegarde.

## Contrat et validation réelle

Références consultées le 17 novembre 2026 :

- [Documentation du fournisseur](https://github.com/SunkwiBOT/twitch-drops-api)
- [Types de réponse](https://github.com/SunkwiBOT/twitch-drops-api/blob/main/types.ts)
- [Endpoint interrogé](https://drops.sunkwi.com/api/twitch/drops)
- [Snapshot du fournisseur](https://github.com/SunkwiBOT/twitch-drops-api/blob/main/drops.json)

Le GET réel a répondu 200 à 15:44:09 UTC, avec `refreshedAt` et
`x-last-updated-at` égaux à `2026-11-17T15:44:01.154Z`.
Les en-têtes annonçaient `RateLimit-Limit: 1`, `RateLimit-Remaining: 0`,
`RateLimit-Reset: 300` et `RateLimit-Policy: 1;w=300`, avec un ETag.
Le contrat documente une requête anonyme par cinq minutes et par IP, des 304
gratuits et `Retry-After` en secondes sur les 429. Aucune clé n’est embarquée.
Les tests de 429 utilisent un serveur local et ne saturent pas le fournisseur.

Une campagne AION 2 était présente : « Ludra Awakens », statut `ACTIVE`,
ID `fab904fd-96c0-4b4c-8775-d2cb829f4563`, du 4 novembre à 14:00 UTC au
18 novembre à 07:59:59.999 UTC. Le snapshot du dépôt, scanné à
`2026-11-17T07:38:02.075Z`, contient le même ID. Cette comparaison confirme
la continuité sur deux scans ; le fournisseur ne garantit pas contractuellement
la stabilité éternelle ni l’absence de réutilisation. L’identité locale reste
donc le couple source Sunkwi / ID de campagne, jamais un titre ou des dates.
Un ID dupliqué invalide la réponse plutôt que de fusionner deux entrées.

La récompense `Rare Theostone Chest x2` porte le nom de palier
`Rare Theostone Chest x1` : le nom du bénéfice est prioritaire et les deux
sont conservés. Les métadonnées brutes conservent descriptions, images, liens,
conditions de chaîne, préconditions et champs inconnus, y compris les drops
événementiels dont le format n’est pas documenté. Une liste de récompenses
absente reste `None`, distincte d’une liste vide. Aucun champ de progression
du compte n’est ajouté.

La source ne documente pas de région. Les campagnes restent en région non
confirmée ; ni le nom du jeu, ni les URL, ni une trame réseau ne la déterminent.
Une décision explicite de région dans `UserState` doit comporter une
justification. Les campagnes marquées incompatibles ne sont pas actives pour
les consommateurs européens ; leur historique reste disponible.

## Persistance et états

`twitch-drops.sqlite3` vit dans le dossier de données personnel du compagnon.
Le schéma utilise les migrations et sauvegardes personnelles existantes.
Le snapshot est écrit atomiquement, avec les campagnes observées et l’état
de la source. Les décisions utilisateur ont une table indépendante, conservée
même après la purge d’une campagne : une réapparition récupère le masquage,
la région et les identifiants de rappels déjà délivrés. Ces derniers sont un
support de persistance pour #74 ; aucun rappel n’est émis par #71.

Les dates sont normalisées en millisecondes Unix UTC. Une période absente,
invalide ou inversée invalide toute la réponse, sans modifier les présences.
Chaque campagne conserve son premier et dernier téléchargement d’observation
ainsi que la date du scan source correspondant. L’historique est conservé
365 jours après sa fin ; les petites entrées d’état utilisateur ne sont pas
purgées avec lui.

`refreshed_at` détermine la fraîcheur ; `downloaded_at` et `last_checked_at`
restent distincts. Un scan absent, invalide ou situé dans le futur donne une
fraîcheur inconnue. Un scan plus ancien que le dernier scan connu est rejeté.
À 48 heures exactement, widget et rappels deviennent inadmissibles ; le
calendrier peut continuer à lire les campagnes datées. Ce calcul se fait à
la lecture et expire donc aussi hors ligne, sans attendre le worker.

Une campagne active doit être présente, avoir le statut exact `ACTIVE`, une
période `début <= maintenant < fin`, une région admissible et un scan frais.
Le masquage est un filtre supplémentaire du widget et ne change pas l’activité
ni les rappels. Une disparition dans une liste valide et fraîche passe
`present` à faux : elle signifie « plus présente dans la source », jamais
« annulée ». Les erreurs et scans périmés ne déclarent pas de disparition.
`Snapshot::state()` distingue chargement, absence, disponibilité, péremption,
fraîcheur inconnue et indisponibilité initiale. `last_error` expose séparément
une panne alors qu’un cache reste utilisable.

## Réseau et reprise

Le worker s’active au démarrage et respecte l’échéance persistée, y compris
après un redémarrage : relancer l’application ne contourne pas un 429.
Il consulte ensuite la source toutes les heures. Une requête expire après
20 secondes, la connexion après 10 ; la réponse est limitée à 8 Mio et les
redirections sont refusées. Une erreur entraîne une reprise progressive de
10 minutes à une heure, avec un décalage aléatoire inférieur à une minute.
`Retry-After` (secondes ou date HTTP), `RateLimit-Reset` et `rateLimit.resetAt`
peuvent repousser l’échéance au-delà de cette reprise. Cela permet de céder
aux autres installations partageant l’IP, sans coordination entre elles.

L’ETag est envoyé tant que le scan connu restera frais à la prochaine échéance
horaire. Un 304 ne change jamais la date du scan ni celle du téléchargement.
À l’approche de l’expiration, une requête sans condition récupère le corps et
sa date de scan : une liste inchangée ne peut donc pas bloquer indéfiniment
le renouvellement de fraîcheur. Une erreur ne renouvelle jamais la fraîcheur.

## Validation reproductible

Les tests ne contactent pas Sunkwi. La fixture est un extrait de la réponse
réelle ci-dessus, contenant le jeu AION 2 et un autre jeu pour le filtrage.
Les variations synthétiques couvrent campagnes simultanées, changement de
titre/dates, champs inconnus, périodes invalides, réponse vide, disparition,
réapparition, restauration SQLite, rétention, région et expiration hors ligne.
Un serveur HTTP local vérifie les statuts, ETag, erreurs et délais de reprise.

```sh
cargo test --locked --lib --no-default-features twitch_drops
```

La validation réelle est volontairement séparée des tests. Pour la renouveler,
effectuer un GET anonyme en conservant corps et en-têtes, après au moins cinq
minutes sans requête depuis cette IP, et respecter `Retry-After` si nécessaire.
Comparer les IDs AION 2 à un autre snapshot daté ; ne pas conclure à une région
européenne à partir des liens ou des noms.
