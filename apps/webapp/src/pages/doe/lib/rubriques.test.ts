import { describe, expect, it } from 'vitest'
import {
	INFO_TAB,
	infoSummary,
	PLAN_TAB,
	planSummary,
	resolveTab,
	rubriques,
	tabSearch,
	validateDoeSearch,
} from '#/pages/doe/lib/rubriques'
import type { DoeForm, DoeSection } from '#/pages/doe/types'

function section(id: string, enabled: boolean): DoeSection {
	return {
		id,
		title: `Thème ${id}`,
		enabled,
		content: { kind: 'blank_pages', count: 2 },
	}
}

function form(sections: DoeSection[]): DoeForm {
	return {
		title: 'Chantier',
		place: '',
		lot: '',
		issuedOn: '2026-01-01',
		author: '',
		revision: 'A',
		coverPhotoKey: null,
		parties: [
			{
				role: 'project_owner',
				name: 'Mairie',
				address: '',
				phone: '',
				email: '',
				logoKey: null,
				contactName: '',
				contactRole: '',
				contactEmail: '',
				contactPhone: '',
			},
			{
				role: 'contractor',
				name: ' ',
				address: '',
				phone: '',
				email: '',
				logoKey: null,
				contactName: '',
				contactRole: '',
				contactEmail: '',
				contactPhone: '',
			},
		],
		sections,
	}
}

describe('rubriques', () => {
	it('lists the fixed rubriques then the enabled themes, numbered', () => {
		const items = rubriques(form([section('a', true), section('b', false)]))

		expect(items.map((item) => item.id)).toEqual([INFO_TAB, PLAN_TAB, 'a'])
		expect(items[2]?.label).toBe('1. Thème a')
		expect(items[2]?.summary).toBe('2 pages libres')
	})

	it('summarises the parties and the plan', () => {
		expect(infoSummary(form([]))).toBe('1 intervenant renseigné')
		expect(planSummary([section('a', true), section('b', false)])).toBe(
			'1 sur 2 thèmes inclus',
		)
	})

	it('falls back to the information rubrique for a disabled or unknown tab', () => {
		const sections = [section('a', true), section('b', false)]

		expect(resolveTab('a', sections)).toBe('a')
		expect(resolveTab('b', sections)).toBe(INFO_TAB)
		expect(resolveTab('zzz', sections)).toBe(INFO_TAB)
		expect(resolveTab(undefined, sections)).toBe(INFO_TAB)
		expect(resolveTab(PLAN_TAB, sections)).toBe(PLAN_TAB)
	})

	it('keeps the search minimal', () => {
		expect(tabSearch(INFO_TAB)).toEqual({})
		expect(tabSearch('a')).toEqual({ onglet: 'a' })
		expect(validateDoeSearch({ onglet: ' ' })).toEqual({})
		expect(validateDoeSearch({ onglet: 3 })).toEqual({ onglet: '3' })
	})
})
