import { describe, expect, it } from 'vitest'

describe('the quote screens sources', () => {
	const sources = import.meta.glob('/src/pages/quotes/**/*.{ts,tsx}', {
		query: '?raw',
		import: 'default',
		eager: true,
	}) as Record<string, string>

	const production = Object.entries(sources).filter(
		([path]) =>
			!path.endsWith('.test.tsx') &&
			!path.endsWith('.test.ts') &&
			!path.endsWith('quote-sources.test.ts'),
	)

	it('read no localStorage', () => {
		const offenders = production
			.filter(([, source]) => source.includes('localStorage'))
			.map(([path]) => path)

		expect(production.length).toBeGreaterThan(30)
		expect(offenders).toEqual([])
	})

	it('import no quote-extras mock', () => {
		const offenders = production
			.filter(
				([, source]) =>
					source.includes('quote-extras') ||
					source.includes('QuoteExtras') ||
					source.includes('mocks/'),
			)
			.map(([path]) => path)

		expect(offenders).toEqual([])
	})

	it('keep no client side recurring totals', () => {
		const offenders = production
			.filter(([, source]) => /recurringTotals\s*\(/.test(source))
			.map(([path]) => path)

		expect(offenders).toEqual([])
	})
})
