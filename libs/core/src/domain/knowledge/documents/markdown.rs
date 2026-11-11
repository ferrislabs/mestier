use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel as MdHeading, Options, Parser, Tag, TagEnd,
};
use uuid::Uuid;

use super::content::{
    Block, BlockId, BlockKind, CalloutTone, Caption, CodeLanguage, ContentError, DocumentContent,
    HeadingLevel, Href, ImageSource, ImageWidth, Inline, MAX_CAPTION_CHARS, MAX_DEPTH, StyledText,
    TextStyles,
};
use crate::{
    KnowledgeArticleId, KnowledgeResource, KnowledgeResourceType,
    domain::knowledge::tables::{KnowledgeTableId, TableViewId},
};

const CALLOUT_LABELS: [(CalloutTone, &str); 4] = [
    (CalloutTone::Info, "Info :"),
    (CalloutTone::Success, "Succès :"),
    (CalloutTone::Warning, "Attention :"),
    (CalloutTone::Danger, "Danger :"),
];

const MAX_CONTAINERS: usize = 32;
const FILE_LABEL: &str = "Fichier";
const VIEW_LABEL: &str = "Vue";

pub fn render(content: &DocumentContent) -> String {
    render_blocks(content.blocks())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Dash,
    Number,
}

fn family(kind: &BlockKind) -> Option<Family> {
    match kind {
        BlockKind::BulletItem(_) | BlockKind::CheckItem { .. } => Some(Family::Dash),
        BlockKind::NumberedItem(_) => Some(Family::Number),
        _ => None,
    }
}

fn render_blocks(blocks: &[Block]) -> String {
    let mut out = String::new();
    let mut previous: Option<Family> = None;
    let mut number = 0;
    for block in blocks {
        let current = family(block.kind());
        number = match (current, previous) {
            (Some(Family::Number), Some(Family::Number)) => number + 1,
            _ => 1,
        };
        let (text, flattened) = render_block(block, number);
        if !text.is_empty() {
            if !out.is_empty() {
                out.push_str(if current.is_some() && current == previous {
                    "\n"
                } else {
                    "\n\n"
                });
            }
            out.push_str(&text);
            previous = current;
        }
        if !flattened.is_empty() {
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&flattened);
            previous = None;
        }
    }
    out
}

fn render_block(block: &Block, number: usize) -> (String, String) {
    let flattened = if matches!(block.kind(), BlockKind::Paragraph(_)) {
        render_blocks(block.children())
    } else {
        String::new()
    };
    let text = match block.kind() {
        BlockKind::Paragraph(content) => render_inlines(content),
        BlockKind::Heading { level, content } => {
            let text = render_inlines(content).replace("\\\n", " ");
            let marks = "#".repeat(usize::from(level.get()));
            if text.is_empty() {
                marks
            } else {
                format!("{marks} {text}")
            }
        }
        BlockKind::BulletItem(content) => item(block, "- ".to_owned(), content),
        BlockKind::CheckItem { checked, content } => {
            let marker = if *checked { "- [x] " } else { "- [ ] " };
            item(block, marker.to_owned(), content)
        }
        BlockKind::NumberedItem(content) => item(block, format!("{number}. "), content),
        BlockKind::Quote(content) => prefix_lines(&render_inlines(content), "> "),
        BlockKind::Callout { tone, content } => {
            let label = CALLOUT_LABELS
                .iter()
                .find(|(candidate, _)| candidate == tone)
                .map_or("Info :", |(_, label)| label);
            let text = render_inlines_after_marker(content);
            let head = if text.is_empty() {
                format!("**{label}**")
            } else {
                format!("**{label}** {text}")
            };
            prefix_lines(&head, "> ")
        }
        BlockKind::Divider => "---".to_owned(),
        BlockKind::Code { language, text } => render_code(language.as_ref(), text),
        BlockKind::Image {
            source,
            id,
            caption,
            ..
        } => {
            let caption = caption
                .as_ref()
                .map_or(String::new(), |c| escape_plain(c.as_str()));
            format!("![{caption}](mestier:{}/{id})", source.as_str())
        }
        BlockKind::File { article_id } => format!("[{FILE_LABEL}](mestier:file/{})", article_id.0),
        BlockKind::TableView { table_id, view_id } => {
            format!(
                "[{VIEW_LABEL}](mestier:table/{}?view={})",
                table_id.0, view_id.0
            )
        }
    };
    (text, flattened)
}

fn item(block: &Block, marker: String, content: &[Inline]) -> String {
    let width = if marker.starts_with('-') {
        2
    } else {
        marker.len()
    };
    let text = render_inlines(content);
    let head = if text.is_empty() {
        marker.trim_end().to_owned()
    } else {
        let mut lines = text.split('\n');
        let first = lines.next().unwrap_or_default();
        let mut head = format!("{marker}{first}");
        for line in lines {
            head.push('\n');
            if !line.is_empty() {
                head.push_str(&" ".repeat(width));
                head.push_str(line);
            }
        }
        head
    };
    if block.children().is_empty() {
        return head;
    }
    let children = render_blocks(block.children());
    let nested_list = block
        .children()
        .first()
        .is_some_and(|c| family(c.kind()).is_some());
    let separator = if nested_list { "\n" } else { "\n\n" };
    format!("{head}{separator}{}", indent(&children, width))
}

fn indent(text: &str, width: usize) -> String {
    let pad = " ".repeat(width);
    text.split('\n')
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn prefix_lines(text: &str, prefix: &str) -> String {
    text.split('\n')
        .map(|line| {
            if line.is_empty() {
                prefix.trim_end().to_owned()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_code(language: Option<&CodeLanguage>, text: &str) -> String {
    let longest = text
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or_default();
    let fence = "`".repeat((longest + 1).max(3));
    let language: String = language
        .map(|language| {
            language
                .as_str()
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '`')
                .collect()
        })
        .unwrap_or_default();
    if text.is_empty() {
        format!("{fence}{language}\n{fence}")
    } else {
        format!("{fence}{language}\n{text}\n{fence}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Strike,
    Bold,
    Italic,
    Underline,
}

const LAYERS: [Layer; 4] = [Layer::Strike, Layer::Bold, Layer::Italic, Layer::Underline];

impl Layer {
    fn open(self, html: bool) -> &'static str {
        match (self, html) {
            (Self::Bold, false) => "**",
            (Self::Italic, false) => "*",
            (Self::Strike, _) => "<del>",
            (Self::Bold, true) => "<strong>",
            (Self::Italic, true) => "<em>",
            (Self::Underline, _) => "<u>",
        }
    }

    fn close(self, html: bool) -> &'static str {
        match (self, html) {
            (Self::Strike, _) => "</del>",
            (Self::Bold, true) => "</strong>",
            (Self::Italic, true) => "</em>",
            (Self::Underline, _) => "</u>",
            (other, false) => other.open(false),
        }
    }

    fn wanted(self, styles: TextStyles) -> bool {
        match self {
            Self::Strike => styles.strike,
            Self::Bold => styles.bold,
            Self::Italic => styles.italic,
            Self::Underline => styles.underline,
        }
    }
}

type Hints = [bool; 4];

fn edge(run: &StyledText, first: bool) -> char {
    if run.styles.code {
        return '`';
    }
    let text = run.text.trim();
    let found = if first {
        text.chars().next()
    } else {
        text.chars().next_back()
    };
    found.unwrap_or('x')
}

fn hints_for(runs: &[Option<&StyledText>], index: usize) -> Hints {
    let mut hints = [false; 4];
    let Some(run) = runs[index] else {
        return hints;
    };
    for (slot, layer) in LAYERS.iter().enumerate() {
        if !layer.wanted(run.styles) {
            continue;
        }
        let mut last = index;
        while let Some(Some(next)) = runs.get(last + 1) {
            if !layer.wanted(next.styles) {
                break;
            }
            last += 1;
        }
        let closing = runs[last].map_or('x', |closing| edge(closing, false));
        let tag_at_edge =
            run.styles.underline || runs[last].is_some_and(|run| run.styles.underline);
        hints[slot] =
            !edge(run, true).is_alphanumeric() || !closing.is_alphanumeric() || tag_at_edge;
    }
    hints
}

struct Writer {
    out: String,
    active: Vec<(Layer, bool)>,
    pending: String,
    line_start: bool,
    closed_at: Option<usize>,
}

impl Writer {
    fn new(line_start: bool) -> Self {
        Self {
            out: String::new(),
            active: Vec::new(),
            pending: String::new(),
            line_start,
            closed_at: None,
        }
    }

    fn settle(&mut self, styles: TextStyles) {
        let diverges = self
            .active
            .iter()
            .position(|(layer, _)| !layer.wanted(styles));
        if let Some(at) = diverges {
            while self.active.len() > at {
                if let Some((layer, html)) = self.active.pop() {
                    self.out.push_str(layer.close(html));
                    self.closed_at = (!html && matches!(layer, Layer::Bold | Layer::Italic))
                        .then_some(self.out.len());
                }
            }
        }
        let pending = std::mem::take(&mut self.pending);
        self.out.push_str(&pending);
    }

    fn open_missing(&mut self, styles: TextStyles, hints: Hints) {
        for (slot, layer) in LAYERS.into_iter().enumerate() {
            if layer.wanted(styles) && !self.active.iter().any(|(open, _)| *open == layer) {
                let html = hints[slot] || self.closed_at == Some(self.out.len());
                self.out.push_str(layer.open(html));
                self.active.push((layer, html));
                self.line_start = false;
            }
        }
    }

    fn close_all(&mut self) {
        self.settle(TextStyles::default());
    }

    fn raw(&mut self, text: &str) {
        self.close_all();
        self.out.push_str(text);
        self.line_start = false;
    }

    fn text(&mut self, run: &StyledText, before_link: bool, hints: Hints) {
        if run.text.is_empty() {
            return;
        }
        let layers = TextStyles {
            code: false,
            ..run.styles
        };
        if run.styles.code {
            self.settle(layers);
            self.open_missing(layers, hints);
            self.out.push_str(&code_span(&run.text));
            self.line_start = false;
            return;
        }
        let core = run.text.trim();
        if core.is_empty() {
            self.settle(TextStyles::default());
            self.push_escaped(&run.text, before_link);
            return;
        }
        let start = run.text.len() - run.text.trim_start().len();
        let end = run.text.trim_end().len();
        let (lead, trail) = (&run.text[..start], &run.text[end..]);
        let styled = layers != TextStyles::default();
        self.settle(layers);
        self.push_escaped(lead, false);
        self.open_missing(layers, hints);
        self.push_escaped(core, before_link && trail.is_empty());
        if styled {
            self.pending.push_str(trail);
        } else {
            self.push_escaped(trail, before_link);
        }
    }

    fn push_escaped(&mut self, text: &str, before_link: bool) {
        let chars: Vec<char> = text.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let c = chars[index];
            if c == '\n' {
                self.out.push_str("\\\n");
                self.line_start = true;
                index += 1;
                continue;
            }
            if self.line_start && c.is_whitespace() {
                index += 1;
                continue;
            }
            let mut escape = match c {
                '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '~' => true,
                '&' => entity_follows(&chars[index + 1..]),
                '!' => before_link && index + 1 == chars.len(),
                _ => false,
            };
            if self.line_start {
                if matches!(c, '#' | '+' | '-' | '=') {
                    escape = true;
                }
                if c.is_ascii_digit() {
                    let digits = chars[index..]
                        .iter()
                        .take_while(|d| d.is_ascii_digit())
                        .count();
                    if matches!(chars.get(index + digits), Some('.' | ')')) {
                        self.out.extend(&chars[index..index + digits]);
                        self.out.push('\\');
                        self.out.push(chars[index + digits]);
                        self.line_start = false;
                        index += digits + 1;
                        continue;
                    }
                }
            }
            if escape {
                self.out.push('\\');
            }
            self.out.push(c);
            self.line_start = false;
            index += 1;
        }
    }

    fn finish(mut self) -> String {
        self.close_all();
        self.out
    }
}

fn entity_follows(rest: &[char]) -> bool {
    let name = rest
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || **c == '#')
        .count();
    name > 0 && rest.get(name) == Some(&';')
}

fn code_span(text: &str) -> String {
    let text = text.replace('\n', " ");
    let longest = text
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or_default();
    let fence = "`".repeat(longest + 1);
    let padded =
        !text.trim().is_empty() && (text.starts_with(['`', ' ']) || text.ends_with(['`', ' ']));
    if padded {
        format!("{fence} {text} {fence}")
    } else {
        format!("{fence}{text}{fence}")
    }
}

fn escape_plain(text: &str) -> String {
    let mut writer = Writer::new(false);
    writer.push_escaped(text, false);
    writer.out
}

fn escape_destination(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if matches!(c, '\\' | '(' | ')' | '<' | '>' | '&') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn render_inlines(inlines: &[Inline]) -> String {
    write_inlines(inlines, true)
}

fn render_inlines_after_marker(inlines: &[Inline]) -> String {
    write_inlines(inlines, false)
}

fn write_inlines(inlines: &[Inline], line_start: bool) -> String {
    let mut writer = Writer::new(line_start);
    let runs: Vec<Option<&StyledText>> = inlines
        .iter()
        .map(|inline| match inline {
            Inline::Text(run) => Some(run),
            _ => None,
        })
        .collect();
    for (index, inline) in inlines.iter().enumerate() {
        let before_link = matches!(
            inlines.get(index + 1),
            Some(Inline::Link { .. } | Inline::Mention { .. })
        );
        match inline {
            Inline::Text(run) => writer.text(run, before_link, hints_for(&runs, index)),
            Inline::Link { href, content } => {
                let mut inner = Writer::new(false);
                let inner_runs: Vec<Option<&StyledText>> = content.iter().map(Some).collect();
                for (position, run) in content.iter().enumerate() {
                    inner.text(run, false, hints_for(&inner_runs, position));
                }
                let text = inner.finish();
                writer.raw(&format!("[{text}]({})", escape_destination(href.as_str())));
            }
            Inline::Mention { resource, label } => {
                let mut inner = Writer::new(false);
                inner.text(
                    &StyledText::new(label.as_str(), TextStyles::default()),
                    false,
                    [false; 4],
                );
                let text = inner.finish();
                writer.raw(&format!(
                    "[{text}](mestier:{}/{})",
                    resource.resource_type.as_str(),
                    resource.id
                ));
            }
        }
    }
    writer.finish()
}

pub fn from_markdown(markdown: &str) -> Result<DocumentContent, ContentError> {
    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut reader = Reader::default();
    for event in Parser::new_ext(markdown, options) {
        reader.accept(event)?;
    }
    reader.flush_leaf();
    let mut next = 0;
    let blocks = build(reader.root, 1, &mut next)?;
    DocumentContent::new(blocks)
}

struct Draft {
    kind: BlockKind,
    children: Vec<Draft>,
}

fn build(drafts: Vec<Draft>, depth: usize, next: &mut usize) -> Result<Vec<Block>, ContentError> {
    if depth > MAX_DEPTH && !drafts.is_empty() {
        return Err(ContentError::TooDeep);
    }
    drafts
        .into_iter()
        .map(|draft| {
            *next += 1;
            let id = BlockId::new(&format!("b{next}"))?;
            let children = build(draft.children, depth + 1, next)?;
            Block::new(id, draft.kind, children)
        })
        .collect()
}

enum Frame {
    Item(ItemFrame),
    Quote { first: bool },
}

struct ItemFrame {
    ordered: bool,
    checked: Option<bool>,
    content: Option<Vec<Inline>>,
    children: Vec<Draft>,
}

enum Piece {
    Inline(Inline),
    Block(BlockKind, String),
}

enum LeafKind {
    Paragraph,
    Heading(HeadingLevel),
    Html,
}

struct Leaf {
    kind: LeafKind,
    pieces: Vec<Piece>,
}

struct LinkState {
    dest: String,
    texts: Vec<StyledText>,
}

struct ImageState {
    dest: String,
    alt: String,
}

struct CodeState {
    language: Option<CodeLanguage>,
    text: String,
}

#[derive(Default)]
struct Counters {
    bold: u32,
    italic: u32,
    strike: u32,
    underline: u32,
}

#[derive(Default)]
struct Reader {
    root: Vec<Draft>,
    frames: Vec<Frame>,
    lists: Vec<bool>,
    leaf: Option<Leaf>,
    link: Option<LinkState>,
    image: Option<ImageState>,
    code: Option<CodeState>,
    counters: Counters,
}

impl Reader {
    fn styles(&self) -> TextStyles {
        TextStyles {
            bold: self.counters.bold > 0,
            italic: self.counters.italic > 0,
            strike: self.counters.strike > 0,
            underline: self.counters.underline > 0,
            code: false,
        }
    }

    fn accept(&mut self, event: Event<'_>) -> Result<(), ContentError> {
        match event {
            Event::Start(tag) => self.start(tag)?,
            Event::End(end) => self.end(end),
            Event::Text(text) => self.add_text(&text, false),
            Event::Code(text) => self.add_text(&text, true),
            Event::SoftBreak => self.add_text(" ", false),
            Event::HardBreak => self.add_text("\n", false),
            Event::InlineHtml(html) => self.inline_html(&html),
            Event::Html(html) => self.add_text(&html, false),
            Event::Rule => {
                self.flush_leaf();
                self.push_block(Draft {
                    kind: BlockKind::Divider,
                    children: Vec::new(),
                });
            }
            Event::TaskListMarker(checked) => {
                if let Some(Frame::Item(item)) = self.frames.last_mut() {
                    item.checked = Some(checked);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn inline_html(&mut self, html: &str) {
        let tag = html.trim().to_ascii_lowercase();
        let (counter, opens) = match tag.as_str() {
            "<u>" => (&mut self.counters.underline, true),
            "</u>" => (&mut self.counters.underline, false),
            "<strong>" | "<b>" => (&mut self.counters.bold, true),
            "</strong>" | "</b>" => (&mut self.counters.bold, false),
            "<em>" | "<i>" => (&mut self.counters.italic, true),
            "</em>" | "</i>" => (&mut self.counters.italic, false),
            "<del>" | "<s>" => (&mut self.counters.strike, true),
            "</del>" | "</s>" => (&mut self.counters.strike, false),
            _ => {
                self.add_text(html, false);
                return;
            }
        };
        *counter = if opens {
            *counter + 1
        } else {
            counter.saturating_sub(1)
        };
    }

    fn start(&mut self, tag: Tag<'_>) -> Result<(), ContentError> {
        match tag {
            Tag::Paragraph => {
                self.flush_leaf();
                self.leaf = Some(Leaf {
                    kind: LeafKind::Paragraph,
                    pieces: Vec::new(),
                });
            }
            Tag::HtmlBlock => {
                self.flush_leaf();
                self.leaf = Some(Leaf {
                    kind: LeafKind::Html,
                    pieces: Vec::new(),
                });
            }
            Tag::Heading { level, .. } => {
                self.flush_leaf();
                self.leaf = Some(Leaf {
                    kind: LeafKind::Heading(heading_level(level)),
                    pieces: Vec::new(),
                });
            }
            Tag::BlockQuote(_) => {
                self.flush_leaf();
                self.enter(Frame::Quote { first: true })?;
            }
            Tag::List(start) => {
                self.flush_leaf();
                self.lists.push(start.is_some());
            }
            Tag::Item => {
                self.flush_leaf();
                self.enter(Frame::Item(ItemFrame {
                    ordered: self.lists.last().copied().unwrap_or(false),
                    checked: None,
                    content: None,
                    children: Vec::new(),
                }))?;
            }
            Tag::CodeBlock(kind) => {
                self.flush_leaf();
                let language = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .and_then(|word| CodeLanguage::new(word).ok()),
                    CodeBlockKind::Indented => None,
                };
                self.code = Some(CodeState {
                    language,
                    text: String::new(),
                });
            }
            Tag::Emphasis => self.counters.italic += 1,
            Tag::Strong => self.counters.bold += 1,
            Tag::Strikethrough => self.counters.strike += 1,
            Tag::Link { dest_url, .. } => {
                self.link = Some(LinkState {
                    dest: dest_url.into_string(),
                    texts: Vec::new(),
                });
            }
            Tag::Image { dest_url, .. } => {
                self.image = Some(ImageState {
                    dest: dest_url.into_string(),
                    alt: String::new(),
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn enter(&mut self, frame: Frame) -> Result<(), ContentError> {
        if self.frames.len() >= MAX_CONTAINERS {
            return Err(ContentError::TooDeep);
        }
        self.frames.push(frame);
        Ok(())
    }

    fn end(&mut self, end: TagEnd) {
        match end {
            TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::HtmlBlock => self.flush_leaf(),
            TagEnd::BlockQuote(_) => {
                self.flush_leaf();
                self.frames.pop();
            }
            TagEnd::List(_) => {
                self.flush_leaf();
                self.lists.pop();
            }
            TagEnd::Item => {
                self.flush_leaf();
                if let Some(Frame::Item(item)) = self.frames.pop() {
                    let content = item.content.unwrap_or_default();
                    let kind = match (item.checked, item.ordered) {
                        (Some(checked), _) => BlockKind::CheckItem { checked, content },
                        (None, true) => BlockKind::NumberedItem(content),
                        (None, false) => BlockKind::BulletItem(content),
                    };
                    self.push_block(Draft {
                        kind,
                        children: item.children,
                    });
                }
            }
            TagEnd::CodeBlock => {
                if let Some(code) = self.code.take() {
                    let text = code
                        .text
                        .strip_suffix('\n')
                        .unwrap_or(&code.text)
                        .to_owned();
                    self.push_block(Draft {
                        kind: BlockKind::Code {
                            language: code.language,
                            text,
                        },
                        children: Vec::new(),
                    });
                }
            }
            TagEnd::Emphasis => self.counters.italic = self.counters.italic.saturating_sub(1),
            TagEnd::Strong => self.counters.bold = self.counters.bold.saturating_sub(1),
            TagEnd::Strikethrough => {
                self.counters.strike = self.counters.strike.saturating_sub(1);
            }
            TagEnd::Link => self.end_link(),
            TagEnd::Image => self.end_image(),
            _ => {}
        }
    }

    fn add_text(&mut self, text: &str, code: bool) {
        if let Some(image) = &mut self.image {
            image.alt.push_str(text);
            return;
        }
        if let Some(block) = &mut self.code {
            block.text.push_str(text);
            return;
        }
        let styles = TextStyles {
            code,
            ..self.styles()
        };
        if let Some(link) = &mut self.link {
            push_styled(&mut link.texts, text, styles);
            return;
        }
        let pieces = &mut self.ensure_leaf().pieces;
        match pieces.last_mut() {
            Some(Piece::Inline(Inline::Text(last))) if last.styles == styles => {
                last.text.push_str(text);
            }
            _ => pieces.push(Piece::Inline(Inline::text(text, styles))),
        }
    }

    fn ensure_leaf(&mut self) -> &mut Leaf {
        self.leaf.get_or_insert_with(|| Leaf {
            kind: LeafKind::Paragraph,
            pieces: Vec::new(),
        })
    }

    fn push_piece(&mut self, piece: Piece) {
        let styles = self.styles();
        if let Some(link) = &mut self.link {
            let text = match piece {
                Piece::Inline(Inline::Text(run)) => run.text,
                Piece::Block(_, label) => label,
                Piece::Inline(_) => String::new(),
            };
            push_styled(&mut link.texts, &text, styles);
            return;
        }
        self.ensure_leaf().pieces.push(piece);
    }

    fn end_image(&mut self) {
        let Some(image) = self.image.take() else {
            return;
        };
        let alt = image.alt.trim().to_owned();
        let piece = match source_of(&image.dest) {
            Some((source, id)) => {
                let caption: String = alt.chars().take(MAX_CAPTION_CHARS).collect();
                let caption = (!caption.is_empty())
                    .then(|| Caption::new(&caption).ok())
                    .flatten();
                Piece::Block(
                    BlockKind::Image {
                        source,
                        id,
                        caption,
                        width: ImageWidth::FULL,
                    },
                    alt,
                )
            }
            None => {
                let shown = if alt.is_empty() { image.dest } else { alt };
                Piece::Inline(Inline::text(shown, self.styles()))
            }
        };
        self.push_piece(piece);
    }

    fn end_link(&mut self) {
        let Some(link) = self.link.take() else {
            return;
        };
        let label: String = link.texts.iter().map(|run| run.text.as_str()).collect();
        let piece = if let Some(resource) = mention_of(&link.dest) {
            Some(Piece::Inline(Inline::Mention { resource, label }))
        } else if let Some(kind) = block_of(&link.dest) {
            Some(Piece::Block(kind, label))
        } else if let Ok(href) = Href::new(&link.dest) {
            Some(Piece::Inline(Inline::Link {
                href,
                content: link.texts.clone(),
            }))
        } else {
            None
        };
        match piece {
            Some(piece) => self.push_piece(piece),
            None => {
                for run in link.texts {
                    self.push_piece(Piece::Inline(Inline::Text(run)));
                }
            }
        }
    }

    fn push_block(&mut self, draft: Draft) {
        for frame in self.frames.iter_mut().rev() {
            if let Frame::Item(item) = frame {
                item.children.push(draft);
                return;
            }
        }
        self.root.push(draft);
    }

    fn flush_leaf(&mut self) {
        let Some(leaf) = self.leaf.take() else {
            return;
        };
        match leaf.kind {
            LeafKind::Heading(level) => {
                let content = into_inlines(leaf.pieces);
                self.push_block(Draft {
                    kind: BlockKind::Heading { level, content },
                    children: Vec::new(),
                });
            }
            LeafKind::Html => {
                let mut content = into_inlines(leaf.pieces);
                if let Some(Inline::Text(last)) = content.last_mut() {
                    last.text.truncate(last.text.trim_end().len());
                }
                self.finish_paragraph(None, content);
            }
            LeafKind::Paragraph => {
                let standalone = match leaf.pieces.as_slice() {
                    [Piece::Block(kind, _)] => Some(kind.clone()),
                    _ => None,
                };
                self.finish_paragraph(standalone, into_inlines(leaf.pieces));
            }
        }
    }

    fn finish_paragraph(&mut self, standalone: Option<BlockKind>, content: Vec<Inline>) {
        if let Some(kind) = standalone {
            if let Some(Frame::Quote { first }) = self.frames.last_mut() {
                *first = false;
            }
            self.push_block(Draft {
                kind,
                children: Vec::new(),
            });
            return;
        }
        match self.frames.last_mut() {
            Some(Frame::Item(item)) if item.content.is_none() => item.content = Some(content),
            Some(Frame::Quote { first }) => {
                let was_first = std::mem::replace(first, false);
                let kind = match was_first.then(|| split_callout(&content)).flatten() {
                    Some((tone, rest)) => BlockKind::Callout {
                        tone,
                        content: rest,
                    },
                    None => BlockKind::Quote(content),
                };
                self.push_block(Draft {
                    kind,
                    children: Vec::new(),
                });
            }
            _ => self.push_block(Draft {
                kind: BlockKind::Paragraph(content),
                children: Vec::new(),
            }),
        }
    }
}

fn heading_level(level: MdHeading) -> HeadingLevel {
    match level {
        MdHeading::H1 => HeadingLevel::One,
        MdHeading::H2 => HeadingLevel::Two,
        _ => HeadingLevel::Three,
    }
}

fn push_styled(runs: &mut Vec<StyledText>, text: &str, styles: TextStyles) {
    match runs.last_mut() {
        Some(last) if last.styles == styles => last.text.push_str(text),
        _ => runs.push(StyledText::new(text, styles)),
    }
}

fn into_inlines(pieces: Vec<Piece>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::with_capacity(pieces.len());
    for piece in pieces {
        let inline = match piece {
            Piece::Inline(inline) => inline,
            Piece::Block(_, label) => Inline::text(label, TextStyles::default()),
        };
        match (&inline, out.last_mut()) {
            (Inline::Text(run), _) if run.text.is_empty() => {}
            (Inline::Text(run), Some(Inline::Text(last))) if last.styles == run.styles => {
                last.text.push_str(&run.text);
            }
            _ => out.push(inline),
        }
    }
    out
}

fn split_callout(content: &[Inline]) -> Option<(CalloutTone, Vec<Inline>)> {
    let Some(Inline::Text(first)) = content.first() else {
        return None;
    };
    let bold_only = TextStyles {
        bold: true,
        ..TextStyles::default()
    };
    if first.styles != bold_only {
        return None;
    }
    let tone = CALLOUT_LABELS
        .iter()
        .find(|(_, label)| *label == first.text)
        .map(|(tone, _)| *tone)?;
    let mut rest = content[1..].to_vec();
    if let Some(Inline::Text(next)) = rest.first_mut() {
        if next.text.starts_with(' ') {
            next.text.remove(0);
        }
        if next.text.is_empty() {
            rest.remove(0);
        }
    }
    Some((tone, rest))
}

fn mestier_path(dest: &str) -> Option<(&str, &str)> {
    dest.strip_prefix("mestier:")?.split_once('/')
}

fn source_of(dest: &str) -> Option<(ImageSource, Uuid)> {
    let (kind, id) = mestier_path(dest)?;
    let source = match kind {
        "drive" => ImageSource::Drive,
        "photo" => ImageSource::Photo,
        _ => return None,
    };
    Some((source, Uuid::parse_str(id).ok()?))
}

fn mention_of(dest: &str) -> Option<KnowledgeResource> {
    let (kind, id) = mestier_path(dest)?;
    Some(KnowledgeResource {
        resource_type: kind.parse::<KnowledgeResourceType>().ok()?,
        id: Uuid::parse_str(id).ok()?,
    })
}

fn block_of(dest: &str) -> Option<BlockKind> {
    let (kind, rest) = mestier_path(dest)?;
    match kind {
        "file" => Some(BlockKind::File {
            article_id: KnowledgeArticleId(Uuid::parse_str(rest).ok()?),
        }),
        "table" => {
            let (table, view) = rest.split_once("?view=")?;
            Some(BlockKind::TableView {
                table_id: KnowledgeTableId(Uuid::parse_str(table).ok()?),
                view_id: TableViewId(Uuid::parse_str(view).ok()?),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn plain(text: &str) -> Inline {
        Inline::text(text, TextStyles::default())
    }

    fn styled(text: &str, styles: TextStyles) -> Inline {
        Inline::text(text, styles)
    }

    fn bold() -> TextStyles {
        TextStyles {
            bold: true,
            ..TextStyles::default()
        }
    }

    fn block(index: usize, kind: BlockKind, children: Vec<Block>) -> Block {
        Block::new(BlockId::new(&format!("t{index}")).unwrap(), kind, children).unwrap()
    }

    fn document(blocks: Vec<Block>) -> DocumentContent {
        DocumentContent::new(blocks).unwrap()
    }

    fn kinds(markdown: &str) -> Vec<BlockKind> {
        fn flatten(blocks: &[Block], out: &mut Vec<BlockKind>) {
            for block in blocks {
                out.push(block.kind().clone());
                flatten(block.children(), out);
            }
        }
        let mut out = Vec::new();
        flatten(from_markdown(markdown).unwrap().blocks(), &mut out);
        out
    }

    #[test]
    fn headings_lists_checks_quotes_code_and_links_become_blocks() {
        let markdown = "# Titre\n\nUn texte avec **gras** et [un lien](https://exemple.fr).\n\n\
- un\n- [x] fait\n- [ ] à faire\n\n1. premier\n2. second\n\n> citation\n\n```rust\nlet a = 1;\n```\n\n---";
        let found = kinds(markdown);
        assert_eq!(
            found[0],
            BlockKind::Heading {
                level: HeadingLevel::One,
                content: vec![plain("Titre")]
            }
        );
        assert_eq!(
            found[1],
            BlockKind::Paragraph(vec![
                plain("Un texte avec "),
                styled("gras", bold()),
                plain(" et "),
                Inline::Link {
                    href: Href::new("https://exemple.fr").unwrap(),
                    content: vec![StyledText::new("un lien", TextStyles::default())],
                },
                plain("."),
            ])
        );
        assert_eq!(found[2], BlockKind::BulletItem(vec![plain("un")]));
        assert_eq!(
            found[3],
            BlockKind::CheckItem {
                checked: true,
                content: vec![plain("fait")]
            }
        );
        assert_eq!(
            found[4],
            BlockKind::CheckItem {
                checked: false,
                content: vec![plain("à faire")]
            }
        );
        assert_eq!(found[5], BlockKind::NumberedItem(vec![plain("premier")]));
        assert_eq!(found[6], BlockKind::NumberedItem(vec![plain("second")]));
        assert_eq!(found[7], BlockKind::Quote(vec![plain("citation")]));
        assert_eq!(
            found[8],
            BlockKind::Code {
                language: Some(CodeLanguage::new("rust").unwrap()),
                text: "let a = 1;".to_owned()
            }
        );
        assert_eq!(found[9], BlockKind::Divider);
    }

    #[test]
    fn mestier_links_become_mentions_images_files_and_table_views() {
        let (customer, drive, photo, file, table, view) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let markdown = format!(
            "Voir [Dupont](mestier:customer/{customer}) demain.\n\n\
![Façade](mestier:drive/{drive})\n\n![](mestier:photo/{photo})\n\n\
[Devis](mestier:file/{file})\n\n[Suivi](mestier:table/{table}?view={view})"
        );
        let found = kinds(&markdown);
        assert_eq!(
            found[0],
            BlockKind::Paragraph(vec![
                plain("Voir "),
                Inline::Mention {
                    resource: KnowledgeResource {
                        resource_type: KnowledgeResourceType::Customer,
                        id: customer
                    },
                    label: "Dupont".to_owned()
                },
                plain(" demain."),
            ])
        );
        assert_eq!(
            found[1],
            BlockKind::Image {
                source: ImageSource::Drive,
                id: drive,
                caption: Some(Caption::new("Façade").unwrap()),
                width: ImageWidth::FULL
            }
        );
        assert_eq!(
            found[2],
            BlockKind::Image {
                source: ImageSource::Photo,
                id: photo,
                caption: None,
                width: ImageWidth::FULL
            }
        );
        assert_eq!(
            found[3],
            BlockKind::File {
                article_id: KnowledgeArticleId(file)
            }
        );
        assert_eq!(
            found[4],
            BlockKind::TableView {
                table_id: KnowledgeTableId(table),
                view_id: TableViewId(view)
            }
        );
    }

    #[test]
    fn a_file_link_inside_a_sentence_is_only_text() {
        let file = Uuid::now_v7();
        let found = kinds(&format!("Lire [le devis](mestier:file/{file}) vite"));
        assert_eq!(
            found,
            vec![BlockKind::Paragraph(vec![plain("Lire le devis vite")])]
        );
    }

    #[test]
    fn what_markdown_knows_nothing_about_becomes_text() {
        let found = kinds("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(matches!(&found[0], BlockKind::Paragraph(content) if !content.is_empty()));
        let found = kinds("![logo](https://exemple.fr/logo.png)");
        assert_eq!(found, vec![BlockKind::Paragraph(vec![plain("logo")])]);
        let found = kinds("[clic](javascript:alert(1))");
        assert_eq!(found, vec![BlockKind::Paragraph(vec![plain("clic")])]);
        let found = kinds("<div>bloc</div>");
        assert_eq!(
            found,
            vec![BlockKind::Paragraph(vec![plain("<div>bloc</div>")])]
        );
    }

    #[test]
    fn a_quote_starting_with_a_bold_label_is_a_callout() {
        let found = kinds("> **Attention :** le chantier ferme\n\n> **Info :**");
        assert_eq!(
            found,
            vec![
                BlockKind::Callout {
                    tone: CalloutTone::Warning,
                    content: vec![plain("le chantier ferme")]
                },
                BlockKind::Callout {
                    tone: CalloutTone::Info,
                    content: Vec::new()
                },
            ]
        );
    }

    #[test]
    fn deep_levels_four_to_six_collapse_to_level_three() {
        let found = kinds("#### Détail");
        assert_eq!(
            found,
            vec![BlockKind::Heading {
                level: HeadingLevel::Three,
                content: vec![plain("Détail")]
            }]
        );
    }

    #[test]
    fn nested_lists_become_children() {
        let content = from_markdown("- a\n  - b\n    - c\n- d").unwrap();
        assert_eq!(content.blocks().len(), 2);
        assert_eq!(content.blocks()[0].children().len(), 1);
        assert_eq!(content.blocks()[0].children()[0].children().len(), 1);
    }

    #[test]
    fn too_deep_a_nesting_is_refused() {
        let markdown: String = (0..8)
            .map(|level| format!("{}- item\n", "  ".repeat(level)))
            .collect();
        assert_eq!(from_markdown(&markdown), Err(ContentError::TooDeep));
        let pathological = ">".repeat(5_000);
        assert_eq!(from_markdown(&pathological), Err(ContentError::TooDeep));
    }

    #[test]
    fn block_ids_are_sequential_and_unique() {
        let content = from_markdown("a\n\n- b\n  - c\n\nd").unwrap();
        assert_eq!(content.blocks()[0].id().as_str(), "b1");
        assert_eq!(content.blocks()[1].id().as_str(), "b2");
        assert_eq!(content.blocks()[1].children()[0].id().as_str(), "b3");
        assert_eq!(content.blocks()[2].id().as_str(), "b4");
    }

    #[test]
    fn rendering_uses_the_documented_forms() {
        let (customer, drive, file, table, view) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let content = document(vec![
            block(
                1,
                BlockKind::Callout {
                    tone: CalloutTone::Success,
                    content: vec![plain("fini")],
                },
                vec![],
            ),
            block(
                2,
                BlockKind::Paragraph(vec![Inline::Mention {
                    resource: KnowledgeResource {
                        resource_type: KnowledgeResourceType::Quote,
                        id: customer,
                    },
                    label: "D-12".to_owned(),
                }]),
                vec![],
            ),
            block(
                3,
                BlockKind::Image {
                    source: ImageSource::Drive,
                    id: drive,
                    caption: Some(Caption::new("Façade").unwrap()),
                    width: ImageWidth::new(40).unwrap(),
                },
                vec![],
            ),
            block(
                4,
                BlockKind::File {
                    article_id: KnowledgeArticleId(file),
                },
                vec![],
            ),
            block(
                5,
                BlockKind::TableView {
                    table_id: KnowledgeTableId(table),
                    view_id: TableViewId(view),
                },
                vec![],
            ),
        ]);
        let expected = format!(
            "> **Succès :** fini\n\n[D-12](mestier:quote/{customer})\n\n![Façade](mestier:drive/{drive})\n\n\
[Fichier](mestier:file/{file})\n\n[Vue](mestier:table/{table}?view={view})"
        );
        assert_eq!(render(&content), expected);
    }

    #[test]
    fn a_checklist_and_a_numbered_list_render_as_markdown() {
        let content = document(vec![
            block(
                1,
                BlockKind::CheckItem {
                    checked: true,
                    content: vec![plain("a")],
                },
                vec![],
            ),
            block(
                2,
                BlockKind::CheckItem {
                    checked: false,
                    content: vec![plain("b")],
                },
                vec![],
            ),
            block(3, BlockKind::NumberedItem(vec![plain("c")]), vec![]),
            block(4, BlockKind::NumberedItem(vec![plain("d")]), vec![]),
        ]);
        assert_eq!(render(&content), "- [x] a\n- [ ] b\n\n1. c\n2. d");
    }

    #[test]
    fn special_characters_are_escaped_and_read_back() {
        let content = document(vec![block(
            1,
            BlockKind::Paragraph(vec![plain("1. # *a* _b_ [c](d) <e> & `f`")]),
            vec![],
        )]);
        let markdown = render(&content);
        assert_eq!(
            from_markdown(&markdown).unwrap().blocks()[0].kind(),
            content.blocks()[0].kind()
        );
    }

    #[test]
    fn generated_ids_never_collide_with_a_valid_document() {
        let value = json!({ "version": 1, "blocks": [] });
        assert_eq!(
            DocumentContent::parse(&value).unwrap(),
            DocumentContent::empty()
        );
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Shape(BlockKind, Vec<Shape>);

    fn shape(blocks: &[Block]) -> Vec<Shape> {
        blocks
            .iter()
            .map(|block| {
                let kind = match block.kind() {
                    BlockKind::Image {
                        source,
                        id,
                        caption,
                        ..
                    } => BlockKind::Image {
                        source: *source,
                        id: *id,
                        caption: caption.clone(),
                        width: ImageWidth::FULL,
                    },
                    BlockKind::Code { text, .. } if text.is_empty() => BlockKind::Code {
                        language: None,
                        text: String::new(),
                    },
                    other => other.clone(),
                };
                Shape(kind, shape(block.children()))
            })
            .collect()
    }

    fn text_strategy() -> impl Strategy<Value = String> {
        proptest::collection::vec("[a-zé0-9 .+#*_<>&~\\[\\]()!:|=/'\\-`\\\\]{1,8}", 1..3)
            .prop_map(|parts| parts.join("\n"))
    }

    fn styles_strategy() -> impl Strategy<Value = TextStyles> {
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            0..6u8,
        )
            .prop_map(|(bold, italic, underline, strike, code)| TextStyles {
                bold,
                italic,
                underline,
                strike,
                code: code == 0,
            })
    }

    fn tidy_newlines(text: &str) -> String {
        text.split('\n')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn canonical_run(mut run: StyledText) -> Option<StyledText> {
        run.text = tidy_newlines(&run.text);
        if run.styles.code {
            run.text = run.text.replace('\n', " ");
            if run.text.trim().is_empty() {
                return None;
            }
        } else if run.styles != TextStyles::default() {
            run.text = run.text.trim().to_owned();
        }
        (!run.text.is_empty()).then_some(run)
    }

    fn push_merged(runs: &mut Vec<StyledText>, run: StyledText) {
        match runs.last_mut() {
            Some(last) if last.styles == run.styles => last.text.push_str(&run.text),
            _ => runs.push(run),
        }
    }

    fn canonical_runs(raw: Vec<StyledText>) -> Vec<StyledText> {
        let mut out = Vec::new();
        for run in raw.into_iter().filter_map(canonical_run) {
            push_merged(&mut out, run);
        }
        out
    }

    fn canonical_inlines(raw: Vec<Inline>) -> Vec<Inline> {
        let mut out: Vec<Inline> = Vec::new();
        for inline in raw {
            match inline {
                Inline::Text(run) => {
                    if let Some(run) = canonical_run(run) {
                        match out.last_mut() {
                            Some(Inline::Text(last)) if last.styles == run.styles => {
                                last.text.push_str(&run.text);
                            }
                            _ => out.push(Inline::Text(run)),
                        }
                    }
                }
                Inline::Link { href, content } => {
                    let content = canonical_runs(content);
                    if !content.is_empty() {
                        out.push(Inline::Link { href, content });
                    }
                }
                mention => out.push(mention),
            }
        }
        loop {
            match out.first_mut() {
                Some(Inline::Text(run)) if !run.styles.code => {
                    run.text = run.text.trim_start().to_owned();
                    if run.text.is_empty() {
                        out.remove(0);
                        continue;
                    }
                }
                _ => {}
            }
            break;
        }
        loop {
            match out.last_mut() {
                Some(Inline::Text(run)) if !run.styles.code => {
                    run.text = run.text.trim_end().to_owned();
                    if run.text.is_empty() {
                        out.pop();
                        continue;
                    }
                }
                _ => {}
            }
            break;
        }
        out
    }

    fn single_line(content: Vec<Inline>) -> Vec<Inline> {
        let flattened = content
            .into_iter()
            .map(|inline| match inline {
                Inline::Text(run) => {
                    Inline::Text(StyledText::new(run.text.replace('\n', " "), run.styles))
                }
                Inline::Link { href, content } => Inline::Link {
                    href,
                    content: content
                        .into_iter()
                        .map(|run| StyledText::new(run.text.replace('\n', " "), run.styles))
                        .collect(),
                },
                other => other,
            })
            .collect();
        let tidy = canonical_inlines(flattened);
        if tidy.is_empty() {
            vec![plain("x")]
        } else {
            tidy
        }
    }

    fn inline_strategy() -> impl Strategy<Value = Inline> {
        let run = (text_strategy(), styles_strategy())
            .prop_map(|(text, styles)| StyledText::new(text, styles));
        let link_run = (text_strategy(), styles_strategy()).prop_map(|(text, styles)| {
            StyledText::new(
                text,
                TextStyles {
                    code: false,
                    ..styles
                },
            )
        });
        prop_oneof![
            8 => run.prop_map(Inline::Text),
            1 => (
                prop_oneof![
                    Just("https://example.com/a?b=1&c=2"),
                    Just("http://x.fr/(p)"),
                    Just("mailto:a@b.fr")
                ],
                proptest::collection::vec(link_run, 1..3)
            )
                .prop_map(|(href, content)| Inline::Link {
                    href: Href::new(href).unwrap(),
                    content,
                }),
            1 => (0..3u8, "[a-zé]{1,6}").prop_map(|(kind, label)| Inline::Mention {
                resource: KnowledgeResource {
                    resource_type: match kind {
                        0 => KnowledgeResourceType::Customer,
                        1 => KnowledgeResourceType::Project,
                        _ => KnowledgeResourceType::Quote,
                    },
                    id: Uuid::from_u128(u128::from(kind) + 1),
                },
                label,
            }),
        ]
    }

    fn inlines_strategy() -> impl Strategy<Value = Vec<Inline>> {
        proptest::collection::vec(inline_strategy(), 0..5).prop_map(canonical_inlines)
    }

    fn non_empty_inlines() -> impl Strategy<Value = Vec<Inline>> {
        inlines_strategy().prop_map(|inlines| {
            if inlines.is_empty() {
                vec![plain("x")]
            } else {
                inlines
            }
        })
    }

    #[derive(Debug, Clone)]
    enum Draft {
        Leaf(BlockKind),
        Item(BlockKind, Vec<Draft>),
    }

    fn leaf_strategy() -> impl Strategy<Value = BlockKind> {
        prop_oneof![
            4 => non_empty_inlines().prop_map(BlockKind::Paragraph),
            2 => (1..4u64, non_empty_inlines()).prop_map(|(level, content)| BlockKind::Heading {
                level: HeadingLevel::new(level).unwrap(),
                content: single_line(content),
            }),
            2 => non_empty_inlines().prop_map(BlockKind::Quote),
            2 => (0..4u8, inlines_strategy()).prop_map(|(tone, content)| BlockKind::Callout {
                tone: match tone {
                    0 => CalloutTone::Info,
                    1 => CalloutTone::Success,
                    2 => CalloutTone::Warning,
                    _ => CalloutTone::Danger,
                },
                content
            }),
            1 => Just(BlockKind::Divider),
            2 => (
                proptest::option::of("[a-z]{1,10}"),
                proptest::collection::vec("[ a-z`~{}()#>*-]{0,12}", 0..4)
            )
                .prop_map(|(language, lines)| BlockKind::Code {
                    language: language.map(|l| CodeLanguage::new(&l).unwrap()),
                    text: lines.join("\n"),
                }),
            1 => (any::<bool>(), 1..9u128, proptest::option::of("[a-zé ]{1,10}")).prop_map(
                |(photo, id, caption)| BlockKind::Image {
                    source: if photo { ImageSource::Photo } else { ImageSource::Drive },
                    id: Uuid::from_u128(id),
                    caption: caption
                        .map(|c| c.trim().to_owned())
                        .filter(|c| !c.is_empty())
                        .map(|c| Caption::new(&c).unwrap()),
                    width: ImageWidth::FULL,
                }
            ),
            1 => (1..9u128).prop_map(|id| BlockKind::File {
                article_id: KnowledgeArticleId(Uuid::from_u128(id))
            }),
            1 => (1..9u128, 1..9u128).prop_map(|(table, view)| BlockKind::TableView {
                table_id: KnowledgeTableId(Uuid::from_u128(table)),
                view_id: TableViewId(Uuid::from_u128(view)),
            }),
        ]
    }

    fn item_kind_strategy() -> impl Strategy<Value = BlockKind> {
        prop_oneof![
            non_empty_inlines().prop_map(BlockKind::BulletItem),
            non_empty_inlines().prop_map(BlockKind::NumberedItem),
            (any::<bool>(), non_empty_inlines())
                .prop_map(|(checked, content)| BlockKind::CheckItem { checked, content }),
        ]
    }

    fn draft_strategy() -> impl Strategy<Value = Draft> {
        let leaf = leaf_strategy().prop_map(Draft::Leaf);
        leaf.prop_recursive(2, 12, 3, |inner| {
            prop_oneof![
                inner.clone(),
                (item_kind_strategy(), proptest::collection::vec(inner, 0..3))
                    .prop_map(|(kind, children)| Draft::Item(kind, children)),
            ]
        })
    }

    fn materialize(drafts: Vec<Draft>, next: &mut usize) -> Vec<Block> {
        drafts
            .into_iter()
            .map(|draft| {
                *next += 1;
                let id = BlockId::new(&format!("g{next}")).unwrap();
                match draft {
                    Draft::Leaf(kind) => Block::new(id, kind, vec![]).unwrap(),
                    Draft::Item(kind, children) => {
                        let children = materialize(children, next);
                        Block::new(id, kind, children).unwrap()
                    }
                }
            })
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(400))]

        #[test]
        fn rendering_then_reading_markdown_gives_the_same_blocks(
            drafts in proptest::collection::vec(draft_strategy(), 1..6)
        ) {
            let mut next = 0;
            let content = DocumentContent::new(materialize(drafts, &mut next)).unwrap();
            let markdown = render(&content);
            let back = from_markdown(&markdown).unwrap();
            prop_assert_eq!(shape(back.blocks()), shape(content.blocks()), "markdown:\n{}", markdown);
        }
    }
}
