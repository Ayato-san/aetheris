# Sources de position du personnage dans les projets locaux

Recherche du 14 novembre 2026. Les inventaires `INVENTAIRE_FONCTIONNALITES_AION.md` et `SYNTHESE_COMPARATIVE_AION.md` ont servi d'index ; les conclusions ci-dessous viennent des sources des projets. Aucun essai de ces applications en jeu n'a été effectué ici. Une affirmation dans un commentaire d'analyse n'est pas une validation indépendante sur la configuration de l'utilisateur.

## Sources figées

Les chemins cités partent de `/home/ayato/digging/aion-metters/`.

| Projet | Commit local examiné | Rôle |
|---|---|---|
| Aion-DPS-Meter, Daevas Power Suite | `32e57ea3aa3705b4a17ffb091a2fb3841d168131` | Carte réseau, position exacte occasionnelle et estimation par les entités proches |
| aion2-nyerk | `cec75a70cd009f4707d0e9b97daa773e0d8f5850` | Position réseau et heuristiques de suivi |
| NOIA2 | `00fa90340cd6e33710b7d9bca7f884a15278204a` | Identité et buffs, sans source continue de position trouvée dans le répartiteur |
| Aion2Meter | `3194fc3d03805397baf671c53b2fbff207baeb37` | Identité locale, sans source continue de position trouvée dans le répartiteur |
| aion2-map-overlay | `45a1c733b7e6b50a378ded2453187a2a9ef5a64f` | Reconnaissance d'image de carte, coordonnées de pixels de référence |
| a2kit | `b1ffe3d6a1248bc26d906983f96896f99d740195` | Décodeur structuré de jauges, sans preuve d'identité locale pour toute jauge |

## Deux sources absentes du guidage au début de la recherche

Les octets d'opcode sont écrits dans l'ordre réseau (`21 36`), indépendamment de la représentation entière utilisée par chaque dépôt. Les offsets de **payload** excluent les deux octets d'opcode et le préfixe de longueur.

### Chargement ou déplacement de zone : `21 36`

Disposition documentée et décodée : `counter u32 LE | zone u32 LE | 8 octets | x f32 LE | y f32 LE | z f32 LE | ...`. Les XYZ commencent aux offsets **16, 20, 24 du payload** ; la zone à l'offset 4. Daevas lit XY aux offsets 18 et 22 du frame qui inclut l'opcode, puis appelle `NoteSpawnPosition`. Il décrit cette position comme celle du joueur après chargement/téléportation et la seule source sans action. Le code Nyerk lit indépendamment les mêmes offsets. Ce message ne commence donc pas par un identifiant d'entité varint à adopter comme identité locale. Sources : [Daevas DecodeZone](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Protocol/Aion2FrameDecoder.cs:207), notamment lignes 232–243 ; [Nyerk parse_map_load](/home/ayato/digging/aion-metters/aion2-nyerk/aion2_tracker/protocol.py:483) ; [analyse Daevas](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/position-frames.md:22).

Limite des preuves : aucun vecteur complet de `21 36` avec assertions XYZ n'a été trouvé dans les tests examinés. Les exemples de l'analyse primaire sont zone 1010, XY `(59953, 17085)`, puis `(-110292, -106862)`. Daevas ne valide que XY dans ce chemin ; l'intégration Aetheris doit également valider Z et la longueur totale nécessaire à sa propre représentation 3D. La concordance de deux implémentations n'est pas un test sur la capture de l'utilisateur.

### Correction / dash / téléportation : `3c 37`

Daevas a une famille dédiée `localPosition`, opcode décimal `15415` = `0x3c37`. Son décodeur lit `entity varint | 03 | x f32 LE | y f32 LE | ...`, exige le marqueur `03` et compare l'entité à l'identité locale connue avant de mettre à jour la position du joueur. Les autres entités ne vont que dans les pistes de replay. Sources : [mapping](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/assets/aion2/protocol/opcodes.json:446), [décodeur](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Protocol/Aion2FrameDecoder.cs:2576).

Un test contient un frame réel abrégé seulement ici par espaces :

```text
3c 37 d1 9a 01 03 76 d2 3a 47 71 04 b0 46 00 40 03 47 14 46 e9 c2 07 02 01 a4 aa f5 00 ef a6 f6 5f a4 03
```

Le test vérifie l'entité **19793**, X proche de **47826.46**, Y proche de **22530.22** ; Z suit immédiatement XY dans le vecteur, mais le test Daevas n'asserte pas Z et concerne une entité non locale. Il valide donc le découpage spatial, pas la cadence des propres corrections. Source : [SelfCheckReplay lignes 17–27](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Combat/SelfCheckReplay.cs:17).

L'analyse Daevas décrit aussi un marqueur `00` pour certains `3c 37`, mais son implémentation n'accepte que `03`. Ne pas élargir ce marqueur par simple généralisation. Sources : [analyse des mouvements, ligne 138](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/bewegungs-frames.md:138), décodeur ci-dessus.

## L'identité doit survivre seulement aux bonnes frontières

`33 36` est bien le record du personnage local. Le test MIT Daevas fournit le préfixe réel `33 36 cb 02 5f a1 c1 28 37 04 41 61 68 7a 18 05 ...` : varint `cb02` = **331**, cinq octets, nom de longueur quatre **Aahz**. Il vérifie `LocalPlayerId == 331`. La présence des cinq octets après l'identité est cohérente avec le préfixe déjà accepté par Aetheris. Sources : [fixture et assertions](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Combat/SelfCheckAion2.cs:1317), [DecodeCharacter](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Protocol/Aion2FrameDecoder.cs:2914), [SetLocalCharacter](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:214).

Daevas mémorise l'identité explicite, sinon utilise un nom configuré, sinon une inférence : destinataire des statistiques détaillées nettement dominant ou lanceur de compétences sans nom. Ces replis sont des **heuristiques**, pas une preuve permettant de prendre le premier acteur qui bouge. Nyerk complète également l'identité à partir des crédits de kills ou votes. Sources : [Daevas LocalPlayerId](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:161), [InferLocalPlayer](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:3062), [Nyerk attribution](/home/ayato/digging/aion-metters/aion2-nyerk/aion2_tracker/track.py:295), [Nyerk _is_me](/home/ayato/digging/aion-metters/aion2-nyerk/aion2_tracker/track.py:426).

NOIA2 et Aion2Meter reconnaissent également `33 36` comme source d'identité. Leur code examiné ne fournit pas de nouvelle source de coordonnées locales continue : [NOIA processor](/home/ayato/digging/aion-metters/NOIA2/src-tauri/src/dps_meter/capture/processor.rs:380), [Aion2Meter dispatcher](/home/ayato/digging/aion-metters/Aion2Meter/src/A2Meter/Dps/Protocol/PacketDispatcher.cs:657).

Dans Aetheris à l'ouverture de cette recherche, chaque `CubeConsumer::reset`, y compris une perte TCP puis resynchronisation, efface `local_entity`. Or le record `33 36` n'arrive pas en continu : reconnecter ne suffit pas si une perte ultérieure efface l'identité avant la prochaine correction. C'est une explication possible de l'attente permanente, à instrumenter et tester, pas une preuve d'attribution du symptôme de l'utilisateur. Une récupération après trou TCP sur **la même connexion reconnue** et une nouvelle session/personnage ne doivent pas partager aveuglément la même politique. Source locale : [CubeConsumer reset](/home/ayato/projects/aetheris/src/overlay/cube_decoder.rs:181).

Autres marqueurs examinés : `22 37` concerne également d'autres joueurs (182 propres identités sur 1329 frames) ; `47 36` retire n'importe quel joueur ; `4e 36` porte des angles pour divers joueurs. `44 37 | entity varint | 00 00` est plus prometteur : l'analyse Daevas compte **281 sur 281** pour sa propre identité, en rafales au chargement et avec l'énergie de vol. Elle laisse toutefois son sens au statut d'hypothèse, sans décodeur d'identité ni test indépendant. Les jauges `00 8d | entity | 01 01 | 03/06 | u32` correspondent à l'endurance/vol dans les observations ; a2kit décode `00 8d` comme jauges de toute entité. Déduire automatiquement une identité locale de ces seuls messages serait donc une nouvelle règle à valider, pas une reprise d'un contrat établi. Sources : [analyse mouvements lignes 132–140](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/bewegungs-frames.md:132), [analyse 44 37](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/skill-frames.md:199), [a2kit gauges](/home/ayato/digging/aion-metters/a2kit/game/parse.go:350), [fixture jauge de vol](/home/ayato/digging/aion-metters/a2kit/game/parse_test.go:294).

Ordre des frontières : Daevas distingue explicitement une position issue du chargement (`_positionIsSpawn`) d'une correction ordinaire. `NoteSpawnPosition` la marque ainsi, et `ForgetPlace` la conserve pendant le changement d'identité si elle est récente (moins de 60 secondes dans cette application). C'est la preuve d'un besoin d'ordre de messages, sans justifier de porter ce délai de confiance dans Aetheris : un point du nouveau chargement peut précéder le record de la nouvelle identité. Sources : [NoteSpawnPosition](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:824), [ForgetPlace](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:1032), [SetLocalCharacter et appel de ForgetPlace](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:252).

## Cadence : aucune position locale continue démontrée

L'analyse primaire Daevas de 95 dumps rapporte que `1a/1b/1c 37` décrivent les **autres** entités ; les occurrences apparentes de l'identité locale provenaient de sa réutilisation dans une autre zone. Les propres `3c 37` sont rares, et les frames `22 37` / `44 37` observés pour le joueur se situent au chargement. Marcher ou voler ne donne pas un flux de coordonnées locales serveur continu. Les paquets client-vers-serveur d'environ 20 Hz sont décrits comme chiffrés ; aucune implémentation de déchiffrement de leur mouvement n'a été trouvée ici. Sources : [analyse mouvements ligne 172](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/bewegungs-frames.md:172), [analyse position lignes 20, 36, 52](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/position-frames.md:20).

La carte Daevas reste mobile grâce à une **estimation** du milieu des entités proches après quatre secondes sans point exact, puis éventuellement son dernier point ancien. Elle ne prouve donc pas l'existence d'un flux exact manquant dans Aetheris. L'estimation exige au moins trois entités, avec filtrage, et reste en XY ; les commentaires donnent des erreurs médianes de l'ordre de 10–15 m sur certaines captures. Ce n'est pas une position 3D exacte utilisable silencieusement pour une flèche vers un cube proche. Source : [EnvironmentEstimate et BestPosition](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Aion2EntityDirectory.cs:667), notamment lignes 754–803.

## Effets : apport possible mais attribution contradictoire

Daevas décode structurellement `2a 38` / `2b 38` : première entité = **cible** ; pour l'ajout, octet `01` ; type ; instance varint ; effet u32 ; durée u32 ; maintien u32 ; horloge u64 ; deuxième entité = **lanceur** ; suite. Il associe les 12 derniers octets XYZ au lanceur et ne les accepte pour le joueur que si cette deuxième entité est locale. NOIA nomme aussi la première entité cible et celle après l'horloge acteur. Sources : [Daevas DecodeBuff](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Protocol/Aion2FrameDecoder.cs:2252), [NOIA buff](/home/ayato/digging/aion-metters/NOIA2/src-tauri/src/dps_meter/capture/parser/buff.rs:17).

Nyerk emploie l'attribution inverse dans ses commentaires et extrait une position par balayage après une horloge plausible : première entité appelée source, deuxième appelée cible. Il affirme des mesures sur captures, mais ce balayage n'est pas une disposition de frame entièrement validée. Source : [Nyerk parse_effect_position](/home/ayato/digging/aion-metters/aion2-nyerk/aion2_tracker/protocol.py:557).

Un test Daevas contient un effet **sur soi**, les deux identités valant 5174, donc il ne tranche pas la contradiction : [SelfCheckCombatOverlay lignes 40–45](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Combat/SelfCheckCombatOverlay.cs:40). Un décodeur conservateur peut commencer par les effets entièrement décodés dont **les deux identités sont locales**, avec XYZ final validé ; attribuer tous les effets d'une seule identité exige une fixture indépendante avec source et cible distinctes.

Les casts ne sont pas une substitution sûre par simple lecture de floats : Nyerk mesure la position de la **cible** sur `02/03 38` ; la documentation Daevas avertit que certains champs de `02 38` sont des vecteurs de direction. Conserver une lecture structurée et conditionnée à l'identité de la cible ; ne pas réutiliser le lanceur ou les premiers floats plausibles. Sources : [Nyerk parse_cast](/home/ayato/digging/aion-metters/aion2-nyerk/aion2_tracker/protocol.py:503), [Daevas position](/home/ayato/digging/aion-metters/Aion-DPS-Meter/docs/protocol/position-frames.md:30).

## Carte par image : autre mécanisme

`aion2-map-overlay` reconnaît des points de la carte visible et calcule une homographie/affinité vers une image de référence. Le « joueur » est un point d'ancrage configuré dans la région capturée, reprojeté par l'inverse de cette matrice. La publication dépend de `auto_progress`, d'une transformation connue et d'un déplacement supérieur à un pixel de référence. Ce résultat n'a pas de Z et n'est pas directement en coordonnées monde XYZ. Il demande une carte visible, un ancrage et une calibration ; ce n'est pas un remplacement immédiat du décodeur réseau. Sources : [tracker RANSAC](/home/ayato/digging/aion-metters/aion2-map-overlay/src/map_overlay/vision/tracker.py:143), [ancrage et inverse](/home/ayato/digging/aion-metters/aion2-map-overlay/src/map_overlay/vision/engine.py:214), [publication](/home/ayato/digging/aion-metters/aion2-map-overlay/src/map_overlay/vision/engine.py:435).

## Conséquences pour Aetheris

1. Ajouter les formats documentés `21 36` et `3c 37`, en conservant les contrôles de longueur, identité et XYZ. Le premier fournit une position locale sans dépendre de l'identité d'entité ; le second exige une identité locale confirmée.
2. Tester séparément perte TCP sur la même connexion, nouvelle connexion, zone et personnage. Ne pas conserver une position fraîche ou un cube par simple inertie pendant une perte ; ne pas perdre indéfiniment l'identité à chaque resynchronisation de la même session si une politique fiable permet de la distinguer.
3. Instrumenter les causes d'attente dans le panneau privé : identité inconnue, aucun format de position reçu, format rejeté ou point périmé. Aucun identifiant/XYZ ne doit devenir un diagnostic public capturable.
4. Ne pas promettre une position exacte continue pendant marche/immobilité sur la base des cartes concurrentes : les sources examinées emploient des estimations ou de vieux points. Allonger seulement le délai de quatre secondes masquerait le manque de source sans le corriger.

## Complément : radar de cubes et rapports de zone répétés

Examen de `Aion2-Hidden-Cube-Radar`, commit local `b45c959b6a873e64bbc9ac936d7d3637260119d4` : [décodeur](/home/ayato/digging/aion-metters/Aion2-Hidden-Cube-Radar/aion2_nav_radar.py:467). Son branchement `13857` (`21 36` en octets) met à jour la position sans effacer les cubes. Il efface le suivi sur `13859` (`23 36`). Il adopte aussi le préfixe de `21 36` comme identité, contrairement à la disposition documentée de Daevas : ce raccourci n'est pas repris.

Le défaut de la bêta 19 Aetheris était d'effacer le suivi à **chaque** rapport `21 36`, y compris une répétition de la même zone. [Daevas DecodeZone](/home/ayato/digging/aion-metters/Aion-DPS-Meter/Client/Aion2/Protocol/Aion2FrameDecoder.cs:245) compare explicitement `zoneId == CurrentZoneId` et distingue ce cas d'une nouvelle zone. Aetheris conserve désormais zone, identité, cubes et durée lors d'une répétition ; la première observation établit la référence. Une différence d'identifiant de zone invalide l'ancien monde. Changement de personnage et interruption réseau restent des frontières distinctes.

Le radar utilise aussi les effets `2a/2b 38` comme positions d'action, mais prend sans validation complète les douze derniers octets et peut adopter la première entité de ces messages. Aetheris reprend uniquement le fait protocolaire, avec le vecteur MIT de `SelfCheckCombatOverlay` : cible locale, statut d'ajout `01` si nécessaire, type, instance varint, effet u32, durée u32, maintien u32, horloge u64, lanceur local, métadonnées exactement `01 00`, puis XYZ et fin de message. Les autres suffixes sont rejetés. Le renouvellement omet l'octet d'ajout, conformément au décodeur Daevas ; son test Aetheris dérive ce format de la fixture d'ajout, sans prétendre disposer d'une seconde capture réelle. L'exigence des deux identités locales évite l'attribution contradictoire constatée entre projets. Aucun code du radar sans licence identifiée n'est copié.
