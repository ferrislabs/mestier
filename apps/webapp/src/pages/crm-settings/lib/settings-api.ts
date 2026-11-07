import type { Schemas } from '#/api/api.client'
import type {
	BillingSettings,
	BillingSettingsBody,
	NumberSeriesKind,
	SeriesNumbering,
} from '#/hooks/use-billing-settings'
import type { Organization } from '#/hooks/use-organizations'
import {
	type BillingPatch,
	formatNumber,
	SEQUENCE_LABELS,
	type SequenceRow,
} from '#/pages/crm-settings/lib/settings-model'
import type {
	Company,
	SequenceKind,
	Settings,
	VatRate,
} from '#/pages/crm-settings/types'
import { centsToInput, parseAmount } from '#/pages/customers/lib/format'

const KIND_OF_SERIES: Record<NumberSeriesKind, SequenceKind> = {
	QUOTE: 'quote',
	INVOICE: 'invoice',
	CREDIT_NOTE: 'creditNote',
}

const SERIES_OF_KIND: Record<SequenceKind, NumberSeriesKind> = {
	quote: 'QUOTE',
	invoice: 'INVOICE',
	creditNote: 'CREDIT_NOTE',
}

const VAT_RATES_BY_BASIS_POINTS: Record<number, VatRate> = {
	0: 0,
	550: 5.5,
	1000: 10,
	2000: 20,
}

export type BillingValues = BillingPatch & { iban: string; bic: string }

export type StripeValues = Pick<Settings, 'stripeEndpoint' | 'stripeSecretSet'>

export const DEFAULT_BILLING: BillingValues = {
	paymentTermsDays: 30,
	quoteValidityDays: 30,
	defaultVatRate: 20,
	latePenalty: 'trois fois le taux d’intérêt légal',
	recoveryFee: 4000,
	invoiceFooter: '',
	numberPadding: 4,
	iban: '',
	bic: '',
}

export function seriesOfKind(kind: SequenceKind): NumberSeriesKind {
	return SERIES_OF_KIND[kind]
}

export function vatRateOfBasisPoints(basisPoints: number): VatRate {
	return VAT_RATES_BY_BASIS_POINTS[basisPoints] ?? 20
}

export function basisPointsOfVatRate(rate: VatRate): number {
	return Math.round(rate * 100)
}

export function billingOf(response: BillingSettings): BillingValues {
	return {
		paymentTermsDays: response.payment_terms_days,
		quoteValidityDays: response.quote_validity_days,
		defaultVatRate: vatRateOfBasisPoints(response.default_vat_rate_bp),
		latePenalty: response.late_penalty,
		recoveryFee: response.recovery_fee_cents,
		invoiceFooter: response.invoice_footer,
		numberPadding: response.number_padding,
		iban: response.iban ?? '',
		bic: response.bic ?? '',
	}
}

function nullIfBlank(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function billingBody(
	values: BillingPatch,
	bank: { iban: string; bic: string },
): BillingSettingsBody {
	return {
		payment_terms_days: values.paymentTermsDays,
		quote_validity_days: values.quoteValidityDays,
		default_vat_rate_bp: basisPointsOfVatRate(values.defaultVatRate),
		late_penalty: values.latePenalty,
		recovery_fee_cents: values.recoveryFee,
		invoice_footer: values.invoiceFooter,
		number_padding: values.numberPadding,
		iban: nullIfBlank(bank.iban),
		bic: nullIfBlank(bank.bic),
	}
}

export function companyOf(
	organization: Organization,
	billing: Pick<BillingValues, 'iban' | 'bic'>,
): Company {
	return {
		name: organization.legal_name ?? '',
		legalForm: organization.legal_form ?? '',
		capital:
			organization.share_capital_cents == null
				? ''
				: centsToInput(organization.share_capital_cents),
		legalId: organization.registration_number ?? '',
		vatNumber:
			organization.vat_status?.type === 'subject'
				? organization.vat_status.vat_number
				: '',
		address: {
			street: organization.address_line1 ?? '',
			complement: organization.address_line2 ?? '',
			postalCode: organization.address_postal_code ?? '',
			city: organization.address_city ?? '',
			country: organization.address_country ?? '',
		},
		email: organization.contact_email ?? '',
		phone: organization.contact_phone ?? '',
		iban: billing.iban,
		bic: billing.bic,
		insurance: organization.insurance_mention ?? '',
	}
}

export function legalIdentityBody(
	organization: Organization,
	company: Company,
): Schemas.UpdateLegalIdentityRequest {
	return {
		legal_name: nullIfBlank(company.name),
		legal_form: nullIfBlank(company.legalForm),
		registration_number: nullIfBlank(company.legalId),
		vat_status: {
			type: 'subject',
			vat_number: company.vatNumber.trim(),
		},
		share_capital_cents:
			company.capital.trim() === '' ? null : parseAmount(company.capital),
		address_line1: nullIfBlank(company.address.street),
		address_line2: nullIfBlank(company.address.complement),
		address_postal_code: nullIfBlank(company.address.postalCode),
		address_city: nullIfBlank(company.address.city),
		address_country: nullIfBlank(company.address.country),
		contact_email: nullIfBlank(company.email),
		contact_phone: nullIfBlank(company.phone),
		insurance_mention: nullIfBlank(company.insurance),
		vat_on_debits: organization.vat_on_debits,
	}
}

export function settingsOf(
	organization: Organization,
	billing: BillingValues,
	stripe: StripeValues,
): Settings {
	const { iban, bic, ...rest } = billing
	return {
		company: companyOf(organization, { iban, bic }),
		...rest,
		...stripe,
	}
}

export function sequenceRowsOf(
	numbering: readonly SeriesNumbering[],
): SequenceRow[] {
	return numbering.map((entry) => {
		const kind = KIND_OF_SERIES[entry.kind]
		const last = entry.last_assigned_number
		return {
			kind,
			label: SEQUENCE_LABELS[kind],
			prefix: entry.prefix,
			year: entry.year,
			last,
			lastNumber:
				last > 0
					? formatNumber(entry.prefix, entry.year, last, entry.padding)
					: null,
			next: entry.next_reference,
		}
	})
}
