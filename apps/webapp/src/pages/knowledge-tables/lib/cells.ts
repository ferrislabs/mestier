import { columnOptions } from '#/pages/knowledge-tables/lib/columns'
import {
	centsInputText,
	formatCents,
	formatIsoDate,
	formatNumber,
	numberInputText,
	type Parsed,
	parseDateInput,
	parseEurosInput,
	parseNumberInput,
} from '#/pages/knowledge-tables/lib/format'
import type {
	CellLabel,
	ColumnKind,
	LinkValue,
	TableColumn,
	TableRow,
} from '#/pages/knowledge-tables/lib/types'

export const UNTITLED = 'Sans titre'
export const DELETED_ELEMENT = 'Élément supprimé'
export const TEXT_MAX_CHARS = 1000
export const LONG_TEXT_MAX_CHARS = 20000

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function isEmptyValue(value: unknown): boolean {
	if (value === undefined || value === null) return true
	if (typeof value === 'string') return value.trim() === ''
	if (Array.isArray(value)) return value.length === 0
	return false
}

export function cellLabel(row: TableRow, columnId: string): CellLabel | null {
	const entry = row.labels[columnId]
	if (!isRecord(entry)) return null
	return {
		label: typeof entry.label === 'string' ? entry.label : null,
		missing: entry.missing === true,
	}
}

export function linkValue(value: unknown): LinkValue | null {
	if (!isRecord(value)) return null
	const { kind, id } = value
	if (typeof id !== 'string') return null
	if (kind !== 'customer' && kind !== 'project' && kind !== 'quote') {
		return null
	}
	return { kind, id }
}

export function optionIds(value: unknown): string[] {
	if (Array.isArray(value)) {
		return value.filter((item): item is string => typeof item === 'string')
	}
	return typeof value === 'string' ? [value] : []
}

export function displayCell(
	column: TableColumn,
	value: unknown,
	label: CellLabel | null,
): string {
	if (isEmptyValue(value)) return ''
	switch (column.kind) {
		case 'text':
		case 'long_text':
			return typeof value === 'string' ? value : String(value)
		case 'number':
			return typeof value === 'number'
				? formatNumber(value, column.config.decimals)
				: String(value)
		case 'currency':
			return typeof value === 'number' ? formatCents(value) : String(value)
		case 'date':
			return typeof value === 'string' ? formatIsoDate(value) : String(value)
		case 'checkbox':
			return value === true ? 'Oui' : 'Non'
		case 'single_select':
		case 'multi_select': {
			const options = columnOptions(column)
			return optionIds(value)
				.map((id) => options.find((option) => option.id === id)?.label)
				.filter((text): text is string => Boolean(text))
				.join(', ')
		}
		case 'link':
		case 'file':
		case 'member':
			if (label?.missing) return DELETED_ELEMENT
			return label?.label ?? ''
	}
}

export function rowTitle(row: TableRow, primary: TableColumn | null): string {
	if (!primary) return UNTITLED
	const value = row.values[primary.id]
	return typeof value === 'string' && value.trim() !== ''
		? value.trim()
		: UNTITLED
}

export function cellEditText(column: TableColumn, value: unknown): string {
	if (isEmptyValue(value)) return ''
	switch (column.kind) {
		case 'number':
			return typeof value === 'number' ? numberInputText(value) : ''
		case 'currency':
			return typeof value === 'number' ? centsInputText(value) : ''
		case 'date':
			return typeof value === 'string' ? formatIsoDate(value) : ''
		default:
			return typeof value === 'string' ? value : ''
	}
}

function parseText(text: string, max: number): Parsed<unknown> {
	if (text.trim() === '') return { kind: 'empty' }
	if ([...text].length > max) return { kind: 'invalid' }
	return { kind: 'value', value: text }
}

export function parseCellText(kind: ColumnKind, text: string): Parsed<unknown> {
	switch (kind) {
		case 'text':
			return parseText(text, TEXT_MAX_CHARS)
		case 'long_text':
			return parseText(text, LONG_TEXT_MAX_CHARS)
		case 'number':
			return parseNumberInput(text)
		case 'currency':
			return parseEurosInput(text)
		case 'date':
			return parseDateInput(text)
		default:
			return { kind: 'invalid' }
	}
}

export function invalidCellMessage(kind: ColumnKind): string {
	switch (kind) {
		case 'text':
			return `Un texte compte ${TEXT_MAX_CHARS} caractères au plus.`
		case 'long_text':
			return `Un texte long compte ${LONG_TEXT_MAX_CHARS} caractères au plus.`
		case 'number':
			return 'Saisis un nombre, par exemple 12,5.'
		case 'currency':
			return 'Saisis un montant en euros, par exemple 1 250,50.'
		case 'date':
			return 'Saisis une date au format JJ/MM/AAAA.'
		default:
			return 'Cette valeur ne convient pas à la colonne.'
	}
}

export function referenceId(
	column: TableColumn,
	value: unknown,
): string | null {
	if (column.kind === 'link') return linkValue(value)?.id ?? null
	return typeof value === 'string' && value !== '' ? value : null
}

export function referenceValue(
	column: TableColumn,
	id: string | null,
): unknown {
	if (id === null) return null
	if (column.kind === 'link') {
		return { kind: column.config.link_kind ?? 'customer', id }
	}
	return id
}
