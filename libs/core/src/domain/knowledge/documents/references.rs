use std::collections::HashSet;

use uuid::Uuid;

use super::content::{Block, BlockKind, DocumentContent, ImageSource, Inline};
use crate::{
    KnowledgeArticleId, KnowledgeResource,
    domain::knowledge::tables::{KnowledgeTableId, TableViewId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentReference {
    DriveFile(KnowledgeArticleId),
    Photo(Uuid),
    TableView {
        table_id: KnowledgeTableId,
        view_id: TableViewId,
    },
    Resource(KnowledgeResource),
}

impl DocumentContent {
    pub fn references(&self) -> HashSet<DocumentReference> {
        let mut found = HashSet::new();
        collect(self.blocks(), &mut found);
        found
    }
}

fn collect(blocks: &[Block], found: &mut HashSet<DocumentReference>) {
    for block in blocks {
        match block.kind() {
            BlockKind::Image {
                source: ImageSource::Drive,
                id,
                ..
            } => {
                found.insert(DocumentReference::DriveFile(KnowledgeArticleId(*id)));
            }
            BlockKind::Image {
                source: ImageSource::Photo,
                id,
                ..
            } => {
                found.insert(DocumentReference::Photo(*id));
            }
            BlockKind::File { article_id } => {
                found.insert(DocumentReference::DriveFile(*article_id));
            }
            BlockKind::TableView { table_id, view_id } => {
                found.insert(DocumentReference::TableView {
                    table_id: *table_id,
                    view_id: *view_id,
                });
            }
            other => {
                for inline in other.inlines() {
                    if let Inline::Mention { resource, .. } = inline {
                        found.insert(DocumentReference::Resource(*resource));
                    }
                }
            }
        }
        collect(block.children(), found);
    }
}

pub fn added(previous: Option<&DocumentContent>, next: &DocumentContent) -> Vec<DocumentReference> {
    let before = previous
        .map(DocumentContent::references)
        .unwrap_or_default();
    next.references()
        .into_iter()
        .filter(|reference| !before.contains(reference))
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::KnowledgeResourceType;

    fn content(blocks: Vec<Value>) -> DocumentContent {
        DocumentContent::parse(&json!({ "version": 1, "blocks": blocks })).unwrap()
    }

    fn block(id: &str, kind: &str, props: Value, inline: Vec<Value>) -> Value {
        json!({ "id": id, "type": kind, "props": props, "content": inline, "children": [] })
    }

    #[test]
    fn every_kind_of_reference_is_found_even_in_children() {
        let (drive, photo, file, table, view, customer) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let mut parent = block(
            "p",
            "bullet_item",
            json!({}),
            vec![json!({ "type": "mention", "resource_type": "customer",
                "resource_id": customer, "label": "Dupont" })],
        );
        parent["children"] = json!([block(
            "t",
            "table_view",
            json!({ "table_id": table, "view_id": view }),
            vec![]
        )]);
        let found = content(vec![
            block(
                "i",
                "image",
                json!({ "source": "drive", "id": drive }),
                vec![],
            ),
            block(
                "j",
                "image",
                json!({ "source": "photo", "id": photo }),
                vec![],
            ),
            block("f", "file", json!({ "article_id": file }), vec![]),
            parent,
        ])
        .references();
        assert_eq!(found.len(), 5);
        assert!(found.contains(&DocumentReference::DriveFile(KnowledgeArticleId(drive))));
        assert!(found.contains(&DocumentReference::Photo(photo)));
        assert!(found.contains(&DocumentReference::DriveFile(KnowledgeArticleId(file))));
        assert!(found.contains(&DocumentReference::TableView {
            table_id: KnowledgeTableId(table),
            view_id: TableViewId(view)
        }));
        assert!(
            found.contains(&DocumentReference::Resource(KnowledgeResource {
                resource_type: KnowledgeResourceType::Customer,
                id: customer
            }))
        );
    }

    #[test]
    fn only_the_references_missing_from_the_previous_content_are_added() {
        let (kept, fresh) = (Uuid::now_v7(), Uuid::now_v7());
        let image = |id: &str, source_id: Uuid| {
            block(
                id,
                "image",
                json!({ "source": "photo", "id": source_id }),
                vec![],
            )
        };
        let previous = content(vec![image("a", kept)]);
        let next = content(vec![image("a", kept), image("b", fresh)]);
        assert_eq!(
            added(Some(&previous), &next),
            vec![DocumentReference::Photo(fresh)]
        );
        assert_eq!(added(None, &next).len(), 2);
        assert!(added(Some(&next), &previous).is_empty());
    }
}
