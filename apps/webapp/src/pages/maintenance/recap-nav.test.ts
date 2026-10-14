import { describe, expect, it } from 'vitest'
import { MODULES } from '#/modules/registry'
import { resolveScope } from '#/modules/scope'

const entretien = MODULES.find(
	(module) => module.id === 'planification',
)?.sections.find((section) => section.id === 'maintenance')

describe('Entretien navigation', () => {
	it('is one menu entry whose tabs hold the recap and the queue', () => {
		expect(entretien?.to).toBe('/planification/maintenance')
		expect(entretien?.requiredPermission).toBe('MANAGE_PLANNING')
		expect(entretien?.tabs).toEqual(
			expect.arrayContaining([
				expect.objectContaining({
					label: 'Récap',
					to: '/planification/maintenance/recap',
				}),
				expect.objectContaining({
					label: 'À valider',
					to: '/planification/maintenance/validation',
				}),
			]),
		)
	})

	it('shows those tabs on both screens', () => {
		for (const path of [
			'/o/acme/planification/maintenance/recap',
			'/o/acme/planification/maintenance/validation',
		]) {
			const scope = resolveScope(path)
			expect(scope.label).toBe('Entretien')
			expect(scope.tabs.map((tab) => tab.label)).toEqual(
				expect.arrayContaining(['Récap', 'À valider']),
			)
		}
	})
})
