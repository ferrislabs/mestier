import { describe, expect, it } from 'vitest'
import {
	cellEditText,
	cellLabel,
	displayCell,
	invalidCellMessage,
	isEmptyValue,
	linkValue,
	parseCellText,
	referenceId,
	referenceValue,
	rowTitle,
	UNTITLED,
} from '#/pages/knowledge-tables/lib/cells'
import type {
	ColumnKind,
	TableColumn,
	TableRow,
} from '#/pages/knowledge-tables/lib/types'

const spaces = (text: string) => text.replace(/\s/g, ' ')

function column(kind: ColumnKind, overrides: Partial<TableColumn> = {}) {
	return {
		id: 'c1',
		name: 'Colonne',
		kind,
		config: {},
		is_primary: false,
		position: 1,
		...overrides,
	} satisfies TableColumn
}

function row(
	values: Record<string, unknown>,
	labels: Record<string, unknown> = {},
): TableRow {
	return {
		id: 'r1',
		values,
		labels,
		created_at: '2026-11-11T08:00:00Z',
		updated_at: '2026-11-11T08:00:00Z',
	}
}

const status = column('single_select', {
	config: {
		options: [
			{ id: 'o1', label: 'En cours', color: 'blue' },
			{ id: 'o2', label: 'Fini', color: 'green' },
		],
	},
})

describe('isEmptyValue', () => {
	it('treats absent, null, blank text and empty lists as empty', () => {
		expect(isEmptyValue(undefined)).toBe(true)
		expect(isEmptyValue(null)).toBe(true)
		expect(isEmptyValue('  ')).toBe(true)
		expect(isEmptyValue([])).toBe(true)
		expect(isEmptyValue(0)).toBe(false)
		expect(isEmptyValue(false)).toBe(false)
	})
})

describe('cellLabel', () => {
	it('reads the resolved label of a reference cell', () => {
		expect(
			cellLabel(row({}, { c1: { label: 'Dupont', missing: false } }), 'c1'),
		).toEqual({ label: 'Dupont', missing: false })
		expect(cellLabel(row({}, { c1: { missing: true } }), 'c1')).toEqual({
			label: null,
			missing: true,
		})
		expect(cellLabel(row({}), 'c1')).toBeNull()
	})
})

describe('displayCell', () => {
	it('shows a number with the decimals of its column', () => {
		expect(
			spaces(
				displayCell(
					column('number', { config: { decimals: 2 } }),
					1250.5,
					null,
				),
			),
		).toBe('1 250,50')
	})

	it('shows an amount in euros', () => {
		expect(spaces(displayCell(column('currency'), 125050, null))).toBe(
			'1 250,50 €',
		)
	})

	it('shows a date as JJ/MM/AAAA', () => {
		expect(displayCell(column('date'), '2026-11-05', null)).toBe('05/11/2026')
	})

	it('shows a checkbox as Oui or Non', () => {
		expect(displayCell(column('checkbox'), true, null)).toBe('Oui')
		expect(displayCell(column('checkbox'), false, null)).toBe('Non')
	})

	it('shows choices by their label', () => {
		expect(displayCell(status, 'o2', null)).toBe('Fini')
		expect(
			displayCell({ ...status, kind: 'multi_select' }, ['o1', 'o2'], null),
		).toBe('En cours, Fini')
	})

	it('shows a reference by its resolved name, or as deleted', () => {
		const link = column('link', { config: { link_kind: 'customer' } })
		expect(
			displayCell(
				link,
				{ kind: 'customer', id: 'x' },
				{
					label: 'Dupont',
					missing: false,
				},
			),
		).toBe('Dupont')
		expect(
			displayCell(
				link,
				{ kind: 'customer', id: 'x' },
				{
					label: null,
					missing: true,
				},
			),
		).toBe('Élément supprimé')
		expect(
			displayCell(column('member'), 'u1', { label: null, missing: true }),
		).toBe('Élément supprimé')
	})

	it('shows nothing for an empty cell', () => {
		expect(displayCell(column('number'), undefined, null)).toBe('')
		expect(displayCell(column('text'), null, null)).toBe('')
	})
})

describe('rowTitle', () => {
	const primary = column('text', { id: 'p', is_primary: true })

	it('is the primary value', () => {
		expect(rowTitle(row({ p: 'Chantier Martin' }), primary)).toBe(
			'Chantier Martin',
		)
	})

	it('is Sans titre when the primary cell is empty', () => {
		expect(rowTitle(row({}), primary)).toBe(UNTITLED)
		expect(rowTitle(row({ p: '  ' }), primary)).toBe('Sans titre')
		expect(rowTitle(row({ p: 'x' }), null)).toBe('Sans titre')
	})
})

describe('cellEditText', () => {
	it('writes the value the way it is typed', () => {
		expect(cellEditText(column('number'), 12.5)).toBe('12,5')
		expect(cellEditText(column('currency'), 125050)).toBe('1250,50')
		expect(cellEditText(column('currency'), 1200)).toBe('12')
		expect(cellEditText(column('date'), '2026-11-05')).toBe('05/11/2026')
		expect(cellEditText(column('text'), 'abc')).toBe('abc')
		expect(cellEditText(column('text'), undefined)).toBe('')
	})
})

describe('parseCellText', () => {
	it('reads text, empty as a cleared cell', () => {
		expect(parseCellText('text', 'Bonjour')).toEqual({
			kind: 'value',
			value: 'Bonjour',
		})
		expect(parseCellText('text', '')).toEqual({ kind: 'empty' })
	})

	it('refuses a text longer than its column allows', () => {
		expect(parseCellText('text', 'a'.repeat(1001))).toEqual({
			kind: 'invalid',
		})
		expect(parseCellText('long_text', 'a'.repeat(1001)).kind).toBe('value')
		expect(parseCellText('long_text', 'a'.repeat(20001))).toEqual({
			kind: 'invalid',
		})
	})

	it('reads numbers, amounts and dates the French way', () => {
		expect(parseCellText('number', '1 250,5')).toEqual({
			kind: 'value',
			value: 1250.5,
		})
		expect(parseCellText('currency', '12,30')).toEqual({
			kind: 'value',
			value: 1230,
		})
		expect(parseCellText('date', '05/11/2026')).toEqual({
			kind: 'value',
			value: '2026-11-05',
		})
		expect(parseCellText('date', '31/02/2026')).toEqual({ kind: 'invalid' })
	})
})

describe('invalidCellMessage', () => {
	it('explains what each kind expects', () => {
		expect(invalidCellMessage('number')).toBe(
			'Saisis un nombre, par exemple 12,5.',
		)
		expect(invalidCellMessage('date')).toBe(
			'Saisis une date au format JJ/MM/AAAA.',
		)
	})
})

describe('linkValue', () => {
	it('reads a stored link', () => {
		expect(linkValue({ kind: 'project', id: 'p1' })).toEqual({
			kind: 'project',
			id: 'p1',
		})
		expect(linkValue('p1')).toBeNull()
		expect(linkValue({ kind: 'nope', id: 'p1' })).toBeNull()
	})
})

describe('referenceId and referenceValue', () => {
	const link = column('link', { config: { link_kind: 'quote' } })

	it('reads the id of a link, a file or a member', () => {
		expect(referenceId(link, { kind: 'quote', id: 'q1' })).toBe('q1')
		expect(referenceId(column('member'), 'u1')).toBe('u1')
		expect(referenceId(column('file'), undefined)).toBeNull()
	})

	it('writes a link with the kind of its column', () => {
		expect(referenceValue(link, 'q1')).toEqual({ kind: 'quote', id: 'q1' })
		expect(referenceValue(column('file'), 'f1')).toBe('f1')
		expect(referenceValue(link, null)).toBeNull()
	})
})
