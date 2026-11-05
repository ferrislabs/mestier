import type { Supplier } from '#/hooks/use-suppliers'
import { normalizeText } from '#/pages/catalog/catalog-model'

export function formatLegalId(value: string): string {
	const digits = value.replace(/\s/g, '')
	if (/^\d{14}$/.test(digits))
		return `${digits.slice(0, 3)} ${digits.slice(3, 6)} ${digits.slice(6, 9)} ${digits.slice(9)}`
	if (/^\d{9}$/.test(digits))
		return `${digits.slice(0, 3)} ${digits.slice(3, 6)} ${digits.slice(6)}`
	return value
}

export function filterSuppliers(
	suppliers: readonly Supplier[],
	query: string,
): Supplier[] {
	const wanted = normalizeText(query)
	return suppliers
		.filter(
			(supplier) =>
				wanted === '' ||
				normalizeText(
					[
						supplier.name,
						supplier.registration_number ?? '',
						supplier.vat_number ?? '',
						supplier.email ?? '',
						supplier.phone ?? '',
						supplier.address ?? '',
					].join(' '),
				).includes(wanted),
		)
		.sort((a, b) => a.name.localeCompare(b.name, 'fr'))
}
