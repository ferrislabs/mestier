import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { PlanningToolbar } from '#/pages/planning/ui/planning-toolbar'

function baseProps() {
	return {
		view: 'week' as const,
		date: '2026-08-07',
		windowFrom: '2026-08-03',
		windowTo: '2026-08-09',
		onViewChange: vi.fn(),
		onDateChange: vi.fn(),
	}
}

describe('PlanningToolbar — bascule de vue', () => {
	it('shows the three tabs and marks the active view', () => {
		render(<PlanningToolbar {...baseProps()} />)

		const semaine = screen.getByRole('tab', { name: 'Semaine' })
		expect(semaine.getAttribute('data-state')).toBe('active')
		expect(screen.getByRole('tab', { name: 'Jour' })).toBeDefined()
		expect(screen.getByRole('tab', { name: 'Mois' })).toBeDefined()
	})

	it('calls onViewChange with the clicked view', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('tab', { name: 'Mois' }))

		expect(props.onViewChange).toHaveBeenCalledWith('month')
	})
})

describe('PlanningToolbar — previous/next navigation', () => {
	it('steps back one week in week view', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période précédente' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-07-31')
	})

	it('steps forward one week in week view', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-08-14')
	})

	it('steps forward one day in day view', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), view: 'day' as const }
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-08-08')
	})

	it('steps forward one month in month view', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), view: 'month' as const }
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-09-07')
	})
})

describe('PlanningToolbar — bouton Aujourd’hui', () => {
	it('targets the controlled date rather than the real clock', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), today: '2027-01-15' }
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Aujourd’hui' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2027-01-15')
	})
})

describe('PlanningToolbar — period label', () => {
	it('shows the visible period in the calendar trigger', () => {
		render(<PlanningToolbar {...baseProps()} />)

		expect(screen.getByText('3 août – 9 août')).toBeDefined()
	})
})

describe('PlanningToolbar — no network call', () => {
	let fetchSpy: ReturnType<typeof createFetchSpy>

	function createFetchSpy() {
		return vi.spyOn(global, 'fetch').mockImplementation(() => {
			throw new Error('le composant ui/ ne doit jamais appeler fetch')
		})
	}

	beforeEach(() => {
		fetchSpy = createFetchSpy()
	})

	afterEach(() => {
		fetchSpy.mockRestore()
	})

	it('fires no fetch on render nor during interactions', async () => {
		const user = userEvent.setup()
		render(<PlanningToolbar {...baseProps()} />)

		await user.click(screen.getByRole('tab', { name: 'Jour' }))
		await user.click(screen.getByRole('button', { name: 'Aujourd’hui' }))

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
