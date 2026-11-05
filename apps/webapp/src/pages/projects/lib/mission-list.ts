import { MISSION_STATUS_LABELS } from '#/pages/projects/lib/missions'
import type { Mission, MissionStatus } from '#/pages/projects/lib/model'

export type MissionFilter = 'toutes' | 'a-faire' | 'en-cours' | 'terminees'

export const MISSION_FILTERS: readonly {
	value: MissionFilter
	label: string
}[] = [
	{ value: 'toutes', label: 'Toutes' },
	{ value: 'a-faire', label: MISSION_STATUS_LABELS.todo },
	{ value: 'en-cours', label: MISSION_STATUS_LABELS.in_progress },
	{ value: 'terminees', label: 'Terminées' },
]

const FILTER_STATUS: Record<Exclude<MissionFilter, 'toutes'>, MissionStatus> = {
	'a-faire': 'todo',
	'en-cours': 'in_progress',
	terminees: 'done',
}

export function isMissionFilter(value: unknown): value is MissionFilter {
	return MISSION_FILTERS.some((filter) => filter.value === value)
}

export function missionFilterLabel(filter: MissionFilter): string {
	return MISSION_FILTERS.find((item) => item.value === filter)?.label ?? ''
}

export type MissionSearch = { statut?: Exclude<MissionFilter, 'toutes'> }

export function parseMissionSearch(
	raw: Record<string, unknown>,
): MissionSearch {
	const statut = raw.statut
	return isMissionFilter(statut) && statut !== 'toutes' ? { statut } : {}
}

export function filterMissions(
	missions: readonly Mission[],
	filter: MissionFilter,
): Mission[] {
	if (filter === 'toutes') return [...missions]
	return missions.filter((mission) => mission.status === FILTER_STATUS[filter])
}

export function moveAvailability(
	index: number,
	count: number,
	filter: MissionFilter,
): { up: boolean; down: boolean } {
	if (filter !== 'toutes') return { up: false, down: false }
	return { up: index > 0, down: index < count - 1 }
}

export function searchOfMissionFilter(filter: MissionFilter): MissionSearch {
	return parseMissionSearch({ statut: filter })
}
