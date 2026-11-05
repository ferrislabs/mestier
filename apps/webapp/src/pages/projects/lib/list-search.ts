import {
	CLIENT_ALL,
	DEFAULT_FILTERS,
	type ProjectFilters,
} from '#/pages/projects/lib/project-list'
import type { ProjectsSearch } from '#/pages/projects/types'

export function filtersOfSearch(search: ProjectsSearch): ProjectFilters {
	return {
		query: search.q ?? DEFAULT_FILTERS.query,
		status:
			search.statut ?? (search.archived ? 'tous' : DEFAULT_FILTERS.status),
		client: search.client ?? DEFAULT_FILTERS.client,
	}
}

export function searchOfFilters(
	filters: ProjectFilters,
	projectId: string | undefined,
): ProjectsSearch {
	const query = filters.query.trim()
	return {
		archived: false,
		...(projectId ? { projectId } : {}),
		...(filters.status !== 'actifs' ? { statut: filters.status } : {}),
		...(query !== '' ? { q: filters.query } : {}),
		...(filters.client !== CLIENT_ALL ? { client: filters.client } : {}),
	}
}
