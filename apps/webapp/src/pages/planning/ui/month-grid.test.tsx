import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { buildMonthModel } from '#/pages/planning/lib/build-month-model'
import type { PlanningEntry, PlanningResource } from '#/pages/planning/types'
import type { CalendarEventCallbacks } from './event-popover'
import { MonthGrid } from './month-grid'

const RESOURCE: PlanningResource = {
	member_id: 'e-1',
	resource_id: 'r-1',
	display_name: 'Marie Leroy',
	weekly_contract_minutes: 2100,
}

const TIMED_TASK: PlanningEntry = {
	kind: 'task',
	id: 't-1',
	title: 'Taille de haie',
	starts_at: '2026-03-02T09:00:00Z',
	ends_at: '2026-03-02T11:00:00Z',
	all_day: false,
	blocks_availability: true,
	child_count: 0,
	member_ids: ['e-1'],
	labels: [],
	status: 'PLANNED',
} as PlanningEntry

const ALL_DAY_TASK: PlanningEntry = {
	kind: 'task',
	id: 't-2',
	title: 'Chantier toiture',
	starts_at: '2026-03-02T00:00:00Z',
	ends_at: '2026-03-03T00:00:00Z',
	all_day: true,
	blocks_availability: true,
	child_count: 0,
	member_ids: ['e-1'],
	labels: [],
	status: 'PLANNED',
} as PlanningEntry

const CALLBACKS: CalendarEventCallbacks = {
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

describe('MonthGrid — attendees', () => {
	it('shows a timed entry’s assignee on its row', () => {
		const model = buildMonthModel({
			from: '2026-03-02',
			to: '2026-03-08',
			month: '2026-03',
			entries: [TIMED_TASK],
			resources: [RESOURCE],
			timeZone: 'UTC',
			today: '2026-03-02',
			filter: 'all',
		})

		render(<MonthGrid model={model} callbacks={CALLBACKS} />)

		expect(screen.getByText('ML')).toBeDefined()
	})

	it('shows an all-day entry’s assignee on its banner', () => {
		const model = buildMonthModel({
			from: '2026-03-02',
			to: '2026-03-08',
			month: '2026-03',
			entries: [ALL_DAY_TASK],
			resources: [RESOURCE],
			timeZone: 'UTC',
			today: '2026-03-02',
			filter: 'all',
		})

		render(<MonthGrid model={model} callbacks={CALLBACKS} />)

		expect(screen.getByText('ML')).toBeDefined()
	})
})
