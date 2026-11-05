// MOCK: project extras (place, description, dates, owner) — no backend endpoint yet. Replace with fields on `ProjectResponse` and `PATCH /projects/{id}`.
import {
	afterLatency,
	createMockStore,
	useMockStore,
} from '#/pages/projects/feature/mocks/store'
import { EMPTY_EXTRAS, type ProjectExtras } from '#/pages/projects/lib/model'

const store = createMockStore<Readonly<Record<string, ProjectExtras>>>({})

export function useProjectExtrasMap(): Readonly<Record<string, ProjectExtras>> {
	return useMockStore(store)
}

export function useProjectExtras(projectId: string): {
	data: ProjectExtras
	isLoading: false
} {
	const all = useMockStore(store)
	return { data: all[projectId] ?? EMPTY_EXTRAS, isLoading: false }
}

export function saveProjectExtras(
	projectId: string,
	extras: ProjectExtras,
): Promise<void> {
	return afterLatency(() => {
		store.set((current) => ({ ...current, [projectId]: extras }))
	})
}
