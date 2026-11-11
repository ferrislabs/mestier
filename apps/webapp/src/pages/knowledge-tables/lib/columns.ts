import type {
	ColumnConfig,
	ColumnKind,
	FilterOperator,
	LinkKind,
	SelectColor,
	SelectOption,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'

export const COLUMN_KINDS: readonly { kind: ColumnKind; label: string }[] = [
	{ kind: 'text', label: 'Texte' },
	{ kind: 'long_text', label: 'Texte long' },
	{ kind: 'number', label: 'Nombre' },
	{ kind: 'currency', label: 'Montant' },
	{ kind: 'date', label: 'Date' },
	{ kind: 'checkbox', label: 'Case à cocher' },
	{ kind: 'single_select', label: 'Choix unique' },
	{ kind: 'multi_select', label: 'Choix multiple' },
	{ kind: 'link', label: 'Lien' },
	{ kind: 'file', label: 'Fichier du Drive' },
	{ kind: 'member', label: 'Membre' },
]

export function kindLabel(kind: ColumnKind): string {
	return COLUMN_KINDS.find((entry) => entry.kind === kind)?.label ?? kind
}

export const LINK_KINDS: readonly { kind: LinkKind; label: string }[] = [
	{ kind: 'customer', label: 'Client' },
	{ kind: 'project', label: 'Projet' },
	{ kind: 'quote', label: 'Devis' },
]

export function linkKindLabel(kind: LinkKind): string {
	return LINK_KINDS.find((entry) => entry.kind === kind)?.label ?? kind
}

export const SELECT_COLORS: readonly SelectColor[] = [
	'gray',
	'red',
	'orange',
	'yellow',
	'green',
	'teal',
	'blue',
	'purple',
	'pink',
]

export const SELECT_COLOR_LABELS: Record<SelectColor, string> = {
	gray: 'Gris',
	red: 'Rouge',
	orange: 'Orange',
	yellow: 'Jaune',
	green: 'Vert',
	teal: 'Turquoise',
	blue: 'Bleu',
	purple: 'Violet',
	pink: 'Rose',
}

export const SELECT_COLOR_CLASSES: Record<SelectColor, string> = {
	gray: 'bg-zinc-100 text-zinc-800 dark:bg-zinc-800 dark:text-zinc-100',
	red: 'bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-200',
	orange:
		'bg-orange-100 text-orange-800 dark:bg-orange-950 dark:text-orange-200',
	yellow:
		'bg-yellow-100 text-yellow-800 dark:bg-yellow-950 dark:text-yellow-200',
	green: 'bg-green-100 text-green-800 dark:bg-green-950 dark:text-green-200',
	teal: 'bg-teal-100 text-teal-800 dark:bg-teal-950 dark:text-teal-200',
	blue: 'bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-200',
	purple:
		'bg-purple-100 text-purple-800 dark:bg-purple-950 dark:text-purple-200',
	pink: 'bg-pink-100 text-pink-800 dark:bg-pink-950 dark:text-pink-200',
}

export function nextOptionColor(
	options: readonly { color?: SelectColor | null }[],
): SelectColor {
	return SELECT_COLORS[options.length % SELECT_COLORS.length]
}

export function columnOptions(column: {
	config: ColumnConfig
}): SelectOption[] {
	const options = column.config.options ?? []
	return options.flatMap((option) =>
		option.id
			? [{ id: option.id, label: option.label, color: option.color ?? 'gray' }]
			: [],
	)
}

export const DECIMAL_CHOICES: readonly number[] = [0, 1, 2, 3, 4]

export function defaultConfig(kind: ColumnKind): ColumnConfig {
	switch (kind) {
		case 'number':
			return { decimals: 0 }
		case 'link':
			return { link_kind: 'customer' }
		case 'single_select':
		case 'multi_select':
			return { options: [] }
		default:
			return {}
	}
}

const TEXT_OPERATORS: FilterOperator[] = [
	'contains',
	'is',
	'is_empty',
	'is_not_empty',
]
const NUMBER_OPERATORS: FilterOperator[] = [
	'eq',
	'ne',
	'lt',
	'le',
	'gt',
	'ge',
	'is_empty',
	'is_not_empty',
]
const REFERENCE_OPERATORS: FilterOperator[] = ['is', 'is_empty', 'is_not_empty']

export function filterOperators(kind: ColumnKind): FilterOperator[] {
	switch (kind) {
		case 'text':
		case 'long_text':
			return TEXT_OPERATORS
		case 'number':
		case 'currency':
			return NUMBER_OPERATORS
		case 'date':
			return ['before', 'after', 'on', 'is_empty', 'is_not_empty']
		case 'checkbox':
			return ['checked', 'unchecked']
		case 'single_select':
			return ['is', 'is_not', 'any_of', 'is_empty']
		case 'multi_select':
			return ['any_of', 'all_of', 'is_empty']
		case 'link':
		case 'file':
		case 'member':
			return REFERENCE_OPERATORS
	}
}

const OPERATOR_LABELS: Record<FilterOperator, string> = {
	contains: 'contient',
	is: 'est',
	is_not: 'n’est pas',
	any_of: 'parmi',
	all_of: 'contient tous',
	is_empty: 'est vide',
	is_not_empty: 'n’est pas vide',
	eq: '=',
	ne: '≠',
	lt: '<',
	le: '≤',
	gt: '>',
	ge: '≥',
	before: 'avant le',
	after: 'après le',
	on: 'le',
	checked: 'cochée',
	unchecked: 'non cochée',
}

export function operatorLabel(
	kind: ColumnKind,
	operator: FilterOperator,
): string {
	if (kind === 'multi_select' && operator === 'any_of') {
		return 'contient l’un de'
	}
	return OPERATOR_LABELS[operator]
}

export function operatorNeedsValue(operator: FilterOperator): boolean {
	return !['is_empty', 'is_not_empty', 'checked', 'unchecked'].includes(
		operator,
	)
}

export function isSortable(kind: ColumnKind): boolean {
	return kind !== 'link' && kind !== 'file' && kind !== 'member'
}

export function conversionTargets(
	column: Pick<TableColumn, 'kind' | 'is_primary'>,
): ColumnKind[] {
	if (column.is_primary) return []
	switch (column.kind) {
		case 'text':
			return ['long_text']
		case 'long_text':
			return ['text']
		case 'single_select':
			return ['text', 'long_text', 'multi_select']
		case 'multi_select':
			return ['text', 'long_text', 'single_select']
		default:
			return ['text', 'long_text']
	}
}

export function primaryColumn(
	columns: readonly TableColumn[],
): TableColumn | null {
	return columns.find((column) => column.is_primary) ?? columns[0] ?? null
}

export function orderedColumns(columns: readonly TableColumn[]): TableColumn[] {
	return [...columns].sort((a, b) => {
		if (a.is_primary !== b.is_primary) return a.is_primary ? -1 : 1
		return a.position - b.position
	})
}
