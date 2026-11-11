import { describe, expect, it } from 'vitest'
import {
	CSV_ACCEPT,
	csvFileRefusal,
	exportFileName,
	hiddenDiffers,
	importSummary,
	invalidReasonLabel,
	MAX_CSV_BYTES,
	moreInvalidCells,
	tableNameFromFile,
} from '#/pages/knowledge-tables/lib/csv-transfer'
import type { TableImportReport } from '#/pages/knowledge-tables/lib/types'

function report(change: Partial<TableImportReport> = {}): TableImportReport {
	return {
		rows_created: 0,
		ignored_columns: [],
		invalid_cells: [],
		invalid_cell_count: 0,
		...change,
	}
}

describe('tableNameFromFile', () => {
	it('drops the extension and the folder', () => {
		expect(tableNameFromFile('Articles.csv')).toBe('Articles')
		expect(tableNameFromFile('C:\\Exports\\stock 2026.CSV')).toBe('stock 2026')
		expect(tableNameFromFile('inventaire.v2.csv')).toBe('inventaire.v2')
	})

	it('keeps a name without extension and bounds it to 100 characters', () => {
		expect(tableNameFromFile('Fournisseurs')).toBe('Fournisseurs')
		expect(tableNameFromFile(`${'a'.repeat(120)}.csv`)).toHaveLength(100)
		expect(tableNameFromFile('.csv')).toBe('.csv')
	})
})

describe('csvFileRefusal', () => {
	it('refuses a file over 5 MB before sending it', () => {
		expect(csvFileRefusal({ size: MAX_CSV_BYTES })).toBeNull()
		expect(csvFileRefusal({ size: MAX_CSV_BYTES + 1 })).toBe(
			'Le fichier dépasse 5 Mo.',
		)
		expect(csvFileRefusal({ size: 0 })).toBe('Le fichier est vide.')
	})

	it('accepts csv and text files in the picker', () => {
		expect(CSV_ACCEPT).toContain('.csv')
		expect(CSV_ACCEPT).toContain('text/csv')
	})
})

describe('exportFileName', () => {
	it('names the table and the view without path separators', () => {
		expect(exportFileName('Matériaux', 'En cours')).toBe(
			'Matériaux - En cours.csv',
		)
		expect(exportFileName('A/B', 'C\\D')).toBe('A_B - C_D.csv')
	})
})

describe('invalidReasonLabel', () => {
	it('says each reason in French and keeps an unknown one readable', () => {
		expect(invalidReasonLabel('not_date')).toBe(
			'Date attendue (JJ/MM/AAAA ou AAAA-MM-JJ)',
		)
		expect(invalidReasonLabel('unknown_option')).toBe(
			'Option inconnue dans la colonne',
		)
		expect(invalidReasonLabel('unknown_target')).toBe(
			'Élément introuvable dans l’organisation',
		)
		expect(invalidReasonLabel('something_new')).toBe('Valeur refusée')
	})
})

describe('importSummary', () => {
	it('counts the rows, the ignored columns and the invalid cells', () => {
		expect(importSummary(report({ rows_created: 1 }))).toBe('1 ligne importée.')
		expect(
			importSummary(
				report({
					rows_created: 1250,
					ignored_columns: ['Remarque', 'Code'],
					invalid_cell_count: 3,
				}),
			),
		).toBe(
			'1\u202f250 lignes importées, 2 colonnes ignorées, 3 cellules laissées vides.',
		)
		expect(importSummary(report())).toBe('Aucune ligne importée.')
	})
})

describe('moreInvalidCells', () => {
	it('counts the invalid cells the report does not list', () => {
		const listed = report({
			invalid_cells: [
				{
					row: 2,
					column: 'Début',
					value: '31/02/2026',
					reason: 'not_date',
					message: '',
				},
			],
			invalid_cell_count: 250,
		})
		expect(moreInvalidCells(listed)).toBe(249)
		expect(moreInvalidCells(report())).toBe(0)
	})
})

describe('hiddenDiffers', () => {
	it('compares the hidden columns whatever their order', () => {
		expect(hiddenDiffers(['a', 'b'], ['b', 'a'])).toBe(false)
		expect(hiddenDiffers(['a'], ['a', 'b'])).toBe(true)
		expect(hiddenDiffers([], [])).toBe(false)
	})
})
