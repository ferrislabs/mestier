import { describe, expect, it } from 'vitest'
import { filterProjects } from '#/pages/projects/lib/filter-projects'

const PROJECTS = [
	{ id: 'a', name: 'Lapalisse' },
	{ id: 'b', name: 'Jardin Duval' },
	{ id: 'c', name: 'Parc de Vichy' },
]
const hasDoe = (id: string) => id === 'b'

describe('filterProjects', () => {
	it('keeps every project when no filter is set', () => {
		expect(
			filterProjects(PROJECTS, { search: '', withDoeOnly: false }, hasDoe),
		).toHaveLength(3)
	})

	it('keeps only the projects with a DOE', () => {
		expect(
			filterProjects(PROJECTS, { search: '', withDoeOnly: true }, hasDoe).map(
				(project) => project.id,
			),
		).toEqual(['b'])
	})

	it('combines the name search and the DOE filter', () => {
		expect(
			filterProjects(PROJECTS, { search: 'vichy', withDoeOnly: true }, hasDoe),
		).toEqual([])
		expect(
			filterProjects(
				PROJECTS,
				{ search: ' VICHY ', withDoeOnly: false },
				hasDoe,
			),
		).toHaveLength(1)
	})
})
