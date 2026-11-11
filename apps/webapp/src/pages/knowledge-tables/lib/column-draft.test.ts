import { describe, expect, it } from 'vitest'
import {
	addOption,
	columnDraftError,
	columnUpdate,
	configFromDraft,
	draftFromColumn,
	EMPTY_COLUMN_DRAFT,
	kindChoices,
} from '#/pages/knowledge-tables/lib/column-draft'
import type { TableColumn } from '#/pages/knowledge-tables/lib/types'

const status: TableColumn = {
	id: 'status',
	name: 'Statut',
	kind: 'single_select',
	config: {
		options: [
			{ id: 'o1', label: 'En cours', color: 'blue' },
			{ id: 'o2', label: 'Fini', color: 'green' },
		],
	},
	is_primary: false,
	position: 2,
}

describe('draftFromColumn', () => {
	it('reads the name, the kind and the options of a column', () => {
		const draft = draftFromColumn(status)
		expect(draft.name).toBe('Statut')
		expect(draft.kind).toBe('single_select')
		expect(
			draft.options.map((option) => [option.id, option.label, option.color]),
		).toEqual([
			['o1', 'En cours', 'blue'],
			['o2', 'Fini', 'green'],
		])
	})
})

describe('configFromDraft', () => {
	it('sends the decimals of a number', () => {
		expect(
			configFromDraft({ ...EMPTY_COLUMN_DRAFT, kind: 'number', decimals: 2 }),
		).toEqual({ decimals: 2 })
	})

	it('sends the kind of a link', () => {
		expect(
			configFromDraft({
				...EMPTY_COLUMN_DRAFT,
				kind: 'link',
				linkKind: 'quote',
			}),
		).toEqual({ link_kind: 'quote' })
	})

	it('sends the options of a choice, new ones without id', () => {
		const draft = addOption(draftFromColumn(status), 'Annulé')
		expect(configFromDraft(draft)).toEqual({
			options: [
				{ id: 'o1', label: 'En cours', color: 'blue' },
				{ id: 'o2', label: 'Fini', color: 'green' },
				{ label: 'Annulé', color: 'orange' },
			],
		})
	})

	it('sends nothing for a plain text', () => {
		expect(configFromDraft({ ...EMPTY_COLUMN_DRAFT, kind: 'text' })).toEqual({})
	})
})

describe('columnUpdate', () => {
	it('sends only what changed', () => {
		expect(
			columnUpdate(status, { ...draftFromColumn(status), name: 'État' }),
		).toEqual({ name: 'État' })
		expect(columnUpdate(status, draftFromColumn(status))).toEqual({})
	})

	it('sends the new kind and its config', () => {
		expect(
			columnUpdate(status, {
				...draftFromColumn(status),
				kind: 'multi_select',
			}),
		).toEqual({
			kind: 'multi_select',
			config: {
				options: [
					{ id: 'o1', label: 'En cours', color: 'blue' },
					{ id: 'o2', label: 'Fini', color: 'green' },
				],
			},
		})
	})

	it('sends the options when one is renamed', () => {
		const draft = draftFromColumn(status)
		const renamed = {
			...draft,
			options: draft.options.map((option) =>
				option.id === 'o2' ? { ...option, label: 'Terminé' } : option,
			),
		}
		expect(columnUpdate(status, renamed)).toEqual({
			config: {
				options: [
					{ id: 'o1', label: 'En cours', color: 'blue' },
					{ id: 'o2', label: 'Terminé', color: 'green' },
				],
			},
		})
	})
})

describe('columnDraftError', () => {
	it('needs a name', () => {
		expect(columnDraftError({ ...EMPTY_COLUMN_DRAFT, name: ' ' })).toBe(
			'Donne un nom à la colonne.',
		)
	})

	it('needs a label on every option and no two alike', () => {
		const draft = { ...draftFromColumn(status) }
		expect(columnDraftError(addOption(draft, ''))).toBe(
			'Chaque option a besoin d’un libellé.',
		)
		expect(columnDraftError(addOption(draft, 'fini'))).toBe(
			'Deux options portent le même libellé.',
		)
		expect(columnDraftError(draft)).toBeNull()
	})
})

describe('kindChoices', () => {
	it('offers every kind at creation', () => {
		expect(kindChoices(null)).toHaveLength(11)
	})

	it('offers only the lossless conversions of an existing column', () => {
		expect(kindChoices(status)).toEqual([
			'single_select',
			'text',
			'long_text',
			'multi_select',
		])
		expect(kindChoices({ ...status, kind: 'text', is_primary: true })).toEqual([
			'text',
		])
	})
})
