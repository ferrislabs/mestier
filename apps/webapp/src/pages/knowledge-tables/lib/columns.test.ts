import { describe, expect, it } from 'vitest'
import {
	COLUMN_KINDS,
	columnOptions,
	conversionTargets,
	defaultConfig,
	filterOperators,
	isSortable,
	nextOptionColor,
	operatorLabel,
	operatorNeedsValue,
	SELECT_COLORS,
} from '#/pages/knowledge-tables/lib/columns'
import type {
	ColumnKind,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'

function column(kind: ColumnKind, overrides: Partial<TableColumn> = {}) {
	return {
		id: `c-${kind}`,
		name: kind,
		kind,
		config: {},
		is_primary: false,
		position: 1,
		...overrides,
	} satisfies TableColumn
}

describe('COLUMN_KINDS', () => {
	it('names the eleven kinds of the spec in French', () => {
		expect(COLUMN_KINDS.map((entry) => entry.kind)).toEqual([
			'text',
			'long_text',
			'number',
			'currency',
			'date',
			'checkbox',
			'single_select',
			'multi_select',
			'link',
			'file',
			'member',
		])
		expect(COLUMN_KINDS.find((entry) => entry.kind === 'currency')?.label).toBe(
			'Montant',
		)
	})
})

describe('filterOperators', () => {
	it('gives text contains, is, empty and not empty', () => {
		expect(filterOperators('text')).toEqual([
			'contains',
			'is',
			'is_empty',
			'is_not_empty',
		])
		expect(filterOperators('long_text')).toEqual(filterOperators('text'))
	})

	it('gives numbers and amounts the six comparisons and emptiness', () => {
		const expected = [
			'eq',
			'ne',
			'lt',
			'le',
			'gt',
			'ge',
			'is_empty',
			'is_not_empty',
		]
		expect(filterOperators('number')).toEqual(expected)
		expect(filterOperators('currency')).toEqual(expected)
	})

	it('gives dates before, after, on and emptiness', () => {
		expect(filterOperators('date')).toEqual([
			'before',
			'after',
			'on',
			'is_empty',
			'is_not_empty',
		])
	})

	it('gives a checkbox checked and unchecked only', () => {
		expect(filterOperators('checkbox')).toEqual(['checked', 'unchecked'])
	})

	it('gives choices their own operators', () => {
		expect(filterOperators('single_select')).toEqual([
			'is',
			'is_not',
			'any_of',
			'is_empty',
		])
		expect(filterOperators('multi_select')).toEqual([
			'any_of',
			'all_of',
			'is_empty',
		])
	})

	it('gives links, files and members is and emptiness', () => {
		for (const kind of ['link', 'file', 'member'] as const) {
			expect(filterOperators(kind)).toEqual(['is', 'is_empty', 'is_not_empty'])
		}
	})
})

describe('operatorLabel', () => {
	it('says any_of differently for a single and a multiple choice', () => {
		expect(operatorLabel('single_select', 'any_of')).toBe('parmi')
		expect(operatorLabel('multi_select', 'any_of')).toBe('contient l’un de')
	})

	it('writes the comparisons as symbols', () => {
		expect(operatorLabel('number', 'le')).toBe('≤')
		expect(operatorLabel('currency', 'ne')).toBe('≠')
	})
})

describe('operatorNeedsValue', () => {
	it('is false for emptiness and checkbox operators', () => {
		expect(operatorNeedsValue('is_empty')).toBe(false)
		expect(operatorNeedsValue('is_not_empty')).toBe(false)
		expect(operatorNeedsValue('checked')).toBe(false)
		expect(operatorNeedsValue('unchecked')).toBe(false)
		expect(operatorNeedsValue('contains')).toBe(true)
		expect(operatorNeedsValue('any_of')).toBe(true)
	})
})

describe('isSortable', () => {
	it('refuses links, files and members', () => {
		expect(isSortable('link')).toBe(false)
		expect(isSortable('file')).toBe(false)
		expect(isSortable('member')).toBe(false)
		expect(isSortable('text')).toBe(true)
		expect(isSortable('single_select')).toBe(true)
	})
})

describe('conversionTargets', () => {
	it('lets text and long text swap', () => {
		expect(conversionTargets(column('text'))).toEqual(['long_text'])
		expect(conversionTargets(column('long_text'))).toEqual(['text'])
	})

	it('lets any other kind become text or long text only', () => {
		for (const kind of [
			'number',
			'currency',
			'date',
			'checkbox',
			'link',
			'file',
			'member',
		] as const) {
			expect(conversionTargets(column(kind))).toEqual(['text', 'long_text'])
		}
	})

	it('lets the choices swap', () => {
		expect(conversionTargets(column('single_select'))).toEqual([
			'text',
			'long_text',
			'multi_select',
		])
		expect(conversionTargets(column('multi_select'))).toEqual([
			'text',
			'long_text',
			'single_select',
		])
	})

	it('never changes the primary column', () => {
		expect(conversionTargets(column('text', { is_primary: true }))).toEqual([])
	})
})

describe('columnOptions', () => {
	it('keeps the options that have an id, gray when uncolored', () => {
		const status = column('single_select', {
			config: {
				options: [
					{ id: 'o1', label: 'En cours', color: 'blue' },
					{ id: 'o2', label: 'Fini' },
					{ label: 'Sans id' },
				],
			},
		})
		expect(columnOptions(status)).toEqual([
			{ id: 'o1', label: 'En cours', color: 'blue' },
			{ id: 'o2', label: 'Fini', color: 'gray' },
		])
	})

	it('is empty for a column without options', () => {
		expect(columnOptions(column('text'))).toEqual([])
	})
})

describe('nextOptionColor', () => {
	it('cycles through the colors', () => {
		expect(nextOptionColor([])).toBe(SELECT_COLORS[0])
		expect(nextOptionColor([{ color: 'gray' }, { color: 'red' }])).toBe(
			SELECT_COLORS[2],
		)
	})
})

describe('defaultConfig', () => {
	it('gives a link column its kind and a number its decimals', () => {
		expect(defaultConfig('link')).toEqual({ link_kind: 'customer' })
		expect(defaultConfig('number')).toEqual({ decimals: 0 })
		expect(defaultConfig('single_select')).toEqual({ options: [] })
		expect(defaultConfig('text')).toEqual({})
	})
})
