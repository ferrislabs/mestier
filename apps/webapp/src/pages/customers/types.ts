import type {
	Customer,
	CustomerContact,
	CustomerContext,
	CustomerPipelineStage,
	CustomerStatus,
} from '#/hooks/use-customers'

export interface CustomerFormValues {
	status: CustomerStatus
	pipelineStage: CustomerPipelineStage
	/** Legal or full name — a customer is an entity, not a person. */
	name: string
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
