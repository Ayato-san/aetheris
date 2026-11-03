# Décodage AION 2 : preuves et cadrage du premier jalon

État au 3 novembre 2026 : jalon partiellement implémenté dans le décodeur
`aion2-experimental-4`. L’[ADR 0006](adr/0006-compressed-message-provenance.md)
est implémenté ; la validation terrain et plusieurs contrôles d’exploration
restent à réaliser. Les décisions Q1 à Q13 sont conservées plus bas comme
historique du cadrage, avec l’évolution autorisée ensuite pour les captures
démarrées en cours de connexion.

## État de l’implémentation

Le même analyseur sert les imports PCAPNG, leur réanalyse et le live Windows.
Les modifications réalisées sont les suivantes :

- **Frontières :** longueurs ULEB128 minimales, positions relatives au corps,
  padding nul entre messages ; les erreurs arrêtent la branche affectée.
- **Compression et provenance :** enveloppes LZ4 normales et marquées F0..FE,
  imbriquées, avec une ligne par message interne. Les positions dans le flux
  TCP et dans la sortie décompressée d’une enveloppe sont distinctes. Le détail
  expose la chaîne des enveloppes et les paquets sources.
- **Conservation :** migrations SQLite pour la provenance et l’indexation des
  sections ; réanalyse transactionnelle préservant les paquets originaux.
  Les analyses anciennes doivent être réanalysées ; l’interface le signale.
- **Catalogue :** les 47 opcodes S2C nommés sont représentés par une énumération
  Rust avec `Unknown([u8; 2])`, nom documentaire, traduction facultative,
  phase, source et confiance. Les traductions sont vides initialement.
  Le registre utilisé est figé à la révision
  `fe2122f0e54e1de854c11e91e3c7bf51916a4f52`, référencée dans
  [opcode.rs](../src/combat_debug/opcode.rs).
- **Interprétation et affichage :** messages directs et décompressés partagent
  le même interpréteur. La table affiche code et nom ; le filtre accepte
  l’hexadécimal compact, espacé ou le nom documentaire. Le détail conserve
  source et confiance communautaires, distinctes d’une confirmation locale.
  « Identifié » signifie que l’opcode est nommé ; « décodé » s’applique ici
  à l’horodatage `unix_ms` de `SERVER_TICK`, dont la taille est vérifiée.
  Les autres corps restent bruts. L’absence d’opcode reste distincte d’un
  opcode inconnu. Les anciennes extractions d’entité, HP et dégâts sont retirées.

### Évolution autorisée : démarrer après la connexion

L’utilisateur a demandé d’appliquer une synchronisation automatique en cours de
connexion. Cela complète le cadrage initial qui exigeait le SYN ou un
alignement offline imposé. L’analyseur reconnaît trois messages serveur
`00 36` contenant des horodatages croissants, reliés par des frontières de
messages valides. Il décode à partir du premier de ces ticks et signale la
synchronisation déduite dans l’analyse. La direction provient des métadonnées
locales ou est déduite des ticks ; elle reste distincte de la frontière reconnue.

La recherche porte sur les premiers 256 Kio du flux réassemblé, avec un budget
maximal d’un million d’étapes et 512 messages de confirmation par candidat.
Les horodatages doivent être compris entre 2000 inclus et 2100 exclu et espacés
de 1 à 60 000 ms. Les octets antérieurs restent bruts. Une reconnaissance
ambiguë dans les deux directions ou une connexion TLS identifiée empêche le
décodage AION 2. Les trous TCP et les contradictions n’entraînent aucune
recherche de reprise après l’erreur. L’alignement expérimental imposé reste un
repli offline explicite pour les flux non synchronisés ; il ne déchiffre pas
les corps client. Le déchiffrement C2S reste hors du périmètre réalisé.

### Limites de compression

La sortie d’une enveloppe est limitée à 8 Mio, avec un ratio maximal de 256,
quatre niveaux de décompression, et des budgets cumulés par enveloppe racine
de 32 Mio décompressés et 65 536 messages dérivés. La taille réelle doit
correspondre à la taille annoncée. Les enveloppes en erreur et les suffixes
décompressés non interprétés restent inspectables.

### Validation effectuée

La suite automatisée exécutée compte 78 tests réussis ; le test dépendant des
captures locales est ignoré dans cette suite et a été exécuté séparément avec
succès. Elle vérifie notamment le catalogue complet, les noms et offsets des
messages directs et décompressés, l’invariance au découpage TCP, les erreurs,
les limites de reconnaissance, les migrations et la conservation des preuves.
La compilation avec GUI, Clippy sans fonctionnalités par défaut et les contrôles
de formatage passent également.

| Capture locale | Messages serveur reconstruits | Opcodes documentés distincts | Messages d’opcode inconnu | Enveloppes LZ4 reconnues |
|---|---:|---:|---:|---:|
| `capture.pcapng` | 1 971 | 11 | 202 | 0 |
| `capture-2.pcapng` | 406 | 11 | 42 | 0 |

Ces résultats proviennent du rejeu, pas d’une attente fondée sur les anciens
compteurs. Par exemple, la première capture expose 110 `ACTION_OR_CAST`,
46 `NPC_POSITION` et 56 `COMBAT_BATCH`. Les paquets originaux restent identiques
après réanalyse. L’absence de LZ4 dans ces deux captures signifie que la
compression et sa provenance sont validées sur des vecteurs synthétiques,
pas encore sur une enveloppe observée dans une session réelle complète.

Commandes utilisées :

```sh
cargo test --locked --no-default-features --lib
cargo test --locked --no-default-features reference_captures -- --ignored --nocapture
cargo check --locked --all-targets
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo fmt --check
```

### Travail restant

- Contrôles offline d’attribution manuelle du client et de sélection
  « AION 2 supposé », distincts du bouton d’alignement déjà disponible.
- Identification explicite des connexions lobby et monde par leurs handshakes.
  Le SYN donne une frontière TCP, sans valider à lui seul le protocole AION 2.
- Interprétation des corps autres que l’horodatage `SERVER_TICK`. Un nom de
  message ne confirme aucun champ de dégâts, de HP ou d’attribution à un acteur.
- Vérification manuelle du live Windows, capture complète depuis la connexion,
  et confirmation par actions contrôlées en jeu. La révision du client des
  captures locales reste à identifier.

## Cadrage initial et sources

Les sections suivantes conservent les constats et décisions antérieurs à
l’implémentation. L’état actuel et l’évolution de synchronisation sont décrits
ci-dessus ; les écarts historiques ne décrivent plus tous le code actuel.

## Synthèse du jalon convenu

Le premier jalon fournit une inspection structurelle du trafic AION 2
serveur → client dans la page Combat · Debug, par le même décodeur offline
et live. Il reconstruit les messages avec une longueur ULEB128, ouvre les
enveloppes LZ4 bornées et affiche chaque message interne avec sa provenance.
Les corps bruts et les opcodes inconnus sont conservés. Les extractions
actuelles d’entité, HP et dégâts sont retirées du nouveau décodeur.

Le catalogue prend la forme d’une énumération Rust avec les codes sur le fil,
les noms documentaires exacts et des traductions françaises facultatives
maintenues dans le code. L’affichage privilégie la traduction renseignée,
sinon le nom documentaire ; le code hexadécimal reste visible et filtrable.
La variante inconnue préserve les deux octets ; l’absence d’opcode reste
distincte. Le détail permet de consulter le nom documentaire, sa source et
sa confiance même lorsque le nom affiché est traduit.

Le mode strict demeure le défaut. Pour les anciennes captures offline,
attribution manuelle du client, sélection « AION 2 supposé » et alignement
initial expérimental restent trois informations distinctes et traçables.
L’acquisition reste passive ; la lecture mémoire et le déchiffrement C2S
sont reportés. Les connexions TLS identifiées sont exclues du décodeur AION 2.

Une erreur arrête la branche affectée sans inventer une nouvelle frontière ;
les résultats antérieurs et les octets non interprétés restent consultables.
La validation porte sur des résultats de référence, l’invariance au découpage
TCP, la décompression et la provenance. Les captures actuelles permettent
l’exploration et la régression. Une capture complète depuis la connexion et
une vérification manuelle Windows restent nécessaires pour la validation
sur une session réelle complète et pour le live, respectivement.

Les décisions Q1 à Q13 et cette synthèse sont confirmées par l’utilisateur.
L’arbre de conception du jalon est épuisé et l’entretien est terminé.
L’état réalisé et les validations restantes sont désormais recensés dans
la section « État de l’implémentation » ci-dessus.

## Sources et portée

Les sept sources communautaires fournies ont été consultées :

- [SOURCES.md](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/docs/SOURCES.md) : provenance et niveaux de preuve ;
- [PROTOCOL.md](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/docs/PROTOCOL.md) : transport, framing, compression et chiffrement ;
- [OPCODES.md](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/docs/OPCODES.md) : catalogue et champs connus ;
- [GETTING_STARTED.md](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/docs/GETTING_STARTED.md) : utilisation et prérequis des outils ;
- [C2S_RESEARCH.md](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/docs/C2S_RESEARCH.md) : recherches et expériences C2S ;
- [watch_session_key.py](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/tools/watch_session_key.py) : récupération de clé dans le processus ;
- [analyze_pcaps.py](https://raw.githubusercontent.com/lunixxx37/aion2-protocol-research/refs/heads/main/tools/analyze_pcaps.py) : inspection statistique des captures.

Ces liens suivent une branche mutable. Le protocole décrit un instantané
d’octobre 2026, client Global Windows révisions 3526/3527. Une validation dans
ce dépôt devra identifier la révision de ses propres captures. Une observation
« locally-confirmed » chez l’auteur reste une preuve externe pour Aetheris.
Les nombres de sessions diffèrent entre les instantanés documentaires ; ils
ne constituent pas un critère de validation de notre décodeur.

## Chaîne de décodage décrite par les sources

1. Reconstruire TCP séparément par connexion et direction, en dédupliquant
   les retransmissions et en signalant les trous ou contradictions.
2. Distinguer les connexions lobby, monde et TLS ; le port monde observé
   13328 ne remplace pas l’identification de la connexion.
3. Lire une longueur ULEB128 minimale `L` de largeur `w` : le corps mesure
   `L - 4` octets et le message physique `w + L - 4` octets. Le décalage de
   quatre est logique, pas un en-tête physique. Les décodeurs cités traitent
   aussi un padding nul entre messages.
4. Après le handshake monde, interpréter S2C en clair. Pour C2S, la longueur
   reste claire mais le corps est chiffré : ses deux premiers octets ne sont
   donc pas encore un opcode lisible.
5. En S2C, reconnaître les bundles `FF FF | taille:u32le | bloc LZ4 brut`
   ainsi que la variante précédée d’un marqueur F0..FE. Leur sortie contient
   des messages complets à analyser à nouveau. Borner taille, ratio et profondeur.
6. Décoder les champs selon l’opcode, avec des positions relatives au corps
   et une confiance propre à chaque interprétation. `04 38` est l’ordre sur
   le fil ; sa représentation numérique u16 LE est `0x3804`.

Le handshake `10 36` / `11 36` établit, selon les sources, une clé RC4 de
214 octets issue de RSA-2048 OAEP-SHA1. L’état RC4 est continu par connexion
sur les seuls corps C2S, sans réinitialisation par message ni RC4-drop.
Une reconnexion change la clé. La clé publique et le bloc RSA capturés seuls
ne donnent pas la clé RC4 du client officiel.

Le watcher lit la mémoire du processus et récupère une clé, sans fournir
un état RC4 synchronisé avec un point quelconque d’une capture. Il faut aussi
retrouver le début du flux chiffré ou disposer d’un état synchronisé équivalent.
Une clé d’une nouvelle connexion ne déchiffre pas une ancienne capture.
`analyze_pcaps.py` fournit des observations statistiques ; il ne réalise pas
le déchiffrement C2S. Le zlib de `40 8D` est un contenu spécifique à cet opcode
après déchiffrement, distinct des bundles LZ4 externes.

## Écart avec Aetheris au moment du cadrage

| Sujet | État du dépôt | Conséquence |
|---|---|---|
| Réassemblage TCP | Déjà présent, conservateur, avec provenance des paquets | Socle réutilisable |
| Framing | `analysis.rs` lit un octet et soustrait 3 | Équivalent à la formule externe seulement pour une longueur sur un octet ; incorrect pour une longueur multioctet |
| Opcode | Positions fixes `bytes[1..3]`, sans contexte cryptographique | À rendre relatif au corps ; C2S chiffré ne doit pas être nommé comme un opcode clair |
| Compression | Aucun LZ4 ni zlib | Messages internes aux bundles encore inaccessibles |
| Sémantique | Deux opcodes candidats, entité sur trois octets, HP à position fixe, recherche de `E6 64` | Hypothèses à tester ; pas une grammaire de dégâts validée |
| Stockage | Octets et provenance conservés ; réanalyse transactionnelle | Permet de corriger les interprétations sans remplacer les preuves |
| Captures de référence | Deux fichiers locaux sans SYN, alignement expérimental explicite | Les décomptes actuels mesurent une reproductibilité, pas la justesse des frontières |

Dans les imports PCAPNG actuels, aucune adresse locale n’est attribuée aux
paquets. La direction des messages dépend du handshake TCP capturé : sans
SYN, les messages restent donc de direction inconnue, même avec l’alignement
expérimental. Une attribution explicite ou une autre preuve sera nécessaire
pour appliquer le chemin S2C à ces captures.

Le stockage relie déjà chaque message aux paquets sources, mais ne représente
pas encore la relation entre une enveloppe compressée et ses messages internes.
Un offset dans des octets décompressés ne doit pas être présenté comme une
position dans le paquet brut. Ces constats motivent les questions du second tour.

Repères : [framing actuel](../src/combat_debug/analysis.rs),
[tests](../src/combat_debug/tests.rs),
[contrat de capture](adr/0003-debug-combat-capture.md).

Le catalogue attribue notamment `00 8D` aux HP, `04 38` aux dégâts directs,
`05 38` aux dégâts périodiques et `3D 38` à un batch de combat. Ces noms
n’établissent pas à eux seuls une grammaire complète des champs, l’attribution
à un acteur, ni l’égalité entre montant encodé, variation de HP et montant
affiché. Le glossaire existant distingue déjà ces trois notions.

En particulier, ces sept fichiers ne donnent pas la disposition des champs
de dégâts de `04 38` : la recherche actuelle de `E6 64` ne peut pas être
validée avec ce seul corpus. Il faudra une source complémentaire décrivant
ces champs et une confrontation aux observations locales.

## Décisions acceptées — premier tour

Les réponses Q1, Q2 et Q3 ont été acceptées explicitement par l’utilisateur.

- **Q1 — Premier jalon :** reconstruire les messages serveur → client,
  décompresser les bundles et afficher les opcodes avec provenance et
  confiance. Le compteur de dégâts viendra après validation des champs métier.
- **Q2 — Acquisition :** conserver l’observation réseau passive. La lecture
  mémoire et le déchiffrement C2S sont reportés. Le watcher n’est pas intégré.
- **Q3 — Validation :** distinguer documentation communautaire, reproduction
  sur les captures Aetheris et confirmation par actions contrôlées en jeu.
  Un nom d’opcode ou un montant candidat ne suffit pas à confirmer un dégât.

Ces choix précisent le jalon de travail dans le périmètre de l’ADR 0003.

## Décisions acceptées — deuxième tour

Les réponses Q4, Q5, Q6 et Q7 ont été acceptées explicitement par l’utilisateur.

- **Q4 — Anciennes captures :** conserver le mode strict par défaut et
  permettre en exploration offline une attribution manuelle du client pour
  chaque connexion. Un alignement initial supposé reste marqué expérimental.
  Désigner le client ne prouve ni la frontière initiale ni le protocole.
- **Q5 — Messages internes :** afficher une ligne par message interne avec
  un lien vers son enveloppe LZ4 consultable et les paquets sources. Les
  positions décompressées sont distinctes des positions dans les octets bruts.
- **Q6 — Erreurs :** arrêter le décodage de la branche affectée par une
  longueur invalide ou un échec de décompression. Conserver les messages déjà
  reconstruits et les octets non interprétés. Imposer des limites de taille
  et de profondeur. Toute reprise nécessite une frontière étayée.
- **Q7 — Champs métier :** retirer du nouveau décodeur les extractions
  positionnelles d’entité, HP et dégâts. Leur réintroduction nécessitera une
  grammaire documentée et vérifiée. Ce jalon expose frontières, bundles,
  opcodes nommés et corps bruts. L’ancienne version du code reste le repère
  pour reproduire les anciens candidats ; leur conservation simultanée dans
  la base n’a pas été décidée.

La provenance des messages compressés fait l’objet de
[l’ADR 0006](adr/0006-compressed-message-provenance.md), car elle structure
durablement les relations conservées et empêche d’assimiler octets dérivés
et octets capturés.

## Décisions acceptées — troisième tour

Les réponses Q8, Q9, Q10 et Q11 ont été acceptées explicitement par l’utilisateur.

- **Q8 — Identification :** reconnaître AION 2 à partir des handshakes et
  exclure les connexions TLS identifiées du décodeur AION 2. En exploration
  offline, permettre une sélection manuelle « AION 2 supposé » quand le début
  manque. Le port seul ne constitue pas une identification suffisante.
- **Q9 — Offline et live :** partager le nouveau décodeur entre les deux
  modes existants ; valider automatiquement offline puis vérifier manuellement
  le live Windows. Attribution manuelle et alignement expérimental restent
  réservés à l’offline.
- **Q10 — Catalogue :** intégrer les noms documentés des opcodes S2C avec
  source et confiance communautaire, conserver les corps bruts et les opcodes
  inconnus. Le nom d’un opcode ne déclenche aucune extraction métier.
  L’utilisateur précise que le catalogue doit être une énumération contenant
  l’opcode, le nom exact de la documentation et un nom traduit facultatif,
  qui peut rester vide initialement. L’affichage choisit le nom traduit
  lorsqu’il est renseigné, sinon le nom documentaire.
- **Q11 — Réussite du jalon :** vérifier les frontières de vecteurs de
  référence, l’indépendance au découpage TCP, la décompression, la provenance
  et la conservation des preuves en présence d’erreurs. Les anciens nombres
  de messages ne sont pas des oracles. Les deux captures actuelles servent
  à l’exploration et à la régression ; une capture complète depuis la
  connexion sera nécessaire pour revendiquer une validation sur une session
  réelle complète.

Exemple du contrat de nommage accepté :

| Opcode sur le fil | Nom documentaire | Nom traduit initial | Nom affiché initial |
|---|---|---|---|
| `04 38` | `DIRECT_DAMAGE` | vide | `DIRECT_DAMAGE` |

## Décisions acceptées — quatrième tour

Les réponses Q12 et Q13 ont été acceptées explicitement par l’utilisateur.

- **Q12 — Inconnus :** énumération Rust des opcodes connus, avec une variante
  `Unknown` conservant les deux octets inconnus. Un inconnu est affiché
  `AB CD · Opcode inconnu`. Un paquet sans opcode reste distinct. Le code
  hexadécimal reste visible et utilisable dans les filtres indépendamment
  du nom affiché.
- **Q13 — Traductions :** libellés français facultatifs maintenus dans le
  code avec le catalogue pour ce jalon. Le nom documentaire exact reste
  accessible dans le détail avec source et confiance. Une traduction
  conserve le sens du nom communautaire sans renforcer sa certitude.

Il ne reste pas de décision de périmètre ouverte pour ce jalon. Les bornes
chiffrées et le choix d’une bibliothèque générale de décompression relèvent
de l’implémentation, dans les contraintes acceptées de taille, ratio,
profondeur et conservation des preuves. La vérification terrain reste un
travail de validation ultérieur, pas une preuve déjà obtenue par cet entretien.

Constats historiques du troisième tour : le live appelait déjà le même
analyseur que l’import et la réanalyse offline. Aucune identification AION 2
ni exclusion TLS applicative n’existait alors : tout TCP admissible arrivait
au framing provisoire. La direction du flux et son identification
comme AION 2 sont donc deux questions distinctes.

Arbre de conception à ce stade :

- Jalon structurel S2C accepté.
  - Acquisition passive et décodeur commun offline/live acceptés.
  - Exploration des anciennes captures et identification explicite acceptées.
  - Enveloppes et messages internes distincts avec provenance acceptés.
  - Arrêt de la branche en erreur accepté → bornes techniques à fixer pendant l’implémentation.
  - Suppression des heuristiques métier et catalogue S2C acceptés.
    - Énumération avec nom documentaire et nom traduit facultatif acceptée.
    - Traductions françaises maintenues dans le code et variante inconnue acceptées.
  - Validation en trois étapes et critères de réussite acceptés.

## Séquence d’implémentation initialement proposée

1. Introduire le catalogue d’opcodes et séparer identification de connexion,
   framing et interprétation. Remplacer le framing mono-octet et supprimer
   les extractions métier provisoires.
2. Ajouter les enveloppes LZ4 et leur provenance dans le modèle et le
   stockage, avec réanalyse transactionnelle conservant les paquets originaux.
3. Exposer les libellés, le détail des enveloppes et les contrôles offline
   d’attribution et d’exploration ; conserver le chemin d’analyse commun au live.
4. Vérifier les vecteurs de référence, les variations de segmentation TCP,
   les erreurs et limites, la provenance et la conservation des données ;
   rejouer les captures locales sans reprendre leurs anciens compteurs
   comme oracle. Consigner séparément les vérifications terrain manquantes.

Cette séquence est conservée comme plan initial. Elle ne constitue pas une
liste entièrement terminée : la section « État de l’implémentation » distingue
les changements réalisés, leurs validations et le travail restant.
