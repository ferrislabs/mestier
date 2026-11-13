# Cursus: separate Apprentis and Tuteurs pages — design

## Intent
The "Participants" page mixes apprentices and company tutors. Users think of them as two directories: participants are students; tutors are created elsewhere.

## Decisions (approved)
- Backend unchanged: a participant keeps its kind(s) and the list endpoint already filters on `kind`.
- Sidebar, group Annuaire: Apprentis, Tuteurs, Entreprises (counts per kind).
- `/cursus/apprentis` replaces `/cursus/participants`: apprentices only, no Type select; search + status toggle (Tous / En attente / Actifs / Inactifs). Create in 3 steps: Identité (Nom, Prénom) · Contact (E-mail, Téléphone, Adresse) · Apprentissage (Date de naissance, Code INE).
- `/cursus/tuteurs` (new): tutors only; columns Nom, Entreprise, Fonction, Statut; status toggle + Entreprise select (`host_company_id`). Create in 3 steps: Identité (Nom, Prénom) · Contact (E-mail, Téléphone) · Entreprise (Entreprise d'accueil, Fonction). Invite works as for apprentices.
- Details: `/cursus/apprentis/<id>` and `/cursus/tuteurs/<id>` reuse the participant detail UI.
- Host company detail: each tutor name links to its Tuteur page.
- Any link that pointed to `/cursus/participants[/<id>]` goes to the right page for the person's kind.
