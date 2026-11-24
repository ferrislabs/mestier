import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	buildUpdateBody,
	canSave,
	dafToForm,
	formErrors,
	MAX_ATTACHMENTS,
} from '#/pages/daf/lib/daf-form'

const DAF: Schemas.DafResponse = {
	id: 'daf-1',
	project_id: 'project-1',
	number: 1,
	designation: 'Ganivelle',
	attachments: [],
	status: 'pending',
	issued_on: '2026-11-18',
	created_at: '2026-11-18T08:00:00Z',
	updated_at: '2026-11-18T09:00:00Z',
}

describe('dafToForm', () => {
	it('turns missing optional values into empty strings', () => {
		const form = dafToForm(DAF)
		expect(form.number).toBe('1')
		expect(form.priceRef).toBe('')
		expect(form.supplierName).toBe('')
		expect(form.projectOwner).toBe('')
		expect(form.photoKey).toBe('')
		expect(form.sentOn).toBe('')
	})

	it('flattens the supplier', () => {
		const form = dafToForm({
			...DAF,
			supplier: { name: 'Acme', phone: '0102' },
		})
		expect(form.supplierName).toBe('Acme')
		expect(form.supplierPhone).toBe('0102')
		expect(form.supplierEmail).toBe('')
	})
})

describe('buildUpdateBody', () => {
	it('sends empty strings as null and keeps the concurrency token', () => {
		const body = buildUpdateBody(dafToForm(DAF), DAF.updated_at)
		expect(body).toEqual({
			number: 1,
			designation: 'Ganivelle',
			price_ref: null,
			supplier: null,
			installer: null,
			specs: null,
			observations: null,
			photo_key: null,
			attachments: [],
			project_owner: null,
			project_manager: null,
			status: 'pending',
			sent_on: null,
			issued_on: '2026-11-18',
			returned_on: null,
			review_note: null,
			expected_updated_at: '2026-11-18T09:00:00Z',
		})
	})

	it('sends the overrides, supplier and status when filled', () => {
		const form = {
			...dafToForm(DAF),
			designation: '  Ganivelle châtaignier ',
			projectManager: ' Atelier Casa ',
			supplierName: 'Acme',
			supplierEmail: 'a@b.fr',
			status: 'accepted' as const,
			returnedOn: '2026-11-20',
			attachments: [
				{ key: 'k1', name: ' Fiche ' },
				{ key: 'k2', name: '   ' },
			],
		}
		const body = buildUpdateBody(form, DAF.updated_at)
		expect(body.designation).toBe('Ganivelle châtaignier')
		expect(body.project_manager).toBe('Atelier Casa')
		expect(body.project_owner).toBeNull()
		expect(body.supplier).toEqual({
			name: 'Acme',
			address: null,
			phone: null,
			email: 'a@b.fr',
		})
		expect(body.status).toBe('accepted')
		expect(body.returned_on).toBe('2026-11-20')
		expect(body.attachments).toEqual([
			{ key: 'k1', name: 'Fiche' },
			{ key: 'k2', name: 'Pièce jointe' },
		])
	})
})

describe('validation', () => {
	it('rejects a number out of range, an empty designation and too many attachments', () => {
		const base = dafToForm(DAF)
		expect(canSave(base)).toBe(true)
		expect(canSave({ ...base, number: '0' })).toBe(false)
		expect(canSave({ ...base, number: '1000' })).toBe(false)
		expect(canSave({ ...base, number: '' })).toBe(false)
		expect(canSave({ ...base, designation: '  ' })).toBe(false)
		expect(canSave({ ...base, issuedOn: '' })).toBe(false)
		const many = Array.from({ length: MAX_ATTACHMENTS + 1 }, (_, i) => ({
			key: `k${i}`,
			name: `f${i}`,
		}))
		expect(canSave({ ...base, attachments: many })).toBe(false)
	})

	it('asks for a supplier name once another supplier field is filled', () => {
		const form = { ...dafToForm(DAF), supplierPhone: '0102' }
		expect(formErrors(form)).toContain('Le nom du fournisseur est requis.')
		expect(canSave(form)).toBe(false)
	})
})
