# Inventaire des clients — issue 40

`aetheris-game-data inspect <dossier>` accepte une installation ou une copie,
sans lanceur. Il inventorie les fichiers `.pak`, `.utoc` et `.ucas` en conservant
chemins relatifs, tailles et SHA-256. Les chemins et résultats sont triés ; deux
copies identiques ont la même empreinte. Aucune extraction n’est effectuée.

`aetheris-game-data discover [racines…]` cherche les dossiers contenant
`Aion2/Content/Paks` ou `Content/Paks`. Une racine peut être un dossier client,
son parent ou un dossier Steam. Les bibliothèques déclarées dans
`steamapps/libraryfolders.vdf` sont parcourues et les doublons supprimés.
Sans argument, l’outil consulte `AETHERIS_GAME_ROOTS` (liste de chemins au format
de la plateforme), les emplacements Steam habituels et, sous Windows, la valeur
`SteamPath` du registre utilisateur. Le résultat contient les installations et
les erreurs de découverte ; une métadonnée illisible ne masque pas les autres
installations trouvées. Une installation hors de ces racines se choisit toujours
par son chemin explicite.

L’inventaire ne déduit pas une région d’un nom de dossier ou d’un build. Pour
une installation sous `steamapps/common`, il conserve le build du manifeste
Steam dont `installdir` correspond exactement au dossier, avec le chemin et le
hash du manifeste. La région reste inconnue. Un dossier copié peut ne présenter
aucune métadonnée de version.

Une déclaration facultative de l’opérateur peut être fournie dans
`aetheris-client.json` :

```json
{"version":"version-verifiee-par-operateur","region":"region-verifiee-par-operateur"}
```

Cette déclaration reste identifiée comme telle dans `metadata` ; elle ne
constitue pas une vérification du client. Aetheris n’écrit pas ce fichier.

Une source absente, une archive illisible, un chemin d’archive non Unicode ou
un lien symbolique vers une archive font échouer l’inventaire. Les répertoires
symboliques ne sont pas parcourus. Les compagnons IoStore absents sont signalés,
y compris pour les partitions `_s1.ucas`, `_s2.ucas`, etc. Leur présence ne
prouve pas que le format sera décodable.

`Inventory::verify_unchanged` reconstitue l’inventaire et compare les archives et
les métadonnées. Le pipeline doit l’appeler avant et après son traitement pour
détecter ajouts, suppressions, remplacements ou changement d’identité. Cela
vérifie les états observés ; une modification puis restauration parfaite entre
ces contrôles ne peut pas être détectée sans verrouillage des fichiers.

```sh
cargo test --locked --no-default-features --test game_data_inspect
```

Les fixtures sont synthétiques et redistribuables. Elles couvrent les copies,
sources manquantes, métadonnées Steam, changements de source et partitions.
La validation de l’extraction sur un client réel appartient aux issues 41 et 47.
