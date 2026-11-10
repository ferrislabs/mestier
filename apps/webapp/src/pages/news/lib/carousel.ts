export function clampIndex(index: number, length: number): number {
	if (length <= 0) return 0
	return Math.min(Math.max(index, 0), length - 1)
}

export function nextIndex(index: number, length: number): number {
	return clampIndex(index + 1, length)
}

export function prevIndex(index: number, length: number): number {
	return clampIndex(index - 1, length)
}

export function indexForKey(
	key: string,
	index: number,
	length: number,
): number | null {
	if (key === 'ArrowRight') return nextIndex(index, length)
	if (key === 'ArrowLeft') return prevIndex(index, length)
	return null
}
