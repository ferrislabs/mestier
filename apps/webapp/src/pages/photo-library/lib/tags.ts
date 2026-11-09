export const MAX_TAGS = 20
export const MAX_TAG_LENGTH = 40

export function normalizeTag(raw: string): string {
	return raw.trim().replace(/\s+/g, ' ').toLowerCase()
}

export function normalizeTags(raw: readonly string[]): string[] {
	const seen = new Set<string>()
	const tags: string[] = []
	for (const entry of raw) {
		const tag = normalizeTag(entry)
		if (tag === '' || seen.has(tag)) continue
		seen.add(tag)
		tags.push(tag)
	}
	return tags
}

export function parseTagInput(input: string): string[] {
	return normalizeTags(input.split(/[,;\n]/))
}

export function addTags(current: readonly string[], input: string): string[] {
	return normalizeTags([...current, ...parseTagInput(input)])
}

export function removeTag(current: readonly string[], tag: string): string[] {
	return current.filter((entry) => entry !== tag)
}

export function tagsError(tags: readonly string[]): string | null {
	if (tags.length > MAX_TAGS) {
		return `Une photo porte ${MAX_TAGS} étiquettes au plus.`
	}
	if (tags.some((tag) => tag.length > MAX_TAG_LENGTH)) {
		return `Une étiquette fait ${MAX_TAG_LENGTH} caractères au plus.`
	}
	return null
}
