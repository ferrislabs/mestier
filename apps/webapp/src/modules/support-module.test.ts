import { describe, expect, it } from 'vitest'
import { moduleLandingPath } from '#/modules/landing'
import { MODULES } from '#/modules/registry'

const supportModule = MODULES.find((candidate) => candidate.id === 'support')

describe('support module', () => {
	it('is registered, available and gated on VIEW_SUPPORT', () => {
		expect(supportModule?.status).toBe('available')
		expect(supportModule?.basePath).toBe('/support')
		expect(supportModule?.requiredPermission).toBe('VIEW_SUPPORT')
	})

	it('holds one Tickets section gated on VIEW_SUPPORT', () => {
		expect(supportModule?.sections).toHaveLength(1)
		expect(supportModule?.sections[0].label).toBe('Tickets')
		expect(supportModule?.sections[0].requiredPermission).toBe('VIEW_SUPPORT')
	})

	it('offers the kanban and the table as the tabs of that section', () => {
		const tabs = supportModule?.sections[0].tabs ?? []

		expect(tabs.map((tab) => [tab.label, tab.to, tab.exact === true])).toEqual([
			['Kanban', '/support', true],
			['Tableau', '/support/table', false],
		])
	})

	it('lands on the kanban', () => {
		expect(moduleLandingPath('support')).toBe('/support')
	})
})
