# Ressources signées — sous-issue 45 en cours

Les ressources peuvent être préparées et installées indépendamment du binaire
d’Aetheris. Un dossier de distribution contient `resources.json`, la signature
Ed25519 brute de ses octets dans `resources.sig` et les fichiers déclarés.
Le manifeste expose `format: 1`, une `revision` entière croissante, les couples
client/région compatibles et la liste des fichiers avec chemin relatif,
SHA-256, taille et type (`schema`, `schema_set`, `mapping` ou `catalogue`).

```sh
cargo run --locked --no-default-features --bin aetheris-game-data -- install-resources /bundle /ressources identity.json
cargo run --locked --no-default-features --bin aetheris-game-data -- resource-status /ressources
```

Ces commandes utilisent la clé publique de distribution configurée à la
compilation dans `AETHERIS_PUBLIC_KEY`, comme le compagnon. Aucune commande
n’accepte de contourner sa vérification. L’API accepte une clé explicitement
fournie pour les tests et les outils de préparation.

Avant activation, l’installeur vérifie signature, version de format, révision,
compatibilité du client, tailles et empreintes. Les chemins doivent rester
relatifs, sans traversée de dossiers ni liens symboliques. Les doublons sont
refusés indépendamment de la casse. Les schémas JSON et les catalogues SQLite
sont ouverts et leur compatibilité est recoupée avec le manifeste signé.
Un mapping seul reçoit un contrôle d’intégrité. Un `schema_set` lie ce mapping
par son empreinte et impose sa présence dans le bundle ; son parsing, les layouts,
la révision du lecteur et la couverture version/région sont vérifiés avant
installation. Le [contrat initial de #42](game-data-schema.md) est disponible ;
le mapping et les clés restent des entrées locales.

Les limites sont de 64 Kio par manifeste, 512 Mio par fichier et 1 Gio par
bundle. Les fichiers sont copiés dans un dossier temporaire, vérifiés puis
conservés sous `versions/<hash-du-manifeste>/`. Seul le remplacement atomique
d’`active.json` active un bundle. Un verrou empêche deux installations
concurrentes. Les anciens bundles restent présents en cas d’erreur.

Une révision plus ancienne que la version active est refusée. Une même révision
ne peut pas désigner un autre contenu. Une nouvelle révision signée peut réparer
les fichiers endommagés d’une version précédente ; la signature de son manifeste
reste nécessaire pour établir la révision précédente.

La lecture de la version installée recontrôle signature et empreintes sans
connexion réseau. Une source de mise à jour indisponible n’efface rien. Les tests
suppriment effectivement la source après installation et vérifient la lecture
hors ligne. Ils couvrent aussi les mises à jour corrompues, incompatibles,
indisponibles, les chemins invalides et les attaques par modification de révision.

Le catalogue initial et le bundle réel signé contenant les ressources
distribuables, le canal de téléchargement et sa vérification différée dans
l'application restent à réaliser. Le contrat de schémas réel est disponible
avec #42 ; sa présence ne constitue pas la distribution d'un mapping. Aucun bundle synthétique des tests
n’est distribué comme ressource du jeu. L’issue 45 reste ouverte.

```sh
cargo test --locked --no-default-features --test game_data_resources
```
