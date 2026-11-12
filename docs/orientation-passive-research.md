# Orientation du personnage par capture passive

Recherche du 12 novembre 2026. Périmètre : client officiel Windows et capture réseau passive, sans lecture de mémoire, injection ni substitution de client. Les sources communautaires ci-dessous décrivent leurs propres observations ; elles ne constituent pas une validation sur les captures ou la version du jeu de l'utilisateur.

## Résultat

Aucune méthode passive documentée et vérifiée dans les sources consultées ne fournit aujourd'hui une orientation continue et attribuée au personnage local. Cela ne démontre pas qu'elle soit impossible : des messages serveur en clair liés au mouvement existent et restent une piste à examiner par captures contrôlées. Leur nom ne démontre ni qu'ils concernent le joueur local, ni qu'ils contiennent son angle, ni leur fréquence.

## Trafic client

La [recherche de protocole, révision figée fe2122f](https://github.com/lunixxx37/aion2-protocol-research/blob/fe2122f0e54e1de854c11e91e3c7bf51916a4f52/docs/PROTOCOL.md#52-server-to-client-11-36) décrit une paire RSA éphémère par connexion, le transfert de la clé RC4 par RSA-OAEP et le chiffrement continu des corps C2S. La clé publique et le bloc chiffré capturés ne fournissent pas directement la clé privée ou la clé de session. L'auteur indique qu'un client autonome pourrait posséder sa propre clé privée, ce qui ne résout pas l'observation passive du client officiel. Les longueurs restent en clair mais les opcodes sont dans les corps chiffrés. Une capture commencée en cours de connexion ajoute le problème de la synchronisation de l'état RC4.

Le [registre d'opcodes figé](https://github.com/lunixxx37/aion2-protocol-research/blob/fe2122f0e54e1de854c11e91e3c7bf51916a4f52/docs/OPCODES.md#01-37--client_movement_update) attribue aux messages C2S `00 37` et `01 37` un angle `facing_heading_degrees` au décalage 17 du corps ; son statut est **inferred**, pas confirmé. Il distingue l'angle de déplacement de l'orientation. Il décrit aussi une forme d'arrêt de `01 37` et d'autres familles, notamment aériennes, qui demanderaient leur propre prise en charge. Il ne démontre pas une mise à jour continue lors d'une rotation immobile ni l'orientation de la caméra. Ces champs ne sont pas accessibles à partir du seul trafic chiffré avec le décodeur actuel.

## Piste serveur

Le même [registre](https://github.com/lunixxx37/aion2-protocol-research/blob/fe2122f0e54e1de854c11e91e3c7bf51916a4f52/docs/OPCODES.md) nomme `1A 37` à `1D 37` comme messages S2C liés au mouvement, avec une confiance *parser-based*, et `33 36` comme état du joueur local. Il ne documente pas pour ces messages un champ d'orientation ni une attribution des mises à jour de mouvement au joueur local. `03 38` est nommé `NPC_POSITION`, ce qui ne suffit pas à en faire une source de direction du joueur. Le trafic S2C est décrit comme lisible après handshake, avec des enveloppes LZ4 possibles ([protocole](https://github.com/lunixxx37/aion2-protocol-research/blob/fe2122f0e54e1de854c11e91e3c7bf51916a4f52/docs/PROTOCOL.md#53-post-handshake-c2s-cipher)).

Dans Aetheris, le [catalogue](../src/combat_debug/opcode.rs) référence cette même révision et [l'interpréteur](../src/combat_debug/interpretation.rs) ne décode actuellement que l'horodatage `SERVER_TICK`. Le [bilan des captures locales](protocol-decoding.md#validation-effectuée) rapporte notamment 46 `NPC_POSITION` pour une capture, sans validation de leurs champs ou de leur acteur ; il ne prouve pas la disponibilité d'une orientation locale. Le mode live et les imports partagent l'analyseur, mais la validation en jeu reste nécessaire ([état de l'implémentation](protocol-decoding.md#état-de-limplémentation)).

## Expérience proposée

Réaliser une capture passive horodatée avec actions contrôlées et version du client identifiée : immobile, rotation du personnage par paliers connus, caméra seule, marche avant, recul et strafe, puis reconnexion. Comparer les messages S2C directs et décompressés, notamment `33 36` et `1A`–`1D 37`, à ces marqueurs et aux identifiants d'acteurs. Répéter avec et sans autres joueurs à proximité pour distinguer leurs mouvements. Conserver les corps et leur provenance et vérifier les valeurs candidates, le sens, le nord et la fraîcheur ; une absence d'émission ou une valeur ambiguë ne doit pas être présentée comme un cap fiable. Des captures négatives dans ces conditions limitent la piste pour les scénarios testés, sans prouver une impossibilité générale.
