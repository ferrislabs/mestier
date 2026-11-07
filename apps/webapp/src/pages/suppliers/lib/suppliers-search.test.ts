import { describe, expect, it } from 'vitest'
import {
	parseSuppliersSearch,
	suppliersSearchOf,
} from '#/pages/suppliers/lib/suppliers-search'

describe('suppliers search', () => {
	it('reads the archived flag', () => {
		expect(parseSuppliersSearch({ archives: 1 })).toEqual({ archives: 1 })
		expect(parseSuppliersSearch({ archives: '1' })).toEqual({ archives: 1 })
		expect(parseSuppliersSearch({ archives: true })).toEqual({ archives: 1 })
		expect(parseSuppliersSearch({ archives: 0 })).toEqual({})
		expect(parseSuppliersSearch({})).toEqual({})
	})

	it('writes the flag only when archived', () => {
		expect(suppliersSearchOf(true)).toEqual({ archives: 1 })
		expect(suppliersSearchOf(false)).toEqual({})
	})
})
