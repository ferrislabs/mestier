import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { TriadSummary } from '#/pages/cursus/triads/feature/use-triads'
import { TriadsList } from '#/pages/cursus/triads/ui/triads-list'

const TRIAD: TriadSummary = {
	id: 'triad-1',
	organization_id: 'org-1',
	apprentice_id: 'app-1',
	promotion_id: 'promo-1',
	apprentice_name: 'Lea Martin',
	promotion_label: 'CAP 2026',
	training_name: 'CAP Menuisier',
	pedagogic_tutor_name: 'Alix Martin',
	current_host_company_name: 'Atelier Bois',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

function props(canManage: boolean, triads: TriadSummary[] = [TRIAD]) {
	return {
		triads,
		total: triads.length,
		promotions: [],
		promotionId: undefined,
		status: 'all' as const,
		search: '',
		isLoading: false,
		canManage,
		onPromotionChange: vi.fn(),
		onStatusChange: vi.fn(),
		onSearchChange: vi.fn(),
		onResetFilters: vi.fn(),
		onOpen: vi.fn(),
		onCreate: vi.fn(),
	}
}

describe('TriadsList', () => {
	it('renders apprentice, promotion, tutor, company and status', () => {
		render(<TriadsList {...props(true)} />)

		expect(screen.getByText('Lea Martin')).toBeDefined()
		expect(screen.getByText('CAP 2026 · CAP Menuisier')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
		expect(screen.getByText('Atelier Bois')).toBeDefined()
		expect(screen.getByText('En cours')).toBeDefined()
	})

	it('marks a closed triad', () => {
		render(
			<TriadsList {...props(true, [{ ...TRIAD, closed_at: '2027-01-01' }])} />,
		)

		expect(screen.getByText('Clôturé')).toBeDefined()
	})

	it('opens a triad on row click', async () => {
		const user = userEvent.setup()
		const p = props(true)
		render(<TriadsList {...p} />)

		await user.click(screen.getByText('Lea Martin'))

		expect(p.onOpen).toHaveBeenCalledWith('triad-1')
	})

	it('hides the creation button without the triads bit', () => {
		render(<TriadsList {...props(false)} />)

		expect(screen.queryByRole('button', { name: /Nouveau trinôme/ })).toBeNull()
	})

	it('counts the listed triads in the header', () => {
		render(<TriadsList {...props(true)} status="open" />)

		expect(screen.getByText('1 trinôme ouvert')).toBeTruthy()
	})

	it('offers to reset a non-default filter when nothing matches', () => {
		render(<TriadsList {...props(true, [])} status="closed" />)

		expect(screen.getByText('Aucun trinôme ne correspond.')).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		).toBeTruthy()
	})

	it('keeps the headers and spans the empty state across the columns', () => {
		render(<TriadsList {...props(true, [])} />)

		expect(screen.getByRole('columnheader', { name: 'Apprenti' })).toBeDefined()
		const cell = screen.getByText('Aucun trinôme ne correspond.').closest('td')
		expect(cell?.getAttribute('colspan')).toBe('5')
	})
})
