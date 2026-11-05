import {
	CONTRACT_LABELS,
	type ContractPhase,
	phaseOf,
} from '#/pages/hr/lib/rh/contracts'
import { diffDays, parseIso, WEEKDAY_SHORT } from '#/pages/hr/lib/rh/dates'
import { formatHours } from '#/pages/hr/lib/rh/members'
import { workDayMinutes } from '#/pages/hr/lib/rh/schedule'
import type {
	Contract,
	IsoDate,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

const DATE = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'short',
	year: 'numeric',
	timeZone: 'UTC',
})

const DAYS_PER_MONTH = 30.4375

type Span = Pick<Contract, 'startDate' | 'endDate'>

export function dayHoursLabel(day: WorkDay): string {
	return formatHours(Math.round((workDayMinutes(day) / 60) * 10) / 10)
}

export function dueLabel(daysLeft: number): string {
	if (daysLeft === 0) return 'aujourd’hui'
	if (daysLeft === 1) return 'demain'
	return `dans ${daysLeft} jours`
}

export function formatDate(iso: IsoDate): string {
	const text = DATE.format(parseIso(iso))
	return text.startsWith('1 ') ? `1er ${text.slice(2)}` : text
}

export function tenureLabel(years: number): string {
	if (years < 1) return 'moins d’un an'
	return years === 1 ? '1 an' : `${years} ans`
}

export function contractCountLabel(count: number): string {
	return count === 1 ? '1 contrat' : `${count} contrats`
}

export function contractPeriod(span: Span, today?: IsoDate): string {
	if (span.endDate)
		return `${formatDate(span.startDate)} au ${formatDate(span.endDate)}`
	return today && span.startDate > today
		? `à partir du ${formatDate(span.startDate)}`
		: `depuis le ${formatDate(span.startDate)}`
}

export function contractTerm(span: Span, today: IsoDate): string {
	const phase: ContractPhase = phaseOf(span, today)
	if (phase === 'upcoming') return `À partir du ${formatDate(span.startDate)}`
	if (phase === 'ended' && span.endDate)
		return `Terminé le ${formatDate(span.endDate)}`
	return span.endDate
		? `Jusqu’au ${formatDate(span.endDate)}`
		: `Depuis le ${formatDate(span.startDate)}`
}

export function durationLabel(
	startDate: IsoDate,
	endDate: IsoDate | null,
	today: IsoDate,
): string | null {
	const end = endDate ?? today
	if (end < startDate) return null
	const days = diffDays(startDate, end) + 1
	if (days < 60) return days === 1 ? '1 jour' : `${days} jours`
	const months = Math.round(days / DAYS_PER_MONTH)
	if (months < 12) return `${months} mois`
	const years = Math.floor(months / 12)
	const rest = months % 12
	const yearsLabel = years === 1 ? '1 an' : `${years} ans`
	return rest === 0 ? yearsLabel : `${yearsLabel} et ${rest} mois`
}

export function workedDaysLabel(schedule: WeekSchedule): string {
	const worked = schedule.flatMap((day, index) => (day ? [index] : []))
	if (worked.length === 0) return 'Aucun jour travaillé'
	const runs: number[][] = []
	for (const index of worked) {
		const last = runs.at(-1)
		if (last && last.at(-1) === index - 1) last.push(index)
		else runs.push([index])
	}
	return runs
		.map((run) =>
			run.length >= 3
				? `${WEEKDAY_SHORT[run[0]]}–${WEEKDAY_SHORT[run[run.length - 1]]}`
				: run.map((index) => WEEKDAY_SHORT[index]).join(', '),
		)
		.join(', ')
}

export function scheduleCaption(
	contract: Pick<Contract, 'type' | 'startDate' | 'endDate'>,
	today: IsoDate,
): string {
	const prefix = {
		upcoming: 'Contrat à venir',
		current: 'Contrat en cours',
		ended: 'Contrat terminé',
	}[phaseOf(contract, today)]
	return `${prefix} : ${CONTRACT_LABELS[contract.type]}, ${contractPeriod(contract, today)}`
}
