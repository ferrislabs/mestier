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
	it('affiche les trois onglets et marque la vue active', () => {
		render(<PlanningToolbar {...baseProps()} />)

		const semaine = screen.getByRole('tab', { name: 'Semaine' })
		expect(semaine.getAttribute('data-state')).toBe('active')
		expect(screen.getByRole('tab', { name: 'Jour' })).toBeDefined()
		expect(screen.getByRole('tab', { name: 'Mois' })).toBeDefined()
	})

	it('appelle onViewChange avec la vue cliquée', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('tab', { name: 'Mois' }))

		expect(props.onViewChange).toHaveBeenCalledWith('month')
	})
})

describe('PlanningToolbar — navigation précédent/suivant', () => {
	it('recule d’une semaine en vue semaine', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période précédente' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-07-31')
	})

	it('avance d’une semaine en vue semaine', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-08-14')
	})

	it('avance d’un jour en vue jour', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), view: 'day' as const }
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-08-08')
	})

	it('avance d’un mois en vue mois', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), view: 'month' as const }
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2026-09-07')
	})
})

describe('PlanningToolbar — bouton Aujourd’hui', () => {
	it('cible la date fournie en contrôle plutôt que l’horloge réelle', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), today: '2027-01-15' }
		render(<PlanningToolbar {...props} />)

		await user.click(screen.getByRole('button', { name: 'Aujourd’hui' }))

		expect(props.onDateChange).toHaveBeenCalledWith('2027-01-15')
	})
})

describe('PlanningToolbar — libellé de la période', () => {
	it('affiche la période visible dans le déclencheur du calendrier', () => {
		render(<PlanningToolbar {...baseProps()} />)

		expect(screen.getByText('3 août – 9 août')).toBeDefined()
	})
})

describe('PlanningToolbar — pas d’appel réseau', () => {
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

	it('ne déclenche aucun fetch au rendu ni pendant les interactions', async () => {
		const user = userEvent.setup()
		render(<PlanningToolbar {...baseProps()} />)

		await user.click(screen.getByRole('tab', { name: 'Jour' }))
		await user.click(screen.getByRole('button', { name: 'Aujourd’hui' }))

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
