# Aetheris

Ce glossaire définit les notions métier de l’application compagnon Aetheris pour Aion 2.

## Paramétrage du jeu

**Éditeur Engine.ini**:
Page de l’application qui permet de modifier des réglages de `Engine.ini`, regroupés en catégories logiques et accompagnés d’explications et de valeurs d’optimisation conseillées.
_Avoid_: Page de recommandations

**Copie de référence**:
Copie des sections `[Core.System]` et `[/Script/Engine.RendererSettings]` d’un `Engine.ini` propre d’Aion 2, servant de référence au retour à l’état d’origine.
_Avoid_: Sauvegarde complète du fichier courant

**Page de recommandations**:
Page en lecture seule qui présente des options accessibles dans Aion 2 pour réduire les saccades, notamment en PvP de masse, en limitant le traitement d’informations non essentielles; elle porte sur les réglages en jeu, pas sur les propriétés ajoutées à `Engine.ini`.
_Avoid_: Éditeur Engine.ini, Paramétrage Engine.ini

**Saccades**:
Interruptions ou à-coups de l’affichage qui surviennent notamment en PvP de masse lorsque le jeu traite de nombreuses informations de jeu.
_Avoid_: Latence réseau, ping

**État d’Aion 2**:
Statut indiquant si le client Aion 2 est fermé, lancé ou indéterminé; il est visible dans la barre de titre et conditionne l’autorisation de modifier `Engine.ini`.
_Avoid_: État du lanceur

**Réglage d’optimisation recommandé**:
Propriété ajoutée à `Engine.ini` et conseillée par Aetheris pour l’optimisation ou une amélioration que les options du jeu ne permettent pas de régler.
_Avoid_: Réglage en jeu

**État d’origine**:
Valeurs de référence prévues pour un `Engine.ini` sans personnalisation; les propriétés ajoutées par Aetheris sont absentes dans cet état.
_Avoid_: Valeur conseillée

**État conseillé**:
Valeurs d’optimisation proposées par Aetheris pour les propriétés qu’il ajoute à `Engine.ini`.
_Avoid_: État d’origine, État personnalisé

**État personnalisé**:
Valeurs de `Engine.ini` personnalisées par le joueur, y compris celles déjà présentes lorsque Aetheris initialise ses réglages.
_Avoid_: État conseillé

**État non reconnu**:
État personnalisé d’un réglage dont la valeur présente ne peut pas être interprétée ou ne correspond à aucun choix proposé par Aetheris; elle reste inchangée jusqu’à ce que le joueur fasse un choix explicite ou applique les réglages conseillés.
_Avoid_: État d’origine

**État non défini**:
État d’un réglage absent du fichier d’origine; l’interface indique son absence et présente la valeur conseillée séparément, sans l’écrire automatiquement.
_Avoid_: État conseillé

**Profil conseillé DLSS/FSR**:
Choix exclusif entre Off, DLSS et FSR proposé selon le GPU et la compatibilité confirmée avec Aion 2; Aetheris conseille Off si aucune option n’est confirmée compatible.
_Avoid_: État d’origine

**Catégorie de réglages**:
Regroupement logique de réglages de l’éditeur `Engine.ini`, tel que les paramètres d’affichage, d’entrée ou de mémoire graphique; les 13 catégories prévues sont distinctes des sections physiques du fichier.
_Avoid_: Section INI (quand il s’agit d’un regroupement de l’interface)
