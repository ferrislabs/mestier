import { describe, expect, it } from 'vitest'
import { moduleLandingPath } from '#/modules/landing'
import { MODULES } from '#/modules/registry'

function automatisationModule() {
	const module = MODULES.find((module) => module.id === 'automatisation')
	if (!module) throw new Error('registry: automatisation module missing')
	return module
}

describe('MODULES order and labels', () => {
	it('lists the modules in the launcher order', () => {
		expect(MODULES.map((module) => module.id)).toEqual([
			'home',
			'crm',
			'projet',
			'planning',
			'planification',
			'chat',
			'support',
			'cursus',
			'hr',
			'automatisation',
			'purchase',
			'settings',
		])
	})

	it('names the modules as the launcher shows them', () => {
		const labels = Object.fromEntries(
			MODULES.map((module) => [module.id, module.label]),
		)
		expect(labels.hr).toBe('RH')
		expect(labels.automatisation).toBe('Workflows')
		expect(labels.chat).toBe('Discussions')
	})
})

describe('automatisation module entry', () => {
	it('lists the sections in the prototype order', () => {
		expect(
			automatisationModule().sections.map((section) => [
				section.label,
				section.to,
			]),
		).toEqual([
			['Workflows', '/automatisation/workflows'],
			['Exécutions', '/automatisation/executions'],
			['Collections', '/automatisation/collections'],
			['Identifications', '/automatisation/identifications'],
		])
	})

	it('keeps Workflows active on its sub paths, the editor included', () => {
		const workflows = automatisationModule().sections.find(
			(section) => section.label === 'Workflows',
		)

		expect(workflows?.exact).toBeUndefined()
	})

	it('opens a workflow draft from the main action, without a search parameter', () => {
		expect(automatisationModule().primaryAction).toEqual({
			label: 'Nouveau workflow',
			to: '/automatisation/workflows/nouveau',
		})
	})

	it('lands on the workflow list, since the base path only redirects', () => {
		expect(moduleLandingPath('automatisation')).toBe(
			'/automatisation/workflows',
		)
	})
})
