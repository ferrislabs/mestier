import { describe, expect, it } from 'vitest'
import {
	buildUpdatePayload,
	canSave,
	doeToForm,
	isRevisionValid,
} from '#/pages/doe/lib/payload'
import type { Doe, DoeForm } from '#/pages/doe/types'

const DOE: Doe = {
	id: 'd1',
	project_id: 'p1',
	title: 'Avenue du 8 mai',
	place: null,
	lot: 'Lot 3',
	issued_on: '2026-10-01',
	author: null,
	revision: 'A',
	parties: [
		{ role: 'contractor', name: 'Treyve Paysages', phone: '0102' },
		{ role: 'project_owner', name: 'Commune', address: 'Mairie' },
	],
	sections: [],
	created_at: '2026-10-01T00:00:00Z',
	updated_at: '2026-10-01T00:00:00Z',
}

describe('doeToForm', () => {
	it('always yields the three roles in order, blank when absent', () => {
		const form = doeToForm(DOE)
		expect(form.parties.map((party) => party.role)).toEqual([
			'project_owner',
			'project_manager',
			'contractor',
		])
		expect(form.parties[1]?.name).toBe('')
		expect(form.parties[0]?.address).toBe('Mairie')
		expect(form.place).toBe('')
	})
})

describe('buildUpdatePayload', () => {
	it('drops blank parties and trims text', () => {
		const form: DoeForm = {
			...doeToForm(DOE),
			title: '  Avenue  ',
			place: '   ',
			revision: ' B ',
		}
		const names = ['  Commune ', '   ', 'Treyve']
		form.parties = form.parties.map((party, index) => ({
			...party,
			name: names[index] ?? '',
			phone: index === 0 ? ' 01 ' : party.phone,
		}))
		const payload = buildUpdatePayload(form)
		expect(payload.title).toBe('Avenue')
		expect(payload.place).toBeNull()
		expect(payload.revision).toBe('B')
		expect(payload.parties.map((party) => party.name)).toEqual([
			'Commune',
			'Treyve',
		])
		expect(payload.parties[0]?.phone).toBe('01')
		expect(payload.parties[0]?.email).toBeNull()
	})
})

describe('revision and save guard', () => {
	it('accepts 1 to 8 characters', () => {
		expect(isRevisionValid('A')).toBe(true)
		expect(isRevisionValid('12345678')).toBe(true)
		expect(isRevisionValid('123456789')).toBe(false)
		expect(isRevisionValid('  ')).toBe(false)
	})

	it('refuses a blank title', () => {
		const form = doeToForm(DOE)
		expect(canSave(form)).toBe(true)
		expect(canSave({ ...form, title: ' ' })).toBe(false)
	})
})
