import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	CursusSidebar,
	type CursusSidebarProps,
} from '#/pages/cursus/nav/ui/cursus-sidebar'
import { renderWithRouter } from '#/test/render-with-router'

const PATH = '/o/dupont/cursus/entreprises'

function props(
	overrides: Partial<CursusSidebarProps> = {},
): CursusSidebarProps {
	return {
		organizationSlug: 'dupont',
		active: 'companies',
		counts: { promotions: 4, companies: 21 },
		canView: true,
		isParticipant: true,
		isCollapsed: () => false,
		onToggleGroup: () => {},
		...overrides,
	}
}

describe('CursusSidebar', () => {
	it('groups the entries and shows their counts', async () => {
		await renderWithRouter(<CursusSidebar {...props()} />, PATH)

		expect(screen.getByText('Annuaire')).toBeTruthy()
		expect(screen.getByText('Référentiel')).toBeTruthy()
		const companies = screen.getByRole('link', { name: /Entreprises/ })
		expect(companies.getAttribute('href')).toBe('/o/dupont/cursus/entreprises')
		expect(companies.getAttribute('aria-current')).toBe('page')
		expect(companies.textContent).toContain('21')
		expect(
			screen.getByRole('link', { name: /Promotions/ }).getAttribute(
				'aria-current',
			),
		).toBeNull()
	})

	it('hides a count that is not loaded yet', async () => {
		await renderWithRouter(<CursusSidebar {...props()} />, PATH)

		expect(screen.getByRole('link', { name: /Formations/ }).textContent).toBe(
			'Formations',
		)
	})

	it('shows only my triads to a participant without cursus access', async () => {
		await renderWithRouter(
			<CursusSidebar {...props({ canView: false })} />,
			PATH,
		)

		expect(screen.getAllByRole('link')).toHaveLength(1)
		expect(screen.getByRole('link', { name: /Mes trinômes/ })).toBeTruthy()
		expect(screen.queryByText('Annuaire')).toBeNull()
		expect(screen.queryByText('Référentiel')).toBeNull()
	})

	it('hides my triads from a non participant', async () => {
		await renderWithRouter(
			<CursusSidebar {...props({ isParticipant: false })} />,
			PATH,
		)

		expect(screen.queryByRole('link', { name: /Mes trinômes/ })).toBeNull()
	})

	it('toggles a group', async () => {
		const onToggleGroup = vi.fn()
		await renderWithRouter(
			<CursusSidebar {...props({ onToggleGroup })} />,
			PATH,
		)

		await userEvent.click(screen.getByRole('button', { name: /Annuaire/ }))

		expect(onToggleGroup).toHaveBeenCalledWith('directory')
	})
})
