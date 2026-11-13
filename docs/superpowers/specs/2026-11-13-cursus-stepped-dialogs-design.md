# Cursus stepped creation dialogs — design

## Intent
The long Cursus creation dialogs show every field at once. Split them into steps so each screen asks one kind of question.

## Scope
- Stepped: Nouveau participant, Nouvelle entreprise, contract dialog (create and edit).
- Not stepped (3–5 fields, steps would only add clicks): promotion, triad, training, establishment.
- The host company detail page keeps its inline single-page edit form.

## Approaches
1. One shared pure `StepperDialog` building block, each dialog declares its steps. **Picked.**
2. Per-dialog ad-hoc step state. Rejected: three copies of the same navigation and validation logic.

## Steps
- Participant: 1 Identité (Nom, Prénom, Type) · 2 Contact (E-mail, Téléphone, Adresse) · 3 depends on Type — Apprenti (Date de naissance, Code INE) or Tuteur (Entreprise d'accueil, Fonction).
- Entreprise: 1 Identité légale (Nom, SIRET, NAF, Forme juridique, Groupe, Capital, Directeur) · 2 Coordonnées (Téléphone, E-mail, Site, adresses) · 3 Activité (effectifs entreprise/groupe, domaines, missions).
- Contrat: 1 Contrat (Entreprise d'accueil, Type, Numéro) · 2 Période (Début, Fin, Durée) · 3 Commentaire.

## Behaviour (my calls, user did not object)
- A step indicator at the top: done steps are clickable to go back, future steps are not.
- "Suivant" validates only the current step's fields and stays put on error; Enter does the same.
- Footer: Annuler on step 1, Précédent afterwards; the last step's primary button keeps the dialog's current submit label.
- Nothing is sent before the last step. A server error shows on the last step; the user can go back without losing input.
- Closing and reopening the dialog starts at step 1 with empty fields (create) or the record's values (edit).
