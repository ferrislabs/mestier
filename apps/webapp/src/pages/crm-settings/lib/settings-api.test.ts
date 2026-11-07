import { describe, expect, it } from 'vitest'
import type { Organization } from '#/hooks/use-organizations'
import {
	basisPointsOfVatRate,
	billingBody,
	billingOf,
	companyOf,
	legalIdentityBody,
	sequenceRowsOf,
	settingsOf,
	vatRateOfBasisPoints,
} from '#/pages/crm-settings/lib/settings-api'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Duval',
	slug: 'duval',
	owner_id: 'user-1',
	legal_name: 'Duval SARL',
	legal_form: null,
	registration_number: '82345678900019',
	vat_status: { type: 'subject', vat_number: 'FR12823456789' },
	share_capital_cents: 5_050_000,
	address_line1: '14 rue des Artisans',
	address_line2: null,
	address_postal_code: '44800',
	address_city: 'Saint-Herblain',
	address_country: 'France',
	contact_email: null,
	contact_phone: null,
	insurance_mention: null,
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	mcp_enabled: false,
	vat_on_debits: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const RESPONSE = {
	payment_terms_days: 45,
	quote_validity_days: 15,
	default_vat_rate_bp: 550,
	late_penalty: 'dix pour cent',
	recovery_fee_cents: 5000,
	invoice_footer: 'Merci.',
	number_padding: 5,
	iban: null,
	bic: 'AGRIFRPP',
}

describe('VAT rate', () => {
	it('converts between basis points and the percentage the screen shows', () => {
		expect([0, 550, 1000, 2000].map(vatRateOfBasisPoints)).toEqual([
			0, 5.5, 10, 20,
		])
		expect(
			[0, 5.5, 10, 20].map((rate) => basisPointsOfVatRate(rate as 0)),
		).toEqual([0, 550, 1000, 2000])
	})
})

describe('billing', () => {
	it('reads the API response into the values the form edits', () => {
		expect(billingOf(RESPONSE)).toEqual({
			paymentTermsDays: 45,
			quoteValidityDays: 15,
			defaultVatRate: 5.5,
			latePenalty: 'dix pour cent',
			recoveryFee: 5000,
			invoiceFooter: 'Merci.',
			numberPadding: 5,
			iban: '',
			bic: 'AGRIFRPP',
		})
	})

	it('writes the whole body back, blank bank details as null', () => {
		const values = billingOf(RESPONSE)

		expect(billingBody(values, { iban: ' ', bic: ' AGRIFRPP ' })).toEqual({
			...RESPONSE,
			iban: null,
		})
	})
})

describe('company', () => {
	it('is read from the legal identity and the bank details of the billing settings', () => {
		const company = companyOf(ORGANIZATION, { iban: 'FR76', bic: 'AGRI' })

		expect(company).toMatchObject({
			name: 'Duval SARL',
			legalForm: '',
			capital: '50500',
			legalId: '82345678900019',
			vatNumber: 'FR12823456789',
			iban: 'FR76',
			bic: 'AGRI',
		})
		expect(company.address).toMatchObject({
			street: '14 rue des Artisans',
			complement: '',
			city: 'Saint-Herblain',
		})
	})

	it('is written as a legal identity keeping the VAT regime of the organization', () => {
		const company = companyOf(ORGANIZATION, { iban: '', bic: '' })

		expect(
			legalIdentityBody(ORGANIZATION, { ...company, capital: '1 000,50' }),
		).toMatchObject({
			legal_name: 'Duval SARL',
			legal_form: null,
			share_capital_cents: 100_050,
			vat_status: { type: 'subject', vat_number: 'FR12823456789' },
			address_line2: null,
			vat_on_debits: true,
		})
	})

	it('sends no capital when the field is blank', () => {
		const company = companyOf(ORGANIZATION, { iban: '', bic: '' })

		expect(
			legalIdentityBody(ORGANIZATION, { ...company, capital: ' ' })
				.share_capital_cents,
		).toBeNull()
	})
})

describe('settingsOf', () => {
	it('composes the organization, the billing values and the Stripe mock', () => {
		const settings = settingsOf(ORGANIZATION, billingOf(RESPONSE), {
			stripeEndpoint: 'https://example.test/hook',
			stripeSecretSet: false,
		})

		expect(settings.paymentTermsDays).toBe(45)
		expect(settings.company.bic).toBe('AGRIFRPP')
		expect(settings.stripeSecretSet).toBe(false)
	})
})

describe('sequenceRowsOf', () => {
	it('lists each series with its prefix, last and next reference', () => {
		const rows = sequenceRowsOf([
			{
				kind: 'CREDIT_NOTE',
				prefix: 'AV',
				year: 2026,
				last_assigned_number: 7,
				next_number: 8,
				next_reference: 'AV-2026-00008',
				padding: 5,
			},
			{
				kind: 'INVOICE',
				prefix: 'FAC',
				year: 2026,
				last_assigned_number: 0,
				next_number: 1,
				next_reference: 'FAC-2026-00001',
				padding: 5,
			},
		])

		expect(rows).toEqual([
			{
				kind: 'creditNote',
				label: 'Avoirs',
				prefix: 'AV',
				year: 2026,
				last: 7,
				lastNumber: 'AV-2026-00007',
				next: 'AV-2026-00008',
			},
			{
				kind: 'invoice',
				label: 'Factures',
				prefix: 'FAC',
				year: 2026,
				last: 0,
				lastNumber: null,
				next: 'FAC-2026-00001',
			},
		])
	})
})
