import { describe, expect, it } from 'vitest'

describe('the treasury, suppliers and catalogue sources', () => {
	const sources = import.meta.glob(
		[
			'/src/pages/treasury/**/*.{ts,tsx}',
			'/src/pages/suppliers/**/*.{ts,tsx}',
			'/src/pages/catalog/**/*.{ts,tsx}',
		],
		{
			query: '?raw',
			import: 'default',
			eager: true,
		},
	) as Record<string, string>

	const production = Object.entries(sources).filter(
		([path]) => !path.endsWith('.test.tsx') && !path.endsWith('.test.ts'),
	)

	it('read no localStorage', () => {
		const offenders = production
			.filter(([, source]) => source.includes('localStorage'))
			.map(([path]) => path)

		expect(production.length).toBeGreaterThan(30)
		expect(offenders).toEqual([])
	})

	it('import no ledger, supplier-extras, catalog-extras or catalog-usage mock', () => {
		const offenders = production
			.filter(
				([, source]) =>
					source.includes('mocks/ledger') ||
					source.includes('supplier-extras') ||
					source.includes('catalog-extras') ||
					source.includes('catalog-usage') ||
					source.includes('mocks/people'),
			)
			.map(([path]) => path)

		expect(offenders).toEqual([])
	})

	it('compute no journal, summary, trend or VAT on the client', () => {
		const offenders = production
			.filter(([, source]) =>
				/\b(summarizeLedger|monthlyTrend|vatSummary|receiptVat|ledgerEntries|DEFAULT_RECEIPT_VAT_RATE)\b/.test(
					source,
				),
			)
			.map(([path]) => path)

		expect(offenders).toEqual([])
	})
})
