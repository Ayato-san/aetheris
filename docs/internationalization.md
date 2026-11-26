# Formatage des nombres

`aetheris::internationalization::Internationalization` est disponible avec ou
sans GUI. Le moteur Rust FormatJS et ses données CLDR sont embarqués : aucun
runtime JavaScript, .NET, ICU système ou téléchargement à l’exécution.

```rust
use aetheris::internationalization::{Internationalization, NumberFormatOptions};

let options: NumberFormatOptions = serde_json::from_value(serde_json::json!({
    "style": "currency",
    "currency": "EUR"
}))?;
let formatted = Internationalization::format_number("fr-FR", 1234.5, &options)?;
assert_eq!(formatted, "1\u{202f}234,50\u{a0}€");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Les options reprennent les noms `camelCase` d’`Intl.NumberFormat`, notamment :

- `style` : `decimal`, `currency`, `percent`, `unit` ; `currency` accepte un code
  ISO 4217 et `unit` un identifiant d’unité. Le moteur valide les options.
- `minimumIntegerDigits`, `minimumFractionDigits`, `maximumFractionDigits`,
  `minimumSignificantDigits`, `maximumSignificantDigits`.
- `notation` : `standard`, `scientific`, `engineering`, `compact` ;
  `compactDisplay` : `short` ou `long`.
- `unitDisplay` : `long`, `short`, `narrow` ; les unités listées dans la demande
  et les composées simples comme `meter-per-second` sont prises en charge.
- `trailingZeroDisplay` : `auto`, `stripIfInteger`, `stripToMinimum`. Ce dernier
  supprime les zéros excédentaires mais respecte les minima explicites ; il suit
  la proposition TC39 de conservation des zéros, déjà implémentée par FormatJS.
- Groupement, signe, affichage de devise et arrondi suivent les options du moteur.

Pour un affichage répété, construire
`Internationalization::number_formatter(language, &options)` puis appeler
`formatter.format(value)`. Les données de langues sont partagées et chargées
à la demande ; le formateur préparé évite de reconstruire ses options dans une
boucle de rendu. La langue est explicite (BCP-47). Une langue valide mais non
prise en charge utilise l’anglais ; une langue invalide renvoie une erreur.
Le choix ne dépend pas de la langue du système.

Les entrées acceptent les nombres Rust et les chaînes décimales exactes : les
entiers 64 bits ne sont pas convertis en `f64`. Les flottants conservent leur
précision propre ; les chaînes sont utiles pour les valeurs décimales exactes.
Les valeurs `NaN`, les infinis et le zéro négatif sont pris en charge.

Aucune conversion métier n’est effectuée : `style: "percent"` attend un ratio
(`0.015` donne `1,5 %`), tandis que `style: "unit", unit: "percent"` attend
une valeur déjà en pourcentage (`1.5` donne `1,5 %`). Le lecteur de données du
jeu reste responsable de convertir ses centièmes avant présentation. Une unité
ou une devise ne convertit pas physiquement ou financièrement la valeur.

## Extension pour le mètre carré

`square-meter` n’est pas une unité simple de la liste ECMA-402. L’application
accepte néanmoins un dénominateur `square-meter`, par exemple
`kilogram-per-square-meter`, avec les trois largeurs et les motifs CLDR localisés.
Le numérateur doit être une unité acceptée par le moteur. Cette extension ne
prétend pas prendre en charge toutes les puissances ou expressions physiques.

Les 113 motifs distincts pour 766 locales proviennent d’Unicode CLDR 48 :
[archive cldr-units-full 48.0.0](https://registry.npmjs.org/cldr-units-full/-/cldr-units-full-48.0.0.tgz).
Empreinte SHA-256 :
`702cec5d8caa9c1c323188eab426262003f45d1a58fc7ad073be4c7128ee8137`.
La licence Unicode est conservée avec les données. Régénération hors ligne :

```sh
python3 tools/generate-square-meter-patterns.py /chemin/cldr-units-full-48.0.0.tgz
```

Références : [API Rust FormatJS](https://formatjs.github.io/docs/rust/intl-runtime/),
[NumberFormat](https://formatjs.github.io/docs/polyfills/intl-numberformat/),
[proposition TC39 sur les zéros](https://tc39.es/proposal-intl-keep-trailing-zeros/).
