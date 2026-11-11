import { describe, expect, it } from 'vitest'
import { tableSearchSchema } from '#/pages/knowledge-tables/lib/search'

describe('tableSearchSchema', () => {
	it('keeps a view id', () => {
		const vue = '0193b7a2-5d4e-7c1f-8a2b-3c4d5e6f7a8b'
		expect(tableSearchSchema.parse({ vue })).toEqual({ vue })
	})

	it('drops a hand-edited value rather than failing', () => {
		expect(tableSearchSchema.parse({ vue: 'nimporte' })).toEqual({
			vue: undefined,
		})
		expect(tableSearchSchema.parse({})).toEqual({})
	})
})
