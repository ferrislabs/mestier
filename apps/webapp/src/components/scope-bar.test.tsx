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
	it('renders nothing when the scope exposes a single screen', async () => {
		const { container } = await renderWithRouter(
			<ScopeBar
				label="RH"
				tabs={[tabs[0] as ModuleTab]}
				organizationSlug="dupont"
			/>,
		)

		expect(container.textContent).toBe('')
	})

	it('renders a bar as soon as actions are provided', async () => {
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

	it('marks the tab matching the current route', async () => {
		await renderWithRouter(
			<ScopeBar label="CRM" tabs={tabs} organizationSlug="dupont" />,
			'/o/dupont/crm/quotes',
		)

		const current = await screen.findByRole('link', { current: 'page' })
		expect(current.textContent).toContain('Devis')
	})

	it('renders an announced tab as a disabled button rather than a link', async () => {
		await renderWithRouter(
			<ScopeBar label="CRM" tabs={tabs} organizationSlug="dupont" />,
			'/o/dupont/crm/quotes',
		)

		const links = screen.getAllByRole('link').map((link) => link.textContent)
		expect(links.some((text) => text?.includes('Factures'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Factures/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('exposes the navigation under a scope label', async () => {
		await renderWithRouter(
			<ScopeBar label="CRM" tabs={tabs} organizationSlug="dupont" />,
			'/o/dupont/crm/quotes',
		)

		expect(screen.getByRole('navigation', { name: 'CRM' })).toBeDefined()
	})
})
