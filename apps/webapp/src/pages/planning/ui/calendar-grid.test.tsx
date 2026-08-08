import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { buildCalendarModel } from '#/pages/planning/lib/build-calendar-model'
import type { PlanningEntry } from '#/pages/planning/types'
import {
	type CalendarEventCallbacks,
	CalendarGrid,
} from '#/pages/planning/ui/calendar-grid'

const TASK = {
	kind: 'task',
	id: 't-1',
	title: 'Taille de haie',
	description: 'Haie de thuyas côté rue',
	customer_name: 'Marie Leroy',
	context_label: 'Jardin nord',
	starts_at: '2026-03-02T09:00:00Z',
	ends_at: '2026-03-02T11:00:00Z',
	all_day: false,
	blocks_availability: true,
	child_count: 0,
	employee_ids: [],
	labels: [],
	status: 'PLANNED',
} as PlanningEntry

const ABSENCE = {
	kind: 'absence',
	id: 'a-1',
	employee_id: 'e-1',
	absence_kind: 'LEAVE',
	note: 'Vacances d’été',
	starts_at: '2026-03-02T00:00:00Z',
	ends_at: '2026-03-03T00:00:00Z',
	all_day: true,
} as PlanningEntry

function renderGrid(
	entries: PlanningEntry[],
	overrides: Partial<CalendarEventCallbacks> = {},
) {
	const callbacks: CalendarEventCallbacks = {
		onOpen: vi.fn(),
		onChangeStatus: vi.fn(),
		onDelete: vi.fn(),
		...overrides,
	}

	render(
		<CalendarGrid
			model={buildCalendarModel({
				from: '2026-03-02',
				to: '2026-03-02',
				entries,
				resources: [],
				workTime: [],
				timeZone: 'UTC',
				today: '2026-03-02',
				filter: 'all',
			})}
			callbacks={callbacks}
			now={new Date('2026-03-02T09:30:00Z')}
		/>,
	)

	return callbacks
}

async function openDetails(name: RegExp) {
	await userEvent.click(screen.getByRole('button', { name }))
	return await screen.findByRole('dialog')
}

describe('CalendarGrid — panneau de détail', () => {
	it("n'ouvre le panneau qu'au clic sur la carte", async () => {
		renderGrid([TASK])

		expect(screen.queryByRole('dialog')).toBeNull()

		const panel = await openDetails(/Taille de haie/)
		expect(panel.textContent).toContain('Taille de haie')
	})

	it('montre ce que porte la tâche sans ouvrir la fiche', async () => {
		renderGrid([TASK])

		const panel = await openDetails(/Taille de haie/)

		expect(panel.textContent).toContain('Marie Leroy')
		expect(panel.textContent).toContain('Jardin nord')
		expect(panel.textContent).toContain('Haie de thuyas côté rue')
		expect(panel.textContent).toMatch(/\d{2}:\d{2} – \d{2}:\d{2}/)
	})

	it('ouvre la fiche complète depuis le panneau', async () => {
		const callbacks = renderGrid([TASK])
		await openDetails(/Taille de haie/)

		await userEvent.click(
			screen.getByRole('button', { name: 'Modifier la tâche' }),
		)

		expect(callbacks.onOpen).toHaveBeenCalledTimes(1)
	})

	it('change le statut depuis le pied du panneau', async () => {
		const callbacks = renderGrid([TASK])
		await openDetails(/Taille de haie/)

		await userEvent.click(screen.getByRole('button', { name: /Terminée/ }))

		expect(callbacks.onChangeStatus).toHaveBeenCalledWith(
			expect.objectContaining({ entryId: 't-1' }),
			'DONE',
		)
	})

	it('propose la suppression sur une absence, pas de changement de statut', async () => {
		const callbacks = renderGrid([ABSENCE])
		const panel = await openDetails(/Congé/)

		expect(panel.textContent).toContain('Vacances d’été')
		expect(screen.queryByRole('button', { name: /Terminée/ })).toBeNull()

		await userEvent.click(
			screen.getByRole('button', { name: /Supprimer cette absence/ }),
		)

		expect(callbacks.onDelete).toHaveBeenCalledTimes(1)
	})

	it('se ferme par son bouton de fermeture', async () => {
		renderGrid([TASK])
		await openDetails(/Taille de haie/)

		await userEvent.click(screen.getByRole('button', { name: 'Fermer' }))

		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
	})
})
