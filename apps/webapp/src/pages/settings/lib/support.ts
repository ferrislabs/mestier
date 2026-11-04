export const MAX_SUPPORT_DISPLAY_FIELDS = 8
export const MAX_SUPPORT_DISPLAY_FIELD_CHARS = 64

export type SupportDisplayFieldsParse =
	| { ok: true; fields: string[] }
	| { ok: false; error: string }

export function parseSupportDisplayFields(
	raw: string,
): SupportDisplayFieldsParse {
	const fields: string[] = []
	const seen = new Set<string>()

	for (const line of raw.split(/\r?\n/)) {
		const key = line.trim()
		if (key === '') continue

		if (Array.from(key).length > MAX_SUPPORT_DISPLAY_FIELD_CHARS) {
			return {
				ok: false,
				error: `La clé « ${key} » dépasse ${MAX_SUPPORT_DISPLAY_FIELD_CHARS} caractères.`,
			}
		}
		if (seen.has(key)) {
			return { ok: false, error: `La clé « ${key} » est listée deux fois.` }
		}
		seen.add(key)
		fields.push(key)
	}

	if (fields.length > MAX_SUPPORT_DISPLAY_FIELDS) {
		return { ok: false, error: 'Huit clés au plus.' }
	}

	return { ok: true, fields }
}

export function supportDisplayFieldsToText(fields: string[]): string {
	return fields.join('\n')
}
