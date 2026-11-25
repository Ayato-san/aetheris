# Inventaire des sources du client (#75)

Outil de maintenance en Python standard, exécuté sur une copie locale déjà
déchiffrée. Il appelle le lecteur Rust `aetheris-game-data probe-table` ; Python
n'est pas une dépendance de l'extraction en production. Il ne lit ni ne modifie
`data.sqlite`, et ne crée aucune projection métier.

Depuis la racine du dépôt, avec un exécutable Rust compilé :

```sh
python3 tools/game-data/inventory_sources.py \
  --executable target/debug/aetheris-game-data \
  --mapping /chemin/vers/client.usmap \
  --tables /copie/AION2/Content/Data/Table \
  --english /copie/AION2/Content/L10N/Text/en-US/L10NString.json \
  --localized /copie/AION2/Content/L10N/Text/fr-FR/L10NString.json \
  --manifest base-doc/data/appmanifest_3393110.acf \
  --resources /copie/AION2/Content/Data \
  --decryption-report /copie/decryption.json \
  --sprites target/game-icons/with-ui-sprites/ui-sprites.json \
  --output target/game-data-validation/source-inventory
```

Les trois derniers arguments d'entrée sont facultatifs. Les chemins sont des
exemples ; le manifeste et les mappings doivent correspondre à la copie étudiée.
L'outil conserve l'identification du manifeste, sans établir lui-même son
rattachement aux archives. La région reste inconnue.

`catalogue-sources.json` déclare les structures candidates, familles et relations
à vérifier. Les préfixes classent des candidats, sans prouver leur sens métier.
Les alias de structures sont soumis au lecteur Rust strict, avec ses limites
de taille et ses diagnostics : l'inventaire ne contourne pas ses refus.

Le répertoire de sortie contient :

- `inventory.json` : empreintes, couverture, diagnostics, exemples, champs,
  disponibilité des textes anglais et localisés, références de ressources,
  contrôles explicites de relations et inventaire des cartes ;
- `probes/*.json` : lignes décodées et provenance produites par Rust. Ces exports
  volumineux restent locaux, sous `target/`, hors du dépôt.

Les contrôles respectent le type des identifiants. La conversion d'une chaîne
numérique est limitée aux règles qui la déclarent. Une référence nulle ou `None`
ne crée pas de lien. Plusieurs cibles ne sont acceptées que pour un groupe
explicitement déclaré. Une règle candidate reste non interprétée même si toutes
ses clés correspondent. `verified` décrit un contrôle de clés, jamais une
validation des unités, des probabilités, de la présence en jeu ou des IDs réseau.
Une recherche sans résultat ne prouve pas l'absence de la fonctionnalité.

Les sondes sont réexécutées à chaque passage. Une modification des entrées
surveillées interrompt la publication du rapport final et préserve le rapport
précédent ; les sondes intermédiaires peuvent déjà avoir été remplacées. Ce
n'est pas un instantané atomique d'un client en cours de mise à jour : travailler
sur une copie stable. Aucune compatibilité générale version/région n'est déclarée.
Les références d'images ne sont pas résolues en textures par cet outil.

Vérification :

```sh
python3 -m unittest discover -s tools/game-data -p 'test_*.py' -v
```

Les résultats sur le client étudié, les limites et l'ordre de développement sont
consignés dans [l'inventaire des familles](../../docs/game-data-sources.md).
