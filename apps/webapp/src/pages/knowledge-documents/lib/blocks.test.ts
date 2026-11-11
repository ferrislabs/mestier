import { describe, expect, it } from 'vitest'
import {
	type EditorBlock,
	toDocumentContent,
	toEditorBlocks,
} from '#/pages/knowledge-documents/lib/blocks'
import type {
	DocumentBlock,
	DocumentContent,
	Inline,
} from '#/pages/knowledge-documents/lib/document-api-types'

const ID = '2f6c1e5a-0000-4000-8000-000000000001'

function textOf(value: string, styles = {}): Inline & { type: 'text' } {
	return { type: 'text', text: value, styles }
}

function block<T extends DocumentBlock>(
	id: string,
	type: T['type'],
	props: T['props'],
	content: T['content'],
	children: DocumentBlock[] = [],
): DocumentBlock {
	return { id, type, props, content, children } as DocumentBlock
}

function content(...blocks: DocumentBlock[]): DocumentContent {
	return { version: 1, blocks }
}

function roundTrip(input: DocumentContent): DocumentContent {
	return toDocumentContent(toEditorBlocks(input))
}

describe('Mestier to editor and back', () => {
	it('keeps a paragraph with every text style', () => {
		const doc = content(
			block('a', 'paragraph', {}, [
				textOf('plain '),
				textOf('bold', { bold: true }),
				textOf('mix', { bold: true, italic: true, underline: true }),
				textOf('gone', { strike: true }),
				textOf('mono', { code: true }),
			]),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps a link and a mention inside a sentence', () => {
		const doc = content(
			block('a', 'paragraph', {}, [
				textOf('Voir '),
				{
					type: 'link',
					href: 'https://mestier.app/aide',
					content: [textOf('l’aide', { bold: true })],
				},
				textOf(' pour '),
				{
					type: 'mention',
					resource_type: 'customer',
					resource_id: ID,
					label: 'Jardins du Parc',
				},
			]),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it.each([
		['bullet_item', 'bulletListItem'],
		['numbered_item', 'numberedListItem'],
	] as const)('keeps %s with nested children', (type, editorType) => {
		const doc = content(
			block(
				'a',
				type,
				{},
				[textOf('parent')],
				[block('b', type, {}, [textOf('child')])],
			),
		)
		expect(toEditorBlocks(doc)[0]?.type).toBe(editorType)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps headings of level 1 to 3', () => {
		const doc = content(
			block('a', 'heading', { level: 1 }, [textOf('Un')]),
			block('b', 'heading', { level: 2 }, [textOf('Deux')]),
			block('c', 'heading', { level: 3 }, [textOf('Trois')]),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps check items checked or not, with children', () => {
		const doc = content(
			block(
				'a',
				'check_item',
				{ checked: true },
				[textOf('fait')],
				[block('b', 'check_item', { checked: false }, [textOf('à faire')])],
			),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps quotes and every callout tone', () => {
		const doc = content(
			block('a', 'quote', {}, [textOf('citation')]),
			block('b', 'callout', { tone: 'info' }, [textOf('i')]),
			block('c', 'callout', { tone: 'success' }, [textOf('s')]),
			block('d', 'callout', { tone: 'warning' }, [textOf('w')]),
			block('e', 'callout', { tone: 'danger' }, [textOf('d')]),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps a divider', () => {
		const doc = content(block('a', 'divider', {}, []))
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps code as one unstyled text, with or without a language', () => {
		const doc = content(
			block('a', 'code', { language: 'typescript' }, [
				textOf('const a = 1\nconst b = 2'),
			]),
			block('b', 'code', {}, [textOf('echo hi')]),
			block('c', 'code', {}, []),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps drive and photo images with caption and width', () => {
		const doc = content(
			block('a', 'image', { source: 'drive', id: ID, width: 100 }, []),
			block(
				'b',
				'image',
				{ source: 'photo', id: ID, caption: 'Haie taillée', width: 40 },
				[],
			),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps a file block and a table view block', () => {
		const doc = content(
			block('a', 'file', { article_id: ID }, []),
			block('b', 'table_view', { table_id: ID, view_id: 'v1' }, []),
		)
		expect(roundTrip(doc)).toEqual(doc)
	})

	it('keeps an empty document empty', () => {
		expect(roundTrip(content())).toEqual(content())
	})

	it('keeps block ids', () => {
		const doc = content(
			block('first', 'paragraph', {}, [textOf('a')]),
			block('second', 'paragraph', {}, [textOf('b')]),
		)
		expect(roundTrip(doc).blocks.map((entry) => entry.id)).toEqual([
			'first',
			'second',
		])
	})
})

describe('editor to Mestier and back', () => {
	it('gives the same editor blocks back for every block type', () => {
		const blocks: EditorBlock[] = [
			{
				id: '1',
				type: 'heading',
				props: { level: 2 },
				content: [{ type: 'text', text: 'Titre', styles: {} }],
				children: [],
			},
			{
				id: '2',
				type: 'callout',
				props: { tone: 'warning' },
				content: [{ type: 'text', text: 'Attention', styles: { bold: true } }],
				children: [],
			},
			{
				id: '3',
				type: 'image',
				props: { source: 'photo', id: ID, caption: 'Vue', width: 60 },
				children: [],
			},
			{
				id: '4',
				type: 'file',
				props: { articleId: ID },
				children: [],
			},
			{
				id: '5',
				type: 'tableView',
				props: { tableId: ID, viewId: 'v1' },
				children: [],
			},
			{
				id: '6',
				type: 'paragraph',
				props: {},
				content: [
					{
						type: 'mention',
						props: {
							resourceType: 'quote',
							resourceId: ID,
							label: 'DEV-12',
						},
					},
				],
				children: [],
			},
			{ id: '7', type: 'divider', props: {}, children: [] },
		]
		const back = toEditorBlocks(toDocumentContent(blocks))
		expect(back).toEqual(blocks)
	})

	it('drops colours, alignment and the default language', () => {
		const result = toDocumentContent([
			{
				id: '1',
				type: 'paragraph',
				props: {
					textColor: 'red',
					backgroundColor: 'blue',
					textAlignment: 'center',
				},
				content: [
					{ type: 'text', text: 'x', styles: { bold: true, textColor: 'red' } },
				],
				children: [],
			},
			{
				id: '2',
				type: 'codeBlock',
				props: { language: 'text' },
				content: [{ type: 'text', text: 'ls', styles: {} }],
				children: [],
			},
		])
		expect(result.blocks[0]).toEqual({
			id: '1',
			type: 'paragraph',
			props: {},
			content: [{ type: 'text', text: 'x', styles: { bold: true } }],
			children: [],
		})
		expect(result.blocks[1]?.props).toEqual({})
	})
})

describe('what the editor can produce but the format cannot hold', () => {
	function paragraph(value: string, children: EditorBlock[] = []): EditorBlock {
		return {
			id: value,
			type: 'paragraph',
			props: {},
			content: value ? [{ type: 'text', text: value, styles: {} }] : [],
			children,
		}
	}

	it('merges neighbouring text with the same styles and drops empty text', () => {
		const result = toDocumentContent([
			{
				id: '1',
				type: 'paragraph',
				props: {},
				content: [
					{ type: 'text', text: 'a', styles: {} },
					{ type: 'text', text: '', styles: { bold: true } },
					{ type: 'text', text: 'b', styles: {} },
				],
				children: [],
			},
		])
		expect(result.blocks[0]?.content).toEqual([textOf('ab')])
	})

	it('trims empty paragraphs at the end of the document', () => {
		const result = toDocumentContent([
			paragraph('a'),
			paragraph(''),
			paragraph(''),
		])
		expect(result.blocks.map((entry) => entry.id)).toEqual(['a'])
	})

	it('keeps an empty paragraph that is followed by something', () => {
		const result = toDocumentContent([paragraph(''), paragraph('a')])
		expect(result.blocks).toHaveLength(2)
	})

	it('hoists the children of a block that cannot have any', () => {
		const result = toDocumentContent([
			{
				id: 'h',
				type: 'heading',
				props: { level: 1 },
				content: [{ type: 'text', text: 'T', styles: {} }],
				children: [paragraph('under')],
			},
		])
		expect(result.blocks.map((entry) => entry.id)).toEqual(['h', 'under'])
		expect(result.blocks[0]?.children).toEqual([])
	})

	it('drops image, file and table view blocks that point at nothing yet', () => {
		const result = toDocumentContent([
			{
				id: '1',
				type: 'image',
				props: { source: 'drive', id: '' },
				children: [],
			},
			{ id: '2', type: 'file', props: { articleId: '' }, children: [] },
			{
				id: '3',
				type: 'tableView',
				props: { tableId: ID, viewId: '' },
				children: [],
			},
			paragraph('kept'),
		])
		expect(result.blocks.map((entry) => entry.id)).toEqual(['kept'])
	})

	it('keeps only http, https and mailto links, and turns the rest into text', () => {
		const link = (href: string): EditorBlock => ({
			id: href,
			type: 'paragraph',
			props: {},
			content: [
				{
					type: 'link',
					href,
					content: [{ type: 'text', text: 'ici', styles: {} }],
				},
			],
			children: [],
		})
		const result = toDocumentContent([
			link('https://a.fr'),
			link('http://a.fr'),
			link('mailto:a@a.fr'),
			link('javascript:alert(1)'),
			link('ftp://a.fr'),
			link('a.fr/page'),
		])
		const kinds = result.blocks.map((entry) => {
			const first = (entry.content as Inline[])[0]
			return first?.type === 'link' ? first.href : first?.type
		})
		expect(kinds).toEqual([
			'https://a.fr',
			'http://a.fr',
			'mailto:a@a.fr',
			'text',
			'text',
			'https://a.fr/page',
		])
	})

	it('clamps the image width and the heading level', () => {
		const result = toDocumentContent([
			{
				id: '1',
				type: 'image',
				props: { source: 'drive', id: ID, width: 3 },
				children: [],
			},
			{
				id: '2',
				type: 'image',
				props: { source: 'drive', id: ID, width: 900 },
				children: [],
			},
			{
				id: '3',
				type: 'heading',
				props: { level: 5 },
				content: [],
				children: [],
			},
		])
		expect(result.blocks.map((entry) => entry.props)).toEqual([
			{ source: 'drive', id: ID, width: 10 },
			{ source: 'drive', id: ID, width: 100 },
			{ level: 3 },
		])
	})

	it('ignores a block type it does not know', () => {
		const result = toDocumentContent([
			{ id: '1', type: 'toggleListItem', props: {}, content: [], children: [] },
			paragraph('a'),
		])
		expect(result.blocks.map((entry) => entry.id)).toEqual(['a'])
	})

	it('gives a fresh id to a block whose id repeats', () => {
		const result = toDocumentContent([
			paragraph('a'),
			{ ...paragraph('b'), id: 'a' },
		])
		const ids = result.blocks.map((entry) => entry.id)
		expect(new Set(ids).size).toBe(2)
	})
})

describe('reading what the server stored', () => {
	it('reads a null content as an empty document', () => {
		expect(toEditorBlocks(null)).toEqual([])
	})
})

describe('ids that do not come from the app', () => {
	it('keeps the label as text when a pasted mention has no valid id', () => {
		const result = toDocumentContent([
			{
				id: '1',
				type: 'paragraph',
				props: {},
				content: [
					{
						type: 'mention',
						props: {
							resourceType: 'customer',
							resourceId: 'x',
							label: 'Dupont',
						},
					},
				],
				children: [],
			},
		])
		expect(result.blocks[0]?.content).toEqual([
			{ type: 'text', text: 'Dupont', styles: {} },
		])
	})

	it('drops image, file and table blocks whose id is not a uuid', () => {
		const result = toDocumentContent([
			{
				id: '1',
				type: 'image',
				props: { source: 'drive', id: 'nope' },
				children: [],
			},
			{ id: '2', type: 'file', props: { articleId: 'nope' }, children: [] },
			{
				id: '3',
				type: 'tableView',
				props: { tableId: 'nope', viewId: 'v1' },
				children: [],
			},
		])
		expect(result.blocks).toEqual([])
	})
})
