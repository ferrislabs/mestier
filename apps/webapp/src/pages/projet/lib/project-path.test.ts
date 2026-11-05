import { describe, expect, it } from 'vitest'
import {
	activeSidebarEntries,
	doeModelIdOf,
	ENTRY,
	projectIdOf,
	projectSectionOf,
	switchTarget,
} from '#/pages/projet/lib/project-path'

describe('projectIdOf', () => {
	it('reads the project id of every project page', () => {
		expect(projectIdOf('/projet/p1')).toBe('p1')
		expect(projectIdOf('/projet/p1/missions')).toBe('p1')
		expect(projectIdOf('/projet/p1/doe/preview')).toBe('p1')
	})

	it('ignores the pages that are not a project', () => {
		expect(projectIdOf('/projet')).toBeNull()
		expect(projectIdOf('/projet/new')).toBeNull()
		expect(projectIdOf('/projet/doe-models')).toBeNull()
		expect(projectIdOf('/projet/doe-models/m1')).toBeNull()
		expect(projectIdOf('/crm/customers')).toBeNull()
	})

	it('decodes the id', () => {
		expect(projectIdOf('/projet/a%20b')).toBe('a b')
	})
})

describe('projectSectionOf', () => {
	it('maps the sub-pages to their section', () => {
		expect(projectSectionOf('/projet/p1')).toBe('overview')
		expect(projectSectionOf('/projet/p1/missions')).toBe('missions')
		expect(projectSectionOf('/projet/p1/doe')).toBe('doe')
		expect(projectSectionOf('/projet/p1/doe/preview')).toBe('doe')
		expect(projectSectionOf('/projet/p1/settings')).toBe('settings')
	})

	it('has no section outside a project', () => {
		expect(projectSectionOf('/projet')).toBeNull()
		expect(projectSectionOf('/projet/new')).toBeNull()
		expect(projectSectionOf('/projet/doe-models')).toBeNull()
	})
})

describe('activeSidebarEntries', () => {
	it('lights the list on the list and on the creation form', () => {
		expect(activeSidebarEntries('/projet')).toEqual(new Set([ENTRY.projects]))
		expect(activeSidebarEntries('/projet/new')).toEqual(
			new Set([ENTRY.projects]),
		)
	})

	it('lights the models on the model pages', () => {
		expect(activeSidebarEntries('/projet/doe-models')).toEqual(
			new Set([ENTRY.models]),
		)
		expect(activeSidebarEntries('/projet/doe-models/m1')).toEqual(
			new Set([ENTRY.models]),
		)
	})

	it('lights the selected project entry', () => {
		expect(activeSidebarEntries('/projet/p1')).toEqual(
			new Set([ENTRY.overview]),
		)
		expect(activeSidebarEntries('/projet/p1/doe/preview')).toEqual(
			new Set([ENTRY.doe]),
		)
		expect(activeSidebarEntries('/projet/p1/settings')).toEqual(
			new Set([ENTRY.settings]),
		)
	})
})

describe('switchTarget', () => {
	it('keeps the section when switching project', () => {
		expect(switchTarget('p2', 'overview')).toBe('/projet/p2')
		expect(switchTarget('p2', 'missions')).toBe('/projet/p2/missions')
		expect(switchTarget('p2', 'doe')).toBe('/projet/p2/doe')
		expect(switchTarget('p2', 'settings')).toBe('/projet/p2/settings')
	})
})

describe('doeModelIdOf', () => {
	it('reads the model id of a model page only', () => {
		expect(doeModelIdOf('/projet/doe-models/m1')).toBe('m1')
		expect(doeModelIdOf('/projet/doe-models')).toBeNull()
		expect(doeModelIdOf('/projet/p1')).toBeNull()
	})
})
