use rmcp::schemars;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{api::Fault, files::Target};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    Supplies,
    Photos,
    Plans,
    BlankPages,
}

impl SectionKind {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Supplies => "supplies",
            Self::Photos => "photos",
            Self::Plans => "plans",
            Self::BlankPages => "blank_pages",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Supplies => "de fournitures",
            Self::Photos => "de photos",
            Self::Plans => "de plans",
            Self::BlankPages => "de pages libres",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub enum MakerLabel {
    Manufacturer,
    Nursery,
}

impl MakerLabel {
    fn tag(self) -> &'static str {
        match self {
            Self::Manufacturer => "manufacturer",
            Self::Nursery => "nursery",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub enum PerPage {
    One,
    Two,
    Four,
}

impl PerPage {
    fn tag(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Two => "two",
            Self::Four => "four",
        }
    }
}

#[derive(Debug, Default)]
pub struct SupplyPatch<'a> {
    pub title: Option<&'a str>,
    pub designation: Option<&'a str>,
    pub dpgf_ref: Option<&'a str>,
    pub destination: Option<&'a str>,
    pub supplier_name: Option<&'a str>,
    pub supplier_address: Option<&'a str>,
    pub supplier_phone: Option<&'a str>,
    pub supplier_email: Option<&'a str>,
    pub maker: Option<&'a str>,
    pub specs: Option<&'a str>,
}

#[derive(Debug)]
pub struct NewSection<'a> {
    pub title: &'a str,
    pub kind: SectionKind,
    pub maker_label: Option<MakerLabel>,
    pub per_page: Option<PerPage>,
    pub count: Option<u8>,
    pub position: Option<usize>,
}

#[derive(Debug, Default)]
pub struct SectionPatch<'a> {
    pub title: Option<&'a str>,
    pub enabled: Option<bool>,
    pub maker_label: Option<MakerLabel>,
    pub per_page: Option<PerPage>,
    pub count: Option<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SectionRef<'a> {
    pub id: Option<&'a str>,
    pub title: Option<&'a str>,
}

fn blank_to_null(value: &str) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}

fn sections(doe: &Value) -> &[Value] {
    doe["sections"].as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn sections_mut(doe: &mut Value) -> Result<&mut Vec<Value>, Fault> {
    doe["sections"]
        .as_array_mut()
        .ok_or_else(|| Fault::new("sections du DOE illisibles"))
}

fn kind_of(section: &Value) -> &str {
    section["content"]["kind"].as_str().unwrap_or("")
}

fn describe(doe: &Value, kind: Option<&str>) -> String {
    sections(doe)
        .iter()
        .filter(|section| kind.is_none_or(|kind| kind_of(section) == kind))
        .map(|section| {
            format!(
                "{} ({})",
                section["title"].as_str().unwrap_or("?"),
                section["id"].as_str().unwrap_or("?")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn pick_section(
    doe: &Value,
    reference: SectionRef<'_>,
    kind: SectionKind,
) -> Result<usize, Fault> {
    let all = sections(doe);
    let tag = kind.tag();
    let of_kind: Vec<usize> = (0..all.len())
        .filter(|&index| kind_of(&all[index]) == tag)
        .collect();
    let found = match (reference.id, reference.title) {
        (Some(id), _) => all
            .iter()
            .position(|section| section["id"].as_str() == Some(id)),
        (None, Some(title)) => all.iter().position(|section| {
            section["title"]
                .as_str()
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(title))
        }),
        (None, None) => {
            return match of_kind.as_slice() {
                [only] => Ok(*only),
                _ => Err(Fault::new(format!(
                    "précisez `section_id` ou `section_title` parmi les sections {} : {}",
                    kind.label(),
                    describe(doe, Some(tag))
                ))),
            };
        }
    };
    let index = found.ok_or_else(|| {
        Fault::new(format!(
            "section introuvable. Sections {} : {}",
            kind.label(),
            describe(doe, Some(tag))
        ))
    })?;
    if !of_kind.contains(&index) {
        return Err(Fault::new(format!(
            "cette section n'est pas une section {}. Sections {} : {}",
            kind.label(),
            kind.label(),
            describe(doe, Some(tag))
        )));
    }
    Ok(index)
}

fn pick_any_section(doe: &Value, reference: SectionRef<'_>) -> Result<usize, Fault> {
    let all = sections(doe);
    match (reference.id, reference.title) {
        (Some(id), _) => all
            .iter()
            .position(|section| section["id"].as_str() == Some(id))
            .ok_or_else(|| {
                Fault::new(format!(
                    "section introuvable. Sections : {}",
                    describe(doe, None)
                ))
            }),
        (None, Some(title)) => {
            let matches: Vec<usize> = (0..all.len())
                .filter(|&index| {
                    all[index]["title"]
                        .as_str()
                        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(title))
                })
                .collect();
            match matches.as_slice() {
                [one] => Ok(*one),
                [] => Err(Fault::new(format!(
                    "section introuvable. Sections : {}",
                    describe(doe, None)
                ))),
                _ => Err(Fault::new(format!(
                    "plusieurs sections portent ce titre, précisez `section_id` parmi : {}",
                    describe(doe, None)
                ))),
            }
        }
        (None, None) => Err(Fault::new(format!(
            "précisez `section_id` ou `section_title`. Sections : {}",
            describe(doe, None)
        ))),
    }
}

fn find_supply(doe: &Value, supply_id: &str) -> Result<(usize, usize), Fault> {
    for (section_index, section) in sections(doe).iter().enumerate() {
        if kind_of(section) != "supplies" {
            continue;
        }
        let found = section["content"]["supplies"]
            .as_array()
            .and_then(|supplies| {
                supplies
                    .iter()
                    .position(|supply| supply["id"].as_str() == Some(supply_id))
            });
        if let Some(supply_index) = found {
            return Ok((section_index, supply_index));
        }
    }
    Err(Fault::new(format!(
        "fourniture introuvable : « {supply_id} ». Les ids des fournitures sont dans get_doe"
    )))
}

pub fn update_supply(
    doe: &mut Value,
    supply_id: &str,
    patch: &SupplyPatch<'_>,
) -> Result<(), Fault> {
    if patch.designation.is_some_and(|text| text.trim().is_empty()) {
        return Err(Fault::new(
            "la désignation de la fourniture ne peut pas être vide",
        ));
    }
    let (section, index) = find_supply(doe, supply_id)?;
    let supply = &mut doe["sections"][section]["content"]["supplies"][index];
    if let Some(designation) = patch.designation {
        supply["designation"] = json!(designation);
    }
    for (name, value) in [
        ("title", patch.title),
        ("dpgf_ref", patch.dpgf_ref),
        ("destination", patch.destination),
        ("maker", patch.maker),
        ("specs", patch.specs),
    ] {
        if let Some(value) = value {
            supply[name] = blank_to_null(value);
        }
    }
    let details = [
        ("address", patch.supplier_address),
        ("phone", patch.supplier_phone),
        ("email", patch.supplier_email),
    ];
    match patch.supplier_name {
        Some(name) if name.trim().is_empty() => supply["supplier"] = Value::Null,
        Some(name) => {
            if !supply["supplier"].is_object() {
                supply["supplier"] =
                    json!({"name": name, "address": null, "phone": null, "email": null});
            }
            supply["supplier"]["name"] = json!(name);
            for (field, value) in details {
                if let Some(value) = value {
                    supply["supplier"][field] = blank_to_null(value);
                }
            }
        }
        None if details.iter().any(|(_, value)| value.is_some()) => {
            if !supply["supplier"].is_object() {
                return Err(Fault::new(
                    "cette fourniture n'a pas de fournisseur : précisez `supplier_name`",
                ));
            }
            for (field, value) in details {
                if let Some(value) = value {
                    supply["supplier"][field] = blank_to_null(value);
                }
            }
        }
        None => {}
    }
    Ok(())
}

pub fn delete_supply(doe: &mut Value, supply_id: &str) -> Result<(), Fault> {
    let (section, index) = find_supply(doe, supply_id)?;
    doe["sections"][section]["content"]["supplies"]
        .as_array_mut()
        .ok_or_else(|| Fault::new("section de fournitures illisible"))?
        .remove(index);
    Ok(())
}

fn check_count(count: u8) -> Result<(), Fault> {
    if (1..=20).contains(&count) {
        Ok(())
    } else {
        Err(Fault::new("`count` doit être compris entre 1 et 20 pages"))
    }
}

fn check_title(title: &str) -> Result<(), Fault> {
    if title.trim().is_empty() {
        Err(Fault::new("le titre de la section ne peut pas être vide"))
    } else {
        Ok(())
    }
}

fn mismatch(setting: &str, kind: &str) -> Fault {
    Fault::new(format!(
        "le réglage `{setting}` ne s'applique pas à une section de type {kind}"
    ))
}

pub fn add_section(doe: &mut Value, new: &NewSection<'_>) -> Result<Value, Fault> {
    check_title(new.title)?;
    let kind = new.kind;
    if new.maker_label.is_some() && kind != SectionKind::Supplies {
        return Err(mismatch("maker_label", kind.tag()));
    }
    if new.per_page.is_some() && kind != SectionKind::Photos {
        return Err(mismatch("per_page", kind.tag()));
    }
    if new.count.is_some() && kind != SectionKind::BlankPages {
        return Err(mismatch("count", kind.tag()));
    }
    let content = match kind {
        SectionKind::Supplies => json!({
            "kind": "supplies",
            "maker_label": new.maker_label.unwrap_or(MakerLabel::Manufacturer).tag(),
            "supplies": [],
        }),
        SectionKind::Photos => json!({
            "kind": "photos",
            "per_page": new.per_page.unwrap_or(PerPage::Two).tag(),
            "photos": [],
        }),
        SectionKind::Plans => json!({"kind": "plans", "plans": []}),
        SectionKind::BlankPages => {
            let count = new.count.unwrap_or(1);
            check_count(count)?;
            json!({"kind": "blank_pages", "count": count})
        }
    };
    let section = json!({
        "id": Uuid::now_v7().to_string(),
        "title": new.title.trim(),
        "enabled": true,
        "content": content,
    });
    let list = sections_mut(doe)?;
    let index = match new.position {
        None => list.len(),
        Some(position) if (1..=list.len() + 1).contains(&position) => position - 1,
        Some(position) => {
            return Err(Fault::new(format!(
                "position {position} invalide : le DOE a {} section(s), choisissez entre 1 et {}",
                list.len(),
                list.len() + 1
            )));
        }
    };
    list.insert(index, section.clone());
    Ok(section)
}

pub fn update_section(
    doe: &mut Value,
    reference: SectionRef<'_>,
    patch: &SectionPatch<'_>,
) -> Result<(), Fault> {
    let index = pick_any_section(doe, reference)?;
    let kind = kind_of(&doe["sections"][index]).to_owned();
    if let Some(title) = patch.title {
        check_title(title)?;
    }
    if patch.maker_label.is_some() && kind != "supplies" {
        return Err(mismatch("maker_label", &kind));
    }
    if patch.per_page.is_some() && kind != "photos" {
        return Err(mismatch("per_page", &kind));
    }
    if let Some(count) = patch.count {
        if kind != "blank_pages" {
            return Err(mismatch("count", &kind));
        }
        check_count(count)?;
    }
    let section = &mut doe["sections"][index];
    if let Some(title) = patch.title {
        section["title"] = json!(title.trim());
    }
    if let Some(enabled) = patch.enabled {
        section["enabled"] = json!(enabled);
    }
    if let Some(label) = patch.maker_label {
        section["content"]["maker_label"] = json!(label.tag());
    }
    if let Some(per_page) = patch.per_page {
        section["content"]["per_page"] = json!(per_page.tag());
    }
    if let Some(count) = patch.count {
        section["content"]["count"] = json!(count);
    }
    Ok(())
}

pub fn delete_section(doe: &mut Value, section_id: &str) -> Result<(), Fault> {
    let index = pick_any_section(
        doe,
        SectionRef {
            id: Some(section_id),
            title: None,
        },
    )?;
    let section = &doe["sections"][index];
    let title = section["title"].as_str().unwrap_or("?");
    let content = &section["content"];
    let filled = |field: &str, what: &str| {
        let count = content[field].as_array().map_or(0, Vec::len);
        (count > 0).then(|| {
            Fault::new(format!(
                "la section « {title} » n'est pas vide : elle contient {count} {what}. Retirez-les d'abord"
            ))
        })
    };
    let refusal = match kind_of(section) {
        "supplies" => filled("supplies", "fourniture(s)"),
        "photos" => filled("photos", "photo(s)"),
        "plans" => filled("plans", "plan(s)"),
        _ => None,
    };
    if let Some(refusal) = refusal {
        return Err(refusal);
    }
    sections_mut(doe)?.remove(index);
    Ok(())
}

pub fn reorder_sections(doe: &mut Value, ids: &[String]) -> Result<(), Fault> {
    let current: Vec<String> = sections(doe)
        .iter()
        .filter_map(|section| section["id"].as_str().map(str::to_owned))
        .collect();
    for (position, id) in ids.iter().enumerate() {
        if ids[..position].contains(id) {
            return Err(Fault::new(format!(
                "la section {id} apparaît plusieurs fois"
            )));
        }
    }
    let unknown: Vec<&str> = ids
        .iter()
        .filter(|id| !current.contains(id))
        .map(String::as_str)
        .collect();
    if !unknown.is_empty() {
        return Err(Fault::new(format!(
            "sections inconnues : {}. Sections du DOE : {}",
            unknown.join(", "),
            describe(doe, None)
        )));
    }
    let missing: Vec<&str> = current
        .iter()
        .filter(|id| !ids.contains(id))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        return Err(Fault::new(format!(
            "liste incomplète : il manque {}. Indiquez toutes les sections du DOE",
            missing.join(", ")
        )));
    }
    let list = sections_mut(doe)?;
    let mut reordered = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(index) = list
            .iter()
            .position(|section| section["id"].as_str() == Some(id))
        {
            reordered.push(list.remove(index));
        }
    }
    *list = reordered;
    Ok(())
}

pub struct FilePlacement<'a> {
    pub target: Target,
    pub section: SectionRef<'a>,
    pub supply_id: Option<&'a str>,
    pub caption: Option<&'a str>,
}

pub fn add_file(doe: &mut Value, placement: &FilePlacement<'_>, key: &str) -> Result<(), Fault> {
    let entry =
        json!({"key": key, "caption": placement.caption.map_or(Value::Null, blank_to_null)});
    let push = |list: &mut Value| -> Result<(), Fault> {
        list.as_array_mut()
            .ok_or_else(|| Fault::new("liste de fichiers illisible"))?
            .push(entry.clone());
        Ok(())
    };
    match placement.target {
        Target::CoverPhoto => {
            if placement.caption.is_some_and(|c| !c.trim().is_empty()) {
                return Err(Fault::new(
                    "la photo de couverture n'a pas de légende : retirez `caption`",
                ));
            }
            doe["cover_photo_key"] = json!(key);
            Ok(())
        }
        Target::Photo => {
            let index = pick_section(doe, placement.section, SectionKind::Photos)?;
            push(&mut doe["sections"][index]["content"]["photos"])
        }
        Target::Plan => {
            let index = pick_section(doe, placement.section, SectionKind::Plans)?;
            push(&mut doe["sections"][index]["content"]["plans"])
        }
        Target::SupplyPhoto | Target::SupplyDatasheet => {
            let supply_id = placement
                .supply_id
                .ok_or_else(|| Fault::new("précisez `supply_id` pour une pièce de fourniture"))?;
            let (section, index) = find_supply(doe, supply_id)?;
            let supply = &mut doe["sections"][section]["content"]["supplies"][index];
            if placement.target == Target::SupplyPhoto {
                return push(&mut supply["photos"]);
            }
            if placement.caption.is_some_and(|c| !c.trim().is_empty()) {
                return Err(Fault::new(
                    "une fiche technique n'a pas de légende : retirez `caption`",
                ));
            }
            supply["datasheet_keys"]
                .as_array_mut()
                .ok_or_else(|| Fault::new("fiches techniques illisibles"))?
                .push(json!(key));
            Ok(())
        }
    }
}

enum Place {
    SectionFile {
        section: usize,
        field: &'static str,
        index: usize,
    },
    SupplyPhoto {
        section: usize,
        supply: usize,
        index: usize,
    },
}

fn locate_file(doe: &Value, key: &str) -> Option<Place> {
    for (section, entry) in sections(doe).iter().enumerate() {
        let content = &entry["content"];
        for field in ["photos", "plans"] {
            let found = content[field]
                .as_array()
                .and_then(|list| list.iter().position(|file| file["key"] == key));
            if let Some(index) = found {
                return Some(Place::SectionFile {
                    section,
                    field,
                    index,
                });
            }
        }
        if let Some(supplies) = content["supplies"].as_array() {
            for (supply, item) in supplies.iter().enumerate() {
                let found = item["photos"]
                    .as_array()
                    .and_then(|list| list.iter().position(|file| file["key"] == key));
                if let Some(index) = found {
                    return Some(Place::SupplyPhoto {
                        section,
                        supply,
                        index,
                    });
                }
            }
        }
    }
    None
}

fn has_datasheet(doe: &Value, key: &str) -> bool {
    sections(doe).iter().any(|section| {
        section["content"]["supplies"]
            .as_array()
            .is_some_and(|supplies| {
                supplies.iter().any(|supply| {
                    supply["datasheet_keys"]
                        .as_array()
                        .is_some_and(|keys| keys.iter().any(|k| k == key))
                })
            })
    })
}

fn unknown_file(key: &str) -> Fault {
    Fault::new(format!(
        "fichier introuvable : « {key} ». Les clés des fichiers sont dans get_doe"
    ))
}

pub fn update_file(
    doe: &mut Value,
    key: &str,
    caption: Option<&str>,
    new_section: Option<&str>,
) -> Result<(), Fault> {
    if caption.is_none() && new_section.is_none() {
        return Err(Fault::new(
            "rien à modifier : précisez `caption` ou `section_id`",
        ));
    }
    let Some(place) = locate_file(doe, key) else {
        if doe["cover_photo_key"] == key {
            return Err(Fault::new(
                "la photo de couverture n'a ni légende ni section : remplacez-la ou supprimez-la",
            ));
        }
        if has_datasheet(doe, key) {
            return Err(Fault::new(
                "une fiche technique n'a ni légende ni section : seule sa suppression est possible",
            ));
        }
        return Err(unknown_file(key));
    };
    if let Some(destination) = new_section {
        let Place::SectionFile {
            section,
            field,
            index,
        } = place
        else {
            return Err(Fault::new(
                "la photo d'une fourniture ne se déplace pas vers une section",
            ));
        };
        let source_kind = kind_of(&doe["sections"][section]).to_owned();
        let target = pick_any_section(
            doe,
            SectionRef {
                id: Some(destination),
                title: None,
            },
        )?;
        if kind_of(&doe["sections"][target]) != source_kind {
            return Err(Fault::new(format!(
                "déplacement refusé : la section cible n'est pas une section {}",
                if field == "photos" {
                    "de photos"
                } else {
                    "de plans"
                }
            )));
        }
        if target != section {
            let mut file = doe["sections"][section]["content"][field]
                .as_array_mut()
                .ok_or_else(|| Fault::new("liste de fichiers illisible"))?
                .remove(index);
            if let Some(caption) = caption {
                file["caption"] = blank_to_null(caption);
            }
            doe["sections"][target]["content"][field]
                .as_array_mut()
                .ok_or_else(|| Fault::new("liste de fichiers illisible"))?
                .push(file);
            return Ok(());
        }
    }
    if let Some(caption) = caption {
        let file = match place {
            Place::SectionFile {
                section,
                field,
                index,
            } => &mut doe["sections"][section]["content"][field][index],
            Place::SupplyPhoto {
                section,
                supply,
                index,
            } => &mut doe["sections"][section]["content"]["supplies"][supply]["photos"][index],
        };
        file["caption"] = blank_to_null(caption);
    }
    Ok(())
}

pub fn delete_file(doe: &mut Value, key: &str) -> Result<(), Fault> {
    let mut removed = false;
    if doe["cover_photo_key"] == key {
        doe["cover_photo_key"] = Value::Null;
        removed = true;
    }
    let Some(all) = doe["sections"].as_array_mut() else {
        return Err(unknown_file(key));
    };
    let mut strip = |list: &mut Value| {
        if let Some(list) = list.as_array_mut() {
            let before = list.len();
            list.retain(|item| item["key"] != key && item.as_str() != Some(key));
            removed |= list.len() != before;
        }
    };
    for section in all {
        let content = &mut section["content"];
        strip(&mut content["photos"]);
        strip(&mut content["plans"]);
        if let Some(supplies) = content["supplies"].as_array_mut() {
            for supply in supplies {
                strip(&mut supply["photos"]);
                strip(&mut supply["datasheet_keys"]);
            }
        }
    }
    if removed {
        Ok(())
    } else {
        Err(unknown_file(key))
    }
}
