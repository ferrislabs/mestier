#let data = json("/data.json")
#let doc = data.doc
#let files = data.files
#let style = doc.style
#let primary = rgb(style.primary)
#let secondary = rgb(style.secondary)
#let on_primary = rgb(style.on_primary)
#let value_fill = white
#let label_fill = white
#let band_fill = white
#let rule = 0.5pt + luma(150)

#set text(lang: "fr", font: style.body_font, size: 9.5pt)
#set par(justify: false)
#set smartquote(enabled: false)
#set page(paper: "a4", margin: (x: 1.3cm, y: 1.2cm))
#set block(spacing: 0pt)

#let titled(..args) = text(font: style.heading_font, ..args)

#let fr(date) = if date == none { "" } else {
  let parts = date.split("-")
  parts.at(2) + "/" + parts.at(1) + "/" + parts.at(0)
}

#let picture(key, ..args) = image(files.at(key).path, ..args)

#let lines_of(..items) = {
  let kept = items.pos().filter(item => item != none and item != "")
  kept.map(item => item.replace("\r", "").split("\n")).flatten().join(linebreak())
}

#let multiline(value) = if value == none or value == "" {
  []
} else {
  value.replace("\r", "").split("\n").join(linebreak())
}

// A price reference does not wrap ("-" before a digit is no break point),
// so a long one shrinks to stay inside its column.
#let reference(value) = text(size: if value.len() > 16 { 6.5pt } else if value.len() > 13 { 7.5pt } else { 8.5pt }, value)

#let arrow = box(baseline: 1pt, polygon(fill: primary, stroke: none, (0pt, 0pt), (5pt, 3pt), (0pt, 6pt)))
#let label(body) = table.cell(fill: label_fill)[
  #arrow #h(3pt) #text(weight: "bold", body)
]

#let band(body) = table.cell(colspan: 3, fill: band_fill, inset: (x: 8pt, y: 6pt))[
  #titled(primary, weight: "bold", size: 11pt, body)
]

#let banner(tag, date) = {
  let cell_inset = (x: 8pt, y: 6pt)
  table(
    columns: (3.6cm, 1fr, 5cm),
    stroke: rule,
    inset: cell_inset,
    align: horizon,
    table.cell(rowspan: 3, align: center + horizon)[
      #if style.logo_key != none {
        picture(style.logo_key, width: 3.2cm, height: 2.2cm, fit: "contain")
      }
    ],
    table.cell(rowspan: 3, inset: (x: 12pt, y: 8pt))[
      #titled(primary, weight: "bold", size: 17pt, doc.title)
      #v(4pt)
      #text(size: 10pt, [> Demande d'Agrément de Fourniture])
    ],
    table.cell(fill: primary, align: center + horizon)[
      #titled(on_primary, weight: "bold", size: 13pt, tag)
    ],
    [Émetteur : #text(weight: "bold", doc.issuer)],
    [Date de diffusion : #text(weight: "bold", fr(date))],
  )
}

#let attachment_pages(key) = {
  let file = files.at(key)
  for number in range(1, file.pages + 1) {
    if file.kind == "pdf" {
      page(header: none, footer: none, margin: 0pt)[
        #image(file.path, page: number, width: 100%, height: 100%, fit: "contain")
      ]
    } else {
      page(header: none, footer: none, margin: 1.5cm)[
        #image(file.path, width: 100%, height: 100%, fit: "contain")
      ]
    }
  }
}

#let specs_size(specs) = if specs == none { 9.5pt } else {
  let length = specs.len()
  if length > 1400 { 6.5pt } else if length > 900 { 7.5pt } else if length > 450 { 8.5pt } else { 9.5pt }
}

#let visa_height = 1.6cm
#let specs_min = 4.6cm

#let sheet_body(sheet, specs_height) = {
  banner(sheet.label, sheet.issued_on)
  v(8pt)
  let supplier = sheet.supplier
  let names = sheet.attachments.map(attachment => attachment.name)
  let photo = if sheet.photo_key != none {
    block(width: 100%, height: specs_height, align(center + horizon, picture(
      sheet.photo_key,
      width: 100%,
      height: 100%,
      fit: "contain",
    )))
  }
  table(
    columns: (4.2cm, 1fr, 1fr),
    stroke: rule,
    inset: 7pt,
    band([IDENTIFICATION FOURNITURE]),
    label([Désignation]),
    table.cell(colspan: 2, fill: value_fill)[
      #titled(primary, weight: "bold", size: 11pt, sheet.designation)
    ],
    label([Correspondance]),
    table.cell(colspan: 2, fill: value_fill, multiline(sheet.price_ref)),
    label([Fournisseurs]),
    table.cell(colspan: 2, fill: value_fill)[
      #if supplier != none {
        text(weight: "bold", supplier.name)
        if supplier.address != none or supplier.phone != none or supplier.email != none {
          linebreak()
          lines_of(supplier.address, supplier.phone, supplier.email)
        }
      }
    ],
    label([Assemblage et pose]),
    table.cell(colspan: 2, fill: value_fill, multiline(sheet.installer)),
    label([Caractéristiques techniques]),
    table.cell(fill: value_fill)[
      #grid(
        columns: (0pt, 1fr),
        box(height: specs_height),
        text(size: specs_size(sheet.specs), multiline(sheet.specs)),
      )
    ],
    table.cell(align: center + horizon, photo),
    label([Pièces jointes]),
    table.cell(colspan: 2, fill: value_fill, text(size: 8.5pt, names.join([ · ]))),
    band([OBSERVATIONS]),
    table.cell(colspan: 3, fill: value_fill)[
      #grid(
        columns: (0pt, 1fr),
        box(height: 2.2cm),
        text(style: "italic", multiline(sheet.observations)),
      )
    ],
    band([DIFFUSION]),
    label([Maîtrise d'Ouvrage]),
    table.cell(colspan: 2, fill: value_fill, multiline(sheet.project_owner)),
    label([Maîtrise d'Œuvre]),
    table.cell(colspan: 2, fill: value_fill, multiline(sheet.project_manager)),
  )
  table(
    columns: (4.2cm, 1fr, 1fr, 1fr),
    stroke: rule,
    inset: 7pt,
    table.cell(colspan: 4, fill: band_fill, inset: (x: 8pt, y: 6pt))[
      #titled(primary, weight: "bold", size: 11pt, [VISAS])
    ],
    table.cell(fill: label_fill, []),
    ..([Acceptation], [Refus avec obs.], [Refus ferme]).map(head => table.cell(
      fill: label_fill,
      align: center,
    )[#text(weight: "bold", head)]),
    ..([Maîtrise d'Ouvrage], [Maîtrise d'Œuvre]).map(party => (
      label(party),
      ..range(3).map(_ => table.cell(align: top, box(height: visa_height - 14pt, text(size: 8.5pt, [date :])))),
    )).flatten(),
  )
}

// The technical specs stretch to fill whatever the page leaves free.
#let sheet_page(sheet) = {
  layout(size => {
    let first = size.height - measure(width: size.width, sheet_body(sheet, 0pt)).height
    let guess = calc.max(specs_min, first)
    let gap = size.height - measure(width: size.width, sheet_body(sheet, guess)).height
    sheet_body(sheet, calc.max(specs_min, guess + gap - 2pt))
  })
  for attachment in sheet.attachments { attachment_pages(attachment.key) }
}

#let tints = (
  pending: rgb("#fdeeb8"),
  accepted: rgb("#d4efd4"),
  rejected_with_comments: rgb("#fcdcb6"),
  rejected: rgb("#f7c3c3"),
)

#let legend = (
  (tints.pending, [En attente]),
  (tints.accepted, [Acceptée]),
  (tints.rejected_with_comments, [Refus avec observations]),
  (tints.rejected, [Refus ferme]),
)

#let summary_page(summary) = {
  banner([SOMMAIRE], summary.printed_on)
  v(8pt)
  grid(
    columns: (auto, auto, auto, auto),
    column-gutter: 16pt,
    ..legend.map(((tint, name)) => box[
      #box(width: 10pt, height: 10pt, fill: tint, stroke: rule, baseline: 1.5pt)
      #h(4pt)
      #text(size: 9pt, name)
    ]),
  )
  v(8pt)
  block(width: 100%, fill: band_fill, stroke: rule, inset: (x: 8pt, y: 6pt))[
    #titled(primary, weight: "bold", size: 11pt, [Récapitulatif des D.A.F])
  ]
  table(
    columns: (2.3cm, 2.6cm, 1fr, 2.05cm, 2.05cm, 2.1cm, 2.4cm),
    stroke: rule,
    inset: (x: 5pt, y: 6pt),
    align: horizon,
    fill: (_, row) => if row == 0 { label_fill } else { tints.at(summary.rows.at(row - 1).status) },
    table.header(
      ..([N° D.A.F], [N° PRIX], [DESIGNATION], [Envoyé], [Diffusion], [Retour], [Observation]).map(head => text(
        weight: "bold",
        size: 8.5pt,
        head,
      )),
    ),
    ..summary.rows.map(row => (
      text(weight: "bold", row.label),
      if row.price_ref != none { reference(row.price_ref) },
      row.designation,
      fr(row.sent_on),
      fr(row.issued_on),
      if row.returned_on != none { fr(row.returned_on) } else { row.status_label },
      if row.review_note != none { text(size: 8.5pt, multiline(row.review_note)) },
    )).flatten(),
  )
}

#let pages = ()
#if doc.summary != none {
  summary_page(doc.summary)
}
#for (index, sheet) in doc.sheets.enumerate() {
  if index > 0 or doc.summary != none { pagebreak() }
  sheet_page(sheet)
}
