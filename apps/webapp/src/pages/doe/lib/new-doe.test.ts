import { describe, expect, it } from 'vitest'
import { doeTabLabel } from '#/pages/doe/lib/new-doe'

describe('doeTabLabel', () => {
	it('shows the revision once the project has a DOE', () => {
		expect(doeTabLabel({ revision: 'B' })).toBe('DOE · indice B')
	})

	it('stays plain while loading or when there is no DOE yet', () => {
		expect(doeTabLabel('none')).toBe('DOE')
		expect(doeTabLabel('loading')).toBe('DOE')
	})
})
