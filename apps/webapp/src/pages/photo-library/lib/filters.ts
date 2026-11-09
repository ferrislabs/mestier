import { parseTagInput } from '#/pages/photo-library/lib/tags'

export interface LibraryFilters {
	q: string
	tags: string[]
	projectId: string | null
	toSort: boolean
}

export const EMPTY_FILTERS: LibraryFilters = {
	q: '',
	tags: [],
	projectId: null,
	toSort: false,
}

export const PAGE_SIZE = 48

export function hasFilters(filters: LibraryFilters): boolean {
	return (
		filters.q.trim() !== '' ||
		filters.tags.length > 0 ||
		filters.projectId !== null ||
		filters.toSort
	)
}

export function filtersFromInputs(inputs: {
	q: string
	tagInput: string
	projectId: string | null
	toSort: boolean
}): LibraryFilters {
	return {
		q: inputs.q.trim(),
		tags: parseTagInput(inputs.tagInput),
		projectId: inputs.projectId,
		toSort: inputs.toSort,
	}
}

export function listSearchParams(
	filters: LibraryFilters,
	page: number,
	perPage: number = PAGE_SIZE,
): URLSearchParams {
	const params = new URLSearchParams()
	params.set('page', String(page))
	params.set('per_page', String(perPage))
	if (filters.q.trim() !== '') params.set('q', filters.q.trim())
	for (const tag of filters.tags) params.append('tag', tag)
	if (filters.projectId) params.set('project_id', filters.projectId)
	if (filters.toSort) params.set('to_sort', 'true')
	return params
}
