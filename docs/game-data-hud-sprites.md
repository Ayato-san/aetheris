# Recherche locale : ressources HUD absentes du référentiel de sprites

Recherche du 26 novembre 2026. Les constats initiaux utilisent les inventaires existants ; des inspections ciblées de propriétés complètent les preuves ci-dessous. Les archives du jeu ne sont pas modifiées. Cette note distingue les chemins réellement inventoriés des usages supposés d'après leur nom.

## Sources et méthode

- [`tools/game-icons/UiSprites.cs`](../tools/game-icons/UiSprites.cs), méthode `Export` avant cette modification : sélection limitée aux packages `.uasset` sous `AION2/Content/UI/Resource/Sprite/`, puis aux exports `UPaperSprite`. Les alias proviennent exclusivement de `UI_ImageTable_Composite`, `UI_ItemIconTable_Composite`, `UI_SkillIconTable`, et seulement si leur objet a déjà été exporté comme sprite.
- [`tools/game-icons/README.md`](../tools/game-icons/README.md), « Référentiel des sprites UI » : limites et commande d'inspection ciblée.
- `target/game-icons/currency-inventory/inventory.json`, `audit-tables/inventory.json`, `audit-registry/inventory.json` : listes de fichiers identiques. Comptages ci-dessous sur chemins **uniques** : la liste brute contient des doublons provenant des archives montées.
- `target/game-icons/currency-inventory/UI_ImageTable_Composite.json`, `UI_ItemIconTable_Composite.json`, `UI_SkillIconTable.json` : propriétés et lignes des tables décodées.
- `target/game-icons/ui-sprites/ui-sprites.json` et `extraction.json` : images et alias réellement produits.
- `target/game-icons/audit-registry/registry-0.json` et `registry-1.json` : sous-ensemble filtré du registre, donc pas un inventaire complet des classes d'assets.

## Constats

Le filtre initial exclut des images UI réellement présentes. Il existe 10 549 packages uniques sous `UI/Resource/Texture`, 697 sous `UI/Resource/Material`, contre 2 508 sous `UI/Resource/Sprite`. Ces derniers correspondent exactement à 60 atlas, 2 447 sprites décodés et une erreur (`UT_FWindow_Enchant_NextSlot_Sprite`, texture cuite absente). Le dossier `Sprite/HUD` contient 391 packages : l'atlas et 390 sprites déjà décodés.

`UI_ImageTable_Composite` comporte 1 866 lignes : 859 références sous `Sprite`, 960 sous `Texture`, 39 sous `Material`, deux textures et un matériau sous `UI_GL`, cinq ressources vides. Le manifeste ne conserve que 831 alias de cette table, plus cinq de `UI_ItemIconTable_Composite`. Les 28 références Sprite sans alias sont une fonction Housing et 27 entrées Housing : leurs packages ne sont pas dans cet inventaire monté. Une référence en table ne prouve donc pas que son image est livrée.

### Références, ressources distinctes et variantes

Les 1 866 lignes de la table Image ne désignent pas 1 866 images différentes : les `ResourceObject` contiennent 792 chemins distincts de textures (UI et UI_GL), 707 de sprites, 40 de matériaux et un chemin vide. La table Item contient 6 776 lignes pour 6 642 chemins distincts de textures (dont sept UI_GL) et cinq sprites ; Skill contient 713 lignes pour 702 textures distinctes. Ces ensembles peuvent encore partager des ressources entre tables.

La table composite Image déclare déjà huit parents : `UI_ImageTable`, `_Attendance`, `_BattlePass`, `_Social`, `_Marker`, `_Title`, `_Onboarding` et `Global/UI_ImageTable_Global`. Les traiter comme huit familles manquantes serait incorrect : leurs lignes sont déjà fusionnées dans le Composite. Item fusionne de même la table normale et sa variante globale.

Champs alternatifs de la table Image : `ResourceObject_Global` est non vide sur quatre lignes et cible **quatre textures supplémentaires**, toutes montées et absentes de l'ensemble des ResourceObject principaux :

| Ligne | Nom de texture sous `/Game/UI/Resource/Texture/Season/` |
| --- | --- |
| `SeasonSystem_Dungeon` | `UT_Season_SlotBG_PartyDungeon02` |
| `SeasonSystem_Awaken` | `UT_Season_SlotBG_AwakenDungeon02` |
| `HUD_Menu_BG_Awaken` | `UT_Season_Menu_SlotBG_AwakenDungeon02` |
| `HUD_Menu_BG_PartyChallenge` | `UT_Season_Menu_SlotBG_PartyDungeon02` |

Le chemin d'objet de chaque ligne est `préfixe/nom.nom`. Une extension de collecte doit conserver le contexte de variante plutôt que remplacer arbitrairement ResourceObject. `ResourceObject_Mobile` est non vide sur 24 lignes (24 chemins distincts), mais tous ces chemins existent aussi dans les ResourceObject principaux d'autres lignes : aucun fichier supplémentaire identifié par ce champ, mais des associations différentes. `ResourceObjects_L10N` est vide sur toutes les lignes. Les tables Item et Skill n'ont aucun champ alternatif de ressource non vide dans ces dumps.

## Familles demandées

### Icônes de classe

Fait confirmé par les lignes et les chemins montés : dix classes (`Gladiator`, `Templar`, `Assassin`, `Sorcerer`, `Cleric`, `Novice`, `Elementalist`, `Chanter`, `Ranger`, `Fighter`) ont deux variantes distinctes.

| Clé de table | ResourceObject (exemple Gladiator) | Couverture actuelle |
| --- | --- | --- |
| `Class_Gladiator_Large` | `/Game/UI/Resource/Texture/Icon/UT_Class_Gladiator_Large.UT_Class_Gladiator_Large` | Hors sélection Sprite |
| `Class_Gladiator_Small` | `/Game/UI/Resource/Sprite/Common/UT_Common_Icon_Class_Gladiator_Sprite.UT_Common_Icon_Class_Gladiator_Sprite` | Sprite déjà décodé et aliasé |

Le même modèle de chemin existe pour les dix classes. Le widget `HUD_UserInfo` confirme directement ce manque : son export `/78`, `AionImage P_Img_Class`, a un Brush pointant sur `Texture/Icon/UT_Class_Gladiator_Large`. Les clés `Daevanion_Node_Start_*` désignent aussi les grandes textures (neuf classes, sans Novice). Les textures ne sont pas des agrandissements des PNG Small : ce sont des ressources distinctes.

### Barre de vie du joueur

Faits : trois sprites existent déjà dans le manifeste : `UI/Sprite/HUD/UT_HUD_HPGauge_BG_Sprite` (188 × 37), `UT_HUD_HpMpBar_Effect_01_Sprite` (24 × 21), `HUD_HpMp_Bar_Effect_03_Sprite` (27 × 27). Leur nom et leur dimension ne prouvent pas qu'ils suffisent à reconstruire la barre visible.

Preuve directe supplémentaire : [`target/game-icons/hud-research/user-info/asset.json`](../target/game-icons/hud-research/user-info/asset.json) contient les 619 exports du widget `HUD_UserInfo`. Les exports `/109` et `/108` ont leurs `WidgetStyle.FillImage` et `LowFillImage` reliés respectivement à `MI_UI_Gauge_001` et `MI_UI_Gauge_002` ; `/110`, `/111` et `/113` utilisent `MI_UI_Gauge_007`. `Img_HpBG` (`/65`) référence le sprite `UT_HUD_HPGauge_BG_Sprite` ; `Img_MpBG` (`/73`) réutilise ce sprite avec `ScaleX=-1`. Le widget combine donc bien fonds Sprite et remplissages Material, hors couverture de la collecte Sprite pour ces derniers. `P_PBar_Hp` porte aussi une couleur de remplissage (`FillColorAndOpacity`) et plusieurs styles de perte/récupération ; son `Percent` sérialisé est une valeur de définition, pas la vie d’un personnage observé en jeu.

Autres pistes montées : matériaux `UI/Resource/Material/MI/MI_UI_Gauge_001.uasset` à `MI_UI_Gauge_010.uasset`, ainsi que `Material/MM/M_UI_GaugeLinear.uasset`. Le lien des variantes 001, 002 et 007 avec ce widget est prouvé ; le rôle des autres variantes reste à établir. Un matériau dynamique n'est pas une image statique directement décodable.

L’inspection de [`gauge-material/asset.json`](../target/game-icons/hud-research/gauge-material/asset.json) confirme que `MI_UI_Gauge_001` hérite de `M_UI_ScaleToFit` et référence les textures `UT_Trail_002`, `UT_Gradient_007` et `UT_Trail_057` via quatre paramètres de textures, avec 33 paramètres scalaires et quatre vectoriels. Exporter ces textures fournit les entrées du matériau, pas l’image finale de la barre avec ses couleurs, dimensions et état.

### Inventaire et fonds des emplacements d'items

Faits : les boutons `UT_Common_Btn_Inventory_*_Sprite` et les fonds `UI/Sprite/Common_Slot/UT_Common_Slot_Empty_Sprite`, `UT_Common_Slot_Empty_Equip_Sprite`, `UT_Common_Slot_Empty_Virtual_Sprite` sont déjà décodés (fonds : 104 × 104). Le manifeste contient aussi les silhouettes d'équipement et les marqueurs de slot.

Autres packages montés à examiner :

- Widgets `UI/Widget/Common/Common_Inventory.uasset`, `Common_Inventory_CubeListItem.uasset`, `UI/Widget/Slot/Slot_InventoryItem.uasset`, `Slot_InventoryItem_Round.uasset`, `Slot_Item.uasset`.
- Famille `UI/Resource/Material/MI/MI_UI_ItemGrade` : base, `_Circle`, `_NineGrid`, `_Pantheon_L`, `_Pantheon_S`.
- Famille `UI/Resource/Material/MI/MI_UI_ItemSlot` : base, `_Circle`, `_FX`, `_RectRound`, `_RectRound_Customizing`, `_RectRound_NineGrid`, `_RectRound_NineGrid_Customizing`.
- Tables montées mais hors sélection d'alias actuelle : `UnrealData/UI_SlotSettings.uasset`, `UI_IconMaterialParamDataTable.uasset`, `UI_ItemIconFxTable.uasset`.

Preuves complémentaires : `target/game-icons/hud-research/material-params/asset.json` contient 32 lignes de `UI_IconMaterialParamDataTable` avec paramètres de teinte, saturation et luminosité selon le grade ; `slot-settings/asset.json` contient 71 lignes de `UI_SlotSettings` avec notamment `bHideGrade` et `bHideSlotBG`. Ces tables démontrent l'existence de paramètres de rendu et de visibilité, sans montrer à elles seules leur application dans l'inventaire.

Le dump direct [`target/game-icons/hud-research/slot-item/asset.json`](../target/game-icons/hud-research/slot-item/asset.json) (163 exports de `Slot_Item`) confirme cette composition :

| Export / image | Brush.ResourceObject |
| --- | --- |
| `[15] P_Img_GradeBG` | `Material/MI/MI_UI_ItemGrade_NineGrid` |
| `[17] P_Img_Icon` | `Material/MI/MI_UI_ItemSlot_RectRound` |
| `[26] P_Img_SlotBG` | `Sprite/Common_Slot/UT_Common_Slot_Empty_Sprite` (déjà décodé) |
| `[27] P_Img_SoulCrystal` | `Material/MI/MI_UI_ItemSlot` |

Les exports `[3..7]` utilisent aussi les matériaux d'effets `MI_UI_Item_002_Loop`, `MI_UI_Item_001_Loop`, `MI_UI_Item_002`, `MI_UI_Item_001`. Les chemins sont relatifs à `/Game/UI/Resource/`. Le fond de grade est donc bien un matériau, tandis que le fond vide est un sprite : les deux familles existent dans un même widget. Le rendu exact par grade et l’affectation dynamique de l’icône demandent encore de suivre les paramètres ; une référence par défaut ne suffit pas à reconstruire chaque état affiché.

L’inspection directe de [`inventory-item/asset.json`](../target/game-icons/hud-research/inventory-item/asset.json) (329 exports de `Slot_InventoryItem`) retrouve les mêmes matériaux dans l’inventaire lui-même : `[38] P_Img_GradeBG → MI_UI_ItemGrade_NineGrid` et `[39] P_Img_Icon → MI_UI_ItemSlot_RectRound`. Il ne s’agit donc plus seulement d’une piste déduite des noms des packages. Les images de surbrillance et de sélection utilisent également matériaux et textures UIFX.

L’inspection de [`item-grade-material/asset.json`](../target/game-icons/hud-research/item-grade-material/asset.json) confirme la chaîne `MI_UI_ItemGrade_NineGrid → M_UI_Item_NineGrid`, avec `SlotImage → Texture/ETC/UT_SlotGrade_Common` et `TextureSlotMask → Texture/UIFX/UT_Rect_2r`. La table Image contient aussi sept clés `ItemGrade_{Common,Rare,Legend,Unique,Epic,Mythic,Special}` vers sept textures `UT_SlotGrade_*`. Ces clés conservent les codes source, sans renommer les grades. L’existence de ces textures par grade ne démontre pas que le widget les sélectionne toutes directement en jeu : son brush par défaut reste le matériau.

### Fonds larges de fiche d'item : preuve distincte des fonds carrés

L'inspection `target/game-icons/hud-research/popup-item/asset.json` contient les 409 exports du widget `/Game/UI/Widget/Popup/Popup_Item`. Ses images de fond de grade référencent directement une famille de **textures distincte** de `UT_SlotGrade_*` :

| Export | Image | Brush.ResourceObject sous `/Game/UI/Resource/Texture/ETC/` |
| --- | --- | --- |
| `[92]` | `Img_GradeBG_Common` | `UT_ItemTooltipGrade_Common.0` |
| `[93]` | `Img_GradeBG_Epic` | `UT_ItemTooltipGrade_Epic.0` |
| `[94]` | `Img_GradeBG_Legend` | `UT_ItemTooltipGrade_Legend.0` |
| `[95]` | `Img_GradeBG_Mythic` | `UT_ItemTooltipGrade_Mythic.0` |
| `[96]` | `Img_GradeBG_Rare` | `UT_ItemTooltipGrade_Rare.0` |
| `[97]` | `Img_GradeBG_Special` | `UT_ItemTooltipGrade_Special.0` |
| `[98]` | `Img_GradeBG_Unique` | `UT_ItemTooltipGrade_Unique.0` |

Ces sept exports sont des `AionImageEffect` dont le `Brush.ImageSize` vaut **512 × 220**. Il s'agit de la taille déclarée du Brush dans le widget ; les PNG décodés mesurent **512 × 256**. Leurs slots Canvas déclarent aussi une hauteur de 220 et des ancres horizontales étirées. L'export `[157]`, `P_WS_GradeBG`, est un `AionWidgetSwitcher` avec sept panneaux, dans l'ordre Common, Rare, Legend, Unique, Epic, Mythic, Special (slots `[389..395]`). Ce widget prouve donc le rôle de fond large par grade.

Les packages `UT_ItemTooltipGrade_*` sont tous présents dans `target/game-icons/audit-inventory/inventory.json`, y compris une variante `Reserve` absente de ce switcher et une texture `UT_ItemTooltipGrade_Deco_Cloud`. Aucun ResourceObject des tables Image/Item inspectées ne référence cette famille : une collecte limitée aux tables passerait à côté de ces fonds pourtant directement employés par le widget.

À l'inverse, les sept clés `UI_ImageTable_Composite.ItemGrade_{Common,Rare,Legend,Unique,Epic,Mythic,Special}` désignent `Texture/ETC/UT_SlotGrade_<grade>.UT_SlotGrade_<grade>` : il faut conserver les deux familles séparées. `Reserve` existe en fichier pour les deux familles, sans association de grade démontrée ici.

L'inspection `target/game-icons/hud-research/item-grade-wide-material/asset.json` confirme que `MI_UI_ItemGrade` (sans NineGrid) reste un `MaterialInstanceConstant` ayant pour parent `Material/MM/M_UI_Item`, avec `TextureParameterValues.SlotImage` pointant vers `Texture/ETC/UT_SlotGrade_Common` et `TextureSlotMask` vers `Texture/UIFX/UT_Rect_2r`. Malgré son nom générique, ce matériau ne constitue donc pas la source prouvée des fonds larges du header : le widget les référence comme textures `UT_ItemTooltipGrade_*`.

## Autres images HUD hors sélection confirmées

Chaque ResourceObject ci-dessous est présent dans `UI_ImageTable_Composite` et son package correspondant figure dans l'inventaire monté. Préfixe commun des chemins : `/Game/UI/Resource/Texture/` ; le nom après le point est identique au nom de l'asset.

| Clé | ResourceObject relatif au préfixe |
| --- | --- |
| `TargetInfo_GaugeBG_Monster` | `ETC/UT_HUD_TargetInfo_Gauge_Monster_Normal.UT_HUD_TargetInfo_Gauge_Monster_Normal` |
| `TargetInfo_GaugeBG_EliteMonster` | `ETC/UT_HUD_TargetInfo_Gauge_Monster_Elite.UT_HUD_TargetInfo_Gauge_Monster_Elite` |
| `TargetInfo_GaugeBG_BossMonster` | `ETC/UT_HUD_TargetInfo_Gauge_Monster_Hero.UT_HUD_TargetInfo_Gauge_Monster_Hero` |
| `TargetInfo_GaugeBG_LegendMonster` | `ETC/UT_HUD_TargetInfo_Gauge_Monster_Legend.UT_HUD_TargetInfo_Gauge_Monster_Legend` |
| `TargetInfo_GaugeBG_PC` | `ETC/UT_HUD_TargetInfo_Gauge_Normal.UT_HUD_TargetInfo_Gauge_Normal` |
| `HUD_AbyssArtifact_Neutral` | `ETC/UT_HUD_AbyssArtifactList_Neutral.UT_HUD_AbyssArtifactList_Neutral` |
| `HUD_AbyssArtifact_Light` | `ETC/UT_HUD_AbyssArtifactList_Light.UT_HUD_AbyssArtifactList_Light` |
| `HUD_AbyssArtifact_Dark` | `ETC/UT_HUD_AbyssArtifactList_Dark.UT_HUD_AbyssArtifactList_Dark` |
| `QuickSlotMacro_Icon01` | `Icon/UT_QuickSlotMacro_01.UT_QuickSlotMacro_01` |
| `HUD_Gather` | `Icon/UT_Marker_Gather.UT_Marker_Gather` |
| `Minimap_Hero_Area` | `ETC/UT_Worldmap_Quest_Hero.UT_Worldmap_Quest_Hero` |
| `Minimap_Tooltip_MapPin` | `Icon/UT_Marker_Pin.UT_Marker_Pin` |

Les macros vont jusqu'à `QuickSlotMacro_Icon15`. Il existe aussi des textures directes hors table, notamment `Texture/ETC/UT_HUD_QuickSlot_Deco_L`, `_R`, `UT_HUD_DPGauge_Deco`. Une collecte limitée aux clés de table laisserait ces ressources de côté.

## Limites et suites possibles

L'inventaire contient 263 packages `UI/Widget/HUD` et 193 `UI/Widget_M/HUD_M` : leurs propriétés sont la piste nécessaire pour rattacher les images à l'écran utilisateur, puis distinguer texture, sprite et rendu composé. Les tables supplémentaires `UI_HudEditDataTable`, `UI_HudEditDataTable_M`, `UI_CursorTable`, `UI_RichTextImageDataTable`, `UI_PortraitTable`, `UI_DungeonResourceTable` et les tables BMShop figurent aussi dans le montage, sans analyse de leurs lignes dans cette recherche.

Étendre la collecte aux textures directement présentes dans les widgets et aux références Texture de `UI_ImageTable_Composite` compléterait la liste d'images. Les matériaux doivent faire l'objet d'une étude séparée de leurs textures, couleurs et paramètres ; leur aplatissement éventuel nécessite un état de rendu explicite. Les inspections ciblées vérifient des propriétés ; elles ne rendent pas les matériaux et ne testent pas leur export PNG.

## Vérification par extraction ciblée

Dix grandes textures de classes et les sept textures `UT_SlotGrade_*` ont été
extraites des packages exacts, puis leurs PNG, dimensions et SHA-256 vérifiés
par `GameIcons --verify` : **17/17 disponibles**, toutes en **256 × 256**.
Cette vérification confirme leur décodabilité et leur absence du filtre Sprite,
sans ajouter ces échantillons à la base active.

- [Correspondances de clés UI et objets](../target/game-icons/hud-research/static-sources.json).
- [Références exactes demandées](../target/game-icons/hud-research/static-references.json).
- [Rapport d’extraction](../target/game-icons/hud-research/static-images/extraction.json).
- [Exemple : Gladiator Large](../target/game-icons/hud-research/static-images/UI/Resource/Texture/Icon/UT_Class_Gladiator_Large.png).
- [Exemple : fond de grade Rare](../target/game-icons/hud-research/static-images/UI/Resource/Texture/ETC/UT_SlotGrade_Rare.png).

La vérification SQLite retrouve le fond vide `UT_Common_Slot_Empty_Sprite` et
le fond HP `UT_HUD_HPGauge_BG_Sprite` dans `ui_sprites`, mais aucune entrée pour
`UT_Class_Gladiator_Large`, `UT_SlotGrade_Common` ou `UT_SlotGrade_Rare`.
Ces constats décrivent la base avant intégration. La suite de l’implémentation
ajoute les sept fonds larges et les sept fonds carrés à `item_backgrounds`
(format SQLite 15), reliés à leurs PNG partagés par `catalogue_icons`.
L’extraction automatique `--ui-sprites` les collecte désormais. La fiche d’item
emploie le fond large dans son en-tête, avec une icône sans fond ; le carré
est réservé aux affichages sans fond large. La sélection se base
sur le grade source : `Legend` conserve son illustration même si son libellé
anglais est `Epic`. Les images sont chargées une fois à la sélection de la fiche.
Les 14 PNG natifs sont vérifiés, avec des dimensions respectives de 512 × 256
et 256 × 256.
