import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ScopeBar } from '#/components/scope-bar'
import type { ModuleTab } from '#/modules/types'
import { renderWithRouter } from '#/test/render-with-router'

const tabs: ModuleTab[] = [
	{ id: 'customers', label: 'Clients', to: '/crm/customers', exact: true },
	{ id: 'quotes', label: 'Devis', to: '/crm/quotes' },
	{
		id: 'invoices',
		label: 'Factures',
		to: '/crm/invoices',
		status: 'coming-soon',
	},
]

describe('ScopeBar', () => {
	it("ne rend rien quand le scope n'expose qu'un seul écran", async () => {
		const { container } = await renderWithRouter(
			<ScopeBar
				label="RH"
				tabs={[tabs[0] as ModuleTab]}
				organizationSlug="dupont"
			/>,
		)

		expect(container.textContent).toBe('')
	})

	it('rend une barre dès lors que des actions sont fournies', async () => {
		await renderWithRouter(
			<ScopeBar
				label="RH"
				tabs={[tabs[0] as ModuleTab]}
				organizationSlug="dupont"
				actions={<button type="button">Enregistrer</button>}
			/>,
		)

		expect(screen.getByRole('button', { name: 'Enregistrer' })).toBeDefined()
	})

	it("marque l'onglet correspondant à la route courante", async () => {
		await renderWithRouter(
			<ScopeBar label="CRM" tabs={tabs} organizationSlug="dupont" />,
			'/o/dupont/crm/quotes',
		)

		const current = await screen.findByRole('link', { current: 'page' })
		expect(current.textContent).toContain('Devis')
	})

	it('rend un onglet annoncé en bouton désactivé plutôt qu’en lien', async () => {
		await renderWithRouter(
			<ScopeBar label="CRM" tabs={tabs} organizationSlug="dupont" />,
			'/o/dupont/crm/quotes',
		)

		const links = screen.getAllByRole('link').map((link) => link.textContent)
		expect(links.some((text) => text?.includes('Factures'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Factures/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('expose la navigation sous un libellé de scope', async () => {
		await renderWithRouter(
			<ScopeBar label="CRM" tabs={tabs} organizationSlug="dupont" />,
			'/o/dupont/crm/quotes',
		)

		expect(screen.getByRole('navigation', { name: 'CRM' })).toBeDefined()
	})
})
