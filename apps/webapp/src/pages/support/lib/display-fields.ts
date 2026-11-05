export function candidateFieldKeys(
	ticketFieldKeys: readonly (readonly string[])[],
	displayFields: readonly string[],
): string[] {
	const keys = new Set<string>(displayFields)
	for (const group of ticketFieldKeys) {
		for (const key of group) keys.add(key)
	}

	return [...keys].sort((left, right) => left.localeCompare(right, 'fr'))
}

export function toggleDisplayField(
	displayFields: readonly string[],
	candidates: readonly string[],
	key: string,
	checked: boolean,
): string[] {
	if (!checked) return displayFields.filter((entry) => entry !== key)

	return candidates.filter(
		(entry) => entry === key || displayFields.includes(entry),
	)
}
