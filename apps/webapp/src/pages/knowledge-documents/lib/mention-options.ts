import type { MentionKind } from '#/pages/knowledge-documents/lib/document-api-types'

export interface MentionOption {
	kind: MentionKind
	id: string
	label: string
}

export const MENTION_LIMIT = 8

export const MENTION_KIND_LABELS: Record<MentionKind, string> = {
	customer: 'Client',
	project: 'Projet',
	quote: 'Devis',
}

function fold(text: string): string {
	return text
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
}

export function filterMentionOptions(
	options: readonly MentionOption[],
	query: string,
	limit: number = MENTION_LIMIT,
): MentionOption[] {
	const needle = fold(query.trim())
	if (needle === '') return options.slice(0, limit)
	const starts: MentionOption[] = []
	const contains: MentionOption[] = []
	for (const option of options) {
		const label = fold(option.label)
		if (label.startsWith(needle)) starts.push(option)
		else if (label.includes(needle)) contains.push(option)
	}
	return [...starts, ...contains].slice(0, limit)
}
