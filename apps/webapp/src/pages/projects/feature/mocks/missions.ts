// MOCK: missions — no backend endpoint yet. Replace with a project-scoped missions resource (list, create, patch, delete, reorder).
import {
	afterLatency,
	createMockStore,
	useMockStore,
} from '#/pages/projects/feature/mocks/store'
import { todayIso } from '#/pages/projects/lib/dates'
import { movedMissions } from '#/pages/projects/lib/missions'
import type { Mission, MissionDraft } from '#/pages/projects/lib/model'

const store = createMockStore<readonly Mission[]>([])

export function useAllMissions(): readonly Mission[] {
	return useMockStore(store)
}

export function createMission(
	projectId: string,
	draft: MissionDraft,
): Promise<Mission> {
	return afterLatency(() => {
		const siblings = store.get().filter((item) => item.projectId === projectId)
		const mission: Mission = {
			...draft,
			title: draft.title.trim(),
			description: draft.description.trim(),
			id: crypto.randomUUID(),
			projectId,
			position: Math.max(-1, ...siblings.map((item) => item.position)) + 1,
			createdAt: todayIso(),
		}
		store.set((current) => [...current, mission])
		return mission
	})
}

export function updateMission(
	missionId: string,
	draft: MissionDraft,
): Promise<void> {
	return afterLatency(() => {
		store.set((current) =>
			current.map((item) =>
				item.id === missionId
					? {
							...item,
							...draft,
							title: draft.title.trim(),
							description: draft.description.trim(),
						}
					: item,
			),
		)
	})
}

export function setMissionStatus(
	missionId: string,
	status: Mission['status'],
): Promise<void> {
	return afterLatency(() => {
		store.set((current) =>
			current.map((item) =>
				item.id === missionId ? { ...item, status } : item,
			),
		)
	})
}

export function deleteMission(missionId: string): Promise<void> {
	return afterLatency(() => {
		store.set((current) => current.filter((item) => item.id !== missionId))
	})
}

export function moveMission(
	projectId: string,
	missionId: string,
	delta: -1 | 1,
): Promise<void> {
	return afterLatency(() => {
		store.set((current) => {
			const inProject = current.filter((item) => item.projectId === projectId)
			const others = current.filter((item) => item.projectId !== projectId)
			return [...others, ...movedMissions(inProject, missionId, delta)]
		})
	})
}
