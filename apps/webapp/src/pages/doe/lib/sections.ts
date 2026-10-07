import type {
	DoeSection,
	DoeSectionContent,
	DoeSupply,
	NumberedSection,
	SectionKind,
} from '#/pages/doe/types'

export function numberSections(sections: DoeSection[]): NumberedSection[] {
	let counter = 0
	return sections.map((section) => {
		if (!section.enabled) return { section, number: null }
		counter += 1
		return { section, number: counter }
	})
}

export function enabledNumberedSections(
	sections: DoeSection[],
): Array<{ section: DoeSection; number: number }> {
	const result: Array<{ section: DoeSection; number: number }> = []
	for (const entry of numberSections(sections)) {
		if (entry.number !== null) {
			result.push({ section: entry.section, number: entry.number })
		}
	}
	return result
}

export function supplyNumber(sectionNumber: number, index: number): string {
	return `${sectionNumber}.${index + 1}`
}

export function moveItem<T>(items: T[], index: number, delta: -1 | 1): T[] {
	const target = index + delta
	if (index < 0 || index >= items.length) return items
	if (target < 0 || target >= items.length) return items
	const next = [...items]
	const [moved] = next.splice(index, 1)
	next.splice(target, 0, moved as T)
	return next
}

export function defaultContent(kind: SectionKind): DoeSectionContent {
	switch (kind) {
		case 'supplies':
			return { kind: 'supplies', maker_label: 'manufacturer', supplies: [] }
		case 'photos':
			return { kind: 'photos', per_page: 'two', photos: [] }
		case 'plans':
			return { kind: 'plans', plans: [] }
		case 'blank_pages':
			return { kind: 'blank_pages', count: 1 }
	}
}

export function newSection(
	id: string,
	title: string,
	kind: SectionKind,
): DoeSection {
	return {
		id,
		title: title.trim(),
		enabled: true,
		content: defaultContent(kind),
	}
}

export function shortTitle(title: string, max = 18): string {
	const trimmed = title.trim()
	if (trimmed.length <= max) return trimmed
	return `${trimmed.slice(0, max - 1).trimEnd()}…`
}

export function updateSection(
	sections: DoeSection[],
	id: string,
	patch: Partial<DoeSection>,
): DoeSection[] {
	return sections.map((section) =>
		section.id === id ? { ...section, ...patch } : section,
	)
}

export function supplyBadgeCounts(supply: DoeSupply): {
	photos: number
	datasheets: number
	isEmpty: boolean
} {
	const photos = supply.photos.length
	const datasheets = supply.datasheet_keys.length
	return { photos, datasheets, isEmpty: photos === 0 && datasheets === 0 }
}

export function updateContent(
	sections: DoeSection[],
	id: string,
	update: (content: DoeSectionContent) => DoeSectionContent,
): DoeSection[] {
	return sections.map((section) =>
		section.id === id
			? { ...section, content: update(section.content) }
			: section,
	)
}

export function suppliesOf(sections: DoeSection[], id: string): DoeSupply[] {
	const found = sections.find((section) => section.id === id)
	return found?.content.kind === 'supplies' ? found.content.supplies : []
}

export function withSupplies(
	sections: DoeSection[],
	id: string,
	supplies: DoeSupply[],
): DoeSection[] {
	return updateContent(sections, id, (content) =>
		content.kind === 'supplies' ? { ...content, supplies } : content,
	)
}

export function upsertSupply(
	supplies: DoeSupply[],
	supply: DoeSupply,
): DoeSupply[] {
	const index = supplies.findIndex((entry) => entry.id === supply.id)
	if (index === -1) return [...supplies, supply]
	return supplies.map((entry, position) =>
		position === index ? supply : entry,
	)
}

/**
 * A section can be deleted only while it holds nothing, so deleting never
 * loses a supply, a photo or a plan. Whether it came from the model or was
 * added by hand does not matter, and is not known after a reload anyway.
 */
export function isSectionEmpty(section: DoeSection): boolean {
	const content = section.content
	switch (content.kind) {
		case 'supplies':
			return content.supplies.length === 0
		case 'photos':
			return content.photos.length === 0
		case 'plans':
			return content.plans.length === 0
		case 'blank_pages':
			return true
	}
}
