# Extraction des icônes du catalogue

Outil hors de l’application pour extraire les textures référencées par
`catalogue_icons`, à partir des archives locales Aion 2. Il produit des PNG avec
transparence et un rapport `extraction.json`. Il ne modifie ni les archives du
jeu, ni SQLite. .NET n’est pas une dépendance du client Aetheris.

## Préparation

Prérequis : Python 3, SDK .NET 10, copie de CUE4Parse au commit
`4d2301a8eb129f593f1c76ff82612400801f1a94`, USMAP compatible et fichier de clé
AES des archives. Le fichier de clé peut contenir des commentaires `#` ou `//`,
mais doit contenir une seule ligne de clé hexadécimale de 32 octets.

Depuis la racine du dépôt :

```sh
python3 tools/game-icons/prepare.py \
  --cue4parse /chemin/CUE4Parse \
  --database /chemin/data.sqlite

dotnet build tools/game-icons/GameIcons.csproj -c Release \
  -p:Cue4ParseRoot="$PWD/target/game-icons/CUE4Parse" \
  -p:CUE4PARSE_SKIP_NATIVE=true -p:RestoreLockedMode=true
```

La préparation vérifie la révision et l’absence de modifications des sources
utilisées, puis les copie dans `target/game-icons/CUE4Parse`. Elle écrit les
références et le build du catalogue dans `references.json` et `preparation.json`.
Elle épingle Microsoft.Bcl.Memory 10.0.15 dans cette copie, sans modifier le
dépôt CUE4Parse d’origine. Les décodeurs BC sont compilés directement depuis
ces sources ; le convertisseur complet, ImageSharp et les exporteurs de
modèles/animations ne sont pas utilisés.

La première compilation restaure les dépendances NuGet. Le premier lancement
peut télécharger la bibliothèque Oodle via CUE4Parse ; les suivants utilisent
la copie présente près de l’exécutable. Le traitement peut ensuite fonctionner
hors ligne avec ces ressources déjà disponibles. Sous Linux sans ICU, définir
`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1` pour le SDK et l’exécutable.

## Extraction

```sh
dotnet target/game-icons/bin/net10.0/GameIcons.dll \
  /chemin/Aion2/Content/Paks \
  /chemin/mapping.usmap \
  /chemin/archive-key.txt \
  target/game-icons/references.json \
  target/game-icons/images
```

Le dossier de sortie doit être vide et extérieur aux archives. Les noms des
PNG reprennent exactement les références du catalogue, par exemple
`Skill/ICON_ACTIVE_SKILL_001.png`. Les références sont d’abord résolues comme
des clés de ligne dans `UI_ItemIconTable_Composite` (qui inclut la table globale)
et `UI_SkillIconTable`. Le champ `ResourceObject` donne le chemin Unreal exact
de la texture, même si son nom diffère de la clé. Les tables du build vérifié
n’utilisent pas de variante PC par langue ou de champ `ResourceObject_Global`.
Sans correspondance dans ces tables, la recherche utilise le suffixe de chemin
exact, puis un nom de texture unique dans les archives montées.
Une texture déclarée dans une table mais absente des archives reste introuvable.
Les clés `Currency_*` utilisent `UI_ImageTable_Composite`. Elles proviennent de
`CurrencyInfo.IconResource`, associé à `sell_currencies.sell_currency_type` par
`CurrencyInfo.CurrencyCategory`. `Gold` désigne ainsi `Currency_Goldbinding`,
et non la monnaie combinée `GoldCombined`. Les sprites sont découpés dans leur
texture d’atlas avec leurs coordonnées et dimensions cuites ; une rotation
d’atlas est corrigée à partir des sommets du sprite.
Plusieurs candidats ne sont acceptés que si leurs PNG décodés sont identiques ;
tous leurs chemins sont alors conservés. Sinon, le résultat est `ambiguous`,
jamais un choix arbitraire. Les autres états sont `available`, `not_found` et `error`.

Le rapport conserve la table et le chemin d’objet ayant résolu la référence,
le chemin de texture, les dimensions, le format de pixels,
l’empreinte du PNG et celle du mapping. Les formats pris en charge sont
DXT1/3/5, BC4/5/7, BGRA8, RGBA8 et G8 sur une texture 2D classique. Les formats
inconnus, textures virtuelles et dimensions supérieures à 4096 sont signalés.
Seul le premier mip lisible est exporté ; aucun agrandissement n’est appliqué.

Pour vérifier les PNG produits (décodage, dimensions et SHA-256) :

```sh
dotnet target/game-icons/bin/net10.0/GameIcons.dll --verify target/game-icons/images
```

L’option `--compare dossier-reference dossier-resultat` compare les pixels
décodés de tous les PNG du premier dossier avec ceux du second, sans imposer
un encodage PNG identique.

Pour rechercher les correspondances sans extraire les PNG :

```sh
dotnet target/game-icons/bin/net10.0/GameIcons.dll --inventory \
  /chemin/Aion2/Content/Paks /chemin/mapping.usmap /chemin/archive-key.txt \
  target/game-icons/inventory
```

Cette commande écrit l’inventaire des fichiers et archives montées, les tables
Unreal `*IconTable*` et `UI_ImageTable_Composite`, les configurations du jeu et les entrées d’icônes et de
redirection des registres Unreal. Le dossier de sortie doit être vide.

Sur le build Steam `26581260`, la résolution par table retrouve les 618 clés
partagées par les 3 167 items que la recherche par nom ne trouvait pas. Par
exemple, `Icon_WP_GS_0020_T05` désigne
`/Game/UI/Resource/Texture/Item/Weapon/Icon_Equip_WP_L_GS_0020_T05.Icon_Equip_WP_L_GS_0020_T05`.
Les âmes de créatures peuvent désigner des portraits situés hors du dossier
`Texture/Item`. Les six doublons entre `BMShop` et `ETC` sont également résolus
par le chemin déclaré dans la table. Il ne faut donc pas déduire l’image de son
nom, même lorsqu’une texture homonyme existe : 33 références du catalogue
produisaient une image différente avec cette ancienne recherche.

## Intégration au catalogue

### Référentiel des sprites UI

`--ui-sprites <paks> <mapping.usmap> <key-file> <output>` extrait les objets
`PaperSprite` du dossier `UI/Resource/Sprite`, en écartant les packages d’atlas
eux-mêmes. Il écrit les PNG, `extraction.json` et `ui-sprites.json` (chemins
d’assets, dimensions, clés de tables et erreurs). Les atlas décodés sont mis
en cache avec une limite de 64 Mio et 16 textures.

Pour produire en une commande les icônes du catalogue et ce référentiel :

```sh
dotnet target/game-icons/bin/net10.0/GameIcons.dll --include-ui-sprites \
  /chemin/Aion2/Content/Paks /chemin/mapping.usmap /chemin/archive-key.txt \
  target/game-icons/references.json target/game-icons/all-images
```

Fournir ce dossier complet à `export-catalogue`. Le moteur Rust importe le
manifeste dans `ui_sprites` et `ui_sprite_aliases`, avec les PNG dans
`catalogue_icons`. Les clés viennent de `UI_ImageTable_Composite`,
`UI_ItemIconTable_Composite` et `UI_SkillIconTable`, en conservant le nom de la
table pour éviter les collisions.

Sur le build `26581260`, 2 447 sprites sont décodés, avec 836 clés de tables.
`UT_FWindow_Enchant_NextSlot_Sprite` porte une propriété
`BakedSourceTexture=null` dans le client : il reste signalé dans les erreurs,
sans fabriquer d’image. La couverture est donc partielle.

Exemple de récupération de l’image par une clé UI (les `FName` Unreal sont
insensibles à la casse) :

```sql
SELECT s.id, s.width, s.height, i.image, i.mime_type
FROM ui_sprite_aliases a
JOIN ui_sprites s ON s.id = a.sprite_id
JOIN catalogue_icons i ON i.id = s.icon_id
WHERE a.table_name = 'UI_ImageTable_Composite'
  AND a.row_key = 'Currency_Goldbinding' COLLATE NOCASE;
```

`--inspect-asset <paks> <mapping.usmap> <key-file> <asset-path> <output>` écrit
les propriétés d’un package dans `asset.json` pour diagnostiquer un cas isolé.

### Images du catalogue

Après vérification du rapport et de la correspondance entre les archives et le
build du catalogue, fournir `target/game-icons/images` comme dernier argument
de `aetheris-game-data export-catalogue` (voir
[l’export SQLite](../../docs/game-data-export.md)). Cet export régénère la base
avec les PNG et leurs liens existants aux items, compétences et altérations.
Une image absente reste signalée comme absente ; aucune image de remplacement
n’est générée.

Les archives, PNG, bibliothèques restaurées et catalogues restent dans les
dossiers de travail ignorés par Git. CUE4Parse est utilisé sous sa licence
Apache-2.0 ; conserver ses notices lors de la distribution de l’outil compilé.
