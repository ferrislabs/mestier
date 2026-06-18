import type { Customer, Property } from '#/hooks/use-customers'

export interface CustomerFormValues {
	firstName: string
	lastName: string
	email: string
	phone: string
}

export interface PropertyFormValues {
	label: string
	street: string
	zip: string
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

export function propertyToForm(property?: Property): PropertyFormValues {
	return {
		label: property?.label ?? '',
		street: property?.street ?? '',
		zip: property?.zip ?? '',
		city: property?.city ?? '',
		photoKey: property?.photo_key ?? '',
	}
}

export const EMPTY_PROPERTY_FORM: PropertyFormValues = {
	label: '',
	street: '',
	zip: '',
	city: '',
	photoKey: '',
}
