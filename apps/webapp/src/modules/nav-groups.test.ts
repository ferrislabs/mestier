import { describe, expect, it } from 'vitest'
import { buildSidebarGroups } from '#/modules/nav-groups'

function titlesOf(pathname: string): string[] {
	return buildSidebarGroups(pathname).flatMap((group) =>
		group.items.map((item) => item.title),
	)
}

describe('buildSidebarGroups', () => {
	it('expose la nav du module courant', () => {
		expect(titlesOf('/')).toEqual([
			'Accueil',
			'Paramètres',
			'Intégrations',
			'Rapports',
			'Audit',
			'Permissions',
		])
		expect(titlesOf('/crm/customers')).toContain('Clients')
		expect(titlesOf('/crm/customers')).toContain('Pipeline')
	})

	it('ajoute les groupes globaux à tous les modules', () => {
		expect(titlesOf('/')).toContain('Paramètres')
		expect(titlesOf('/discussions')).toContain('Paramètres')
	})

	it('place les groupes du module avant les groupes globaux', () => {
		const labels = buildSidebarGroups('/').map((group) => group.label)
		expect(labels).toEqual(['Activité', 'Configuration', 'Sécurité'])
	})
})
