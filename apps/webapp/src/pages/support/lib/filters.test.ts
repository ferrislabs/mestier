import { describe, expect, it } from 'vitest'
import {
	EMPTY_SUPPORT_FILTERS,
	hasActiveSupportFilters,
	supportFiltersToQuery,
	supportSearchSchema,
} from '#/pages/support/lib/filters'
import type { SupportFilters } from '#/pages/support/types'

function parse(search: unknown): SupportFilters {
	return supportSearchSchema.parse(search)
}

describe('supportSearchSchema', () => {
	it('reads a full filter set out of the search params', () => {
		expect(
			parse({
				assignee: 'member-1',
				badge: 'Urgent',
				custom_field_key: 'ville',
				custom_field_value: 'Lyon',
			}),
		).toEqual({
			assignee: 'member-1',
			badge: 'Urgent',
			custom_field_key: 'ville',
			custom_field_value: 'Lyon',
		})
	})

	it('reads an empty search as no filter at all', () => {
		expect(parse({})).toEqual(EMPTY_SUPPORT_FILTERS)
	})

	it('folds an empty or blank string to undefined and trims the rest', () => {
		expect(parse({ assignee: '' }).assignee).toBeUndefined()
		expect(parse({ badge: '   ' }).badge).toBeUndefined()
		expect(parse({ custom_field_key: '' }).custom_field_key).toBeUndefined()
		expect(parse({ custom_field_value: ' \t ' }).custom_field_value).toBe(
			undefined,
		)
		expect(parse({ badge: '  Urgent  ' }).badge).toBe('Urgent')
	})

	it('drops a value that is not a string', () => {
		expect(parse({ assignee: 42, badge: true, custom_field_key: {} })).toEqual(
			EMPTY_SUPPORT_FILTERS,
		)
		expect(parse({ custom_field_value: ['a'] }).custom_field_value).toBe(
			undefined,
		)
	})

	it('keeps the filters it can read when a neighbour is malformed', () => {
		expect(parse({ assignee: 'member-1', badge: 12 })).toEqual({
			...EMPTY_SUPPORT_FILTERS,
			assignee: 'member-1',
		})
	})

	it('never throws on garbage input', () => {
		expect(() => parse(null)).not.toThrow()
		expect(() => parse(undefined)).not.toThrow()
		expect(() => parse([])).not.toThrow()
		expect(() => parse(['assignee'])).not.toThrow()
		expect(() => parse('assignee=x')).not.toThrow()
		expect(() => parse(7)).not.toThrow()
		expect(parse(null)).toEqual(EMPTY_SUPPORT_FILTERS)
		expect(parse([])).toEqual(EMPTY_SUPPORT_FILTERS)
		expect(parse('assignee=x')).toEqual(EMPTY_SUPPORT_FILTERS)
	})
})

describe('EMPTY_SUPPORT_FILTERS', () => {
	it('carries every key, explicitly undefined', () => {
		expect(Object.keys(EMPTY_SUPPORT_FILTERS).sort()).toEqual([
			'assignee',
			'badge',
			'custom_field_key',
			'custom_field_value',
		])
		expect(Object.values(EMPTY_SUPPORT_FILTERS)).toEqual([
			undefined,
			undefined,
			undefined,
			undefined,
		])
	})
})

describe('hasActiveSupportFilters', () => {
	it('is false for the empty set', () => {
		expect(hasActiveSupportFilters(EMPTY_SUPPORT_FILTERS)).toBe(false)
	})

	it('is true for an assignee alone', () => {
		expect(
			hasActiveSupportFilters({ ...EMPTY_SUPPORT_FILTERS, assignee: 'm' }),
		).toBe(true)
	})

	it('is true for a badge alone', () => {
		expect(
			hasActiveSupportFilters({ ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' }),
		).toBe(true)
	})

	it('is false for a lone custom field key', () => {
		expect(
			hasActiveSupportFilters({
				...EMPTY_SUPPORT_FILTERS,
				custom_field_key: 'ville',
			}),
		).toBe(false)
	})

	it('is false for a lone custom field value', () => {
		expect(
			hasActiveSupportFilters({
				...EMPTY_SUPPORT_FILTERS,
				custom_field_value: 'Lyon',
			}),
		).toBe(false)
	})

	it('is true once the custom field pair is complete', () => {
		expect(
			hasActiveSupportFilters({
				...EMPTY_SUPPORT_FILTERS,
				custom_field_key: 'ville',
				custom_field_value: 'Lyon',
			}),
		).toBe(true)
	})

	it('stays true with a lone key beside an assignee', () => {
		expect(
			hasActiveSupportFilters({
				...EMPTY_SUPPORT_FILTERS,
				assignee: 'm',
				custom_field_key: 'ville',
			}),
		).toBe(true)
	})
})

describe('supportFiltersToQuery', () => {
	it('sends nothing but the page size when no filter is set', () => {
		expect(supportFiltersToQuery(EMPTY_SUPPORT_FILTERS, 100)).toEqual({
			per_page: 100,
		})
	})

	it('emits only the filters that are set', () => {
		const query = supportFiltersToQuery(
			{ ...EMPTY_SUPPORT_FILTERS, assignee: 'member-1', badge: 'Urgent' },
			50,
		)

		expect(query).toEqual({
			per_page: 50,
			assignee: 'member-1',
			badge: 'Urgent',
		})
		expect(Object.keys(query).sort()).toEqual(['assignee', 'badge', 'per_page'])
	})

	it('sends the custom field pair when both halves are set', () => {
		expect(
			supportFiltersToQuery(
				{
					...EMPTY_SUPPORT_FILTERS,
					custom_field_key: 'ville',
					custom_field_value: 'Lyon',
				},
				100,
			),
		).toEqual({
			per_page: 100,
			custom_field_key: 'ville',
			custom_field_value: 'Lyon',
		})
	})

	it('never sends half of the custom field pair', () => {
		expect(
			supportFiltersToQuery(
				{ ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ville' },
				100,
			),
		).toEqual({ per_page: 100 })
		expect(
			supportFiltersToQuery(
				{ ...EMPTY_SUPPORT_FILTERS, custom_field_value: 'Lyon' },
				100,
			),
		).toEqual({ per_page: 100 })
	})

	it('distinguishes one assignee from another', () => {
		expect(
			supportFiltersToQuery({ ...EMPTY_SUPPORT_FILTERS, assignee: 'a' }, 100),
		).not.toEqual(
			supportFiltersToQuery({ ...EMPTY_SUPPORT_FILTERS, assignee: 'b' }, 100),
		)
	})
})
