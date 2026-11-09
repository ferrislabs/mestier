import type { Schemas } from '#/api/api.client'

export interface ContractFormValues {
	hostCompanyId: string
	kind: Schemas.ContractKind
	number: string
	startDate: string
	endDate: string
}

export interface ContractFormErrors {
	hostCompanyId?: string
	startDate?: string
	endDate?: string
}

export const EMPTY_CONTRACT_FORM: ContractFormValues = {
	hostCompanyId: '',
	kind: 'APPRENTICESHIP',
	number: '',
	startDate: '',
	endDate: '',
}

export function contractToForm(
	contract: Schemas.ContractResponse,
): ContractFormValues {
	return {
		hostCompanyId: contract.host_company_id,
		kind: contract.kind,
		number: contract.number ?? '',
		startDate: contract.start_date,
		endDate: contract.end_date,
	}
}

export function contractFormErrors(
	values: ContractFormValues,
): ContractFormErrors {
	const errors: ContractFormErrors = {}
	if (!values.hostCompanyId) errors.hostCompanyId = 'Choisissez une entreprise.'
	if (!values.startDate) errors.startDate = 'La date de début est requise.'
	if (!values.endDate) errors.endDate = 'La date de fin est requise.'
	if (values.startDate && values.endDate && values.endDate < values.startDate) {
		errors.endDate = 'La fin doit suivre le début.'
	}
	return errors
}

export function toContractRequest(
	values: ContractFormValues,
): Schemas.ContractRequest {
	const number = values.number.trim()
	return {
		host_company_id: values.hostCompanyId,
		kind: values.kind,
		number: number === '' ? null : number,
		start_date: values.startDate,
		end_date: values.endDate,
	}
}
