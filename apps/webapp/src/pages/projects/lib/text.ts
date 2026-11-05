export function plural(count: number, one: string, many: string): string {
	return count > 1 ? many : one
}

export function countLabel(count: number, one: string, many: string): string {
	return `${count} ${plural(count, one, many)}`
}

export function normalizeText(text: string): string {
	return text.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim()
}
