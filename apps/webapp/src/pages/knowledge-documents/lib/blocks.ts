import type {
	CalloutTone,
	DocumentBlock,
	DocumentContent,
	ImageSource,
	Inline,
	LinkInline,
	MentionInline,
	MentionKind,
	TextInline,
	TextStyles,
} from '#/pages/knowledge-documents/lib/document-api-types'

export interface EditorBlock {
	id: string
	type: string
	props: Record<string, unknown>
	content?: unknown
	children: EditorBlock[]
}

export const DEFAULT_CODE_LANGUAGE = 'text'
export const MIN_IMAGE_WIDTH = 10
export const MAX_IMAGE_WIDTH = 100
export const MAX_HEADING_LEVEL = 3

const STYLE_NAMES = ['bold', 'italic', 'underline', 'strike', 'code'] as const
const TONES: readonly CalloutTone[] = ['info', 'success', 'warning', 'danger']
const MENTION_KINDS: readonly MentionKind[] = ['customer', 'project', 'quote']
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
const SAFE_LINK = /^(https?:|mailto:)/i
const HAS_SCHEME = /^[a-z][a-z0-9+.-]*:/i
const LOOKS_LIKE_HOST = /^[^\s/:@]+\.[^\s/:@]+(\/\S*)?$/

const CHILDREN_ALLOWED: ReadonlySet<DocumentBlock['type']> = new Set([
	'paragraph',
	'bullet_item',
	'numbered_item',
	'check_item',
])

const EDITOR_TO_DOCUMENT: Readonly<Record<string, DocumentBlock['type']>> = {
	paragraph: 'paragraph',
	heading: 'heading',
	bulletListItem: 'bullet_item',
	numberedListItem: 'numbered_item',
	checkListItem: 'check_item',
	quote: 'quote',
	callout: 'callout',
	divider: 'divider',
	codeBlock: 'code',
	image: 'image',
	file: 'file',
	tableView: 'table_view',
}

const DOCUMENT_TO_EDITOR: Readonly<Record<DocumentBlock['type'], string>> = {
	paragraph: 'paragraph',
	heading: 'heading',
	bullet_item: 'bulletListItem',
	numbered_item: 'numberedListItem',
	check_item: 'checkListItem',
	quote: 'quote',
	callout: 'callout',
	divider: 'divider',
	code: 'codeBlock',
	image: 'image',
	file: 'file',
	table_view: 'tableView',
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function str(value: unknown): string {
	return typeof value === 'string' ? value : ''
}

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value))
}

function stylesOf(raw: unknown): TextStyles {
	const styles: TextStyles = {}
	if (!isRecord(raw)) return styles
	for (const name of STYLE_NAMES) {
		if (raw[name] === true) styles[name] = true
	}
	return styles
}

function sameStyles(a: TextStyles, b: TextStyles): boolean {
	return STYLE_NAMES.every((name) => (a[name] ?? false) === (b[name] ?? false))
}

function mergeText(parts: readonly TextInline[]): TextInline[] {
	const merged: TextInline[] = []
	for (const part of parts) {
		if (part.text === '') continue
		const last = merged.at(-1)
		if (last && sameStyles(last.styles, part.styles)) {
			merged[merged.length - 1] = { ...last, text: last.text + part.text }
		} else {
			merged.push({ ...part, styles: { ...part.styles } })
		}
	}
	return merged
}

function textParts(raw: unknown): TextInline[] {
	if (!Array.isArray(raw)) return []
	return raw.flatMap((entry): TextInline[] =>
		isRecord(entry) && entry.type === 'text'
			? [
					{
						type: 'text',
						text: str(entry.text),
						styles: stylesOf(entry.styles),
					},
				]
			: [],
	)
}

export function safeHref(href: string): string | null {
	const trimmed = href.trim()
	if (trimmed === '') return null
	if (SAFE_LINK.test(trimmed)) return trimmed
	if (HAS_SCHEME.test(trimmed)) return null
	return LOOKS_LIKE_HOST.test(trimmed) ? `https://${trimmed}` : null
}

function inlineFromEditor(raw: unknown): Inline[] {
	if (!Array.isArray(raw)) return []
	const result: Inline[] = []
	let run: TextInline[] = []
	const flush = () => {
		result.push(...mergeText(run))
		run = []
	}
	for (const entry of raw) {
		if (!isRecord(entry)) continue
		if (entry.type === 'text') {
			run.push(...textParts([entry]))
		} else if (entry.type === 'link') {
			const inner = textParts(entry.content)
			const href = safeHref(str(entry.href))
			if (href === null) {
				run.push(...inner)
				continue
			}
			flush()
			const content = mergeText(inner)
			if (content.length > 0) {
				const link: LinkInline = { type: 'link', href, content }
				result.push(link)
			}
		} else if (entry.type === 'mention' && isRecord(entry.props)) {
			const kind = entry.props.resourceType
			const id = str(entry.props.resourceId)
			if (MENTION_KINDS.includes(kind as MentionKind) && UUID.test(id)) {
				flush()
				const mention: MentionInline = {
					type: 'mention',
					resource_type: kind as MentionKind,
					resource_id: id,
					label: str(entry.props.label),
				}
				result.push(mention)
			} else {
				const label = str(entry.props.label)
				if (label !== '') run.push({ type: 'text', text: label, styles: {} })
			}
		}
	}
	flush()
	return result
}

function inlineEntryToEditor(entry: Inline): unknown {
	if (entry.type === 'text') {
		return { type: 'text', text: entry.text, styles: { ...entry.styles } }
	}
	if (entry.type === 'link') {
		return {
			type: 'link',
			href: entry.href,
			content: entry.content.map((part) => ({
				type: 'text',
				text: part.text,
				styles: { ...part.styles },
			})),
		}
	}
	return {
		type: 'mention',
		props: {
			resourceType: entry.resource_type,
			resourceId: entry.resource_id,
			label: entry.label,
		},
	}
}

function inlineToEditor(content: readonly Inline[]): unknown[] {
	return content.map(inlineEntryToEditor)
}

function plainText(raw: unknown): string {
	return textParts(raw)
		.map((part) => part.text)
		.join('')
}

class IdBook {
	private readonly seen = new Set<string>()
	private counter = 0

	take(id: string): string {
		let next = id === '' ? this.fresh() : id.slice(0, 64)
		while (this.seen.has(next)) next = this.fresh()
		this.seen.add(next)
		return next
	}

	private fresh(): string {
		this.counter += 1
		return `b${this.counter}-${Math.random().toString(36).slice(2, 10)}`
	}
}

function toneOf(value: unknown): CalloutTone {
	return TONES.includes(value as CalloutTone) ? (value as CalloutTone) : 'info'
}

function convertBlock(source: EditorBlock, ids: IdBook): DocumentBlock | null {
	const type = EDITOR_TO_DOCUMENT[source.type]
	if (!type) return null
	const props = source.props ?? {}
	const base = { content: [], children: [] as DocumentBlock[] }
	const inline = () => inlineFromEditor(source.content)
	switch (type) {
		case 'paragraph':
		case 'bullet_item':
		case 'numbered_item':
		case 'quote':
			return {
				id: ids.take(source.id),
				type,
				props: {},
				...base,
				content: inline(),
			}
		case 'heading': {
			const level = clamp(Number(props.level) || 1, 1, MAX_HEADING_LEVEL)
			return {
				id: ids.take(source.id),
				type,
				props: { level: level as 1 | 2 | 3 },
				...base,
				content: inline(),
			}
		}
		case 'check_item':
			return {
				id: ids.take(source.id),
				type,
				props: { checked: props.checked === true },
				...base,
				content: inline(),
			}
		case 'callout':
			return {
				id: ids.take(source.id),
				type,
				props: { tone: toneOf(props.tone) },
				...base,
				content: inline(),
			}
		case 'divider':
			return { id: ids.take(source.id), type, props: {}, ...base, content: [] }
		case 'code': {
			const text = plainText(source.content)
			const language = str(props.language)
			return {
				id: ids.take(source.id),
				type,
				props:
					language !== '' && language !== DEFAULT_CODE_LANGUAGE
						? { language: language.slice(0, 32) }
						: {},
				...base,
				content: text === '' ? [] : [{ type: 'text', text, styles: {} }],
			}
		}
		case 'image': {
			const id = str(props.id)
			if (!UUID.test(id)) return null
			const caption = str(props.caption)
			const width = clamp(
				Math.round(Number(props.width) || MAX_IMAGE_WIDTH),
				MIN_IMAGE_WIDTH,
				MAX_IMAGE_WIDTH,
			)
			return {
				id: ids.take(source.id),
				type,
				props: {
					source: (props.source === 'photo' ? 'photo' : 'drive') as ImageSource,
					id,
					...(caption === '' ? {} : { caption: caption.slice(0, 500) }),
					width,
				},
				...base,
				content: [],
			}
		}
		case 'file': {
			const articleId = str(props.articleId)
			if (!UUID.test(articleId)) return null
			return {
				id: ids.take(source.id),
				type,
				props: { article_id: articleId },
				...base,
				content: [],
			}
		}
		case 'table_view': {
			const tableId = str(props.tableId)
			const viewId = str(props.viewId)
			if (!UUID.test(tableId) || viewId === '') return null
			return {
				id: ids.take(source.id),
				type,
				props: { table_id: tableId, view_id: viewId },
				...base,
				content: [],
			}
		}
	}
}

function convertAll(
	sources: readonly EditorBlock[],
	ids: IdBook,
): DocumentBlock[] {
	const result: DocumentBlock[] = []
	for (const source of sources) {
		const converted = convertBlock(source, ids)
		const kids = convertAll(source.children ?? [], ids)
		if (!converted) {
			result.push(...kids)
			continue
		}
		if (CHILDREN_ALLOWED.has(converted.type)) {
			converted.children = kids
			result.push(converted)
		} else {
			result.push(converted, ...kids)
		}
	}
	return result
}

function isEmptyParagraph(block: DocumentBlock): boolean {
	return (
		block.type === 'paragraph' &&
		block.content.length === 0 &&
		block.children.length === 0
	)
}

export function toDocumentContent(
	blocks: readonly EditorBlock[],
): DocumentContent {
	const converted = convertAll(blocks, new IdBook())
	while (
		converted.length > 0 &&
		isEmptyParagraph(converted[converted.length - 1] as DocumentBlock)
	) {
		converted.pop()
	}
	return { version: 1, blocks: converted }
}

function blockToEditor(block: DocumentBlock): EditorBlock {
	const type = DOCUMENT_TO_EDITOR[block.type]
	const children = block.children.map(blockToEditor)
	switch (block.type) {
		case 'paragraph':
		case 'bullet_item':
		case 'numbered_item':
		case 'quote':
			return {
				id: block.id,
				type,
				props: {},
				content: inlineToEditor(block.content),
				children,
			}
		case 'heading':
			return {
				id: block.id,
				type,
				props: { level: block.props.level },
				content: inlineToEditor(block.content),
				children,
			}
		case 'check_item':
			return {
				id: block.id,
				type,
				props: { checked: block.props.checked },
				content: inlineToEditor(block.content),
				children,
			}
		case 'callout':
			return {
				id: block.id,
				type,
				props: { tone: block.props.tone },
				content: inlineToEditor(block.content),
				children,
			}
		case 'divider':
			return { id: block.id, type, props: {}, children }
		case 'code':
			return {
				id: block.id,
				type,
				props: { language: block.props.language ?? DEFAULT_CODE_LANGUAGE },
				content: inlineToEditor(block.content),
				children,
			}
		case 'image':
			return {
				id: block.id,
				type,
				props: {
					source: block.props.source,
					id: block.props.id,
					caption: block.props.caption ?? '',
					width: block.props.width,
				},
				children,
			}
		case 'file':
			return {
				id: block.id,
				type,
				props: { articleId: block.props.article_id },
				children,
			}
		case 'table_view':
			return {
				id: block.id,
				type,
				props: { tableId: block.props.table_id, viewId: block.props.view_id },
				children,
			}
	}
}

export function toEditorBlocks(
	content: DocumentContent | null | undefined,
): EditorBlock[] {
	if (!content || !Array.isArray(content.blocks)) return []
	return content.blocks.map(blockToEditor)
}
