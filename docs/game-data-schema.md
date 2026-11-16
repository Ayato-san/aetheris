# Décodage strict des tables — sous-issue 42 en cours

`game_data::schema::Schema` décrit un layout ordonné, les hashes de colonnes
attendus, la version interne de table et les couples version/région compatibles.
Un client absent de cette liste est refusé avant lecture des lignes. Aucune
version réelle d’Aion 2 n’est déclarée compatible dans le dépôt.

Les primitives comprennent les entiers de 8 à 64 bits, flottants finis,
booléens stricts, chaînes UTF-8 et UTF-16, enums, structures, tableaux simples,
tableaux de structures avec tags Unreal, maps, sets et `FText` (historiques
`None` et `Base`). Le XOR des noms UTF-16 laisse le terminateur brut. Les maps
et sets conservent leurs éléments supprimés dans `Removed` et leurs données
dans `Entries` ou `Values`, pour éviter de perdre une valeur inconnue.

Les tags de tableau sont contrôlés par nom de champ, type, nom de structure,
taille et drapeau de GUID. Les noms de champ `Unknown…` restent inconnus. Les
guides de normalisation doivent continuer à traiter ces valeurs comme telles.
Les flags de `FText` inconnus et ses autres historiques sont refusés.

Chaque décodage exige une correspondance des hashes et du nombre de feuilles
du schéma avec l’en-tête. Le payload doit être intégralement consommé. Les
tailles, comptes, allocations et profondeurs sont bornés. Une erreur indique la
ligne et l’offset ; elle ne renvoie pas une table partiellement considérée valide.

Les tests synthétiques couvrent les types composés, Unicode, GUID, données
tronquées à chaque octet, types invalides et incompatibilités. Ils n’établissent
pas la signification d’un champ sur un client réel. L’import `.usmap`, les
variantes de structures et la préparation des schémas de production restent à
réaliser avant de terminer l’issue 42.

```sh
cargo test --locked --no-default-features --test game_data_schema
```
