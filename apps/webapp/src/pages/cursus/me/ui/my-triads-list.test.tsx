import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { MyTriad } from '#/pages/cursus/me/feature/use-my-cursus'
import { MyTriadsList } from '#/pages/cursus/me/ui/my-triads-list'

const ACTIVE: MyTriad = {
	triad_id: 'triad-1',
	apprentice_name: 'Lea Martin',
	promotion_label: 'CAP 2026',
	training_name: 'CAP Menuisier',
	role: 'main_tutor',
	access: { kind: 'active' },
}

const ARCHIVED: MyTriad = {
	...ACTIVE,
	triad_id: 'triad-2',
	apprentice_name: 'Noe Petit',
	role: 'tutor',
	closed_at: '2026-06-30',
	access: { kind: 'read_only', since: '2026-07-01T00:00:00Z' },
}

function renderList(triads: MyTriad[], isLoading = false) {
	const onOpen = vi.fn()
	render(
		<MyTriadsList
			title="Mes trinômes"
			triads={triads}
			isLoading={isLoading}
			onOpen={onOpen}
		/>,
	)
	return onOpen
}

describe('MyTriadsList', () => {
	it('renders apprentice, promotion, my role and status', () => {
		renderList([ACTIVE])

		expect(screen.getByText('Lea Martin')).toBeDefined()
		expect(screen.getByText('CAP 2026 · CAP Menuisier')).toBeDefined()
		expect(screen.getByText('Tuteur principal')).toBeDefined()
		expect(screen.getByText('En cours')).toBeDefined()
		expect(screen.queryByText('Historique, lecture seule')).toBeNull()
	})

	it('marks a read-only link with its since date', () => {
		renderList([ACTIVE, ARCHIVED])

		expect(screen.getByText('Historique, lecture seule')).toBeDefined()
		expect(screen.getByText('depuis le 01/07/2026')).toBeDefined()
		expect(screen.getByText('Tuteur')).toBeDefined()
	})

	it('opens a triad on row click', async () => {
		const user = userEvent.setup()
		const onOpen = renderList([ACTIVE])

		await user.click(screen.getByText('Lea Martin'))

		expect(onOpen).toHaveBeenCalledWith('triad-1')
	})

	it('shows an empty state', () => {
		renderList([])

		expect(screen.getByText('Aucun trinôme')).toBeDefined()
	})
})
