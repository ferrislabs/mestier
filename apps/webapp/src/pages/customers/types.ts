import type {
	Customer,
	CustomerContact,
	CustomerContext,
	CustomerPipelineStage,
	CustomerStatus,
} from '#/hooks/use-customers'
import type {
	AccountExtras,
	AccountKind,
	LeadSource,
} from '#/pages/customers/feature/mocks/account-extras'

export interface CustomerFormValues {
	status: CustomerStatus
	pipelineStage: CustomerPipelineStage
	/** Legal or full name — a customer is an entity, not a person. */
	name: string
	/**
	 * The customer's own SIREN (or equivalent) — required on an invoice
	 * transmitted electronically once the customer is itself VAT-subject,
	 * per the reform effective 1 September 2026.
	 */
	registrationNumber: string
	email: string
	phone: string
}

export interface CustomerContactFormValues {
	firstName: string
	lastName: string
	role: string
	email: string
	phone: string
	isPrimary: boolean
}

export interface CustomerContextFormValues {
	label: string
	addressLine: string
	postalCode: string
	city: string
	photoKey: string
}

export function customerDisplayName(customer: Customer): string {
	return customer.name.trim()
}

/**
 * Two initials at most. A customer is an entity: "Mairie de Saint-Julien"
 * yields "MS", "Marie Leroy" yields "ML".
 */
export function customerInitials(customer: Customer): string {
	return (
		customer.name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('') || 'C'
	)
}

export function customerToForm(customer: Customer): CustomerFormValues {
	return {
		status: customer.status,
		pipelineStage: customer.pipeline_stage,
		name: customer.name,
		registrationNumber: customer.registration_number ?? '',
		email: customer.email ?? '',
		phone: customer.phone ?? '',
	}
}

export function customerContactToForm(
	customerContact?: CustomerContact,
): CustomerContactFormValues {
	return {
		firstName: customerContact?.first_name ?? '',
		lastName: customerContact?.last_name ?? '',
		role: customerContact?.role ?? '',
		email: customerContact?.email ?? '',
		phone: customerContact?.phone ?? '',
		isPrimary: customerContact?.is_primary ?? false,
	}
}

export function customerStatusLabel(status: CustomerStatus): string {
	if (status === 'PROSPECT') return 'Prospect'
	if (status === 'CLIENT') return 'Client'
	return 'Archivé'
}

export function customerPipelineStageLabel(
	stage: CustomerPipelineStage,
): string {
	if (stage === 'NEW') return 'Nouveau'
	if (stage === 'CONTACTED') return 'Contacté'
	if (stage === 'QUALIFIED') return 'Qualifié'
	if (stage === 'QUOTE_SENT') return 'Devis envoyé'
	if (stage === 'WON') return 'Gagné'
	return 'Perdu'
}

export function customerContextToForm(
	customerContext?: CustomerContext,
): CustomerContextFormValues {
	return {
		label: customerContext?.label ?? '',
		addressLine: customerContext?.address_line ?? '',
		postalCode: customerContext?.postal_code ?? '',
		city: customerContext?.city ?? '',
		photoKey: customerContext?.photo_key ?? '',
	}
}

export const EMPTY_CUSTOMER_CONTEXT_FORM: CustomerContextFormValues = {
	label: '',
	addressLine: '',
	postalCode: '',
	city: '',
	photoKey: '',
}

export const EMPTY_CUSTOMER_CONTACT_FORM: CustomerContactFormValues = {
	firstName: '',
	lastName: '',
	role: '',
	email: '',
	phone: '',
	isPrimary: false,
}

export type AccountStatus = 'prospect' | 'client' | 'archived'

export function accountStatusOf(status: CustomerStatus): AccountStatus {
	if (status === 'PROSPECT') return 'prospect'
	if (status === 'CLIENT') return 'client'
	return 'archived'
}

export const ACCOUNT_STATUS_LABELS: Record<AccountStatus, string> = {
	prospect: 'Prospect',
	client: 'Client',
	archived: 'Archivé',
}

export const ACCOUNT_KIND_LABELS: Record<AccountKind, string> = {
	particulier: 'Particulier',
	professionnel: 'Professionnel',
}

export const LEAD_SOURCE_LABELS: Record<LeadSource, string> = {
	bouche_a_oreille: 'Bouche-à-oreille',
	site_web: 'Site web',
	recommandation: 'Recommandation',
	salon: 'Salon',
	architecte: 'Architecte',
	autre: 'Autre',
}

export const LEAD_SOURCES = Object.keys(LEAD_SOURCE_LABELS) as LeadSource[]

export function kindOf(
	customer: Pick<Customer, 'registration_number'>,
	extras: Pick<AccountExtras, 'kind'>,
): AccountKind {
	if (extras.kind !== null) return extras.kind
	return customer.registration_number ? 'professionnel' : 'particulier'
}

export type CustomerBodyOverrides = Partial<{
	status: CustomerStatus
	pipeline_stage: CustomerPipelineStage
	name: string
	registration_number: string | null
	email: string | null
	phone: string | null
}>

export function customerBody(
	customer: Customer,
	overrides: CustomerBodyOverrides = {},
) {
	return {
		status: customer.status,
		pipeline_stage: customer.pipeline_stage,
		name: customer.name,
		registration_number: customer.registration_number ?? null,
		email: customer.email ?? null,
		phone: customer.phone ?? null,
		...overrides,
	}
}

export function reactivatedStatus(customer: Customer): {
	status: CustomerStatus
	pipeline_stage: CustomerPipelineStage
} {
	return customer.pipeline_stage === 'WON'
		? { status: 'CLIENT', pipeline_stage: 'WON' }
		: { status: 'PROSPECT', pipeline_stage: customer.pipeline_stage }
}
