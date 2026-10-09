import { describe, expect, it } from 'vitest'
import {
	canCreateDoe,
	doeButtonLabel,
	projectsWithoutDoe,
} from '#/pages/doe/lib/new-doe'

describe('projectsWithoutDoe', () => {
	it('drops archived projects and those that already have a DOE', () => {
		const projects = [
			{ id: 'a' },
			{ id: 'b' },
			{ id: 'c', archived_at: '2026-01-01T00:00:00Z' },
			{ id: 'd', archived_at: null },
		]
		expect(
			projectsWithoutDoe(projects, [{ project_id: 'b' }]).map((p) => p.id),
		).toEqual(['a', 'd'])
	})
})

describe('doeButtonLabel', () => {
	it('reflects the DOE state', () => {
		expect(doeButtonLabel({ revision: 'B' })).toBe('Ouvrir le DOE · indice B')
		expect(doeButtonLabel('none')).toBe('Créer le DOE')
		expect(doeButtonLabel('loading')).toBe('DOE')
	})
})

describe('canCreateDoe', () => {
	it('needs a project and a model', () => {
		expect(canCreateDoe('', 'm')).toBe(false)
		expect(canCreateDoe('p', '')).toBe(false)
		expect(canCreateDoe('p', 'm')).toBe(true)
	})
})
