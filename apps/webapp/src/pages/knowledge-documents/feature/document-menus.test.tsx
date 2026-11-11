import { BlockNoteEditor } from '@blocknote/core'
import { fr } from '@blocknote/core/locales'
import { describe, expect, it } from 'vitest'
import {
	documentSlashItems,
	mentionItems,
} from '#/pages/knowledge-documents/feature/document-menus'
import { documentSchema } from '#/pages/knowledge-documents/feature/document-schema'

function editor() {
	return BlockNoteEditor.create({ schema: documentSchema, dictionary: fr })
}

describe('the slash menu', () => {
	it('offers only the blocks of the document format', () => {
		const titles = documentSlashItems(editor()).map((item) => item.title)
		expect(titles.sort()).toEqual(
			[
				'Titre 1',
				'Titre 2',
				'Titre 3',
				'Citation',
				'Liste Numérotée',
				'Liste à puces',
				'Liste de tâches',
				'Paragraphe',
				'Bloc de code',
				'Diviseur',
				'Encadré',
				'Image',
				'Fichier',
				'Vue de table',
			].sort(),
		)
	})

	it('has no table, video, audio, toggle, emoji or heading 4 to 6 entry', () => {
		const titles = documentSlashItems(editor())
			.map((item) => item.title.toLowerCase())
			.join('|')
		for (const forbidden of [
			'vidéo',
			'audio',
			'emoji',
			'titre 4',
			'titre 5',
			'titre 6',
			'déroulant',
		]) {
			expect(titles).not.toContain(forbidden)
		}
		expect(titles).not.toMatch(/(^|\|)tableau(\||$)/)
	})

	it('only knows the block types of the document format', () => {
		expect(Object.keys(editor().schema.blockSchema).sort()).toEqual(
			[
				'paragraph',
				'heading',
				'bulletListItem',
				'numberedListItem',
				'checkListItem',
				'quote',
				'divider',
				'codeBlock',
				'callout',
				'image',
				'file',
				'tableView',
			].sort(),
		)
		expect(Object.keys(editor().schema.inlineContentSchema).sort()).toEqual([
			'link',
			'mention',
			'text',
		])
		expect(Object.keys(editor().schema.styleSchema).sort()).toEqual([
			'bold',
			'code',
			'italic',
			'strike',
			'underline',
		])
	})
})

describe('the mention menu', () => {
	it('lists customers, projects and quotes with their kind', () => {
		const items = mentionItems(editor(), [
			{ kind: 'customer', id: 'c1', label: 'Jardins du Parc' },
			{ kind: 'quote', id: 'q1', label: 'DEV-1' },
		])
		expect(items.map((item) => [item.title, item.subtext])).toEqual([
			['Jardins du Parc', 'Client'],
			['DEV-1', 'Devis'],
		])
	})
})
