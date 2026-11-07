export type SuppliersSearch = { archives?: 1 }

export function parseSuppliersSearch(
	raw: Record<string, unknown>,
): SuppliersSearch {
	return raw.archives === 1 || raw.archives === '1' || raw.archives === true
		? { archives: 1 }
		: {}
}

export function suppliersSearchOf(archived: boolean): SuppliersSearch {
	return archived ? { archives: 1 } : {}
}
