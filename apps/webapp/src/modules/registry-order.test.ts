import { describe, expect, it } from 'vitest'
import { MODULES } from '#/modules/registry'

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
