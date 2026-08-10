import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ModuleNav } from '#/components/module-nav'
import { SidebarProvider } from '#/components/ui/sidebar'
import { renderWithRouter } from '#/test/render-with-router'

function renderNav(initialPath = '/o/dupont') {
	return renderWithRouter(
		<SidebarProvider defaultOpen>
			<ModuleNav organizationSlug="dupont" />
		</SidebarProvider>,
		initialPath,
	)
}

describe('ModuleNav', () => {
	it("lists the active module's sections", async () => {
		await renderNav('/o/dupont/crm/customers')

		for (const label of ['Clients', 'Pipeline', 'Devis', 'Factures']) {
			expect(screen.getByText(label)).toBeDefined()
		}
	})

	it("does not expose other modules' sections", async () => {
		await renderNav('/o/dupont/crm/customers')

		expect(screen.queryByText('Employés')).toBeNull()
		expect(screen.queryByText('Vue équipe')).toBeNull()
	})

	it('prefixes every section link with the tenant', async () => {
		await renderNav('/o/dupont/crm/customers')

		const lien = screen.getByRole('link', { name: /Pipeline/ })
		expect(lien.getAttribute('href')).toBe('/o/dupont/crm/customers/pipeline')
	})

	it('marks the current section', async () => {
		await renderNav('/o/dupont/crm/customers/pipeline')

		const courant = await screen.findByRole('link', { current: 'page' })
		expect(courant.textContent).toContain('Pipeline')
	})

	it('exposes no link for an announced section, but keeps it focusable', async () => {
		await renderNav('/o/dupont/crm/customers')

		const liens = screen.getAllByRole('link').map((lien) => lien.textContent)
		expect(liens.some((texte) => texte?.includes('Factures'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Factures/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('keeps utility modules at the foot of the nav', async () => {
		const { container } = await renderNav('/o/dupont/crm/customers')

		const pied = container.querySelector('[data-slot="sidebar-footer"]')
		const corps = container.querySelector('[data-slot="sidebar-content"]')

		expect(pied?.textContent).toContain('Paramètres')
		expect(corps?.textContent).toContain('Clients')
		expect(corps?.textContent).not.toContain('Paramètres')
	})

	it('shows the current module under the brand', async () => {
		await renderNav('/o/dupont/planning/team')

		expect(screen.getByText('Mestier')).toBeDefined()
		expect(screen.getByText('Planning')).toBeDefined()
	})
})
