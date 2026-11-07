import { describe, expect, it } from 'vitest'

describe('the invoice screens sources', () => {
	const sources = import.meta.glob(
		[
			'/src/pages/invoices/**/*.{ts,tsx}',
			'/src/pages/credit-notes/**/*.{ts,tsx}',
		],
		{
			query: '?raw',
			import: 'default',
			eager: true,
		},
	) as Record<string, string>

	const production = Object.entries(sources).filter(
		([path]) =>
			!path.endsWith('.test.tsx') &&
			!path.endsWith('.test.ts') &&
			!path.includes('/test-support/'),
	)

	it('read no localStorage', () => {
		const offenders = production
			.filter(([, source]) => source.includes('localStorage'))
			.map(([path]) => path)

		expect(production.length).toBeGreaterThan(20)
		expect(offenders).toEqual([])
	})

	it('import no invoice-extras or invoice-activity mock', () => {
		const offenders = production
			.filter(
				([, source]) =>
					source.includes('invoice-extras') ||
					source.includes('invoice-activity') ||
					source.includes('mocks/'),
			)
			.map(([path]) => path)

		expect(offenders).toEqual([])
	})

	it('price no deposit and no full invoice on the client', () => {
		const offenders = production
			.filter(([, source]) =>
				/\b(depositLines|fullInvoiceLines|depositPreviewCents)\b/.test(source),
			)
			.map(([path]) => path)

		expect(offenders).toEqual([])
	})
})
