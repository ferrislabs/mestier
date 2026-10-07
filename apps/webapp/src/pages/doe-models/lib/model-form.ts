import type {
	DoeModel,
	DoeModelSection,
	DoeModelSectionKind,
} from '#/hooks/use-doe-models'
import type { SectionKind } from '#/pages/doe/types'
import type { ModelDraft, ModelSectionDraft } from '#/pages/doe-models/types'

export const MIN_BLANK_PAGES = 1
export const MAX_BLANK_PAGES = 20

export function newModelSectionDraft(
	key: string,
	title: string,
	kind: SectionKind,
): ModelSectionDraft {
	return {
		key,
		title,
		enabled: true,
		kind,
		makerLabel: 'manufacturer',
		perPage: 'two',
		count: 1,
	}
}

export function emptyModelDraft(): ModelDraft {
	return { id: null, name: '', description: '', sections: [] }
}

export function sectionToDraft(
	section: DoeModelSection,
	key: string,
): ModelSectionDraft {
	const base = newModelSectionDraft(key, section.title, section.kind.kind)
	const draft = { ...base, enabled: section.enabled }
	switch (section.kind.kind) {
		case 'supplies':
			return { ...draft, makerLabel: section.kind.maker_label }
		case 'photos':
			return { ...draft, perPage: section.kind.per_page }
		case 'blank_pages':
			return { ...draft, count: section.kind.count }
		case 'plans':
			return draft
	}
}

export function modelToDraft(
	model: Pick<DoeModel, 'name' | 'description' | 'sections'> & { id: string },
	keyFor: (index: number) => string,
): ModelDraft {
	return {
		id: model.id,
		name: model.name,
		description: model.description ?? '',
		sections: model.sections.map((section, index) =>
			sectionToDraft(section, keyFor(index)),
		),
	}
}

function kindFromDraft(section: ModelSectionDraft): DoeModelSectionKind {
	switch (section.kind) {
		case 'supplies':
			return { kind: 'supplies', maker_label: section.makerLabel }
		case 'photos':
			return { kind: 'photos', per_page: section.perPage }
		case 'plans':
			return { kind: 'plans' }
		case 'blank_pages':
			return {
				kind: 'blank_pages',
				count: Math.min(
					MAX_BLANK_PAGES,
					Math.max(MIN_BLANK_PAGES, section.count),
				),
			}
	}
}

export function buildModelRequest(draft: ModelDraft) {
	const description = draft.description.trim()
	return {
		name: draft.name.trim(),
		description: description === '' ? null : description,
		sections: draft.sections.map(
			(section): DoeModelSection => ({
				title: section.title.trim(),
				enabled: section.enabled,
				kind: kindFromDraft(section),
			}),
		),
	}
}

export function canSubmitModel(draft: ModelDraft): boolean {
	return (
		draft.name.trim() !== '' &&
		draft.sections.every((section) => section.title.trim() !== '')
	)
}
