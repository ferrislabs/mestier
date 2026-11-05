export const BADGE_MAX_LENGTH = 24

export function normalizeBadge(label: string): string {
	return label.replace(/\s+/g, ' ').trim()
}

export function sameBadge(a: string, b: string): boolean {
	return a.toLocaleLowerCase('fr') === b.toLocaleLowerCase('fr')
}
