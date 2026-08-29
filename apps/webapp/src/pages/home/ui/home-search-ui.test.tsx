import { fireEvent, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { HomeSearchUI, type SearchGroup } from '#/pages/home/ui/home-search-ui'
import { renderWithRouter } from '#/test/render-with-router'

const GROUPS: SearchGroup[] = [
	{
		label: 'Clients',
		items: [
			{
				id: 'c1',
				label: 'Atelier Dupont',
				to: '/o/atelier-bois/crm/customers/c1',
			},
			{
				id: 'c2',
				label: 'Menuiserie Martin',
				to: '/o/atelier-bois/crm/customers/c2',
			},
		],
	},
	{
		label: 'Devis',
		items: [
			{
				id: 'q1',
				label: 'Terrasse bois',
				sublabel: 'DEV-2026-014',
				to: '/o/atelier-bois/crm/quotes/q1',
			},
		],
	},
]

describe('HomeSearchUI', () => {
	it('shows no dropdown before the input has any text', async () => {
		await renderWithRouter(<HomeSearchUI groups={GROUPS} />)

		expect(screen.queryByText('Atelier Dupont')).toBeNull()
	})

	it('groups matches by entity kind, filtering across all groups', async () => {
		await renderWithRouter(<HomeSearchUI groups={GROUPS} />)

		const input = screen.getByRole('textbox', { name: 'Recherche rapide' })
		fireEvent.focus(input)
		fireEvent.change(input, { target: { value: 'dup' } })

		expect(screen.getByText('Clients')).toBeDefined()
		expect(screen.getByText('Atelier Dupont')).toBeDefined()
		expect(screen.queryByText('Menuiserie Martin')).toBeNull()
		expect(screen.queryByText('Devis')).toBeNull()
	})

	it('matches on the sublabel too, e.g. a quote reference', async () => {
		await renderWithRouter(<HomeSearchUI groups={GROUPS} />)

		const input = screen.getByRole('textbox', { name: 'Recherche rapide' })
		fireEvent.focus(input)
		fireEvent.change(input, { target: { value: '2026-014' } })

		expect(screen.getByText('Terrasse bois')).toBeDefined()
	})

	it('shows an explicit empty state rather than nothing when there is no match', async () => {
		await renderWithRouter(<HomeSearchUI groups={GROUPS} />)

		const input = screen.getByRole('textbox', { name: 'Recherche rapide' })
		fireEvent.focus(input)
		fireEvent.change(input, { target: { value: 'zzz-no-match' } })

		expect(screen.getByText(/Aucun résultat/)).toBeDefined()
	})
})
