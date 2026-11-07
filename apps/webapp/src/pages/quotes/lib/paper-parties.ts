import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { BillingFrequency } from '#/pages/quotes/lib/billing-frequency'
import { postalAddressLines } from '#/pages/quotes/lib/doc-format'
import { formatCents } from '#/pages/quotes/types'
import type {
	PaperIssuer,
	PaperLine,
	PaperRecipient,
} from '#/pages/quotes/ui/doc/document-paper'

export function vatNoticeOf(organization: Organization): string | null {
	return organization.vat_status?.type === 'not_subject'
		? `TVA non applicable, ${organization.vat_status.basis}`
		: null
}

export function paperIssuer(organization: Organization): PaperIssuer {
	return {
		name: organization.legal_name?.trim() || organization.name,
		legalForm: organization.legal_form ?? '',
		capital:
			organization.share_capital_cents != null
				? formatCents(organization.share_capital_cents)
				: '',
		addressLines: postalAddressLines({
			street: organization.address_line1,
			complement: organization.address_line2,
			postalCode: organization.address_postal_code,
			city: organization.address_city,
			country: organization.address_country,
		}),
		phone: organization.contact_phone ?? '',
		email: organization.contact_email ?? '',
		legalId: organization.registration_number ?? '',
		vatNumber:
			organization.vat_status?.type === 'subject'
				? organization.vat_status.vat_number
				: '',
		insurance: organization.insurance_mention ?? '',
	}
}

export function paperRecipient(
	customer: Customer | undefined,
	context: CustomerContext | undefined,
): PaperRecipient {
	return {
		name: customer?.name.trim() ?? '',
		contactName: '',
		addressLines: context
			? postalAddressLines({
					street: context.address_line,
					postalCode: context.postal_code,
					city: context.city,
				})
			: [],
		legalId: customer?.registration_number ?? '',
	}
}

export function paperLine(input: {
	id: string
	label: string
	notes?: string | null
	quantity: string
	unit?: string | null
	unitPriceCents: number
	vatRateBp: number | null
	totalCents: number
	discountPercent?: number
	frequency?: BillingFrequency
}): PaperLine {
	return {
		id: input.id,
		title: input.label.trim(),
		details: input.notes?.trim() ?? '',
		quantity: input.quantity.replace('.', ','),
		unit: input.unit ?? null,
		unitPriceCents: input.unitPriceCents,
		vatRateBp: input.vatRateBp,
		totalCents: input.totalCents,
		...(input.discountPercent !== undefined
			? { discountPercent: input.discountPercent }
			: {}),
		...(input.frequency !== undefined ? { frequency: input.frequency } : {}),
	}
}
