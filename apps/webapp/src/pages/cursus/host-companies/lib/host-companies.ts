import { z } from 'zod'
import type { Schemas } from '#/api/api.client'
import { emptyToNull } from '#/pages/cursus/participants/lib/forms'

export const hostCompaniesSearchSchema = z.object({
	archived: z.literal(true).optional().catch(undefined),
})

export type HostCompaniesSearch = z.infer<typeof hostCompaniesSearchSchema>

export interface AddressValues {
	label: string
	line1: string
	line2: string
	postalCode: string
	city: string
	isMain: boolean
}

export interface HostCompanyValues {
	name: string
	groupName: string
	siret: string
	nafCode: string
	legalForm: string
	capital: string
	directorName: string
	phone: string
	email: string
	website: string
	employeeCount: string
	groupEmployeeCount: string
	activityDomains: string
	missions: string
	addresses: AddressValues[]
}

export const EMPTY_HOST_COMPANY: HostCompanyValues = {
	name: '',
	groupName: '',
	siret: '',
	nafCode: '',
	legalForm: '',
	capital: '',
	directorName: '',
	phone: '',
	email: '',
	website: '',
	employeeCount: '',
	groupEmployeeCount: '',
	activityDomains: '',
	missions: '',
	addresses: [],
}

export function emptyAddress(): AddressValues {
	return {
		label: '',
		line1: '',
		line2: '',
		postalCode: '',
		city: '',
		isMain: false,
	}
}

export function normaliseAddresses(
	addresses: AddressValues[],
): AddressValues[] {
	if (addresses.length === 0) return addresses
	const mainIndex = Math.max(
		addresses.findIndex((address) => address.isMain),
		0,
	)
	return addresses.map((address, index) => ({
		...address,
		isMain: index === mainIndex,
	}))
}

export function addAddress(addresses: AddressValues[]): AddressValues[] {
	return normaliseAddresses([...addresses, emptyAddress()])
}

export function removeAddress(
	addresses: AddressValues[],
	index: number,
): AddressValues[] {
	return normaliseAddresses(addresses.filter((_, i) => i !== index))
}

export function setMainAddress(
	addresses: AddressValues[],
	index: number,
): AddressValues[] {
	return addresses.map((address, i) => ({ ...address, isMain: i === index }))
}

export function updateAddress(
	addresses: AddressValues[],
	index: number,
	patch: Partial<AddressValues>,
): AddressValues[] {
	return addresses.map((address, i) =>
		i === index ? { ...address, ...patch } : address,
	)
}

export function mainAddress(
	addresses: readonly Schemas.HostCompanyAddressResponse[],
): Schemas.HostCompanyAddressResponse | null {
	return addresses.find((address) => address.is_main) ?? addresses[0] ?? null
}

export function toHostCompanyValues(
	company: Schemas.HostCompanyResponse,
): HostCompanyValues {
	return {
		name: company.name,
		groupName: company.group_name ?? '',
		siret: company.siret ?? '',
		nafCode: company.naf_code ?? '',
		legalForm: company.legal_form ?? '',
		capital: company.capital ?? '',
		directorName: company.director_name ?? '',
		phone: company.phone ?? '',
		email: company.email ?? '',
		website: company.website ?? '',
		employeeCount:
			company.employee_count == null ? '' : String(company.employee_count),
		groupEmployeeCount:
			company.group_employee_count == null
				? ''
				: String(company.group_employee_count),
		activityDomains: company.activity_domains ?? '',
		missions: company.missions ?? '',
		addresses: normaliseAddresses(
			company.addresses.map((address) => ({
				label: address.label ?? '',
				line1: address.line1,
				line2: address.line2 ?? '',
				postalCode: address.postal_code ?? '',
				city: address.city ?? '',
				isMain: address.is_main,
			})),
		),
	}
}

function isWholeNumber(value: string): boolean {
	const trimmed = value.trim()
	return trimmed === '' || /^\d+$/.test(trimmed)
}

export function validateHostCompany(values: HostCompanyValues): string | null {
	if (values.name.trim() === '') {
		return "Le nom de l'entreprise est obligatoire."
	}
	if (!isWholeNumber(values.employeeCount)) {
		return "Le nombre d'employés doit être un nombre entier."
	}
	if (!isWholeNumber(values.groupEmployeeCount)) {
		return "Le nombre d'employés du groupe doit être un nombre entier."
	}
	if (values.addresses.some((address) => address.line1.trim() === '')) {
		return "Chaque adresse doit avoir une ligne d'adresse."
	}
	return null
}

function toCount(value: string): number | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : Number(trimmed)
}

function toMarkdown(value: string): string | null {
	return value.trim() === '' ? null : value
}

export function toHostCompanyRequest(
	values: HostCompanyValues,
): Schemas.CreateHostCompanyRequest {
	return {
		name: values.name.trim(),
		group_name: emptyToNull(values.groupName),
		siret: emptyToNull(values.siret),
		naf_code: emptyToNull(values.nafCode),
		legal_form: emptyToNull(values.legalForm),
		capital: emptyToNull(values.capital),
		director_name: emptyToNull(values.directorName),
		phone: emptyToNull(values.phone),
		email: emptyToNull(values.email),
		website: emptyToNull(values.website),
		employee_count: toCount(values.employeeCount),
		group_employee_count: toCount(values.groupEmployeeCount),
		activity_domains: emptyToNull(values.activityDomains),
		missions: toMarkdown(values.missions),
		addresses: normaliseAddresses(values.addresses).map((address) => ({
			label: emptyToNull(address.label),
			line1: address.line1.trim(),
			line2: emptyToNull(address.line2),
			postal_code: emptyToNull(address.postalCode),
			city: emptyToNull(address.city),
			is_main: address.isMain,
		})),
	}
}
