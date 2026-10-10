import { describe, expect, it } from 'vitest'
import {
	canSubmitSupply,
	EMPTY_SUPPLY_FORM,
	formToSupply,
	supplyToForm,
} from '#/pages/doe/lib/supply-form'
import type { DoeSupply } from '#/pages/doe/types'

const SUPPLY: DoeSupply = {
	id: 's1',
	source_product_id: 'p1',
	designation: 'Banc Garden',
	supplier: { name: 'PLAS ECO', phone: '01' },
	photos: [{ key: 'k1', caption: 'Vue' }],
	datasheet_keys: ['d1'],
}

describe('supply form', () => {
	it('round-trips a supply without changing it', () => {
		const rebuilt = formToSupply(SUPPLY, supplyToForm(SUPPLY))
		expect(rebuilt.designation).toBe('Banc Garden')
		expect(rebuilt.source_product_id).toBe('p1')
		expect(rebuilt.supplier).toEqual({
			name: 'PLAS ECO',
			address: null,
			phone: '01',
			email: null,
		})
		expect(rebuilt.photos).toEqual([{ key: 'k1', caption: 'Vue' }])
	})

	it('drops the supplier snapshot when the name is blank', () => {
		const supply = formToSupply(
			{ id: 'n', source_product_id: null },
			{ ...EMPTY_SUPPLY_FORM, designation: ' Pot ', supplierPhone: '01' },
		)
		expect(supply.supplier).toBeNull()
		expect(supply.designation).toBe('Pot')
	})

	it('requires a designation', () => {
		expect(canSubmitSupply(EMPTY_SUPPLY_FORM)).toBe(false)
		expect(canSubmitSupply({ ...EMPTY_SUPPLY_FORM, designation: 'x' })).toBe(
			true,
		)
	})
})
