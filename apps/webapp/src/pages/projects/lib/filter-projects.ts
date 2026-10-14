interface FilterableProject {
	id: string
	name: string
}

export interface ProjectFilters {
	search: string
	/** Keep only the projects that already have a DOE. */
	withDoeOnly: boolean
}

export function filterProjects<T extends FilterableProject>(
	projects: T[],
	filters: ProjectFilters,
	hasDoe: (projectId: string) => boolean,
): T[] {
	const needle = filters.search.trim().toLowerCase()

	return projects.filter(
		(project) =>
			project.name.toLowerCase().includes(needle) &&
			(!filters.withDoeOnly || hasDoe(project.id)),
	)
}
