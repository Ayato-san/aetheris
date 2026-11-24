# Export commun des objets et compétences

Le moteur `game_data::target_export` construit les objets, compétences et
altérations d’état dans un seul catalogue SQLite, conformément à l’ADR 13 et
à `game-data-target.md`. Il reconstruit un fichier temporaire, vérifie son
intégrité et ses clés étrangères, puis remplace atomiquement la destination.
Les anciens exports indépendants et les migrations de schéma de développement
ont été retirés. Une base d’un autre format doit être régénérée ; aucune
modification de son schéma n’est exécutée sur place.

```sh
cargo run --locked --no-default-features --bin aetheris-game-data -- export-catalogue mapping.usmap /client-dechiffre/AION2/Content/Data/Table en-US.json localized.json appmanifest_3393110.acf [sortie.sqlite [fr-FR [images-extraites]]]
```

Les crochets indiquent les arguments facultatifs, à ne pas saisir littéralement.
La sortie par défaut est `%LOCALAPPDATA%\Aetheris\data.sqlite` sous Windows.
Les entrées sont des payloads déjà déchiffrés ; le montage et le déchiffrement
restent des étapes distinctes. Les commandes `probe-table` et `probe-skills`
restent des diagnostics structurels. Un décodage réussi ne déclare pas la
compatibilité d’un mapping avec toutes les versions/régions.

## Sources et couverture

Le profil lit `Item`, les 41 tables de compétences, tous les `Item*.bin`
présents, les tables d’enchantement examinées, `BoxReward`, `RewardItem`,
`RewardPackageItem`, `EnumString` et `UIIcon`. Pour les sources complémentaires,
la disposition candidate `<nom>TableItem` est essayée strictement ; tout échec
est enregistré sans inventer de disposition ni ignorer des octets.

- `catalogue_manifest` conserve build, langue, empreintes du manifeste Steam et
  du mapping, version et empreinte des sources du lecteur, et région inconnue lorsqu’elle n’est pas établie.
- `catalogue_sources` conserve build, mapping, empreinte du payload, structure,
  état et toutes les lignes décodées, compressées avec LZ4. Cela inclut les
  champs inconnus, valeurs nulles, tableaux ordonnés et entrées supprimées des
  maps Unreal. Ce stockage est séparé des tables métier ; il ne contient pas
  de clé de déchiffrement. `source_rows(db, nom)` restitue les valeurs typées.
- `catalogue_coverage` distingue vérifié, non interprété, recherché sans résultat,
  non recherché, échec et couverture partielle. La réussite structurelle d’une
  table ne signifie pas que tous ses champs sont interprétés.
- Une source complémentaire en échec peut être conservée avec son ancien build
  si les autres sources connues sont identiques. Si elles ont changé, la
  cohérence n’est plus établie et le catalogue précédent reste actif.
- Une source principale illisible, une annulation par interruption, une erreur
  SQL ou un changement des sources pendant le traitement ne publie pas le
  fichier temporaire. Les tailles et les lots sont bornés.

## Compléments métier

Les tables existantes restent les projections de base. Le nouvel export ajoute :

| Tables | Informations |
| --- | --- |
| `item_details` | Niveau requis minimal, distinct du niveau d’objet. Les autres valeurs restent dans la source. |
| `item_false_flags` | Permissions explicitement fausses. Les vraies restent dans `item_flags` ; absence des deux signifie inconnue. `item_permission` retourne `Some(true)`, `Some(false)` ou `None`. |
| `item_set_definitions`, `item_set_members`, `item_set_bonuses` | Définitions partageant éventuellement un nom d’ensemble, membres et paliers avec référence d’altération. Les statistiques non interprétées restent accessibles dans la source. |
| `item_progression_links` | Références explicites de groupes vers enchantement, effets d’enchantement, dépassement, succession et ajout d’âme ; niveau source lorsqu’il est identifié. Les références `ExceedEnchant.AdditionalStatName` pointent aussi vers les bonus de `AdditionalStat`. Coûts/statistiques complets restent dans les lignes liées. |
| `item_obtaining_routes` | Références typées repérées dans `ItemGetRoute`, indexées aussi depuis la source d’obtention. Les cibles non extraites restent non résolues. Le chemin renvoie aux conditions originales. |
| `item_rewards` | Contenus, choix, tirages, quantités, références d’items/packages et poids bruts des trois tables de récompenses. Les liens vers les coffres dont `UseEffect=RewardBox` sont explicites. |
| `skill_abnormals` | Noms, descriptions, classification source, niveaux, cumul et remplacement des états, distincts des effets directs. |
| `skill_abnormal_links` | Candidats d’application SE → état, avec position du paramètre et résolution de l’ID. La résolution de l’ID ne prouve pas le sens du paramètre. |
| `catalogue_icons`, `catalogue_entity_icons` | Référentiel partagé des références d’icônes et liens aux objets, compétences et états ; image et empreinte lorsqu’un fichier extrait correspondant est fourni. |
| `catalogue_labels`, `catalogue_texts` | Libellés d’énumérations EN/localisés et états des textes : disponible, repli anglais, manquant ou sans référence. |

Les identifiants internes de référentiels sont bornés à 1–32767. Les IDs source
du jeu gardent leur taille. SQLite choisit la taille physique de chaque entier.
Les sources originales ne sont pas renommées pour inventer une unité.

Les compétences Item/System restent disponibles indépendamment de leur présence
dans le catalogue joueur. Toutes les définitions SE/ABE/SEF et leurs niveaux
sont conservées, avec leurs liens et conditions, même sans marqueur de description.
Une valeur MP zéro est conservée ; un champ inconnu reste NULL lorsque le modèle
le permet. Les descriptions originales gardent leurs marqueurs ; l’API de
résolution est disponible pour l’interface, sans réécrire ces modèles stockés.

## Changements entre builds

`catalogue_entries` conserve, par famille et ID source, le dernier changement
observé, ses builds d’observation et de référence, la dernière observation,
le dernier build des données comparables et une trace minimale des suppressions.
Les empreintes internes servent à comparer les contenus ; les anciennes valeurs
ne sont pas conservées dans un historique de builds.

La première extraction est un état initial sans ajout. Les comparaisons suivantes
incluent les données métier, les champs source supplémentaires et les relations
extraites d’obtention, récompenses, ensembles et progression. Une réapparition
est un ajout. Un changement de langue ignore les différences de traduction ;
les textes de même langue participent à la comparaison. Un changement du profil
de sources ou du mapping donne « comparaison impossible », jamais de fausses
suppressions attribuées au jeu. Les builds de référence et d’observation indiquent
l’intervalle ; ils ne prouvent pas quelle mise à jour intermédiaire a changé la donnée.

## Limites explicites

Les échelles de probabilités, les unités non confirmées et certains paramètres
d’effet restent non interprétés. `SkillGrade=Common` ne prouve pas la rareté
affichée en jeu ; aucune correction individuelle n’est appliquée.
Les restrictions de classes par indices ne sont pas renommées arbitrairement.
Les familles externes (créatures, quêtes, recettes…) restent à extraire pour
résoudre toutes les routes d’obtention. Les chaînes de packages sont conservées
sans calculer de probabilités composées.

Le décodage des textures Unreal n’est pas implémenté : sans répertoire d’images
déjà extraites, les images sont marquées non recherchées. Le répertoire optionnel
accepte PNG/WebP/JPEG, fait correspondre les références exactes et signale les
ambiguïtés. Le profil de compatibilité version/région et l’intégration à la
publication du catalogue actif de l’application restent des travaux distincts.

## Consultation dans l’application de développement

La page **Debug → Database**, compilée uniquement avec les assertions de
débogage, ouvre le catalogue en lecture seule. Elle propose les listes d’items,
compétences et altérations, la recherche par ID ou nom EN/localisé, et une fiche d’item mise en forme selon Penpot (nom, rareté, catégorie, description, statistiques, sets, pierres, race, niveau requis, permissions et prix). Le sélecteur EN/localisé s’applique aussi aux libellés des statistiques, catégories, raretés et ensembles. Les noms de statistiques proviennent des clés `StatName` du client et alimentent `stats` ; un libellé localisé absent utilise l’anglais. La rareté normalisée « Unique » garde ce nom dans les deux langues. Les compétences gardent leur vue de relations et de paramètres.
Les raccourcis de sections permettent d’accéder directement aux statistiques,
permissions, récompenses, conditions ou autres relations présentes.

La base par défaut est `%LOCALAPPDATA%\Aetheris\data.sqlite`. **Ouvrir une base…**
permet de consulter un autre fichier, notamment `target/game-data-validation/data.sqlite`,
sans le copier ni modifier la base active. Pour les compétences, **Source complète** ajoute les valeurs
originales et les lignes sources liées aux relations affichées ; **Copier** copie
la fiche. **Couverture / limites** expose les états et diagnostics d’extraction.
Les lectures s’exécutent en arrière-plan ; les listes sont paginées par 100 et
les relations sont limitées à 200 par section, avec un avertissement explicite
lorsque la fiche est tronquée. Les BLOB sont décrits par leur taille.

Les skills et altérations utilisent le même composant de fiche Penpot que les
items (en-tête, icône, sections et lignes). Le curseur sélectionne les niveaux
source, actualise descriptions, coûts et paramètres des effets SE/ABE/SEF liés.
Les familles SE et ABE restent distinctes, y compris lorsqu’elles partagent un
groupe ou un ID. Les marqueurs non interprétés restent visibles. Les unités de
temps non établies sont étiquetées comme valeurs sources. La vue « Source complète »
conserve l’inspection technique ; l’affichage des relations est borné et signale
les troncatures. Aucune écriture ni reconstruction SQLite n’est nécessaire.
