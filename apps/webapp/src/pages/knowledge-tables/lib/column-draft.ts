import {
	COLUMN_KINDS,
	conversionTargets,
	nextOptionColor,
} from '#/pages/knowledge-tables/lib/columns'
import type {
	ColumnConfig,
	ColumnKind,
	LinkKind,
	SelectColor,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'

export interface OptionDraft {
	key: string
	id: string | null
	label: string
	color: SelectColor
}

export interface ColumnDraft {
	name: string
	kind: ColumnKind
	decimals: number
	linkKind: LinkKind
	options: OptionDraft[]
}

export const EMPTY_COLUMN_DRAFT: ColumnDraft = {
	name: '',
	kind: 'text',
	decimals: 0,
	linkKind: 'customer',
	options: [],
}

let optionKeys = 0

function newKey(): string {
	optionKeys += 1
	return `option-${optionKeys}`
}

export function draftFromColumn(column: TableColumn): ColumnDraft {
	return {
		name: column.name,
		kind: column.kind,
		decimals: column.config.decimals ?? 0,
		linkKind: column.config.link_kind ?? 'customer',
		options: (column.config.options ?? []).map((option) => ({
			key: option.id ?? newKey(),
			id: option.id ?? null,
			label: option.label,
			color: option.color ?? 'gray',
		})),
	}
}

export function addOption(draft: ColumnDraft, label: string): ColumnDraft {
	return {
		...draft,
		options: [
			...draft.options,
			{ key: newKey(), id: null, label, color: nextOptionColor(draft.options) },
		],
	}
}

export function configFromDraft(draft: ColumnDraft): ColumnConfig {
	switch (draft.kind) {
		case 'number':
			return { decimals: draft.decimals }
		case 'link':
			return { link_kind: draft.linkKind }
		case 'single_select':
		case 'multi_select':
			return {
				options: draft.options.map((option) =>
					option.id
						? { id: option.id, label: option.label.trim(), color: option.color }
						: { label: option.label.trim(), color: option.color },
				),
			}
		default:
			return {}
	}
}

export interface ColumnUpdate {
	name?: string
	kind?: ColumnKind
	config?: ColumnConfig
}

export function columnUpdate(
	column: TableColumn,
	draft: ColumnDraft,
): ColumnUpdate {
	const update: ColumnUpdate = {}
	const name = draft.name.trim()
	if (name !== column.name) update.name = name
	const config = configFromDraft(draft)
	const before = configFromDraft(draftFromColumn(column))
	if (draft.kind !== column.kind) {
		update.kind = draft.kind
		update.config = config
	} else if (JSON.stringify(config) !== JSON.stringify(before)) {
		update.config = config
	}
	return update
}

export function columnDraftError(draft: ColumnDraft): string | null {
	if (draft.name.trim() === '') return 'Donne un nom à la colonne.'
	if (draft.kind !== 'single_select' && draft.kind !== 'multi_select') {
		return null
	}
	const labels = draft.options.map((option) =>
		option.label.trim().toLocaleLowerCase('fr'),
	)
	if (labels.some((label) => label === '')) {
		return 'Chaque option a besoin d’un libellé.'
	}
	if (new Set(labels).size !== labels.length) {
		return 'Deux options portent le même libellé.'
	}
	return null
}

export function kindChoices(column: TableColumn | null): ColumnKind[] {
	if (!column) return COLUMN_KINDS.map((entry) => entry.kind)
	return [column.kind, ...conversionTargets(column)]
}
