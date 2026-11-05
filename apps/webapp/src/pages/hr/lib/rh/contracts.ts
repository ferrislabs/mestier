import {
	NO_SCHEDULE,
	validateSchedule,
	weekly,
	weekOn,
} from '#/pages/hr/lib/rh/contract-schedule'
import { addDays, formatRange } from '#/pages/hr/lib/rh/dates'
import type {
	Contract,
	ContractDraft,
	ContractType,
	IsoDate,
	Member,
	WeekSchedule,
} from '#/pages/hr/lib/rh/types'

export const CONTRACT_LABELS: Record<ContractType, string> = {
	cdi: 'CDI',
	cdd: 'CDD',
	apprentissage: 'Apprentissage',
	interim: 'Intérim',
	gerant: 'Gérant',
}

export const FIXED_TERM_CONTRACTS: readonly ContractType[] = [
	'cdd',
	'apprentissage',
	'interim',
]

export { NO_SCHEDULE }

const NO_CONTRACT: Contract = {
	id: 'none',
	type: 'cdi',
	startDate: '',
	endDate: null,
	trialEndDate: null,
	schedule: weekly(NO_SCHEDULE),
}

export type ContractPhase = 'upcoming' | 'current' | 'ended'

export function sortContracts<Item extends Pick<Contract, 'startDate'>>(
	contracts: readonly Item[],
): Item[] {
	return [...contracts].sort((a, b) => a.startDate.localeCompare(b.startDate))
}

export function covers(
	contract: Pick<Contract, 'startDate' | 'endDate'>,
	date: IsoDate,
): boolean {
	return (
		date >= contract.startDate &&
		(contract.endDate === null || date <= contract.endDate)
	)
}

export function contractOn(member: Member, date: IsoDate): Contract | null {
	return member.contracts.find((contract) => covers(contract, date)) ?? null
}

export function currentContract(member: Member, today: IsoDate): Contract {
	const covering = contractOn(member, today)
	if (covering) return covering
	const sorted = sortContracts(member.contracts)
	const started = sorted.filter((contract) => contract.startDate <= today)
	return started.at(-1) ?? sorted[0] ?? NO_CONTRACT
}

export function hireDateOf(member: Member): IsoDate {
	return sortContracts(member.contracts)[0]?.startDate ?? ''
}

export function leaveDateOf(member: Member): IsoDate | null {
	const last = sortContracts(member.contracts).at(-1)
	return last ? last.endDate : null
}

export function scheduleOn(member: Member, date: IsoDate): WeekSchedule {
	const contract = contractOn(member, date)
	return contract ? weekOn(contract.schedule, date).week : NO_SCHEDULE
}

export function blockLabelOn(member: Member, date: IsoDate): string | null {
	const contract = contractOn(member, date)
	if (!contract) return null
	const { week, label } = weekOn(contract.schedule, date)
	return week.every((day) => day === null) ? label : null
}

export function isVariableOn(member: Member, date: IsoDate): boolean {
	return contractOn(member, date)?.schedule.kind === 'variable'
}

export function phaseOf(
	contract: Pick<Contract, 'startDate' | 'endDate'>,
	today: IsoDate,
): ContractPhase {
	if (contract.startDate > today) return 'upcoming'
	return contract.endDate !== null && contract.endDate < today
		? 'ended'
		: 'current'
}

export function contractTrail(member: Member): ContractType[] {
	const trail: ContractType[] = []
	for (const contract of sortContracts(member.contracts)) {
		if (trail.at(-1) !== contract.type) trail.push(contract.type)
	}
	return trail
}

export function validateContracts(
	contracts: readonly ContractDraft[],
): string[] {
	if (contracts.length === 0)
		return ['Un membre doit avoir au moins un contrat.']
	const errors: string[] = []
	for (const contract of contracts) {
		if (contract.startDate === '') {
			errors.push('La date de début du contrat est obligatoire.')
			continue
		}
		if (FIXED_TERM_CONTRACTS.includes(contract.type) && !contract.endDate)
			errors.push('Ce type de contrat demande une date de fin.')
		if (contract.endDate && contract.endDate < contract.startDate)
			errors.push('La date de fin précède la date de début.')
		if (
			contract.trialEndDate &&
			(contract.trialEndDate < contract.startDate ||
				(contract.endDate && contract.trialEndDate > contract.endDate))
		)
			errors.push(
				'La fin de la période d’essai doit se situer pendant le contrat.',
			)
		errors.push(...validateSchedule(contract.schedule))
	}
	const dated = sortContracts(
		contracts.filter((contract) => contract.startDate),
	)
	for (let index = 1; index < dated.length; index += 1) {
		const previous = dated[index - 1]
		const next = dated[index]
		if (previous.endDate === null || previous.endDate >= next.startDate)
			errors.push(
				`Ce contrat chevauche un autre contrat (${formatRange(previous.startDate, previous.endDate ?? next.startDate)}).`,
			)
	}
	return [...new Set(errors)]
}

export function suggestNextContract(
	member: Member,
	today: IsoDate,
): ContractDraft {
	const last = sortContracts(member.contracts).at(-1)
	return {
		type: 'cdi',
		startDate: last?.endDate ? addDays(last.endDate, 1) : today,
		endDate: null,
		trialEndDate: null,
		schedule: last?.schedule ?? weekly(NO_SCHEDULE),
	}
}
