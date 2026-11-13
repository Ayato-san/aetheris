# Abonnements et traitement partagés des widgets

Statut : accepté

Tous les widgets d’overlay actifs déclarent les opcodes et, lorsque c’est possible, les entités nécessaires à leurs fonctionnalités. Ils partagent la capture et les étapes communes de traitement ; seuls les messages et entités nécessaires aux abonnés actifs font l’objet de l’analyse spécifique et de la distribution correspondantes. Cette règle limite le travail réalisé quand des widgets sont désactivés, sans multiplier les captures indépendantes.

Un identifiant d’entité ne peut servir de filtre qu’une fois connu : les messages de découverte nécessaires restent donc analysés selon leurs opcodes, puis les abonnements ou filtres d’entités sont mis à jour au fil du suivi. Une entité retirée ou dont le suivi est réinitialisé ne conserve pas de suivi inutile. Les besoins communs à plusieurs abonnés restent actifs tant qu’au moins l’un d’eux en a besoin ; désactiver un widget libère ses propres abonnements sans interrompre les autres consommateurs, y compris Combat Debug. Les files, ressources et données de suivi doivent rester bornées et être libérées lorsque leur dernier utilisateur se désabonne.

Le filtrage intervient au plus tôt là où les informations nécessaires sont disponibles, sans promettre d’éviter les étapes indispensables à la lecture du flux. En particulier, pour le trafic client chiffré, les opcodes sont dans les corps chiffrés et l’état RC4 est continu par connexion : le flux doit rester synchronisé avant identification des messages. Combat Debug ne déchiffre pas actuellement ces corps. L’existence de cette décision ne signifie pas que les abonnements par opcode ou entité sont déjà raccordés ; leur mise en œuvre doit préserver ces contraintes et être vérifiée.
