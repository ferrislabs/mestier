import { z } from 'zod'
import type { Schemas } from '#/api/api.client'
import { emptyToNull } from '#/pages/cursus/participants/lib/forms'

export const hostCompaniesSearchSchema = z.object({
	archived: z.literal(true).optional().catch(undefined),
})

export type HostCompaniesSearch = z.infer<typeof hostCompaniesSearchSchema>

export interface HostCompanyValues {
	name: string
	siret: string
	nafCode: string
	legalForm: string
	directorName: string
	employeeCount: string
	address: string
}

export const EMPTY_HOST_COMPANY: HostCompanyValues = {
	name: '',
	siret: '',
	nafCode: '',
	legalForm: '',
	directorName: '',
	employeeCount: '',
	address: '',
}

export function toHostCompanyValues(
	company: Schemas.HostCompanyResponse,
): HostCompanyValues {
	return {
		name: company.name,
		siret: company.siret ?? '',
		nafCode: company.naf_code ?? '',
		legalForm: company.legal_form ?? '',
		directorName: company.director_name ?? '',
		employeeCount:
			company.employee_count == null ? '' : String(company.employee_count),
		address: company.address ?? '',
	}
}

export function validateHostCompany(values: HostCompanyValues): string | null {
	if (values.name.trim() === '') return 'La raison sociale est obligatoire.'
	const count = values.employeeCount.trim()
	if (count !== '' && !/^\d+$/.test(count)) {
		return "L'effectif doit être un nombre entier."
	}
	return null
}

export function toHostCompanyRequest(
	values: HostCompanyValues,
): Schemas.CreateHostCompanyRequest {
	const count = values.employeeCount.trim()
	return {
		name: values.name.trim(),
		siret: emptyToNull(values.siret),
		naf_code: emptyToNull(values.nafCode),
		legal_form: emptyToNull(values.legalForm),
		director_name: emptyToNull(values.directorName),
		employee_count: count === '' ? null : Number(count),
		address: emptyToNull(values.address),
	}
}
