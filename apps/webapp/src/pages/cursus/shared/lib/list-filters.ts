export type ArchiveFilter = 'active' | 'archived' | 'all'

export interface FilterOption<Value extends string> {
	value: Value
	label: string
}

export const FEMININE_ARCHIVE_OPTIONS: FilterOption<ArchiveFilter>[] = [
	{ value: 'active', label: 'Actives' },
	{ value: 'archived', label: 'Archivées' },
	{ value: 'all', label: 'Toutes' },
]

export const MASCULINE_ARCHIVE_OPTIONS: FilterOption<ArchiveFilter>[] = [
	{ value: 'active', label: 'Actifs' },
	{ value: 'archived', label: 'Archivés' },
	{ value: 'all', label: 'Tous' },
]

export function parseArchiveSearch(
	raw: unknown,
): Exclude<ArchiveFilter, 'active'> | undefined {
	return raw === 'archived' || raw === 'all' ? raw : undefined
}

export function archiveQuery(filter: ArchiveFilter) {
	return filter === 'active' ? {} : { archived: filter }
}

export function searchQuery(q: string | undefined) {
	const trimmed = q?.trim() ?? ''
	return trimmed === '' ? {} : { q: trimmed }
}
