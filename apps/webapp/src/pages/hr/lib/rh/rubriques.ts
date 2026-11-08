import { isActiveAbsence } from '#/pages/hr/lib/rh/absences'
import {
	CONTRACT_LABELS,
	contractTrail,
	currentContract,
} from '#/pages/hr/lib/rh/contracts'
import { contractCountLabel } from '#/pages/hr/lib/rh/format'
import { scheduleVolume } from '#/pages/hr/lib/rh/schedule-mode'
import type { Absence, IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'

export type MemberTab =
	| 'identite'
	| 'contrats'
	| 'documents'
	| 'emploi-du-temps'

export const MEMBER_TABS: readonly {
	id: MemberTab
	label: string
	description: string
}[] = [
	{
		id: 'identite',
		label: 'Identité',
		description: 'Coordonnées, adresse, poste et rattachement.',
	},
	{
		id: 'contrats',
		label: 'Contrats',
		description: 'Historique des contrats et semaines types associées.',
	},
	{
		id: 'documents',
		label: 'Documents',
		description: 'Pièces du dossier du membre.',
	},
	{
		id: 'emploi-du-temps',
		label: 'Emploi du temps',
		description: 'Calendrier, semaine type, absences et heures extra.',
	},
]

export const DEFAULT_TAB: MemberTab = 'identite'

export function isMemberTab(value: unknown): value is MemberTab {
	return MEMBER_TABS.some((tab) => tab.id === value)
}

export const ABSENCE_LIST_LIMIT = 5

export function identitySummary(member: Member): string {
	const place = [member.mobile, member.address.city]
		.filter((part) => part.trim() !== '')
		.join(' · ')
	return (
		place ||
		member.workEmail ||
		member.personalEmail ||
		'Coordonnées à compléter'
	)
}

export function contractsSummary(member: Member): string {
	if (member.contracts.length === 0) return 'Aucun contrat'
	const trail = contractTrail(member)
		.map((type) => CONTRACT_LABELS[type])
		.join(' → ')
	return `${trail} · ${contractCountLabel(member.contracts.length)}`
}

export function documentsSummary(count: number): string {
	if (count === 0) return 'Aucun document'
	return count === 1 ? '1 document' : `${count} documents`
}

export function absenceLists(
	state: RhState,
	memberId: string,
	today: IsoDate,
	limit = ABSENCE_LIST_LIMIT,
): { upcoming: Absence[]; past: Absence[] } {
	const active = state.absences.filter(
		(absence) => absence.memberId === memberId && isActiveAbsence(absence),
	)
	return {
		upcoming: active
			.filter((absence) => absence.endDate >= today)
			.sort((a, b) => a.startDate.localeCompare(b.startDate))
			.slice(0, limit),
		past: active
			.filter((absence) => absence.endDate < today)
			.sort((a, b) => b.endDate.localeCompare(a.endDate))
			.slice(0, limit),
	}
}

export function scheduleSummary(
	state: RhState,
	member: Member,
	today: IsoDate,
): string {
	const upcoming = state.absences.filter(
		(absence) =>
			absence.memberId === member.id &&
			isActiveAbsence(absence) &&
			absence.endDate >= today,
	).length
	const volume = scheduleVolume(currentContract(member, today).schedule)
	const absences =
		upcoming === 0
			? 'aucune absence à venir'
			: upcoming === 1
				? '1 absence à venir'
				: `${upcoming} absences à venir`
	return `${volume} · ${absences}`
}
