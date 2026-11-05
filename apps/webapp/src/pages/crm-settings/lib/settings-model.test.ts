import { describe, expect, it } from 'vitest'
import {
	billingFormOf,
	companyErrors,
	evaluateBilling,
	evaluateResume,
	formatNumber,
	raiseSequence,
	sequenceRows,
	validateSecret,
} from '#/pages/crm-settings/lib/settings-model'
import type { Company, CrmSettingsState } from '#/pages/crm-settings/types'

const company: Company = {
	name: 'Duval',
	legalForm: '',
	capital: '',
	legalId: '823 456 789 00019',
	registry: '',
	vatNumber: 'FR12823456789',
	address: {
		street: '1 rue X',
		complement: '',
		postalCode: '44000',
		city: 'Nantes',
		country: 'France',
	},
	email: '',
	phone: '',
	iban: '',
	bic: '',
	insurance: '',
}

const state: CrmSettingsState = {
	settings: {
		company,
		paymentTermsDays: 30,
		quoteValidityDays: 30,
		defaultVatRate: 20,
		latePenalty: 'trois fois le taux légal',
		recoveryFee: 4000,
		invoiceFooter: '',
		numberPadding: 3,
		stripeEndpoint: 'https://example.test/hook',
		stripeSecretSet: true,
	},
	sequences: { quote: { '2026': 9 }, invoice: {}, creditNote: {} },
}

describe('companyErrors', () => {
	it('accepts a complete company', () => {
		expect(companyErrors(company)).toEqual({ fields: {}, other: [] })
	})

	it('points at the offending fields', () => {
		const errors = companyErrors({
			...company,
			name: ' ',
			legalId: '123',
			vatNumber: 'DE1',
			address: { ...company.address, street: '' },
		})
		expect(Object.keys(errors.fields).sort()).toEqual([
			'legalId',
			'name',
			'street',
			'vatNumber',
		])
	})
})

describe('evaluateBilling', () => {
	it('turns the form into a patch', () => {
		const evaluation = evaluateBilling(billingFormOf(state.settings))
		expect(evaluation.errors).toEqual({})
		expect(evaluation.patch?.recoveryFee).toBe(4000)
		expect(evaluation.patch?.paymentTermsDays).toBe(30)
	})

	it('reports every invalid field and no patch', () => {
		const evaluation = evaluateBilling({
			...billingFormOf(state.settings),
			paymentTermsDays: '90',
			quoteValidityDays: 'x',
			recoveryFee: 'abc',
			latePenalty: ' ',
			numberPadding: '9',
		})
		expect(evaluation.patch).toBeNull()
		expect(Object.keys(evaluation.errors).sort()).toEqual([
			'latePenalty',
			'numberPadding',
			'paymentTermsDays',
			'quoteValidityDays',
			'recoveryFee',
		])
	})
})

describe('numbering', () => {
	it('formats numbers and lists the next one per series', () => {
		expect(formatNumber('invoice', 2026, 7, 3)).toBe('F-2026-007')
		const rows = sequenceRows(state, 2026)
		expect(rows.find((row) => row.kind === 'quote')).toMatchObject({
			lastNumber: 'D-2026-009',
			next: 'D-2026-010',
		})
		expect(rows.find((row) => row.kind === 'creditNote')?.lastNumber).toBeNull()
	})

	it('only lets a counter move forward', () => {
		const quote = sequenceRows(state, 2026)[0]
		if (!quote) throw new Error('missing row')
		expect(evaluateResume(quote, '', 3)).toEqual({
			error: null,
			last: null,
			preview: null,
		})
		expect(evaluateResume(quote, 'abc', 3).error).toBeDefined()
		expect(evaluateResume(quote, '10', 3).error).toContain('D-2026-010')
		expect(evaluateResume(quote, '120', 3)).toEqual({
			error: null,
			last: 119,
			preview: 'D-2026-120',
		})
		expect(raiseSequence(state.sequences, 'quote', 2026, 5)).toBe(
			state.sequences,
		)
		expect(
			raiseSequence(state.sequences, 'quote', 2026, 119).quote['2026'],
		).toBe(119)
	})
})

describe('validateSecret', () => {
	it('requires the whsec_ prefix', () => {
		expect(validateSecret(' ')).toBe('Saisis le secret de signature.')
		expect(validateSecret('sk_live_1')).toContain('whsec_')
		expect(validateSecret('whsec_')).not.toBeNull()
		expect(validateSecret('whsec_abc')).toBeNull()
	})
})
