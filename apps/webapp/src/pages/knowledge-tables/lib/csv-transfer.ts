import type { TableImportReport } from '#/pages/knowledge-tables/lib/types'

export const MAX_CSV_BYTES = 5 * 1024 * 1024
export const CSV_ACCEPT = '.csv,.txt,text/csv,text/plain'
const MAX_NAME_CHARS = 100

const REASONS: Record<string, string> = {
	not_text: 'Texte attendu',
	text_too_long: 'Texte trop long pour la colonne',
	not_number: 'Nombre attendu',
	not_amount: 'Montant en euros attendu (2 décimales au plus)',
	not_date: 'Date attendue (JJ/MM/AAAA ou AAAA-MM-JJ)',
	not_boolean: 'Oui ou non attendu',
	unknown_option: 'Option inconnue dans la colonne',
	duplicate_option: 'Option donnée deux fois',
	not_option_list: 'Liste d’options attendue',
	not_link: 'Identifiant attendu',
	wrong_link_kind: 'Lien vers un autre genre d’élément',
	not_id: 'Identifiant attendu',
	unknown_target: 'Élément introuvable dans l’organisation',
}

const numbers = new Intl.NumberFormat('fr-FR')

function plural(count: number, one: string, many: string): string {
	return `${numbers.format(count)} ${count === 1 ? one : many}`
}

export function tableNameFromFile(fileName: string): string {
	const base = fileName.split(/[/\\]/).at(-1) ?? fileName
	const dot = base.lastIndexOf('.')
	const stem = dot > 0 ? base.slice(0, dot) : base
	return [...stem.trim()].slice(0, MAX_NAME_CHARS).join('')
}

export function csvFileRefusal(file: { size: number }): string | null {
	if (file.size === 0) return 'Le fichier est vide.'
	if (file.size > MAX_CSV_BYTES) return 'Le fichier dépasse 5 Mo.'
	return null
}

export function exportFileName(tableName: string, viewName: string): string {
	return `${tableName} - ${viewName}.csv`.replace(/[/\\]/g, '_')
}

export function invalidReasonLabel(reason: string): string {
	return REASONS[reason] ?? 'Valeur refusée'
}

export function importSummary(report: TableImportReport): string {
	const parts = [
		report.rows_created === 0
			? 'Aucune ligne importée'
			: plural(report.rows_created, 'ligne importée', 'lignes importées'),
	]
	if (report.ignored_columns.length > 0) {
		parts.push(
			plural(
				report.ignored_columns.length,
				'colonne ignorée',
				'colonnes ignorées',
			),
		)
	}
	if (report.invalid_cell_count > 0) {
		parts.push(
			plural(
				report.invalid_cell_count,
				'cellule laissée vide',
				'cellules laissées vides',
			),
		)
	}
	return `${parts.join(', ')}.`
}

export function moreInvalidCells(report: TableImportReport): number {
	return Math.max(0, report.invalid_cell_count - report.invalid_cells.length)
}

export function hiddenDiffers(
	hidden: readonly string[],
	saved: readonly string[],
): boolean {
	const kept = new Set(saved)
	return hidden.length !== saved.length || hidden.some((id) => !kept.has(id))
}
