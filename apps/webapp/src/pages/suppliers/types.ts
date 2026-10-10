import type { Supplier } from '#/hooks/use-suppliers'

export interface SupplierFormValues {
	name: string
	registrationNumber: string
	vatNumber: string
	address: string
	phone: string
	email: string
}

export const EMPTY_SUPPLIER_FORM: SupplierFormValues = {
	name: '',
	registrationNumber: '',
	vatNumber: '',
	address: '',
	phone: '',
	email: '',
}

export function supplierToFormValues(supplier: Supplier): SupplierFormValues {
	return {
		name: supplier.name,
		registrationNumber: supplier.registration_number ?? '',
		vatNumber: supplier.vat_number ?? '',
		address: supplier.address ?? '',
		phone: supplier.phone ?? '',
		email: supplier.email ?? '',
	}
}

function optional(value: string): string | null {
	const trimmed = value.trim()
	return trimmed.length > 0 ? trimmed : null
}

export function buildSupplierRequest(values: SupplierFormValues) {
	return {
		name: values.name.trim(),
		registration_number: optional(values.registrationNumber),
		vat_number: optional(values.vatNumber),
		address: optional(values.address),
		phone: optional(values.phone),
		email: optional(values.email),
	}
}

export function validateSupplierName(name: string): string | undefined {
	return name.trim().length === 0 ? 'Le nom est obligatoire.' : undefined
}

export function firstAddressLine(address: string | null | undefined): string {
	return address?.split('\n')[0]?.trim() ?? ''
}
