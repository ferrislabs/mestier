// MOCK: per-project DOE style override — no backend endpoint yet. Replace with a project-level style on `DoeSettingsResponse` (today the style is organisation-wide).
import type { CompleteDoeStyle } from '#/pages/doe-style/lib/style'
import {
	afterLatency,
	createMockStore,
	useMockStore,
} from '#/pages/projects/feature/mocks/store'

const store = createMockStore<Readonly<Record<string, CompleteDoeStyle>>>({})

export function useProjectStyleOverrides(): Readonly<
	Record<string, CompleteDoeStyle>
> {
	return useMockStore(store)
}

export function saveProjectStyle(
	projectId: string,
	style: CompleteDoeStyle,
): Promise<void> {
	return afterLatency(() => {
		store.set((current) => ({ ...current, [projectId]: style }))
	})
}

export function clearProjectStyle(projectId: string): Promise<void> {
	return afterLatency(() => {
		store.set((current) => {
			const { [projectId]: _removed, ...rest } = current
			return rest
		})
	})
}
