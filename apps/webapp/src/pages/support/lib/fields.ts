export function displayFieldValue(customFields: unknown, key: string): string {
	if (
		typeof customFields !== 'object' ||
		customFields === null ||
		Array.isArray(customFields)
	) {
		return ''
	}

	const value = (customFields as Record<string, unknown>)[key]
	if (value === undefined || value === null) return ''
	if (typeof value === 'string') return value
	if (typeof value === 'number' || typeof value === 'boolean') {
		return String(value)
	}

	return JSON.stringify(value)
}

export function ticketNumberLabel(number: number): string {
	return `#${number}`
}
