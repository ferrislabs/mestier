import { describe, expect, it } from 'vitest'
import {
	billingFormOf,
	companyErrors,
	evaluateBilling,
	evaluateResume,
	formatNumber,
	type SequenceRow,
	validateSecret,
} from '#/pages/crm-settings/lib/settings-model'
import type { Company, Settings } from '#/pages/crm-settings/types'

const company: Company = {
	name: 'Duval',
	legalForm: '',
	capital: '',
	legalId: '823 456 789 00019',
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

const settings: Settings = {
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
}

const quoteRow: SequenceRow = {
	kind: 'quote',
	label: 'Devis',
	prefix: 'DEV',
	year: 2026,
	last: 9,
	lastNumber: 'DEV-2026-009',
	next: 'DEV-2026-010',
}

describe('companyErrors', () => {
	it('accepts a complete company', () => {
		expect(companyErrors(company)).toEqual({ fields: {}, other: [] })
	})

	it('refuses a capital that is not an amount', () => {
		const errors = companyErrors({ ...company, capital: 'beaucoup' })
		expect(Object.keys(errors.fields)).toEqual(['capital'])
		expect(companyErrors({ ...company, capital: '50 000' }).fields).toEqual({})
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
		const evaluation = evaluateBilling(billingFormOf(settings))
		expect(evaluation.errors).toEqual({})
		expect(evaluation.patch?.recoveryFee).toBe(4000)
		expect(evaluation.patch?.paymentTermsDays).toBe(30)
	})

	it('reports every invalid field and no patch', () => {
		const evaluation = evaluateBilling({
			...billingFormOf(settings),
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
	it('formats a number from the prefix of the series', () => {
		expect(formatNumber('FAC', 2026, 7, 3)).toBe('FAC-2026-007')
		expect(formatNumber('AV', 2026, 7, 5)).toBe('AV-2026-00007')
		expect(formatNumber('DEV', 2026, 1234, 2)).toBe('DEV-2026-1234')
	})

	it('only lets a counter move forward', () => {
		expect(evaluateResume(quoteRow, '', 3)).toEqual({
			error: null,
			nextNumber: null,
			preview: null,
		})
		expect(evaluateResume(quoteRow, 'abc', 3).error).toBeDefined()
		expect(evaluateResume(quoteRow, '10', 3).error).toContain('DEV-2026-010')
		expect(evaluateResume(quoteRow, '120', 3)).toEqual({
			error: null,
			nextNumber: 120,
			preview: 'DEV-2026-120',
		})
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
