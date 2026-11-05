import { describe, expect, it } from 'vitest'
import {
	eligibleProjects,
	noTargetMessage,
	projectOptionLabel,
	type TargetProject,
} from '#/pages/doe-models/lib/create-doe-targets'

const projects: TargetProject[] = [
	{ id: 'b', name: 'Beta', customerName: 'Duval', archived: false },
	{ id: 'a', name: 'Alpha', customerName: null, archived: false },
	{ id: 'c', name: 'Gamma', customerName: null, archived: true },
	{ id: 'd', name: 'Delta', customerName: null, archived: false },
]

describe('create DOE targets', () => {
	it('proposes active projects without a DOE, by name', () => {
		expect(
			eligibleProjects(projects, new Set(['d'])).map((project) => project.id),
		).toEqual(['a', 'b'])
	})

	it('labels a project with its customer or as internal', () => {
		expect(projectOptionLabel(projects[0] as TargetProject)).toBe(
			'Beta · Duval',
		)
		expect(projectOptionLabel(projects[1] as TargetProject)).toBe(
			'Alpha · Projet interne',
		)
	})

	it('explains why no project is proposed', () => {
		expect(noTargetMessage(projects)).toContain('ont déjà un DOE')
		expect(noTargetMessage([])).toContain('Aucun projet actif')
	})
})
