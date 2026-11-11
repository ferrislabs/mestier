import {
	BlockNoteSchema,
	createCodeBlockSpec,
	createHeadingBlockSpec,
	defaultBlockSpecs,
	defaultInlineContentSpecs,
	defaultStyleSpecs,
} from '@blocknote/core'
import {
	createCalloutBlock,
	createFileBlock,
	createImageBlock,
	createTableViewBlock,
} from '#/pages/knowledge-documents/feature/document-blocks'
import { mentionInlineSpec } from '#/pages/knowledge-documents/feature/document-mention'
import { MAX_HEADING_LEVEL } from '#/pages/knowledge-documents/lib/blocks'

export const CODE_LANGUAGES = {
	text: { name: 'Texte brut' },
	bash: { name: 'Shell', aliases: ['sh', 'shell'] },
	css: { name: 'CSS' },
	html: { name: 'HTML' },
	javascript: { name: 'JavaScript', aliases: ['js'] },
	json: { name: 'JSON' },
	markdown: { name: 'Markdown', aliases: ['md'] },
	python: { name: 'Python', aliases: ['py'] },
	rust: { name: 'Rust', aliases: ['rs'] },
	sql: { name: 'SQL' },
	typescript: { name: 'TypeScript', aliases: ['ts'] },
	yaml: { name: 'YAML', aliases: ['yml'] },
}

export const documentSchema = BlockNoteSchema.create({
	blockSpecs: {
		paragraph: defaultBlockSpecs.paragraph,
		heading: createHeadingBlockSpec({
			levels: Array.from({ length: MAX_HEADING_LEVEL }, (_, i) => i + 1),
			allowToggleHeadings: false,
		}),
		bulletListItem: defaultBlockSpecs.bulletListItem,
		numberedListItem: defaultBlockSpecs.numberedListItem,
		checkListItem: defaultBlockSpecs.checkListItem,
		quote: defaultBlockSpecs.quote,
		divider: defaultBlockSpecs.divider,
		codeBlock: createCodeBlockSpec({
			defaultLanguage: 'text',
			supportedLanguages: CODE_LANGUAGES,
		}),
		callout: createCalloutBlock(),
		image: createImageBlock(),
		file: createFileBlock(),
		tableView: createTableViewBlock(),
	},
	inlineContentSpecs: {
		text: defaultInlineContentSpecs.text,
		link: defaultInlineContentSpecs.link,
		mention: mentionInlineSpec,
	},
	styleSpecs: {
		bold: defaultStyleSpecs.bold,
		italic: defaultStyleSpecs.italic,
		underline: defaultStyleSpecs.underline,
		strike: defaultStyleSpecs.strike,
		code: defaultStyleSpecs.code,
	},
})

export type DocumentSchema = typeof documentSchema
