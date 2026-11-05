export interface TargetProject {
	id: string
	name: string
	customerName: string | null
	archived: boolean
}

export function eligibleProjects<T extends TargetProject>(
	projects: readonly T[],
	projectIdsWithDoe: ReadonlySet<string>,
): T[] {
	return projects
		.filter(
			(project) => !project.archived && !projectIdsWithDoe.has(project.id),
		)
		.sort((a, b) => a.name.localeCompare(b.name, 'fr'))
}

export function projectOptionLabel(project: TargetProject): string {
	return `${project.name} · ${project.customerName ?? 'Projet interne'}`
}

export function noTargetMessage(projects: readonly TargetProject[]): string {
	return projects.some((project) => !project.archived)
		? 'Tous les projets actifs ont déjà un DOE : un projet n’en reçoit qu’un.'
		: 'Aucun projet actif pour l’instant. Crée un projet, puis reviens ici.'
}
