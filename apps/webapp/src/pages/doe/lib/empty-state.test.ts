import { describe, expect, it } from 'vitest'
import { activeModels, chooseCreationMode } from '#/pages/doe/lib/empty-state'

describe('chooseCreationMode', () => {
	it('offers the starter when the organization has no model', () => {
		expect(chooseCreationMode([])).toBe('from-starter')
	})

	it('offers the starter when every model is archived', () => {
		expect(
			chooseCreationMode([{ id: 'm', archived_at: '2026-01-01T00:00:00Z' }]),
		).toBe('from-starter')
	})

	it('offers the model picker otherwise', () => {
		expect(chooseCreationMode([{ id: 'm', archived_at: null }])).toBe(
			'pick-model',
		)
	})
})

describe('activeModels', () => {
	it('drops archived models', () => {
		expect(
			activeModels([{ id: 'a' }, { id: 'b', archived_at: 'x' }]).map(
				(model) => model.id,
			),
		).toEqual(['a'])
	})
})
