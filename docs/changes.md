# Notes de version d’Aetheris

Statut : implémenté le 3 novembre 2026. La décision est référencée dans
l’[ADR 0008](adr/0008-player-facing-release-notes.md).
Ce document décrit la spécification et les commandes de maintenance.

## Décisions validées

- Les canaux stable et bêta restent publiés depuis `main`, sans création de
  branches distinctes pour les notes de version.
- `CHANGES.beta.md` contient les notes bêta ; `CHANGES.md` contient les notes
  stables. Chaque fichier regroupe les entrées par version sous les sections
  `Add`, `Modify` et `Fixes`.
- Avant chaque commit ayant un effet pertinent pour le joueur, les notes en
  préparation sont mises à jour sous `PENDING`. Plusieurs commits peuvent
  enrichir une même entrée.
- Les notes expliquent en français simple ce que la mise à jour change pour le
  joueur. Les détails techniques ne sont inclus que s’ils lui sont utiles.
  Les changements sans effet pertinent pour le joueur, notamment le formatage,
  les tests et la documentation interne, ne nécessitent pas d’entrée.
- Les notes stables sont une synthèse rédigée explicitement, entretenue dans
  leur propre `PENDING`, et non une concaténation automatique des notes bêta.
- Une release bêta clôture son `PENDING` en lui attribuant sa version et sa date,
  sans clôturer la synthèse stable en préparation.
- Une release stable clôture la synthèse stable avec sa version et sa date.
  Elle vide également le `PENDING` bêta, sans supprimer les notes des anciennes
  versions bêta.
- Une fonctionnalité ajoutée, modifiée puis corrigée dans trois bêtas successives
  figure dans chacune des versions bêta concernées. Dans la stable, elle figure
  une seule fois dans `Add`, décrite dans son état final.
- Ces historiques serviront de source à une future page de notes de version.
- Un contrôle automatique avant commit exige la mise à jour des `PENDING`
  des canaux concernés pour les changements pertinents pour le joueur et
  vérifie la structure des fichiers. Par défaut, les deux canaux sont concernés.
  Une exemption explicite et justifiée permet les commits sans effet pertinent
  pour le joueur dans un canal. Ce contrôle ne juge pas la pertinence éditoriale.
- Les exemptions figurent dans le message du commit, sur une ligne dédiée
  indiquant le canal et le motif. Exemple :
  `Changelog-Exempt: stable — correction d’un bug apparu uniquement en bêta`.
  Le contrôle exige la mise à jour des notes des canaux non exemptés et vérifie
  la présence d’un motif pour chaque exemption.
- Les notes stables comparent deux versions stables : une correction d’un bug
  introduit et corrigé pendant les bêtas d’une nouvelle fonctionnalité figure
  uniquement dans les notes bêta. La synthèse stable conserve la fonctionnalité
  finale dans `Add`, sans ajouter ce bug dans `Fixes`. Le commit justifie alors
  l’absence de modification des notes stables par une exemption de ce canal.
- La préparation d’une release attribue automatiquement la version et un
  horodatage UTC au format ISO 8601, par exemple
  `1.2.3 — 2026-11-03T14:30:00Z`, puis recrée un `PENDING` vide.
- La section figée du canal publié fournit également les notes de la release
  GitHub correspondante.
- Une release sans ajout, modification ou correction pertinent pour le joueur
  est autorisée. Ses notes portent la mention : « Cette version ne comporte
  aucun changement visible pour le joueur. »
- Si le commit et le tag ont été poussés mais que la publication GitHub échoue,
  la reprise conserve la version, les notes et l’horodatage déjà figés. Elle
  reprend la publication existante sans consommer un nouveau `PENDING` ni créer
  une nouvelle version. Le workflow actuel, qui refuse un tag existant, devra
  être adapté.
- Le suivi commence avec les prochaines releases, sans reconstruction
  automatique des anciennes notes à partir des commits. Une relecture manuelle
  des 15 commits a permis de documenter la première bêta taguée et de préparer
  la synthèse de la première stable, à la demande du responsable du projet.

## Périmètre implémenté

- Créer les deux historiques avec leur structure `PENDING`, `Add`, `Modify`,
  `Fixes`, sans inventer d’historique antérieur.
- Fournir le contrôle avant commit, son installation et sa documentation,
  avec les exemptions justifiées par canal.
- Intégrer le gel des notes, leur publication sur GitHub et la reprise de
  publication aux outils de release existants.
- Documenter la rédaction et la maintenance de la synthèse stable.
- La future page de notes de version reste hors du périmètre de cette mise en
  place ; elle exploitera ces historiques ultérieurement.

## Critères de validation

- Un commit modifiant un comportement joueur sans mise à jour du `PENDING`
  d’un canal non exempté est refusé ; un commit mettant à jour les notes
  concernées est accepté. Le contrôle porte sur les changements inclus dans
  le commit, pas sur des modifications non indexées.
- Une exemption sans canal reconnu ou sans motif est refusée. Une exemption
  justifiée de la stable permet une correction propre à la bêta si ses notes
  bêta sont mises à jour. Un commit sans effet joueur peut être exempté pour
  les deux canaux.
- Une structure invalide des notes est refusée. Le contrôle et les exemptions
  restent compatibles avec les commits automatiques de préparation de release.
- Une release bêta fige uniquement ses notes ; le `PENDING` stable reste
  intact. Une release stable fige sa synthèse et vide les deux `PENDING`,
  en conservant les anciennes versions.
- La version et l’horodatage figés correspondent aux notes utilisées pour la
  release GitHub ; une release sans changement joueur contient la mention
  standard prévue ci-dessus.
- Une reprise après échec de publication conserve la version, l’horodatage
  et les notes du commit déjà tagué, même si `main` a évolué depuis. Aucun
  nouveau `PENDING` n’est consommé.
- L’installation du contrôle local et les commandes permettant de vérifier
  ces scénarios sont documentées. Sa capacité de contrôle structurel reste
  distincte de la relecture humaine de la pertinence des notes.

## Rédaction et contrôle local

Python 3.10 ou plus récent doit être disponible sous le nom `python3` dans le
terminal utilisé par Git (notamment Git Bash sous Windows). Installer le hook
après chaque nouveau clone, depuis la racine du dépôt :

```sh
python3 scripts/install_hooks.py
```

Le script configure `core.hooksPath=.githooks` pour ce dépôt. Il refuse de
remplacer un autre `core.hooksPath` : dans ce cas, intégrer l'appel à
`python3 scripts/changelog.py check-commit "$1"` dans le hook `commit-msg`
existant. Le hook versionné se lance depuis la racine du dépôt.

Pour chaque changement joueur, modifier les listes de `PENDING` dans
`CHANGES.beta.md` et `CHANGES.md`, puis indexer les fichiers avec le code.
Conserver exactement les titres `Add`, `Modify`, `Fixes`, dans cet ordre.
Les dates sont attribuées par l'outil ; ne pas figer les notes à la main.
Le contrôle compare le contenu de `PENDING` dans l'index à celui de `HEAD` :
modifier seulement une ancienne version, un titre ou un fichier non indexé ne
suffit pas. Il vérifie les deux structures, y compris en présence d'exemptions.

La synthèse stable décrit le résultat final depuis la dernière stable. Par
exemple, ajouter une fonctionnalité dans `Add`, ajuster sa description au fil
des bêtas, puis y conserver son état final. Les réglages et corrections propres
à ces bêtas sont détaillés dans leurs notes ; ils ne créent pas d'entrées
`Modify` ou `Fixes` supplémentaires dans la synthèse stable.

Le contrôle exige par défaut une modification de chaque `PENDING` pour tous les
commits, sans tenter de déduire l'effet joueur à partir des fichiers modifiés.
Pour une documentation interne ou ce chantier de publication, conserver les
`PENDING` vides et ajouter les deux lignes suivantes au message du commit :

```text
Changelog-Exempt: stable — outillage interne sans effet joueur
Changelog-Exempt: beta — outillage interne sans effet joueur
```

Chaque ligne concerne un seul canal et exige un motif après le tiret cadratin
`—`. Pour une correction propre à la bêta, mettre à jour et indexer ses notes,
puis justifier seulement l'exemption stable. Les messages de préparation
Conventional Commit produits par le workflow exemptent explicitement les deux
canaux : ils figent des notes déjà rédigées, sans ajouter de changement joueur.

L’historique initial a été relu manuellement à la demande du responsable du
projet : `CHANGES.beta.md` documente `0.1.0-beta.1`, avec l’horodatage UTC du
commit de préparation associé au tag léger. Il ne prétend pas dater la mise
en ligne sur GitHub. Le `PENDING` stable synthétise les fonctionnalités de la
première stable à venir ; aucune version stable n’est encore taguée. Le
`PENDING` bêta documente également, à la demande du responsable du projet,
les améliorations de l’analyse de combat réalisées après ce tag. Ces notes
précisent que cette page est réservée aux builds de développement et absente
des releases joueur.
Les ajustements et corrections antérieurs à la première bêta sont intégrés
à la description finale des ajouts, sans inventer de versions intermédiaires.
La pertinence des notes et des motifs reste une responsabilité de relecture
humaine, distincte du contrôle structurel local.

## Gel et reprise de publication

Les workflows stable et bêta acceptent `version` (`major.minor.patch`) et un
champ optionnel `resume_tag`. Laisser ce dernier vide pour une nouvelle release.
`prepare_release.py` met à jour Cargo et l'historique concerné, attribue une date
UTC et fournit la section figée au workflow dans un fichier Markdown temporaire.
La stable vide aussi le `PENDING` bêta en préservant ses anciennes versions.
Le workflow inclut les historiques dans le commit et pousse le commit et le tag
atomiquement, puis crée un brouillon GitHub avec les mêmes notes. Le ZIP est
chargé avant de rendre la release publique.

Après un échec suivant la poussée du tag, relancer le workflow du même canal
avec la même version de base et le tag exact dans `resume_tag` (par exemple
`version=1.2.3`, `resume_tag=v1.2.3-beta.2`). Le compteur bêta n'est alors pas
incrémenté. L'outil vérifie la version des deux fichiers Cargo et les notes du
tag, puis extrait ce commit en HEAD détachée pour reconstruire le ZIP. Le commit
et le tag ne sont pas recréés ni poussés, et le `PENDING` courant de `main` n'est
pas consommé. La date et les notes proviennent exclusivement du tag.

Un brouillon existant est repris, son ZIP est remplacé si nécessaire, puis il
est publié. Une release déjà publique avec les mêmes notes et le ZIP attendu
est considérée comme terminée. Une erreur réseau lors de la recherche de la
release arrête la reprise. Les tags antérieurs à ce système, dépourvus de notes
figées, ne sont pas éligibles à la reprise automatique.

## Vérifications reproductibles

Depuis la racine du dépôt :

```sh
python3 scripts/changelog.py validate
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/changelog.py check-commit /chemin/message-commit.txt
```

La dernière commande vérifie l'index réel ; les tests utilisent des dépôts Git
temporaires et simulent GitHub sans réseau ni secret. Ils couvrent les notes
indexées, les exemptions, les structures invalides, l'installation et l'exécution
du hook, le gel par canal, les releases vides, la reprise sur un tag malgré de
nouveaux commits, ainsi que la publication et ses échecs. Ils sont également
exécutés par le workflow Windows avant la préparation d'une release.
