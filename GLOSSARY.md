# Aetheris

Ce glossaire définit les notions métier de l’application compagnon Aetheris pour Aion 2.

## Paramétrage du jeu

**Éditeur Engine.ini**:
Page de l’application qui permet de modifier des réglages de `Engine.ini`, regroupés en catégories logiques et accompagnés d’explications et de valeurs d’optimisation conseillées.
_Avoid_: Page de recommandations

**Copie de référence**:
Référence de l’état d’origine contenant les chemins de contenu d’Aion 2 et les valeurs DLSS/FSR d’origine fournies par le joueur. Les personnalisations du fichier courant ne définissent pas cet état d’origine.
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

## Analyse du combat

**Trame réseau capturée**:
Paquet brut observé par une source de capture réseau ou lu depuis un fichier PCAPNG; plusieurs trames peuvent contribuer à un même message applicatif.
_Avoid_: Message applicatif, événement de combat

**Message applicatif**:
Unité du protocole AION 2 reconstruite à partir du flux TCP et affichée sur une ligne de la table de debug, qu’elle soit décodée ou inconnue.
_Avoid_: Trame réseau capturée, paquet TCP

**Connexion TCP**:
Échange TCP identifié par ses deux extrémités réseau; une reconnexion crée une nouvelle connexion, même si elle appartient à la même session de capture.
_Avoid_: Session de capture, flux TCP directionnel

**Flux TCP directionnel**:
Suite ordonnée d’octets allant d’une extrémité d’une connexion TCP vers l’autre; les deux directions d’une connexion sont suivies séparément.
_Avoid_: Paquet TCP, connexion TCP

**Section décodée**:
Interprétation structurée d’une partie d’un message applicatif, comme une mise à jour de HP ou un montant candidat de dégâts; un même message peut en contenir plusieurs.
_Avoid_: Traduction, événement confirmé

**Niveau de confiance de décodage**:
Qualification du soutien disponible pour une section décodée : observé, fortement étayé, probable, hypothèse ou inconnu.
_Avoid_: Valeur décodée, certitude métier

**Alignement expérimental offline**:
Hypothèse explicite d’une frontière de message au premier octet du flux reconstruit d’une capture sans début de connexion; elle permet une exploration sans établir que cet alignement est correct.
_Avoid_: Synchronisation confirmée, connexion complète

**Session de capture**:
Ensemble persistant créé pour l’analyse d’un PCAPNG ou pour un intervalle de capture réseau explicite; il regroupe la source, les données brutes, les messages applicatifs et leurs sections décodées, et peut contenir plusieurs connexions TCP.
_Avoid_: Import isolé, session réseau

**Montant affiché**:
Valeur de dégâts présentée par l’interface du jeu et relevée comme observation de gameplay.
_Avoid_: Montant candidat décodé, variation de HP

**Montant candidat décodé**:
Valeur numérique extraite d’un message applicatif selon une interprétation encore à valider; elle n’est pas assimilée automatiquement au montant affiché.
_Avoid_: Montant affiché, variation de HP

**Variation de HP**:
Différence observée entre deux valeurs successives de HP d’une entité; elle peut être liée à plusieurs causes et ne représente pas automatiquement les dégâts d’un seul coup.
_Avoid_: Montant affiché, montant candidat décodé
