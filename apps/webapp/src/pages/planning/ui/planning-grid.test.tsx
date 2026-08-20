import { fireEvent, render, screen, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { PlanningEntry, PlanningResource } from '#/pages/planning/types'
import { PlanningGrid } from '#/pages/planning/ui/planning-grid'

/** jsdom has no `DataTransfer`; a plain object backed by a `Map` is enough for `setData`/`getData`. */
function fakeDataTransfer() {
	const store = new Map<string, string>()
	return {
		setData: (format: string, data: string) => store.set(format, data),
		getData: (format: string) => store.get(format) ?? '',
	}
}

const TZ = 'Europe/Paris'

function memberResource(
	overrides: Partial<PlanningResource> = {},
): PlanningResource {
	return {
		resource_id: 'member:member-1',
		member_id: 'member-1',
		employee_id: 'employee-1',
		display_name: 'Alix Martin',
		hourly_rate_cents: 1500,
		weekly_contract_minutes: 2100,
		...overrides,
	}
}

function task(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'task',
		labels: [],
		title: 'Tâche',
		blocks_availability: true,
		child_count: 0,
		id: 'wo-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
		status: 'PLANNED',
		member_ids: ['member-1'],
		customer_name: 'Client Dupont',
		context_label: 'Projet toiture',
		...overrides,
	} as PlanningEntry
}

function absence(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'absence',
		id: 'ab-1',
		starts_at: '2026-08-10T00:00:00+02:00',
		ends_at: '2026-08-11T00:00:00+02:00',
		all_day: true,
		absence_kind: 'LEAVE',
		member_id: 'member-1',
		...overrides,
	} as PlanningEntry
}

function unknownKindEntry(): PlanningEntry {
	return {
		kind: 'external_source',
		id: 'ext-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
	} as unknown as PlanningEntry
}

describe('PlanningGrid — axe ressource', () => {
	it('shows one row per resource, whatever the view', () => {
		const resources = [
			memberResource(),
			memberResource({
				resource_id: 'member:member-2',
				member_id: 'member-2',
				display_name: 'Marie Leroy',
			}),
		]

		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={resources}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.getByText('Alix Martin')).toBeDefined()
		expect(screen.getByText('Marie Leroy')).toBeDefined()
		expect(screen.getAllByTestId('grid-row')).toHaveLength(2)
	})

	it('shows the empty state without crashing when there is no resource', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={[]}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.queryByTestId('planning-grid')).toBeNull()
		expect(screen.getByText(/Aucune ressource/)).toBeDefined()
	})
})

describe('PlanningGrid — vue semaine', () => {
	it('renders one column per day of the window', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.getAllByTestId('grid-cell')).toHaveLength(7)
	})

	it('positions a job segment with left/width as percentages', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('data-entry-id')).toBe('wo-1')
		expect(segment.getAttribute('data-tone')).toBe('task')
		expect(segment.style.left).toBe('0%')
		expect(segment.style.width).toBe('100%')
	})
})

describe('PlanningGrid — vue jour', () => {
	it('renders a single cell covering the whole day', () => {
		render(
			<PlanningGrid
				view="day"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.getAllByTestId('grid-cell')).toHaveLength(1)
	})

	it("shows the segment's label (room available)", () => {
		render(
			<PlanningGrid
				view="day"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task({ title: 'Réfection toiture' })]}
				workTime={[]}
			/>,
		)

		expect(screen.getByText('Réfection toiture')).toBeDefined()
	})
})

describe('PlanningGrid — vue mois', () => {
	it('marks the cells where an absence is running', () => {
		render(
			<PlanningGrid
				view="month"
				windowFrom="2026-08-01"
				windowTo="2026-08-31"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[absence()]}
				workTime={[]}
			/>,
		)

		const cellWithAbsence = screen
			.getAllByTestId('grid-cell')
			.find((cell) => cell.getAttribute('data-date') === '2026-08-10')
		expect(cellWithAbsence?.getAttribute('data-has-absence')).toBe('true')

		const cellWithoutAbsence = screen
			.getAllByTestId('grid-cell')
			.find((cell) => cell.getAttribute('data-date') === '2026-08-11')
		expect(cellWithoutAbsence?.getAttribute('data-has-absence')).toBe('false')
	})
})

describe('PlanningGrid — stacking overlaps', () => {
	it('gives a distinct row to each overlapping segment', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[
					task({
						id: 'wo-1',
						starts_at: '2026-08-10T08:00:00Z',
						ends_at: '2026-08-10T10:00:00Z',
					}),
					task({
						id: 'wo-2',
						starts_at: '2026-08-10T09:00:00Z',
						ends_at: '2026-08-10T11:00:00Z',
					}),
				]}
				workTime={[]}
			/>,
		)

		const cell = screen.getByTestId('grid-cell')
		const segments = within(cell).getAllByTestId('grid-segment')
		const tops = segments.map((segment) => segment.style.top)
		expect(new Set(tops).size).toBe(2)
	})
})

describe('PlanningGrid — resilience to unknown kinds', () => {
	it('does not crash when an entry carries an unknown kind, and renders the rest normally', () => {
		expect(() =>
			render(
				<PlanningGrid
					view="week"
					windowFrom="2026-08-10"
					windowTo="2026-08-10"
					timeZone={TZ}
					resources={[memberResource()]}
					entries={[task(), unknownKindEntry()]}
					workTime={[]}
				/>,
			),
		).not.toThrow()

		expect(screen.getAllByTestId('grid-segment')).toHaveLength(1)
	})
})

describe('PlanningGrid — drag & drop', () => {
	function twoResources() {
		return [
			memberResource(),
			memberResource({
				resource_id: 'member:member-2',
				member_id: 'member-2',
				display_name: 'Marie Leroy',
			}),
		]
	}

	it('a job segment is draggable and carries entryId/resourceId/date on dragstart', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('draggable')).toBe('true')
	})

	it('a drop on another row, same date, calls onDropTask with the target resource', () => {
		const onDropTask = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={twoResources()}
				entries={[task()]}
				workTime={[]}
				onDropTask={onDropTask}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEvent.dragStart(segment, { dataTransfer })

		const rows = screen.getAllByTestId('grid-row')
		const targetRow = rows.find(
			(row) => row.getAttribute('data-resource-id') === 'member:member-2',
		)
		if (!targetRow) throw new Error('target row not found')
		const targetCell = within(targetRow).getByTestId('grid-cell')

		fireEvent.drop(targetCell, { dataTransfer })

		expect(onDropTask).toHaveBeenCalledWith({
			entryId: 'wo-1',
			sourceResourceId: 'member:member-1',
			sourceDate: '2026-08-10',
			targetResourceId: 'member:member-2',
			targetDate: '2026-08-10',
		})
	})

	it('a drop on the origin cell (nothing changes) still calls the callback with targets identical to the sources', () => {
		const onDropTask = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
				onDropTask={onDropTask}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEvent.dragStart(segment, { dataTransfer })
		fireEvent.drop(screen.getByTestId('grid-cell'), { dataTransfer })

		expect(onDropTask).toHaveBeenCalledWith({
			entryId: 'wo-1',
			sourceResourceId: 'member:member-1',
			sourceDate: '2026-08-10',
			targetResourceId: 'member:member-1',
			targetDate: '2026-08-10',
		})
	})

	it('without onDropTask, a drop does not crash', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEvent.dragStart(segment, { dataTransfer })
		expect(() =>
			fireEvent.drop(screen.getByTestId('grid-cell'), { dataTransfer }),
		).not.toThrow()
	})
})

describe('PlanningGrid — removing an assignee', () => {
	it("the remove button on a job segment calls onRemoveAssignee with the row's entryId and resourceId", () => {
		const onRemoveAssignee = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
				onRemoveAssignee={onRemoveAssignee}
			/>,
		)

		fireEvent.click(
			screen.getByRole('button', { name: /Retirer cette personne/ }),
		)

		expect(onRemoveAssignee).toHaveBeenCalledWith({
			entryId: 'wo-1',
			resourceId: 'member:member-1',
		})
	})

	it('shows no remove button without onRemoveAssignee', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
			/>,
		)

		expect(
			screen.queryByRole('button', { name: /Retirer cette personne/ }),
		).toBeNull()
	})
})

describe('PlanningGrid — segment d’absence inerte', () => {
	it('clicking an absence segment triggers no interaction (no button role, no onClick)', () => {
		const onOpenTask = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[absence()]}
				workTime={[]}
				onOpenTask={onOpenTask}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('role')).toBeNull()
		expect(segment.getAttribute('tabindex')).toBeNull()
		// A click must not throw and must not carry any accessible "button" affordance.
		fireEvent.click(segment)
		expect(screen.queryAllByRole('button')).toHaveLength(0)
		expect(onOpenTask).not.toHaveBeenCalled()
	})
})

describe("PlanningGrid — opening a task's detail", () => {
	it('clicking a task segment calls onOpenTask with the entryId', () => {
		const onOpenTask = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
				onOpenTask={onOpenTask}
			/>,
		)

		fireEvent.click(screen.getByTestId('grid-segment'))

		expect(onOpenTask).toHaveBeenCalledWith({ entryId: 'wo-1' })
	})

	it('carries a button role and a tabIndex when onOpenTask is provided', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
				onOpenTask={vi.fn()}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('role')).toBe('button')
		expect(segment.getAttribute('tabindex')).toBe('0')
	})

	it('carries neither button role nor tabIndex without onOpenTask, and a click does not crash', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('role')).toBeNull()
		expect(segment.getAttribute('tabindex')).toBeNull()
		expect(() => fireEvent.click(segment)).not.toThrow()
	})

	it('the Enter key on a focused segment calls onOpenTask', () => {
		const onOpenTask = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
				onOpenTask={onOpenTask}
			/>,
		)

		fireEvent.keyDown(screen.getByTestId('grid-segment'), { key: 'Enter' })

		expect(onOpenTask).toHaveBeenCalledWith({ entryId: 'wo-1' })
	})

	it('clicking the remove button does not also fire onOpenTask (stopPropagation)', () => {
		const onOpenTask = vi.fn()
		const onRemoveAssignee = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task()]}
				workTime={[]}
				onOpenTask={onOpenTask}
				onRemoveAssignee={onRemoveAssignee}
			/>,
		)

		fireEvent.click(
			screen.getByRole('button', { name: /Retirer cette personne/ }),
		)

		expect(onRemoveAssignee).toHaveBeenCalledTimes(1)
		expect(onOpenTask).not.toHaveBeenCalled()
	})
})

describe('PlanningGrid — pastilles de labels', () => {
	it('shows one dot per label the task carries', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[
					task({
						labels: [
							{
								id: 'label-1',
								organization_id: 'org-1',
								name: 'Réunion',
								color: '#2563EB',
								created_at: '2026-01-01T00:00:00Z',
								updated_at: '2026-01-01T00:00:00Z',
							},
						],
					}),
				]}
				workTime={[]}
			/>,
		)

		expect(screen.getByTitle('Réunion')).toBeDefined()
	})

	it('a segment with no labels renders no dot', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task({ labels: [] })]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(
			segment.querySelectorAll('[style*="background-color"]'),
		).toHaveLength(0)
	})
})

describe('PlanningGrid — no network call', () => {
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

	it('fires no fetch on render', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={[memberResource()]}
				entries={[task(), absence()]}
				workTime={[]}
			/>,
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
