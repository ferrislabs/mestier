import { isIsoDate } from '#/pages/projects/lib/dates'
import type {
	Mission,
	MissionDraft,
	MissionStatus,
} from '#/pages/projects/lib/model'

export const MISSION_STATUS_LABELS: Record<MissionStatus, string> = {
	todo: 'À faire',
	in_progress: 'En cours',
	done: 'Terminée',
}

export const MISSION_STATUSES: readonly MissionStatus[] = [
	'todo',
	'in_progress',
	'done',
]

export function isMissionStatus(value: string): value is MissionStatus {
	return MISSION_STATUSES.some((status) => status === value)
}

export function validateMissionDraft(draft: MissionDraft): string[] {
	const errors: string[] = []
	if (draft.title.trim() === '')
		errors.push('Le titre de la mission est obligatoire.')
	for (const date of [draft.startDate, draft.endDate]) {
		if (date !== null && !isIsoDate(date))
			errors.push('Une date n’est pas valide.')
	}
	if (
		draft.startDate !== null &&
		draft.endDate !== null &&
		draft.endDate < draft.startDate
	)
		errors.push('La fin ne peut pas précéder le début.')
	return errors
}

export function sortedMissions(missions: readonly Mission[]): Mission[] {
	return [...missions].sort((a, b) => a.position - b.position)
}

export interface MissionStats {
	total: number
	todo: number
	inProgress: number
	done: number
	progress: number
}

export function missionStats(missions: readonly Mission[]): MissionStats {
	const done = missions.filter((mission) => mission.status === 'done').length
	return {
		total: missions.length,
		todo: missions.filter((mission) => mission.status === 'todo').length,
		inProgress: missions.filter((mission) => mission.status === 'in_progress')
			.length,
		done,
		progress:
			missions.length === 0 ? 0 : Math.round((done / missions.length) * 100),
	}
}

export function movedMissions(
	missions: readonly Mission[],
	missionId: string,
	delta: -1 | 1,
): Mission[] {
	const ordered = sortedMissions(missions)
	const index = ordered.findIndex((item) => item.id === missionId)
	const target = index + delta
	if (index < 0 || target < 0 || target >= ordered.length) return [...missions]
	const swapped = [...ordered]
	const [moved] = swapped.splice(index, 1)
	if (moved) swapped.splice(target, 0, moved)
	const positions = new Map(
		swapped.map((item, position) => [item.id, position]),
	)
	return missions.map((item) => ({
		...item,
		position: positions.get(item.id) ?? item.position,
	}))
}
