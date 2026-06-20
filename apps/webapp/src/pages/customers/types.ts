import type { Customer, CustomerContext } from '#/hooks/use-customers'

export interface CustomerFormValues {
	firstName: string
	lastName: string
	email: string
	phone: string
}

export interface CustomerContextFormValues {
	label: string
	addressLine: string
	postalCode: string
	city: string
	photoKey: string
}

export function customerDisplayName(customer: Customer): string {
	return `${customer.first_name} ${customer.last_name}`.trim()
}

export function customerInitials(customer: Customer): string {
	const parts = [customer.first_name, customer.last_name].filter(Boolean)
	return (
		parts
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('') || 'C'
	)
}

export function customerToForm(customer: Customer): CustomerFormValues {
	return {
		firstName: customer.first_name,
		lastName: customer.last_name,
		email: customer.email ?? '',
		phone: customer.phone ?? '',
	}
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
