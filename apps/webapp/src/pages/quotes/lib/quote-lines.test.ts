import { describe, expect, it } from 'vitest'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import { emptyQuoteLine } from '#/pages/quotes/types'
import { copyQuoteLine, lineFromCatalogItem } from './quote-lines'

const SERVICE: CatalogItem = {
	id: 'catalog-1',
	type: 'SERVICE',
	sourceId: 'rate-1',
	label: 'Pose de parquet',
	unit: 'SQUARE_METER',
	unitPriceCents: 4500,
	description: 'Chêne massif',
	defaultVatRateBp: 1000,
}

describe('lineFromCatalogItem', () => {
	it('prefills the line from a service', () => {
		const line = lineFromCatalogItem(SERVICE, 'line-9')

		expect(line).toMatchObject({
			clientId: 'line-9',
			catalogItemType: 'SERVICE',
			serviceRateId: 'rate-1',
			label: 'Pose de parquet',
			unit: 'SQUARE_METER',
			unitPrice: '45.00',
			vatRateBp: '1000',
			notes: 'Chêne massif',
			quantity: '1',
		})
	})

	it('does not link a product to a service rate', () => {
		const line = lineFromCatalogItem(
			{ ...SERVICE, type: 'PRODUCT', defaultVatRateBp: null },
			'line-9',
		)

		expect(line.serviceRateId).toBe('')
		expect(line.vatRateBp).toBe('')
	})
})

describe('copyQuoteLine', () => {
	it('copies the photos instead of sharing them', () => {
		const source = { ...emptyQuoteLine(), photoKeys: ['a'] }
		const copy = copyQuoteLine(source, 'line-2')

		expect(copy.clientId).toBe('line-2')
		expect(copy.photoKeys).toEqual(['a'])
		expect(copy.photoKeys).not.toBe(source.photoKeys)
	})
})
