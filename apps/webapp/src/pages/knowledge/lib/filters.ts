import { parseTagInput } from '#/pages/photo-library/lib/tags'

export interface KnowledgeFilters {
	q: string
	tags: string[]
}

export const PAGE_SIZE = 25

/**
 * Where the articles come from: the folder open in the drive (`null` is the
 * root), alone or with every folder below it. The root with its whole tree is
 * the whole drive.
 */
export interface FolderScope {
	folderId: string | null
	recursive: boolean
}

export function hasFilters(filters: KnowledgeFilters): boolean {
	return filters.q.trim() !== '' || filters.tags.length > 0
}

export function filtersFromInputs(inputs: {
	q: string
	tagInput: string
}): KnowledgeFilters {
	return { q: inputs.q.trim(), tags: parseTagInput(inputs.tagInput) }
}

export function listSearchParams(
	filters: KnowledgeFilters,
	page: number,
	perPage: number = PAGE_SIZE,
	scope: FolderScope | null = null,
): URLSearchParams {
	const params = new URLSearchParams()
	params.set('page', String(page))
	params.set('per_page', String(perPage))
	if (scope && !(scope.folderId === null && scope.recursive)) {
		params.set('folder', scope.folderId ?? 'root')
		if (scope.recursive) params.set('recursive', 'true')
	}
	if (filters.q.trim() !== '') params.set('q', filters.q.trim())
	for (const tag of filters.tags) params.append('tag', tag)
	return params
}
