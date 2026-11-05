import { FIXED_TERM_CONTRACTS } from '#/pages/hr/lib/rh/contracts'
import { defaultContractSchedule } from '#/pages/hr/lib/rh/members'
import type {
	Contract,
	ContractDraft,
	ContractType,
	IsoDate,
} from '#/pages/hr/lib/rh/types'

export type ContractFieldKey = 'startDate' | 'endDate' | 'trialEndDate'

const ERROR_FIELDS: Record<string, ContractFieldKey> = {
	'La date de début du contrat est obligatoire.': 'startDate',
	'Ce type de contrat demande une date de fin.': 'endDate',
	'La date de fin précède la date de début.': 'endDate',
	'La fin de la période d’essai doit se situer pendant le contrat.':
		'trialEndDate',
}

const OVERLAP_PREFIX = 'Ce contrat chevauche un autre contrat'
const NO_CONTRACT_ERROR = 'Un membre doit avoir au moins un contrat.'

export function isFixedTerm(type: ContractType): boolean {
	return FIXED_TERM_CONTRACTS.includes(type)
}

export function emptyContractDraft(today: IsoDate): ContractDraft {
	return {
		type: 'cdi',
		startDate: today,
		endDate: null,
		trialEndDate: null,
		schedule: defaultContractSchedule(),
	}
}

export function contractDraftOf(contract: Contract): ContractDraft {
	return {
		type: contract.type,
		startDate: contract.startDate,
		endDate: contract.endDate,
		trialEndDate: contract.trialEndDate,
		schedule: contract.schedule,
	}
}

function orNull(value: string | null): string | null {
	return value === null || value === '' ? null : value
}

export function finalizeContractDraft(draft: ContractDraft): ContractDraft {
	return {
		...draft,
		endDate: orNull(draft.endDate),
		trialEndDate: orNull(draft.trialEndDate),
	}
}

export type GroupedContractErrors = {
	byField: Partial<Record<ContractFieldKey, string>>
	overlap: string | null
}

export function groupContractErrors(
	errors: readonly string[],
): GroupedContractErrors {
	const grouped: GroupedContractErrors = { byField: {}, overlap: null }
	for (const error of errors) {
		const field = ERROR_FIELDS[error]
		if (field) {
			if (!grouped.byField[field]) grouped.byField[field] = error
		} else if (error.startsWith(OVERLAP_PREFIX) && !grouped.overlap) {
			grouped.overlap = error
		}
	}
	return grouped
}

export function isContractError(message: string): boolean {
	return (
		message in ERROR_FIELDS ||
		message.startsWith(OVERLAP_PREFIX) ||
		message === NO_CONTRACT_ERROR
	)
}
