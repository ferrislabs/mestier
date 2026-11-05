export function moveItem<T>(
	items: readonly T[],
	index: number,
	delta: -1 | 1,
): T[] {
	const target = index + delta
	if (
		target < 0 ||
		target >= items.length ||
		index < 0 ||
		index >= items.length
	) {
		return [...items]
	}
	const next = [...items]
	const current = items[index]
	const other = items[target]
	if (current === undefined || other === undefined) return next
	next[index] = other
	next[target] = current
	return next
}

export function duplicateItem<T>(
	items: readonly T[],
	index: number,
	copy: (source: T) => T,
): T[] {
	const source = items[index]
	if (source === undefined) return [...items]
	return [...items.slice(0, index + 1), copy(source), ...items.slice(index + 1)]
}

export function newLineId(count: number): string {
	return `line-${Date.now()}-${count}`
}
