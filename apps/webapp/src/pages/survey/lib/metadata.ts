export interface MetadataEntry {
	id: string
	key: string
	value: string
}

let sequence = 0

export function newMetadataEntry(key = '', value = ''): MetadataEntry {
	sequence += 1
	return { id: `meta-${sequence}`, key, value }
}

export function entriesOf(
	record: Readonly<Record<string, string>> | null | undefined,
): MetadataEntry[] {
	return Object.entries(record ?? {}).map(([key, value]) =>
		newMetadataEntry(key, value),
	)
}

function isBlank(entry: MetadataEntry): boolean {
	return entry.key.trim() === '' && entry.value.trim() === ''
}

export function recordOf(
	entries: readonly MetadataEntry[],
): Record<string, string> {
	return Object.fromEntries(
		entries
			.filter((entry) => !isBlank(entry))
			.map((entry) => [entry.key.trim(), entry.value.trim()]),
	)
}

export function metadataErrors(entries: readonly MetadataEntry[]): string[] {
	const filled = entries.filter((entry) => !isBlank(entry))
	const errors: string[] = []
	if (filled.some((entry) => entry.key.trim() === '')) {
		errors.push('Une métadonnée n’a pas de clé.')
	}
	if (filled.some((entry) => entry.value.trim() === '')) {
		errors.push('Une métadonnée n’a pas de valeur.')
	}
	const keys = filled.map((entry) => entry.key.trim()).filter(Boolean)
	if (new Set(keys).size !== keys.length) {
		errors.push('Deux métadonnées portent la même clé.')
	}
	return errors
}
