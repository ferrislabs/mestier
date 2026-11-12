import { useState } from 'react'
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'
import type { ArchiveFilter } from '#/pages/cursus/shared/lib/list-filters'

export function useCatalogFilters() {
	const [archived, setArchived] = useState<ArchiveFilter>('active')
	const search = useDebouncedSearch('')

	return {
		filters: { archived, q: search.query },
		toolbar: {
			search: search.text,
			archived,
			onSearchChange: search.setText,
			onArchivedChange: setArchived,
			onResetFilters: () => {
				search.reset()
				setArchived('active')
			},
		},
	}
}
