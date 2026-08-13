import { render, screen, within } from '@testing-library/react'
import { Boxes } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import type { SettingsNavGroup } from '#/pages/settings/nav'
import { AnchorNav } from '#/pages/settings/ui/anchor-nav'

const groups: SettingsNavGroup[] = [
	{
		label: 'Général',
		sections: [
			{
				id: 'organisation',
				label: 'Organisation',
				icon: Boxes,
				Component: () => null,
			},
			{
				id: 'automatisation',
				label: 'Automatisation',
				icon: Boxes,
				Component: () => null,
			},
		],
	},
	{
		label: 'CRM',
		sections: [
			{
				id: 'crm',
				label: 'Catalogue',
				icon: Boxes,
				moduleId: 'crm',
				Component: () => null,
			},
		],
	},
]

describe('AnchorNav', () => {
	it('rend un lien par section, pointant vers son ancre', () => {
		render(<AnchorNav groups={groups} activeId="organisation" />)

		expect(screen.getByRole('link', { name: 'Organisation' })).toHaveProperty(
			'hash',
			'#organisation',
		)
		expect(screen.getByRole('link', { name: 'Catalogue' })).toHaveProperty(
			'hash',
			'#crm',
		)
	})

	it('affiche le libellé de chaque groupe', () => {
		render(<AnchorNav groups={groups} activeId="organisation" />)

		expect(screen.getByText('Général')).toBeDefined()
		expect(screen.getByText('CRM')).toBeDefined()
	})

	it('marque la section active et elle seule', () => {
		render(<AnchorNav groups={groups} activeId="crm" />)

		const current = screen.getAllByRole('link', { current: 'location' })
		expect(current.map((link) => link.textContent)).toEqual(['Catalogue'])
	})

	it('range chaque lien dans son propre groupe, pas un autre', () => {
		render(<AnchorNav groups={groups} activeId="organisation" />)

		const generalGroup = screen.getByRole('list', { name: 'Général' })
		const crmGroup = screen.getByRole('list', { name: 'CRM' })

		expect(
			within(crmGroup).getByRole('link', { name: 'Catalogue' }),
		).toBeDefined()
		expect(
			within(generalGroup).queryByRole('link', { name: 'Catalogue' }),
		).toBeNull()
	})
})
