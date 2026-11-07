import { describe, expect, it } from 'vitest'
import { dealBodyOfDraft } from '#/pages/deals/lib/deal-api'
import {
	buildDraft,
	type DealForm,
	emptyForm,
	formFromDeal,
	isFormChanged,
} from '#/pages/deals/lib/deal-form'
import { deal } from '#/pages/deals/lib/testing'

const PEOPLE = [
	{ id: 'julien', name: 'Julien', hue: 10 },
	{ id: 'baptiste', name: 'Baptiste', hue: 200 },
]

const valid: DealForm = {
	customerId: 'c8',
	title: '  Bureaux  ',
	amount: '12 500,50',
	expectedCloseDate: '2026-12-31',
	ownerId: 'julien',
	source: 'salon',
	notes: ' à rappeler ',
}

describe('emptyForm', () => {
	it('is owned by the current person and has no amount', () => {
		expect(emptyForm('baptiste')).toEqual({
			customerId: '',
			title: '',
			amount: '',
			expectedCloseDate: '',
			ownerId: 'baptiste',
			source: '',
			notes: '',
		})
	})
})

describe('buildDraft', () => {
	it('turns a valid form into a draft in cents', () => {
		const result = buildDraft(valid, PEOPLE)
		expect(result.errors).toEqual({})
		expect(result.draft).toEqual({
			customerId: 'c8',
			title: 'Bureaux',
			amount: 1250050,
			expectedCloseDate: '2026-12-31',
			ownerId: 'julien',
			source: 'salon',
			notes: 'à rappeler',
		})
	})

	it('falls back to null for optional fields and zero for an empty amount', () => {
		const result = buildDraft(
			{
				...valid,
				amount: '',
				expectedCloseDate: '',
				ownerId: '',
				source: '',
			},
			PEOPLE,
		)
		expect(result.draft).toMatchObject({
			amount: 0,
			expectedCloseDate: null,
			ownerId: null,
			source: null,
		})
	})

	it('ignores unknown owners and sources', () => {
		const result = buildDraft(
			{ ...valid, ownerId: 'inconnu', source: 'x' },
			PEOPLE,
		)
		expect(result.draft?.ownerId).toBeNull()
		expect(result.draft?.source).toBeNull()
	})

	it('attaches messages to their field', () => {
		const result = buildDraft({ ...valid, customerId: '', title: ' ' }, PEOPLE)
		expect(result.draft).toBeNull()
		expect(result.errors.title).toBe('L’intitulé est obligatoire.')
		expect(result.errors.customerId).toBe('Choisis un client.')
		expect(result.errors.amount).toBeUndefined()
	})

	it('requires a readable and positive amount', () => {
		const unreadable = buildDraft({ ...valid, amount: 'beaucoup' }, PEOPLE)
		expect(unreadable.draft).toBeNull()
		expect(unreadable.errors.amount).toBe(
			'Saisis un montant valide, par exemple 12 500,00.',
		)
		const negative = buildDraft({ ...valid, amount: '-5' }, PEOPLE)
		expect(negative.draft).toBeNull()
		expect(negative.errors.amount).toBe('Le montant estimé doit être positif.')
	})

	it('requires a real close date', () => {
		const result = buildDraft(
			{ ...valid, expectedCloseDate: '2026-02-30' },
			PEOPLE,
		)
		expect(result.draft).toBeNull()
		expect(result.errors.expectedCloseDate).toBe('Saisis une date valide.')
	})
})

describe('formFromDeal', () => {
	it('round-trips a deal through the form', () => {
		const source = deal({
			amount: 198900,
			ownerId: 'julien',
			source: 'site_web',
			expectedCloseDate: '2026-11-20',
			notes: 'Rappeler',
		})
		const form = formFromDeal(source)
		expect(form.amount).toBe('1989')
		expect(isFormChanged(form, source)).toBe(false)
		expect(isFormChanged({ ...form, title: `${form.title} 2` }, source)).toBe(
			true,
		)
		const draft = buildDraft(form, PEOPLE).draft
		expect(draft).not.toBeNull()
		if (!draft) return
		expect(dealBodyOfDraft(draft)).toEqual({
			customer_id: source.customerId,
			title: source.title,
			amount_cents: 198900,
			expected_close_date: '2026-11-20',
			owner_member_id: 'julien',
			source: 'site_web',
			notes: 'Rappeler',
		})
	})
})

describe('the API body of a built draft', () => {
	it('sends integer cents', () => {
		const draft = buildDraft(valid, PEOPLE).draft
		expect(draft).not.toBeNull()
		if (!draft) return
		expect(dealBodyOfDraft(draft)).toMatchObject({
			amount_cents: 1250050,
			title: 'Bureaux',
		})
	})
})
