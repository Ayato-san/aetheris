# Design System GPUI centralisé et contrôlé par les pages

Status: accepted; automatic scrolling and full-height tables amended on 12 November 2026.

Pour unifier l’interface d’Aetheris et éviter la duplication des styles et des
interactions, l’application principale adopte un Design System partagé sur GPUI,
indépendant des règles métier. Les pages détiennent les valeurs de référence et
acceptent ou refusent les changements utilisateur ; les composants conservent
leurs états transitoires. Cette séparation évite plusieurs sources de vérité et
permet de réutiliser les mêmes contrôles dans des contextes métier différents.

Les composants restent dans `src/ui/components/` et les tokens dans `src/ui/theme/`,
avec une palette de couleurs dédiée et des rôles sémantiques consommés par les
composants. Le thème sombre utilise des gris neutres et un accent turquoise commun.
Cette organisation permet de remplacer la palette sans modifier chaque composant ;
la première version utilise une reconstruction, sans sélection de thème à
l’exécution. Le compagnon reste hors périmètre et indépendant de GPUI, conformément
à l’[ADR 0004](0004-companion-crash-diagnostics-and-updates.md).

Les panneaux superposés partagent une infrastructure de positionnement, de
superposition et de focus pour éviter des règles incompatibles dans les usages
imbriqués. Le panneau de réglages composable est nommé Popover, distinct d’un menu
de commandes. Popup est modale par défaut ; sa fermeture peut être refusée par la
page. La navigation clavier et les états accessibles font partie des contrats
communs, avec vérification réelle des capacités de GPUI pour les lecteurs d’écran.

## Défilement et hauteur des tables

Le layout partagé gère l’état et l’affichage des scrollbars : une barre verticale
apparaît lorsque le contenu dépasse son conteneur et disparaît lorsqu’il tient à
nouveau. Les pages et les contrôles lui confient leur contenu et leurs contraintes
de taille, sans mesurer le dépassement ni monter une scrollbar manuellement.
Les listes virtualisées lui transmettent leur état de liste et conservent les
états nécessaires au rendu, au focus et à la navigation clavier. Cette
centralisation évite les règles de défilement concurrentes dans les conteneurs
imbriqués.

Une Table ordinaire peut afficher toutes ses lignes à leur hauteur naturelle.
Dans ce mode, le layout de page assure le défilement vertical ; la table n’ajoute
pas de zone défilante interne. Les tables à hauteur limitée et les tables
virtualisées conservent un conteneur contraint. Le bilan de l’[ADR 0009](0009-combat-history.md)
utilise un tableau unique à hauteur naturelle pour garder dégâts et indicateurs
sur une même ligne par compétence.

## Conséquences

- Les mises à jour programmatiques sont possibles pour tous les contrôles, sans
  événement de changement utilisateur. Les contraintes sont vérifiées et les mises
  à jour conjointes de configuration et de valeur sont atomiques. Une
  réinitialisation explicite peut abandonner un brouillon.
- Brouillon, valeur validée et valeur enregistrée sont distincts. L’acceptation est
  synchrone ; la persistance et les autres effets asynchrones appartiennent aux
  pages. Les champs valident sur Enter, Tab ou perte de focus : l’attente de deux
  secondes ne suffit plus à appliquer ou enregistrer une saisie dans l’éditeur
  Engine.ini lors de sa migration.
- La Table peut virtualiser son rendu, tandis que la page conserve les données,
  les états des contrôles, la pagination et le tri. Le Timer reste un affichage
  d’échéance, sans devenir un ordonnanceur métier.
- L’adoption est progressive : galerie de développement, puis éditeur Engine.ini
  entièrement converti comme page pilote, avant généralisation. Les valeurs
  visuelles exactes sont validées dans la galerie.

Les contrats des seize composants, les conventions de callbacks et les critères
de validation sont définis dans le [document de conception](../design-system.md).
Le [glossaire](../../GLOSSARY.md) précise les notions métier de brouillon de réglage
et de valeur validée. L’acceptation de cet ADR formalise la conception ; elle ne
signifie pas que sa migration est déjà implémentée.
