import type {
	DoeModel,
	DoeModelSection,
	DoeModelSectionKind,
} from '#/hooks/use-doe-models'
import type { SectionKind } from '#/pages/doe/types'
import type { ModelDraft, ModelSectionDraft } from '#/pages/doe-models/types'
import {
	completeStyle,
	isStyleValid,
	normalizeStyle,
} from '#/pages/doe-style/lib/style'

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
	return { id: null, name: '', description: '', sections: [], style: null }
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
	model: Pick<DoeModel, 'name' | 'description' | 'sections' | 'style'> & {
		id: string
	},
	keyFor: (index: number) => string,
): ModelDraft {
	return {
		id: model.id,
		name: model.name,
		description: model.description ?? '',
		sections: model.sections.map((section, index) =>
			sectionToDraft(section, keyFor(index)),
		),
		style: model.style ? completeStyle(model.style) : null,
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
		style: draft.style ? normalizeStyle(draft.style) : null,
	}
}

export function canSubmitModel(draft: ModelDraft): boolean {
	return (
		draft.name.trim() !== '' &&
		draft.sections.every((section) => section.title.trim() !== '') &&
		(draft.style === null || isStyleValid(draft.style))
	)
}

export interface ModelErrors {
	name: string | undefined
	sections: boolean[]
	messages: string[]
}

export function modelErrors(draft: ModelDraft): ModelErrors {
	const name =
		draft.name.trim() === '' ? 'Le nom du modèle est obligatoire.' : undefined
	const sections = draft.sections.map((section) => section.title.trim() === '')
	const messages = [
		...(name ? [name] : []),
		...(sections.some(Boolean) ? ['Chaque thème doit avoir un titre.'] : []),
	]
	return { name, sections, messages }
}

function comparable(draft: ModelDraft) {
	return {
		name: draft.name.trim(),
		description: draft.description.trim(),
		style: draft.style,
		sections: draft.sections.map((section) => ({
			title: section.title.trim(),
			enabled: section.enabled,
			kind: section.kind,
			makerLabel: section.makerLabel,
			perPage: section.perPage,
			count: section.count,
		})),
	}
}

export function sameModelDraft(a: ModelDraft, b: ModelDraft): boolean {
	return JSON.stringify(comparable(a)) === JSON.stringify(comparable(b))
}

export function canSaveModel(draft: ModelDraft, stored: ModelDraft): boolean {
	return !sameModelDraft(draft, stored) && canSubmitModel(draft)
}
