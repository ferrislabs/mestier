function isPlainObject(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function flattenInto(
	target: Record<string, string>,
	prefix: string,
	value: unknown,
): void {
	if (isPlainObject(value)) {
		for (const [key, child] of Object.entries(value)) {
			flattenInto(target, prefix ? `${prefix}.${key}` : key, child)
		}
		return
	}
	target[prefix] = typeof value === 'string' ? value : JSON.stringify(value)
}

export function flattenItem(item: unknown): Record<string, string> {
	const row: Record<string, string> = {}
	flattenInto(row, '', item)
	return row
}

export function collectColumns(
	rows: readonly Record<string, string>[],
): string[] {
	const columns = new Set<string>()
	for (const row of rows) {
		for (const key of Object.keys(row)) columns.add(key)
	}
	return [...columns]
}

export function toItems(value: unknown): unknown[] {
	if (value === undefined || value === null) return []
	return Array.isArray(value) ? value : [value]
}
