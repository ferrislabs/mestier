#let data = json("/data.json")

#let ink = rgb("#1f3d2b")
#let late = rgb("#b42318")
#let muted = luma(100)
#let rule = luma(190)

#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 1.5cm, top: 1.5cm, bottom: 1.8cm),
  footer: [
    #line(length: 100%, stroke: 0.5pt + rule)
    #v(-4pt)
    #grid(
      columns: (1fr, auto),
      text(size: 8pt, fill: muted)[Édité le #data.generated],
      text(size: 8pt, fill: muted, context counter(page).display()),
    )
  ],
)
#set text(lang: "fr", font: "Inter", size: 9pt, hyphenate: false)
#set par(justify: false)

#let dash(value) = if value == none or value == "" { [—] } else { value }

#let stacked(..items) = {
  let kept = items.pos().filter(item => item != none and item != "")
  if kept.len() == 0 { [—] } else { kept.join(linebreak()) }
}

#block(
  width: 100%,
  fill: ink,
  inset: (x: 12pt, y: 10pt),
  radius: 2pt,
  grid(
    columns: (1fr, auto),
    align: (left + horizon, right + horizon),
    text(fill: white, weight: "bold", size: 15pt)[Prestations d'entretien, #data.week],
    text(fill: white, size: 10pt)[#data.span],
  ),
)

#v(6pt)
#text(size: 10pt)[
  #data.total
  #if data.total > 1 [prestations] else [prestation]
  #if data.overdue > 0 [
    dont #text(fill: late, weight: "bold")[#data.overdue en retard]
  ]
]
#v(4pt)

#let columns = (1.5fr, 1.4fr, 2.7cm, 2.6cm, 1.8cm, 1.6fr)

#let cells(row) = (
  stacked(
    text(weight: "bold", row.site),
    if row.reference != none { text(fill: muted, size: 8pt, row.reference) },
  ),
  row.line,
  dash(row.last_visit),
  if row.overdue {
    stacked(text(fill: late, weight: "bold", row.due_on), text(fill: late, weight: "bold", size: 8pt)[En retard])
  } else {
    row.due_on
  },
  dash(row.planned),
  if row.contact == none { [—] } else {
    stacked(row.contact.name, row.contact.phone, row.contact.email)
  },
)

#if data.groups.len() == 0 [
  #v(1cm)
  #align(center, text(size: 11pt, fill: muted)[Aucune prestation due cette semaine.])
]

#for group in data.groups {
  block(sticky: true, above: 14pt, below: 5pt, text(fill: ink, weight: "bold", size: 12pt, group.town))
  table(
    columns: columns,
    stroke: (x: none, y: 0.5pt + rule),
    inset: (x: 6pt, y: 6pt),
    fill: (_, y) => if y == 0 { ink } else { none },
    table.header(
      ..([Site], [Prestation], [Dernière visite], [Échéance], [Heures], [Contact])
        .map(label => text(fill: white, weight: "bold", label)),
    ),
    ..group.rows.map(cells).flatten(),
  )
}
