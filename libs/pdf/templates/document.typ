// ── Mestier document template (quote / invoice / deposit / balance)
// Data is passed via sys.inputs["data"] as a JSON-encoded DocumentSnapshot.

#import sys: inputs

// ── helpers ──────────────────────────────────────────────────────────────────

// Convert integer cents to a French-formatted euro string.
// e.g. 1234567 → "12 345,67 €"
#let fmt-cents(c) = {
  let c = int(c)
  let euros = int(c / 100)
  let cts = c - euros * 100
  let cts-str = if cts < 10 { "0" + str(cts) } else { str(cts) }

  // Group thousands with a narrow no-break space (U+202F)
  let e-str = str(euros)
  let groups = ()
  let s = e-str
  while s.len() > 3 {
    groups.insert(0, s.slice(s.len() - 3))
    s = s.slice(0, s.len() - 3)
  }
  if s.len() > 0 {
    groups.insert(0, s)
  }
  let formatted = groups.join("\u{202f}")

  formatted + "," + cts-str + " \u{20ac}"
}

// ── data ─────────────────────────────────────────────────────────────────────

#let data = json(inputs.at("data"))

// ── page & text setup ────────────────────────────────────────────────────────

#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.8cm, left: 2cm, right: 2cm),
)

#set text(font: "New Computer Modern", size: 10pt, lang: "fr")

#set par(justify: false, leading: 0.65em)

// ── palette ──────────────────────────────────────────────────────────────────

#let accent     = rgb("#1a1a2e")
#let light-bg   = rgb("#f5f5f5")
#let border-col = rgb("#cccccc")

// ─────────────────────────────────────────────────────────────────────────────
// HEADER: seller (left) + document title / ref / dates (right)
// ─────────────────────────────────────────────────────────────────────────────

#grid(
  columns: (1fr, 1fr),
  gutter: 12pt,

  // ── left: seller identity ──────────────────────────────────────────────────
  stack(dir: ttb, spacing: 3pt,
    text(weight: "bold", size: 13pt)[#data.seller.name],
    if data.seller.address != none [#data.seller.address],
    if data.seller.siret != none [SIRET : #data.seller.siret],
    if data.seller.rcs   != none [RCS : #data.seller.rcs],
    if data.seller.ape   != none [APE : #data.seller.ape],
    if data.seller.vat_intracom != none [N° TVA : #data.seller.vat_intracom],
  ),

  // ── right: document heading ────────────────────────────────────────────────
  align(right,
    stack(dir: ttb, spacing: 6pt,
      block(
        fill: accent,
        inset: (x: 12pt, y: 8pt),
        radius: 4pt,
        text(weight: "bold", size: 14pt, fill: white)[#data.title],
      ),
      text(weight: "bold")[
        #if data.reference != none [#data.reference] else [Brouillon]
      ],
      if data.issued_date != none [Date : #data.issued_date],
      if data.due_date    != none [Échéance : #data.due_date],
    )
  ),
)

#v(12pt)
#line(length: 100%, stroke: 0.5pt + border-col)
#v(14pt)

// ─────────────────────────────────────────────────────────────────────────────
// CUSTOMER BLOCK (right-aligned)
// ─────────────────────────────────────────────────────────────────────────────

#align(right,
  block(
    stroke: 0.5pt + border-col,
    inset: 10pt,
    radius: 4pt,
    width: 52%,
    stack(dir: ttb, spacing: 3pt,
      text(weight: "bold")[Destinataire],
      data.customer.name,
      if data.customer.address != none [#data.customer.address],
    )
  )
)

#v(16pt)

// ─────────────────────────────────────────────────────────────────────────────
// LINE ITEMS TABLE
// ─────────────────────────────────────────────────────────────────────────────

#let zebra-fill(_, row) = {
  if row == 0 { accent }
  else if calc.rem(row, 2) == 0 { light-bg }
  else { white }
}

#let col-align(col, _) = {
  if col == 0 { left } else { right }
}

#block(below: 16pt,
  table(
    columns: (1fr, 5%, 8%, 12%, 8%, 12%),
    inset: 6pt,
    stroke: none,
    fill: zebra-fill,
    align: col-align,

    // header row
    table.cell[#text(fill: white, weight: "bold")[Désignation]],
    table.cell[#text(fill: white, weight: "bold")[Qté]],
    table.cell[#text(fill: white, weight: "bold")[Unité]],
    table.cell[#text(fill: white, weight: "bold")[PU HT]],
    table.cell[#text(fill: white, weight: "bold")[TVA %]],
    table.cell[#text(fill: white, weight: "bold")[Total HT]],

    // data rows — spread each line's 6 cells
    ..data.lines.map(line => (
      line.label,
      line.quantity,
      line.unit,
      fmt-cents(line.unit_price_cents),
      line.vat_rate + " %",
      fmt-cents(line.line_ht_cents),
    )).flatten(),
  )
)

// ─────────────────────────────────────────────────────────────────────────────
// TOTALS TABLE (right-aligned, 45% width)
// ─────────────────────────────────────────────────────────────────────────────

// Build rows dynamically
#let totals-rows = (
  // Total HT
  [Total HT], [#fmt-cents(data.total_ht_cents)],

  // VAT breakdown
  ..data.vat_breakdown.map(b => (
    [TVA #b.rate %],
    [#fmt-cents(b.vat_cents)],
  )).flatten(),
)

// Separator row spanning 2 columns
#let sep-row = (
  table.cell(colspan: 2, line(length: 100%, stroke: 0.5pt + border-col)),
)

// Total TTC row
#let ttc-row = (
  table.cell[#text(weight: "bold")[Total TTC]],
  table.cell[#text(weight: "bold")[#fmt-cents(data.total_ttc_cents)]],
)

// Optional deposit
#let deposit-rows = if data.deposit_amount_cents != none {
  let label = if data.deposit_label != none { data.deposit_label } else { "Acompte" }
  (
    [#label],
    [#fmt-cents(data.deposit_amount_cents)],
  )
} else { () }

// Optional remaining
#let remaining-rows = if data.remaining_cents != none {
  (
    table.cell[#text(weight: "bold")[Reste à payer]],
    table.cell[#text(weight: "bold")[#fmt-cents(data.remaining_cents)]],
  )
} else { () }

#align(right,
  block(
    stroke: 0.5pt + border-col,
    inset: 10pt,
    radius: 4pt,
    width: 45%,
    table(
      columns: (1fr, auto),
      inset: (x: 4pt, y: 4pt),
      stroke: none,
      ..totals-rows,
      ..sep-row,
      ..ttc-row,
      ..deposit-rows,
      ..remaining-rows,
    )
  )
)

#v(16pt)

// ─────────────────────────────────────────────────────────────────────────────
// PAYMENT TERMS
// ─────────────────────────────────────────────────────────────────────────────

#if data.payment_terms != none [
  #block(below: 10pt)[
    #text(weight: "bold")[Conditions de règlement :] #data.payment_terms
  ]
]

// ─────────────────────────────────────────────────────────────────────────────
// BANKING DETAILS
// ─────────────────────────────────────────────────────────────────────────────

#if data.seller.iban != none or data.seller.bic != none [
  #block(below: 10pt)[
    #text(weight: "bold")[Coordonnées bancaires :]
    #if data.seller.iban != none [ IBAN\u{00a0}: #data.seller.iban]
    #if data.seller.bic  != none [ — BIC\u{00a0}: #data.seller.bic]
  ]
]

// ─────────────────────────────────────────────────────────────────────────────
// LEGAL MENTIONS
// ─────────────────────────────────────────────────────────────────────────────

#if data.legal_mentions.len() > 0 [
  #v(16pt)
  #line(length: 100%, stroke: 0.5pt + border-col)
  #v(6pt)
  #for mention in data.legal_mentions [
    #text(size: 8pt)[#mention]
    #v(3pt)
  ]
]

// ─────────────────────────────────────────────────────────────────────────────
// FOOTER
// ─────────────────────────────────────────────────────────────────────────────

#if data.footer != none [
  #place(bottom + center, float: false)[
    #line(length: 100%, stroke: 0.5pt + border-col)
    #v(3pt)
    #text(size: 8pt, fill: rgb("#666666"))[#data.footer]
  ]
]
