use std::collections::HashSet;

use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{
    KnowledgeArticleId, KnowledgeResource, KnowledgeResourceType,
    domain::knowledge::tables::{KnowledgeTableId, TableViewId},
};

pub const VERSION: u64 = 1;
pub const MAX_BLOCKS: usize = 5_000;
pub const MAX_DEPTH: usize = 6;
pub const MAX_TEXT_CHARS: usize = 20_000;
pub const MAX_JSON_BYTES: usize = 1024 * 1024;
pub const MAX_BLOCK_ID_CHARS: usize = 64;
pub const MAX_LANGUAGE_CHARS: usize = 32;
pub const MAX_CAPTION_CHARS: usize = 500;
pub const MAX_HREF_CHARS: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ContentError {
    #[error("content: {0}")]
    Malformed(&'static str),
    #[error("content: only version 1 is supported")]
    UnsupportedVersion,
    #[error("content: a document holds at most 5000 blocks")]
    TooManyBlocks,
    #[error("content: blocks nest at most 6 levels deep")]
    TooDeep,
    #[error("content: a block holds at most 20000 characters of text")]
    TextTooLong,
    #[error("content: a document holds at most 1 MB of JSON")]
    TooLarge,
    #[error("content: a block id holds 1 to 64 characters")]
    BlockIdInvalid,
    #[error("content: two blocks share an id")]
    BlockIdDuplicated,
    #[error("content: unknown block type")]
    UnknownBlockType,
    #[error("content: unknown inline type")]
    UnknownInlineType,
    #[error("content: a heading has a level from 1 to 3")]
    HeadingLevel,
    #[error("content: a callout tone is info, success, warning or danger")]
    UnknownTone,
    #[error("content: an image comes from the drive or the photo library")]
    UnknownImageSource,
    #[error("content: a mention names a customer, a project or a quote")]
    UnknownMention,
    #[error("content: `{0}` is not a valid id")]
    InvalidId(&'static str),
    #[error("content: a link opens http, https or mailto addresses only")]
    LinkScheme,
    #[error("content: a code language holds at most 32 characters")]
    LanguageTooLong,
    #[error("content: an image caption holds at most 500 characters")]
    CaptionTooLong,
    #[error("content: an image is 10 to 100 percent wide")]
    WidthOutOfRange,
    #[error("content: a code block holds a single text without style")]
    CodeNotPlain,
    #[error("content: this block cannot have children")]
    NoChildren,
    #[error("content: this block has no text")]
    NoContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(String);

impl BlockId {
    pub fn new(raw: &str) -> Result<Self, ContentError> {
        let length = raw.chars().count();
        if length == 0 || length > MAX_BLOCK_ID_CHARS {
            return Err(ContentError::BlockIdInvalid);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Href(String);

impl Href {
    pub fn new(raw: &str) -> Result<Self, ContentError> {
        let allowed = ["http://", "https://", "mailto:"].iter().any(|scheme| {
            raw.get(..scheme.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(scheme))
        });
        let clean = !raw.chars().any(|c| c.is_whitespace() || c.is_control());
        if !allowed || !clean || raw.chars().count() > MAX_HREF_CHARS {
            return Err(ContentError::LinkScheme);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyles {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
}

impl TextStyles {
    fn to_value(self) -> Value {
        let mut styles = Map::new();
        for (name, on) in [
            ("bold", self.bold),
            ("italic", self.italic),
            ("underline", self.underline),
            ("strike", self.strike),
            ("code", self.code),
        ] {
            if on {
                styles.insert(name.to_owned(), Value::Bool(true));
            }
        }
        Value::Object(styles)
    }

    fn parse(value: Option<&Value>) -> Result<Self, ContentError> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let map = object(value, "`styles` must be an object")?;
        only_keys(map, &["bold", "italic", "underline", "strike", "code"])?;
        let flag = |name: &str| match map.get(name) {
            None => Ok(false),
            Some(Value::Bool(on)) => Ok(*on),
            Some(_) => Err(ContentError::Malformed("a style is true or absent")),
        };
        Ok(Self {
            bold: flag("bold")?,
            italic: flag("italic")?,
            underline: flag("underline")?,
            strike: flag("strike")?,
            code: flag("code")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledText {
    pub text: String,
    pub styles: TextStyles,
}

impl StyledText {
    pub fn new(text: impl Into<String>, styles: TextStyles) -> Self {
        Self {
            text: text.into(),
            styles,
        }
    }

    fn to_value(&self) -> Value {
        json!({ "type": "text", "text": self.text, "styles": self.styles.to_value() })
    }

    fn parse(map: &Map<String, Value>) -> Result<Self, ContentError> {
        only_keys(map, &["type", "text", "styles"])?;
        let text = map
            .get("text")
            .and_then(Value::as_str)
            .ok_or(ContentError::Malformed("a text needs a `text` string"))?;
        Ok(Self::new(text, TextStyles::parse(map.get("styles"))?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(StyledText),
    Link {
        href: Href,
        content: Vec<StyledText>,
    },
    Mention {
        resource: KnowledgeResource,
        label: String,
    },
}

impl Inline {
    pub fn text(text: impl Into<String>, styles: TextStyles) -> Self {
        Self::Text(StyledText::new(text, styles))
    }

    fn chars(&self) -> usize {
        match self {
            Self::Text(text) => text.text.chars().count(),
            Self::Link { content, .. } => content.iter().map(|t| t.text.chars().count()).sum(),
            Self::Mention { label, .. } => label.chars().count(),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::Text(text) => text.to_value(),
            Self::Link { href, content } => json!({
                "type": "link",
                "href": href.as_str(),
                "content": content.iter().map(StyledText::to_value).collect::<Vec<_>>(),
            }),
            Self::Mention { resource, label } => json!({
                "type": "mention",
                "resource_type": resource.resource_type.as_str(),
                "resource_id": resource.id,
                "label": label,
            }),
        }
    }

    fn parse(value: &Value) -> Result<Self, ContentError> {
        let map = object(value, "an inline must be an object")?;
        match map.get("type").and_then(Value::as_str) {
            Some("text") => Ok(Self::Text(StyledText::parse(map)?)),
            Some("link") => {
                only_keys(map, &["type", "href", "content"])?;
                let href = Href::new(
                    map.get("href")
                        .and_then(Value::as_str)
                        .ok_or(ContentError::Malformed("a link needs an `href` string"))?,
                )?;
                let content = array(map.get("content"), "a link needs a `content` list")?
                    .iter()
                    .map(|item| {
                        let item = object(item, "a link holds texts")?;
                        if item.get("type").and_then(Value::as_str) != Some("text") {
                            return Err(ContentError::Malformed("a link holds texts only"));
                        }
                        StyledText::parse(item)
                    })
                    .collect::<Result<_, _>>()?;
                Ok(Self::Link { href, content })
            }
            Some("mention") => {
                only_keys(map, &["type", "resource_type", "resource_id", "label"])?;
                let resource_type = map
                    .get("resource_type")
                    .and_then(Value::as_str)
                    .and_then(|raw| raw.parse::<KnowledgeResourceType>().ok())
                    .ok_or(ContentError::UnknownMention)?;
                let id = uuid_of(map.get("resource_id"), "resource_id")?;
                let label = map
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or(ContentError::Malformed("a mention needs a `label` string"))?;
                Ok(Self::Mention {
                    resource: KnowledgeResource { resource_type, id },
                    label: label.to_owned(),
                })
            }
            _ => Err(ContentError::UnknownInlineType),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    One,
    Two,
    Three,
}

impl HeadingLevel {
    pub fn new(level: u64) -> Result<Self, ContentError> {
        match level {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            _ => Err(ContentError::HeadingLevel),
        }
    }

    pub fn get(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutTone {
    Info,
    Success,
    Warning,
    Danger,
}

impl CalloutTone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }

    fn parse(raw: &str) -> Result<Self, ContentError> {
        match raw {
            "info" => Ok(Self::Info),
            "success" => Ok(Self::Success),
            "warning" => Ok(Self::Warning),
            "danger" => Ok(Self::Danger),
            _ => Err(ContentError::UnknownTone),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSource {
    Drive,
    Photo,
}

impl ImageSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Drive => "drive",
            Self::Photo => "photo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageWidth(u8);

impl ImageWidth {
    pub const FULL: Self = Self(100);

    pub fn new(percent: u64) -> Result<Self, ContentError> {
        match u8::try_from(percent) {
            Ok(percent @ 10..=100) => Ok(Self(percent)),
            _ => Err(ContentError::WidthOutOfRange),
        }
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caption(String);

impl Caption {
    pub fn new(raw: &str) -> Result<Self, ContentError> {
        if raw.chars().count() > MAX_CAPTION_CHARS {
            return Err(ContentError::CaptionTooLong);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeLanguage(String);

impl CodeLanguage {
    pub fn new(raw: &str) -> Result<Self, ContentError> {
        if raw.chars().count() > MAX_LANGUAGE_CHARS {
            return Err(ContentError::LanguageTooLong);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph(Vec<Inline>),
    Heading {
        level: HeadingLevel,
        content: Vec<Inline>,
    },
    BulletItem(Vec<Inline>),
    NumberedItem(Vec<Inline>),
    CheckItem {
        checked: bool,
        content: Vec<Inline>,
    },
    Quote(Vec<Inline>),
    Callout {
        tone: CalloutTone,
        content: Vec<Inline>,
    },
    Divider,
    Code {
        language: Option<CodeLanguage>,
        text: String,
    },
    Image {
        source: ImageSource,
        id: Uuid,
        caption: Option<Caption>,
        width: ImageWidth,
    },
    File {
        article_id: KnowledgeArticleId,
    },
    TableView {
        table_id: KnowledgeTableId,
        view_id: TableViewId,
    },
}

impl BlockKind {
    fn type_name(&self) -> &'static str {
        match self {
            Self::Paragraph(_) => "paragraph",
            Self::Heading { .. } => "heading",
            Self::BulletItem(_) => "bullet_item",
            Self::NumberedItem(_) => "numbered_item",
            Self::CheckItem { .. } => "check_item",
            Self::Quote(_) => "quote",
            Self::Callout { .. } => "callout",
            Self::Divider => "divider",
            Self::Code { .. } => "code",
            Self::Image { .. } => "image",
            Self::File { .. } => "file",
            Self::TableView { .. } => "table_view",
        }
    }

    pub fn takes_children(&self) -> bool {
        matches!(
            self,
            Self::Paragraph(_)
                | Self::BulletItem(_)
                | Self::NumberedItem(_)
                | Self::CheckItem { .. }
        )
    }

    pub fn inlines(&self) -> &[Inline] {
        match self {
            Self::Paragraph(content)
            | Self::BulletItem(content)
            | Self::NumberedItem(content)
            | Self::Quote(content)
            | Self::Heading { content, .. }
            | Self::CheckItem { content, .. }
            | Self::Callout { content, .. } => content,
            Self::Divider
            | Self::Code { .. }
            | Self::Image { .. }
            | Self::File { .. }
            | Self::TableView { .. } => &[],
        }
    }

    fn text_chars(&self) -> usize {
        match self {
            Self::Code { text, .. } => text.chars().count(),
            Self::Image { caption, .. } => {
                caption.as_ref().map_or(0, |c| c.as_str().chars().count())
            }
            other => other.inlines().iter().map(Inline::chars).sum(),
        }
    }

    fn props(&self) -> Value {
        match self {
            Self::Heading { level, .. } => json!({ "level": level.get() }),
            Self::CheckItem { checked, .. } => json!({ "checked": checked }),
            Self::Callout { tone, .. } => json!({ "tone": tone.as_str() }),
            Self::Code {
                language: Some(language),
                ..
            } => json!({ "language": language.as_str() }),
            Self::Image {
                source,
                id,
                caption,
                width,
            } => {
                let mut props =
                    json!({ "source": source.as_str(), "id": id, "width": width.get() });
                if let Some(caption) = caption {
                    props["caption"] = json!(caption.as_str());
                }
                props
            }
            Self::File { article_id } => json!({ "article_id": article_id.0 }),
            Self::TableView { table_id, view_id } => {
                json!({ "table_id": table_id.0, "view_id": view_id.0 })
            }
            _ => json!({}),
        }
    }

    fn content_value(&self) -> Vec<Value> {
        match self {
            Self::Code { text, .. } if !text.is_empty() => {
                vec![StyledText::new(text.as_str(), TextStyles::default()).to_value()]
            }
            other => other.inlines().iter().map(Inline::to_value).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    id: BlockId,
    kind: BlockKind,
    children: Vec<Block>,
}

impl Block {
    pub fn new(id: BlockId, kind: BlockKind, children: Vec<Block>) -> Result<Self, ContentError> {
        if !children.is_empty() && !kind.takes_children() {
            return Err(ContentError::NoChildren);
        }
        if kind.text_chars() > MAX_TEXT_CHARS {
            return Err(ContentError::TextTooLong);
        }
        Ok(Self { id, kind, children })
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }

    pub fn kind(&self) -> &BlockKind {
        &self.kind
    }

    pub fn children(&self) -> &[Block] {
        &self.children
    }

    fn to_value(&self) -> Value {
        json!({
            "id": self.id.as_str(),
            "type": self.kind.type_name(),
            "props": self.kind.props(),
            "content": self.kind.content_value(),
            "children": self.children.iter().map(Block::to_value).collect::<Vec<_>>(),
        })
    }

    fn parse(value: &Value, depth: usize, count: &mut usize) -> Result<Self, ContentError> {
        if depth > MAX_DEPTH {
            return Err(ContentError::TooDeep);
        }
        *count += 1;
        if *count > MAX_BLOCKS {
            return Err(ContentError::TooManyBlocks);
        }
        let map = object(value, "a block must be an object")?;
        only_keys(map, &["id", "type", "props", "content", "children"])?;
        let id = BlockId::new(
            map.get("id")
                .and_then(Value::as_str)
                .ok_or(ContentError::BlockIdInvalid)?,
        )?;
        let empty = Map::new();
        let props = match map.get("props") {
            None => &empty,
            Some(props) => object(props, "`props` must be an object")?,
        };
        let content = match map.get("content") {
            None => &[][..],
            Some(content) => array(Some(content), "`content` must be a list")?,
        };
        let kind_name = map
            .get("type")
            .and_then(Value::as_str)
            .ok_or(ContentError::UnknownBlockType)?;
        let kind = parse_kind(kind_name, props, content)?;
        let children = match map.get("children") {
            None => Vec::new(),
            Some(children) => array(Some(children), "`children` must be a list")?
                .iter()
                .map(|child| Block::parse(child, depth + 1, count))
                .collect::<Result<_, _>>()?,
        };
        Block::new(id, kind, children)
    }
}

fn parse_kind(
    name: &str,
    props: &Map<String, Value>,
    content: &[Value],
) -> Result<BlockKind, ContentError> {
    let inlines =
        || -> Result<Vec<Inline>, ContentError> { content.iter().map(Inline::parse).collect() };
    let no_content = || {
        if content.is_empty() {
            Ok(())
        } else {
            Err(ContentError::NoContent)
        }
    };
    match name {
        "paragraph" => {
            only_keys(props, &[])?;
            Ok(BlockKind::Paragraph(inlines()?))
        }
        "bullet_item" => {
            only_keys(props, &[])?;
            Ok(BlockKind::BulletItem(inlines()?))
        }
        "numbered_item" => {
            only_keys(props, &[])?;
            Ok(BlockKind::NumberedItem(inlines()?))
        }
        "quote" => {
            only_keys(props, &[])?;
            Ok(BlockKind::Quote(inlines()?))
        }
        "heading" => {
            only_keys(props, &["level"])?;
            let level = props
                .get("level")
                .and_then(Value::as_u64)
                .ok_or(ContentError::HeadingLevel)?;
            Ok(BlockKind::Heading {
                level: HeadingLevel::new(level)?,
                content: inlines()?,
            })
        }
        "check_item" => {
            only_keys(props, &["checked"])?;
            let checked = match props.get("checked") {
                None => false,
                Some(Value::Bool(checked)) => *checked,
                Some(_) => return Err(ContentError::Malformed("`checked` is true or false")),
            };
            Ok(BlockKind::CheckItem {
                checked,
                content: inlines()?,
            })
        }
        "callout" => {
            only_keys(props, &["tone"])?;
            let tone = props
                .get("tone")
                .and_then(Value::as_str)
                .ok_or(ContentError::UnknownTone)?;
            Ok(BlockKind::Callout {
                tone: CalloutTone::parse(tone)?,
                content: inlines()?,
            })
        }
        "divider" => {
            only_keys(props, &[])?;
            no_content()?;
            Ok(BlockKind::Divider)
        }
        "code" => {
            only_keys(props, &["language"])?;
            let language = match props.get("language") {
                None | Some(Value::Null) => None,
                Some(Value::String(language)) => Some(CodeLanguage::new(language)?),
                Some(_) => return Err(ContentError::Malformed("`language` is a string")),
            };
            let text = match content {
                [] => String::new(),
                [only] => {
                    let only = object(only, "a code block holds a text")?;
                    if only.get("type").and_then(Value::as_str) != Some("text") {
                        return Err(ContentError::CodeNotPlain);
                    }
                    let text = StyledText::parse(only)?;
                    if text.styles != TextStyles::default() {
                        return Err(ContentError::CodeNotPlain);
                    }
                    text.text
                }
                _ => return Err(ContentError::CodeNotPlain),
            };
            Ok(BlockKind::Code { language, text })
        }
        "image" => {
            only_keys(props, &["source", "id", "caption", "width"])?;
            no_content()?;
            let source = match props.get("source").and_then(Value::as_str) {
                Some("drive") => ImageSource::Drive,
                Some("photo") => ImageSource::Photo,
                _ => return Err(ContentError::UnknownImageSource),
            };
            let caption = match props.get("caption") {
                None | Some(Value::Null) => None,
                Some(Value::String(caption)) => Some(Caption::new(caption)?),
                Some(_) => return Err(ContentError::Malformed("`caption` is a string")),
            };
            let width = match props.get("width") {
                None => ImageWidth::FULL,
                Some(width) => {
                    ImageWidth::new(width.as_u64().ok_or(ContentError::WidthOutOfRange)?)?
                }
            };
            Ok(BlockKind::Image {
                source,
                id: uuid_of(props.get("id"), "id")?,
                caption,
                width,
            })
        }
        "file" => {
            only_keys(props, &["article_id"])?;
            no_content()?;
            Ok(BlockKind::File {
                article_id: KnowledgeArticleId(uuid_of(props.get("article_id"), "article_id")?),
            })
        }
        "table_view" => {
            only_keys(props, &["table_id", "view_id"])?;
            no_content()?;
            Ok(BlockKind::TableView {
                table_id: KnowledgeTableId(uuid_of(props.get("table_id"), "table_id")?),
                view_id: TableViewId(uuid_of(props.get("view_id"), "view_id")?),
            })
        }
        _ => Err(ContentError::UnknownBlockType),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentContent {
    blocks: Vec<Block>,
}

impl DocumentContent {
    pub fn empty() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn new(blocks: Vec<Block>) -> Result<Self, ContentError> {
        let mut seen = HashSet::new();
        let mut count = 0;
        check_blocks(&blocks, 1, &mut seen, &mut count)?;
        let content = Self { blocks };
        let size = serde_json::to_vec(&content.to_value()).map_or(usize::MAX, |bytes| bytes.len());
        if size > MAX_JSON_BYTES {
            return Err(ContentError::TooLarge);
        }
        Ok(content)
    }

    pub fn parse(value: &Value) -> Result<Self, ContentError> {
        let root = object(value, "a document must be an object")?;
        only_keys(root, &["version", "blocks"])?;
        if root.get("version").and_then(Value::as_u64) != Some(VERSION) {
            return Err(ContentError::UnsupportedVersion);
        }
        let mut count = 0;
        let blocks = array(root.get("blocks"), "`blocks` must be a list")?
            .iter()
            .map(|block| Block::parse(block, 1, &mut count))
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(blocks)
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn to_value(&self) -> Value {
        json!({
            "version": VERSION,
            "blocks": self.blocks.iter().map(Block::to_value).collect::<Vec<_>>(),
        })
    }
}

fn check_blocks(
    blocks: &[Block],
    depth: usize,
    seen: &mut HashSet<String>,
    count: &mut usize,
) -> Result<(), ContentError> {
    if depth > MAX_DEPTH && !blocks.is_empty() {
        return Err(ContentError::TooDeep);
    }
    for block in blocks {
        *count += 1;
        if *count > MAX_BLOCKS {
            return Err(ContentError::TooManyBlocks);
        }
        if !seen.insert(block.id.as_str().to_owned()) {
            return Err(ContentError::BlockIdDuplicated);
        }
        check_blocks(&block.children, depth + 1, seen, count)?;
    }
    Ok(())
}

fn object<'a>(
    value: &'a Value,
    message: &'static str,
) -> Result<&'a Map<String, Value>, ContentError> {
    value.as_object().ok_or(ContentError::Malformed(message))
}

fn array<'a>(value: Option<&'a Value>, message: &'static str) -> Result<&'a [Value], ContentError> {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(ContentError::Malformed(message))
}

fn only_keys(map: &Map<String, Value>, allowed: &[&str]) -> Result<(), ContentError> {
    if map.keys().all(|key| allowed.contains(&key.as_str())) {
        Ok(())
    } else {
        Err(ContentError::Malformed("an unknown field was sent"))
    }
}

fn uuid_of(value: Option<&Value>, field: &'static str) -> Result<Uuid, ContentError> {
    value
        .and_then(Value::as_str)
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .ok_or(ContentError::InvalidId(field))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph(id: &str, text: &str) -> Value {
        json!({
            "id": id, "type": "paragraph", "props": {},
            "content": [{ "type": "text", "text": text, "styles": {} }],
            "children": []
        })
    }

    fn document(blocks: Vec<Value>) -> Value {
        json!({ "version": 1, "blocks": blocks })
    }

    fn nested(levels: usize) -> Value {
        let mut block = paragraph(&format!("n{levels}"), "x");
        for level in (1..levels).rev() {
            let mut parent = paragraph(&format!("n{level}"), "x");
            parent["children"] = json!([block]);
            block = parent;
        }
        block
    }

    #[test]
    fn a_valid_document_reads_back_the_same_json() {
        let id = Uuid::now_v7();
        let value = document(vec![
            paragraph("a", "Bonjour"),
            json!({ "id": "h", "type": "heading", "props": { "level": 2 },
                    "content": [{ "type": "text", "text": "Titre", "styles": { "bold": true } }],
                    "children": [] }),
            json!({ "id": "c", "type": "code", "props": { "language": "rust" },
                    "content": [{ "type": "text", "text": "fn main() {}", "styles": {} }],
                    "children": [] }),
            json!({ "id": "i", "type": "image",
                    "props": { "source": "photo", "id": id, "caption": "Vue", "width": 50 },
                    "content": [], "children": [] }),
            json!({ "id": "m", "type": "paragraph", "props": {},
                    "content": [
                        { "type": "mention", "resource_type": "customer", "resource_id": id, "label": "Dupont" },
                        { "type": "link", "href": "https://exemple.fr", "content": [
                            { "type": "text", "text": "site", "styles": { "italic": true } }] }
                    ], "children": [] }),
            json!({ "id": "d", "type": "divider", "props": {}, "content": [], "children": [] }),
        ]);
        let parsed = DocumentContent::parse(&value).unwrap();
        assert_eq!(parsed.to_value(), value);
    }

    #[test]
    fn a_missing_props_content_and_styles_are_accepted() {
        let value = json!({ "version": 1, "blocks": [
            { "id": "a", "type": "paragraph",
              "content": [{ "type": "text", "text": "x" }] }
        ] });
        assert!(DocumentContent::parse(&value).is_ok());
    }

    #[test]
    fn an_unknown_block_type_is_refused() {
        let value = document(vec![
            json!({ "id": "a", "type": "video", "props": {}, "content": [], "children": [] }),
        ]);
        assert_eq!(
            DocumentContent::parse(&value),
            Err(ContentError::UnknownBlockType)
        );
    }

    #[test]
    fn a_heading_of_level_four_is_refused() {
        let value = document(vec![
            json!({ "id": "a", "type": "heading", "props": { "level": 4 },
            "content": [], "children": [] }),
        ]);
        assert_eq!(
            DocumentContent::parse(&value),
            Err(ContentError::HeadingLevel)
        );
    }

    #[test]
    fn a_link_only_opens_http_https_and_mailto() {
        for href in [
            "javascript:alert(1)",
            "data:text/html,x",
            "ftp://x",
            " https://x",
            "https://a b",
        ] {
            assert_eq!(Href::new(href), Err(ContentError::LinkScheme), "{href}");
        }
        for href in ["http://x.fr", "HTTPS://x.fr/a?b=1", "mailto:a@b.fr"] {
            assert!(Href::new(href).is_ok(), "{href}");
        }
        let value = document(vec![json!({ "id": "a", "type": "paragraph", "props": {},
            "content": [{ "type": "link", "href": "javascript:alert(1)", "content": [] }],
            "children": [] })]);
        assert_eq!(
            DocumentContent::parse(&value),
            Err(ContentError::LinkScheme)
        );
    }

    #[test]
    fn two_blocks_cannot_share_an_id() {
        let value = document(vec![paragraph("a", "x"), paragraph("a", "y")]);
        assert_eq!(
            DocumentContent::parse(&value),
            Err(ContentError::BlockIdDuplicated)
        );
        let mut parent = paragraph("a", "x");
        parent["children"] = json!([paragraph("a", "y")]);
        assert_eq!(
            DocumentContent::parse(&document(vec![parent])),
            Err(ContentError::BlockIdDuplicated)
        );
    }

    #[test]
    fn a_block_id_holds_one_to_sixty_four_characters() {
        assert_eq!(BlockId::new(""), Err(ContentError::BlockIdInvalid));
        assert!(BlockId::new(&"a".repeat(64)).is_ok());
        assert_eq!(
            BlockId::new(&"a".repeat(65)),
            Err(ContentError::BlockIdInvalid)
        );
    }

    #[test]
    fn six_levels_are_accepted_and_seven_refused() {
        assert!(DocumentContent::parse(&document(vec![nested(6)])).is_ok());
        assert_eq!(
            DocumentContent::parse(&document(vec![nested(7)])),
            Err(ContentError::TooDeep)
        );
    }

    #[test]
    fn five_thousand_blocks_are_accepted_and_one_more_refused() {
        let many = |count: usize| {
            document(
                (0..count)
                    .map(|n| paragraph(&format!("b{n}"), "x"))
                    .collect(),
            )
        };
        assert!(DocumentContent::parse(&many(MAX_BLOCKS)).is_ok());
        assert_eq!(
            DocumentContent::parse(&many(MAX_BLOCKS + 1)),
            Err(ContentError::TooManyBlocks)
        );
    }

    #[test]
    fn a_block_holds_at_most_twenty_thousand_characters() {
        let at_limit = document(vec![paragraph("a", &"x".repeat(MAX_TEXT_CHARS))]);
        assert!(DocumentContent::parse(&at_limit).is_ok());
        let over = document(vec![paragraph("a", &"x".repeat(MAX_TEXT_CHARS + 1))]);
        assert_eq!(
            DocumentContent::parse(&over),
            Err(ContentError::TextTooLong)
        );
    }

    #[test]
    fn a_document_holds_at_most_one_megabyte_of_json() {
        let blocks = (0..60)
            .map(|n| paragraph(&format!("b{n}"), &"x".repeat(MAX_TEXT_CHARS)))
            .collect();
        assert_eq!(
            DocumentContent::parse(&document(blocks)),
            Err(ContentError::TooLarge)
        );
    }

    #[test]
    fn a_code_block_refuses_any_style() {
        let value = document(vec![json!({ "id": "a", "type": "code", "props": {},
            "content": [{ "type": "text", "text": "x", "styles": { "bold": true } }],
            "children": [] })]);
        assert_eq!(
            DocumentContent::parse(&value),
            Err(ContentError::CodeNotPlain)
        );
        let two = document(vec![json!({ "id": "a", "type": "code", "props": {},
            "content": [{ "type": "text", "text": "x" }, { "type": "text", "text": "y" }],
            "children": [] })]);
        assert_eq!(
            DocumentContent::parse(&two),
            Err(ContentError::CodeNotPlain)
        );
    }

    #[test]
    fn only_paragraphs_and_list_items_have_children() {
        let mut heading = json!({ "id": "a", "type": "heading", "props": { "level": 1 },
            "content": [], "children": [] });
        heading["children"] = json!([paragraph("b", "x")]);
        assert_eq!(
            DocumentContent::parse(&document(vec![heading])),
            Err(ContentError::NoChildren)
        );
    }

    #[test]
    fn image_bounds_are_checked() {
        let image = |props: Value| {
            document(vec![json!({ "id": "a", "type": "image", "props": props,
                "content": [], "children": [] })])
        };
        let id = Uuid::now_v7();
        assert_eq!(
            DocumentContent::parse(&image(json!({ "source": "drive", "id": id, "width": 9 }))),
            Err(ContentError::WidthOutOfRange)
        );
        assert_eq!(
            DocumentContent::parse(&image(json!({ "source": "web", "id": id }))),
            Err(ContentError::UnknownImageSource)
        );
        assert_eq!(
            DocumentContent::parse(&image(
                json!({ "source": "drive", "id": id, "caption": "x".repeat(501) })
            )),
            Err(ContentError::CaptionTooLong)
        );
        assert_eq!(
            DocumentContent::parse(&image(json!({ "source": "drive", "id": "nope" }))),
            Err(ContentError::InvalidId("id"))
        );
    }

    #[test]
    fn a_code_language_holds_at_most_thirty_two_characters() {
        let value = document(vec![json!({ "id": "a", "type": "code",
            "props": { "language": "x".repeat(33) }, "content": [], "children": [] })]);
        assert_eq!(
            DocumentContent::parse(&value),
            Err(ContentError::LanguageTooLong)
        );
    }

    #[test]
    fn other_versions_and_shapes_are_refused() {
        assert_eq!(
            DocumentContent::parse(&json!({ "version": 2, "blocks": [] })),
            Err(ContentError::UnsupportedVersion)
        );
        assert!(DocumentContent::parse(&json!([])).is_err());
        assert!(DocumentContent::parse(&json!({ "version": 1 })).is_err());
        assert!(
            DocumentContent::parse(&json!({ "version": 1, "blocks": [], "extra": 1 })).is_err()
        );
    }
}
