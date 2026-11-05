import { describe, expect, it } from 'vitest'
import type { Project } from '#/hooks/use-projects'
import { switcherOptions } from '#/pages/projet/lib/switcher-options'

function project(overrides: Partial<Project>): Project {
	return { id: 'p', name: 'P', ...overrides } as Project
}

describe('switcherOptions', () => {
	const customers = [{ id: 'c1', name: 'Commune de Lapalisse' }]

	it('puts the archived projects last, each group sorted by name', () => {
		const options = switcherOptions(
			[
				project({
					id: 'a',
					name: 'Zèbre',
					archived_at: '2026-01-01T00:00:00Z',
				}),
				project({ id: 'b', name: 'Étable' }),
				project({ id: 'c', name: 'Atelier' }),
			],
			customers,
		)

		expect(options.map((option) => option.id)).toEqual(['c', 'b', 'a'])
		expect(options.map((option) => option.archived)).toEqual([
			false,
			false,
			true,
		])
	})

	it('names the owner, or none for an internal project', () => {
		const options = switcherOptions(
			[
				project({ id: 'a', name: 'A', customer_id: 'c1' }),
				project({ id: 'b', name: 'B' }),
				project({ id: 'c', name: 'C', customer_id: 'unknown' }),
			],
			customers,
		)

		expect(options.map((option) => option.ownerName)).toEqual([
			'Commune de Lapalisse',
			null,
			'Client',
		])
	})
})
