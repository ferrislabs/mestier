# ADR 0003 : les documents de chantier sont composés avec Typst, en processus

- Status: Proposed
- Date: 2026-10-07
- Related: [chantier DOE](../chantiers/doe.md)

## Contexte

Le PDF des devis et factures est écrit à la main (`libs/handlers-quote/src/pdf.rs`) :
une page, une police Type1 de base, du texte ASCII (`escape_pdf_text` remplace
tout le reste par `?`), aucune image. C'est assez pour une facture.

Un DOE fait 20 à 40 pages : page de garde, sommaire avec numéros de page,
tableaux, photos, accents français, et des PDF de fournisseurs et des plans
insérés tels quels. Le renderer actuel ne peut rien de tout ça.

## Décision

Les documents de chantier sont composés avec **Typst** (`typst` + `typst-pdf`,
0.15.x), appelé en bibliothèque depuis un crate dédié `libs/doc-render`.

- Le domaine produit un `DoeDocument` (données pures, déjà numérotées) et
  définit le port `DoeRenderer`. Le crate `doc-render` l'implémente. Le domaine
  ne sait pas que Typst existe.
- Le gabarit Typst est un fichier `.typ` embarqué (`include_str!`), les données
  lui sont passées en JSON. Les fichiers (photos, PDF fournisseurs, plans) lui
  sont fournis en mémoire par un `World` qui ne lit ni le disque ni le réseau.
- Les polices sont embarquées dans le binaire. Aucune dépendance système.
- La composition tourne dans `spawn_blocking` : c'est du calcul, pas de l'I/O.

## Alternatives rejetées

- **Chromium headless / Gotenberg** : un service de plus à déployer, à
  superviser et à sécuriser (HTML injecté depuis des données utilisateur), pour
  un rendu que Typst fait en processus.
- **Étendre le renderer maison** : la pagination, le sommaire, les images et
  l'insertion de PDF, c'est réécrire un moteur de mise en page.
- **`printpdf` / `genpdf`** : bas niveau, pas de sommaire ni de flux de texte
  multipage ; on réécrirait la mise en page à la main.

## Conséquences

- Temps de compilation plus long pour le crate `doc-render` seul ; `mestier-core`
  n'en dépend pas.
- Taille du binaire en hausse (polices).
- **À vérifier avant de s'appuyer dessus (lot W1)** : insertion de pages PDF
  comme images dans Typst 0.15, images JPEG/PNG volumineuses, accents. Si
  l'insertion PDF échoue, repli : Typst compose les pages Mestier, `lopdf` (déjà
  dans `mestier-core`) intercale les pages fournisseurs. Le port ne change pas.
- Les devis et factures restent sur leur renderer. Les migrer est un autre sujet.
