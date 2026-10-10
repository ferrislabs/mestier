export interface ProjectOption {
	id: string
	archived_at?: string | null
}

export interface DoeProjectRef {
	project_id: string
}

export type DoeButtonState = 'loading' | 'none' | { revision: string }

export function projectsWithoutDoe<T extends ProjectOption>(
	projects: T[],
	does: DoeProjectRef[],
): T[] {
	const taken = new Set(does.map((doe) => doe.project_id))
	return projects.filter(
		(project) => !project.archived_at && !taken.has(project.id),
	)
}

export function doeButtonLabel(state: DoeButtonState): string {
	if (state === 'loading') return 'DOE'
	if (state === 'none') return 'Créer le DOE'
	return `Ouvrir le DOE · indice ${state.revision}`
}

export function canCreateDoe(projectId: string, modelId: string): boolean {
	return projectId !== '' && modelId !== ''
}
