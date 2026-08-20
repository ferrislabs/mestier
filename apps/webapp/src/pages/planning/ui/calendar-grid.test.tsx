import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { buildCalendarModel } from '#/pages/planning/lib/build-calendar-model'
import type { PlanningEntry } from '#/pages/planning/types'
import { CalendarGrid } from '#/pages/planning/ui/calendar-grid'
import type { CalendarEventCallbacks } from '#/pages/planning/ui/event-popover'

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
	member_ids: [],
	labels: [],
	status: 'PLANNED',
} as PlanningEntry

const ABSENCE = {
	kind: 'absence',
	id: 'a-1',
	member_id: 'e-1',
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
		onChangeStatus: vi.fn(),
		onDelete: vi.fn(),
		editing: null,
		assignees: [],
		selectedResourceIds: [],
		onEdit: vi.fn(),
		onEditChange: vi.fn(),
		onToggleAssignee: vi.fn(),
		onEditSubmit: vi.fn(),
		onEditCancel: vi.fn(),
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

describe('CalendarGrid — detail panel', () => {
	it('opens the panel on a card click only', async () => {
		renderGrid([TASK])

		expect(screen.queryByRole('dialog')).toBeNull()

		const panel = await openDetails(/Taille de haie/)
		expect(panel.textContent).toContain('Taille de haie')
	})

	it('shows what the task carries without opening the full sheet', async () => {
		renderGrid([TASK])

		const panel = await openDetails(/Taille de haie/)

		expect(panel.textContent).toContain('Marie Leroy')
		expect(panel.textContent).toContain('Jardin nord')
		expect(panel.textContent).toContain('Haie de thuyas côté rue')
		expect(panel.textContent).toMatch(/\d{2}:\d{2} – \d{2}:\d{2}/)
	})

	it('switches the panel to editing from the pencil', async () => {
		const callbacks = renderGrid([TASK])
		await openDetails(/Taille de haie/)

		await userEvent.click(
			screen.getByRole('button', { name: 'Modifier la tâche' }),
		)

		expect(callbacks.onEdit).toHaveBeenCalledTimes(1)
	})

	it("changes the status from the panel's footer", async () => {
		const callbacks = renderGrid([TASK])
		await openDetails(/Taille de haie/)

		await userEvent.click(screen.getByRole('button', { name: /Terminée/ }))

		expect(callbacks.onChangeStatus).toHaveBeenCalledWith(
			expect.objectContaining({ id: 't-1' }),
			'DONE',
		)
	})

	it('offers deletion on an absence, no status change', async () => {
		const callbacks = renderGrid([ABSENCE])
		const panel = await openDetails(/Congé/)

		expect(panel.textContent).toContain('Vacances d’été')
		expect(screen.queryByRole('button', { name: /Terminée/ })).toBeNull()

		await userEvent.click(
			screen.getByRole('button', { name: /Supprimer cette absence/ }),
		)

		expect(callbacks.onDelete).toHaveBeenCalledTimes(1)
	})

	it('edits inside the panel, without opening a drawer', async () => {
		const callbacks = renderGrid([TASK], {
			editing: {
				kind: 'task',
				entryId: 't-1',
				errors: [],
				values: {
					title: 'Taille de haie',
					description: '',
					allDay: false,
					startDate: '2026-03-02',
					startTime: '09:00',
					endDate: '2026-03-02',
					endTime: '11:00',
					blocksAvailability: true,
					customerId: '',
					customerContextId: '',
					quoteId: '',
					labelIds: [],
					assignees: [],
				},
			},
		})
		await openDetails(/Taille de haie/)

		const titre = screen.getByDisplayValue('Taille de haie')
		await userEvent.type(titre, ' haute')

		expect(callbacks.onEditChange).toHaveBeenCalled()

		await userEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))
		expect(callbacks.onEditSubmit).toHaveBeenCalledTimes(1)
	})

	it('blocks saving while the draft is invalid', async () => {
		renderGrid([TASK], {
			editing: {
				kind: 'task',
				entryId: 't-1',
				errors: ['Le titre est obligatoire.'],
				values: {
					title: '',
					description: '',
					allDay: false,
					startDate: '2026-03-02',
					startTime: '09:00',
					endDate: '2026-03-02',
					endTime: '11:00',
					blocksAvailability: true,
					customerId: '',
					customerContextId: '',
					quoteId: '',
					labelIds: [],
					assignees: [],
				},
			},
		})
		await openDetails(/Taille de haie/)

		expect(screen.getByText('Le titre est obligatoire.')).toBeDefined()
		expect(
			screen
				.getByRole('button', { name: 'Enregistrer' })
				.hasAttribute('disabled'),
		).toBe(true)
	})

	it('closes through its close button', async () => {
		renderGrid([TASK])
		await openDetails(/Taille de haie/)

		await userEvent.click(screen.getByRole('button', { name: 'Fermer' }))

		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
	})
})
