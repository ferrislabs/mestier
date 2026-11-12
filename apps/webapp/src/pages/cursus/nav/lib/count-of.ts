interface ListResponse {
	data: readonly unknown[]
	pagination?: { total?: number | null } | null
}

export function countOf(
	response: ListResponse | undefined,
): number | undefined {
	if (!response) return undefined
	return response.pagination?.total ?? response.data.length
}
