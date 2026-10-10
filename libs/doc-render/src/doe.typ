#let data = json("/data.json")
#let doc = data.doc
#let files = data.files
#let style = doc.style
#let primary = rgb(style.primary)
#let secondary = rgb(style.secondary)
#let on_primary = rgb(style.on_primary)
#let muted = luma(90)

#set text(lang: "fr", font: style.body_font, size: 10.5pt)

// Titles use the heading font; everything else inherits the body font.
#let titled(..args) = text(font: style.heading_font, ..args)
#set par(justify: false)

#let dash(value) = if value == none or value == "" { [—] } else { value }

#let stacked(..items) = {
  let kept = items.pos().filter(item => item != none and item != "")
  if kept.len() == 0 { [—] } else { kept.join(linebreak()) }
}

#let multiline(value) = if value == none or value == "" {
  [—]
} else {
  value.replace("\r", "").split("\n").join(linebreak())
}

#let band(body) = block(
  width: 100%,
  fill: primary,
  inset: (x: 12pt, y: 10pt),
  radius: 2pt,
  titled(on_primary, weight: "bold", size: 15pt, body),
)

#let picture(key, ..args) = image(files.at(key).path, ..args)

#let captioned(photo, height) = block(breakable: false, width: 100%)[
  #align(center, picture(photo.key, width: 100%, height: height, fit: "contain"))
  #if photo.caption != none {
    align(center, text(size: 9pt, fill: muted, photo.caption))
  }
]

#let full_pages(key) = {
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

#show heading.where(level: 1): it => band(upper(it.body))
#show heading.where(level: 2): it => {
  v(10pt)
  titled(secondary, weight: "bold", size: 14pt, it.body)
  v(4pt)
}

#let contractor = doc.parties.find(p => p.role == "contractor")

#let party_block(party) = block(breakable: false, width: 100%)[
  #grid(
    columns: (auto, 1fr),
    column-gutter: 14pt,
    if party.logo_key != none {
      picture(party.logo_key, height: 2cm, width: 4cm, fit: "contain")
    },
    [
      #text(secondary, weight: "bold", size: 11pt, party.role_label) \
      #text(weight: "bold", size: 13pt, party.name) \
      #stacked(party.address, party.phone, party.email)
    ],
  )
]

#set page(paper: "a4", margin: (x: 2cm, y: 2.4cm))

#let logo_box = (
  small: (height: 1.6cm, width: 4cm),
  medium: (height: 2.4cm, width: 6cm),
  large: (height: 3.4cm, width: 8.5cm),
).at(style.cover_logo_size)
#let logo_align = (left: left, center: center, right: right).at(style.cover_logo_position)

#page(header: none, footer: none)[
  #if style.logo_key != none {
    align(logo_align, picture(style.logo_key, ..logo_box, fit: "contain"))
  }
  #v(if doc.cover_photo_key != none { 1.2cm } else { 2.5cm })
  #if doc.place != none {
    text(14pt, fill: muted, doc.place)
    parbreak()
  }
  #titled(32pt, weight: "bold", doc.title)
  #if doc.lot != none {
    parbreak()
    text(16pt, doc.lot)
  }
  // The photo takes whatever height the cover has left, so the cover stays
  // one page whatever the logo size and title length.
  #if doc.cover_photo_key != none {
    v(0.8cm)
    block(width: 100%, height: 1fr, clip: true, radius: 2pt,
      picture(doc.cover_photo_key, width: 100%, height: 100%, fit: "cover"))
    v(0.4cm)
  } else {
    v(1.2cm)
  }
  #block(width: 100%, fill: primary, inset: (x: 14pt, y: 16pt))[
    #align(center, titled(on_primary, weight: "bold", size: 17pt, style.cover_title))
  ]
  #v(1cm)
  #grid(
    columns: (auto, 1fr),
    row-gutter: 8pt,
    column-gutter: 18pt,
    text(weight: "bold")[Date], data.issued_fr,
    text(weight: "bold")[Rédacteur], dash(doc.author),
    text(weight: "bold")[Indice], doc.revision,
  )
  #v(if doc.cover_photo_key != none { 0.8cm } else { 1fr })
  #if contractor != none { party_block(contractor) }
]

#set page(
  header: [
    #grid(
      columns: (auto, 1fr, auto),
      column-gutter: if style.header_logo and style.logo_key != none { 8pt } else { 0pt },
      align: horizon,
      if style.header_logo and style.logo_key != none {
        picture(style.logo_key, height: 0.7cm, width: 2.5cm, fit: "contain")
      },
      doc.header, [Indice #doc.revision],
    )
    #v(-4pt)
    #line(length: 100%, stroke: 0.5pt + muted)
  ],
  footer: [
    #line(length: 100%, stroke: 0.5pt + muted)
    #v(-4pt)
    #grid(
      columns: (1fr, auto),
      doc.footer, context counter(page).display(),
    )
  ],
)

#titled(primary, weight: "bold", size: 20pt, style.parties_title)
#v(10pt)
#for party in doc.parties {
  party_block(party)
  v(16pt)
}

#pagebreak(weak: true)
#titled(primary, weight: "bold", size: 20pt, style.contents_title)
#v(10pt)
#outline(title: none, depth: 2)

#for section in doc.sections {
  pagebreak(weak: true)
  let title = [#section.number. #section.title]
  let content = section.content
  if content.kind == "supplies" {
    heading(level: 1, numbering: none, title)
    for (index, entry) in content.supplies.enumerate() {
      if index > 0 { pagebreak(weak: true) }
      let supply = entry.supply
      let supplier = supply.supplier
      heading(level: 2, numbering: none)[#entry.number #entry.heading]
      let datasheet_pages = supply.datasheet_keys.map(key => files.at(key).pages).sum(default: 0)
      let attachments = ()
      if supply.photos.len() > 0 { attachments.push("Photos") }
      if datasheet_pages > 0 { attachments.push("fiche technique (" + str(datasheet_pages) + " p.)") }
      let rows = (
        ([Désignation], dash(supply.designation)),
        ([Correspondance DPGF], dash(supply.dpgf_ref)),
        ([Destination], dash(supply.destination)),
        (
          [Fournisseur],
          if supplier == none { [—] } else { stacked(supplier.name, supplier.address, supplier.phone, supplier.email) },
        ),
        ([#content.maker_label], dash(supply.maker)),
        ([Caractéristiques techniques], multiline(supply.specs)),
        ([Pièces jointes], if attachments.len() == 0 { [—] } else { attachments.join(", ") }),
      )
      table(
        columns: (5cm, 1fr),
        stroke: 0.5pt + luma(180),
        inset: 7pt,
        table.header(table.cell(colspan: 2, fill: primary)[
          #titled(on_primary, weight: "bold", style.supply_table_title)
        ]),
        ..rows.map(row => (text(weight: "bold", row.at(0)), row.at(1))).flatten(),
      )
      if supply.photos.len() > 0 {
        v(12pt)
        grid(
          columns: (1fr, 1fr, 1fr),
          gutter: 10pt,
          ..supply.photos.map(photo => captioned(photo, 4.6cm)),
        )
      }
      for key in supply.datasheet_keys { full_pages(key) }
    }
  } else if content.kind == "photos" {
    let per_page = (one: 1, two: 2, four: 4).at(content.per_page)
    let height = (one: 17cm, two: 8.2cm, four: 6.6cm).at(content.per_page)
    let columns = if per_page == 4 { (1fr, 1fr) } else { (1fr,) }
    let chunks = content.photos.chunks(per_page)
    if chunks.len() == 0 { heading(level: 1, numbering: none, title) }
    for (index, chunk) in chunks.enumerate() {
      if index == 0 {
        heading(level: 1, numbering: none, title)
        v(8pt)
      } else {
        pagebreak()
      }
      grid(columns: columns, gutter: 10pt, ..chunk.map(photo => captioned(photo, height)))
    }
  } else if content.kind == "plans" {
    heading(level: 1, numbering: none, title)
    v(10pt)
    for (index, plan) in content.plans.enumerate() {
      let label = if plan.caption != none { plan.caption } else { "Plan " + str(index + 1) }
      [#(index + 1). #label]
      linebreak()
    }
    for plan in content.plans { full_pages(plan.key) }
  } else if content.kind == "blank_pages" {
    for index in range(content.count) {
      if index == 0 {
        heading(level: 1, numbering: none, title)
        v(14pt)
      } else {
        pagebreak()
      }
      for _ in range(24) {
        block(spacing: 0pt, width: 100%, height: 24pt, stroke: (bottom: 0.4pt + luma(170)))
      }
    }
  }
}
