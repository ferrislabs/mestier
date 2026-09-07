import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { renderWithRouter } from '#/test/render-with-router'
import { buildCalendarModel } from '../lib/build-calendar-model'
import type { PlanningEntry } from '../types'
import type { CalendarEventCallbacks } from './event-popover'
import { PlanningCalendarUI } from './planning-calendar-ui'

Element.prototype.scrollIntoView ??= () => {}

const LEAVE: PlanningEntry = {
	kind: 'absence',
	id: 'a-1',
	member_id: 'e-1',
	absence_kind: 'LEAVE',
	note: 'Vacances',
	starts_at: '2026-03-02T00:00:00Z',
	ends_at: '2026-03-03T00:00:00Z',
	all_day: true,
}

const EVENT_CALLBACKS: CalendarEventCallbacks = {
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
}

function baseProps(
	overrides: Partial<React.ComponentProps<typeof PlanningCalendarUI>> = {},
) {
	return {
		organizationName: 'Atelier Bois & Co',
		organizationSlug: 'atelier-bois',
		pendingReportsCount: null,
		view: 'week' as const,
		date: '2026-03-02',
		windowFrom: '2026-03-02',
		windowTo: '2026-03-02',
		filter: 'all' as const,
		members: [],
		selectedMemberIds: [],
		isLoading: false,
		error: null,
		model: buildCalendarModel({
			from: '2026-03-02',
			to: '2026-03-02',
			entries: [],
			resources: [],
			workTime: [],
			timeZone: 'UTC',
			today: '2026-03-02',
			filter: 'all',
		}),
		monthModel: null,
		onViewChange: vi.fn(),
		onDateChange: vi.fn(),
		onFilterChange: vi.fn(),
		onToggleMember: vi.fn(),
		onResetMembers: vi.fn(),
		onCreate: vi.fn(),
		eventCallbacks: EVENT_CALLBACKS,
		onRetry: vi.fn(),
		now: new Date('2026-03-02T09:00:00Z'),
		assigneeOptions: [],
		onQuickCreate: vi.fn(),
		onQuickCreateMoreOptions: vi.fn(),
		...overrides,
	}
}

describe('PlanningCalendarUI — a period with nothing planned', () => {
	it('shows the grid itself instead of a placeholder screen', async () => {
		await renderWithRouter(<PlanningCalendarUI {...baseProps()} />)

		// The grid's own day header (2 mars) is what tells the period is empty —
		// no separate "nothing here" screen replacing it.
		expect(screen.getByText('02')).toBeDefined()
		expect(screen.queryByText('Rien de planifié sur cette période')).toBeNull()
	})

	it('still calls out entries hidden by a filter — that is not "nothing planned"', async () => {
		const model = buildCalendarModel({
			from: '2026-03-02',
			to: '2026-03-02',
			entries: [LEAVE],
			resources: [],
			workTime: [],
			timeZone: 'UTC',
			today: '2026-03-02',
			filter: 'task',
		})

		await renderWithRouter(
			<PlanningCalendarUI {...baseProps({ model, filter: 'task' })} />,
		)

		expect(screen.getByText('02')).toBeDefined()
		expect(
			screen.getByText(/1 entrée masquée par les filtres en cours/),
		).toBeDefined()
	})
})

describe('PlanningCalendarUI — loading and error', () => {
	it('shows a loading notice instead of the grid while fetching', async () => {
		await renderWithRouter(
			<PlanningCalendarUI {...baseProps({ isLoading: true })} />,
		)

		expect(screen.getByText('Chargement du calendrier…')).toBeDefined()
	})

	it('shows an error notice with a retry action instead of the grid', async () => {
		const onRetry = vi.fn()
		await renderWithRouter(
			<PlanningCalendarUI
				{...baseProps({ error: 'Le serveur ne répond pas', onRetry })}
			/>,
		)

		expect(screen.getByText('Planning indisponible')).toBeDefined()
		expect(screen.getByText('Le serveur ne répond pas')).toBeDefined()
	})
})
