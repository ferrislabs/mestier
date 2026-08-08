import { describe, expect, it } from 'vitest'
import {
	canGoToNextPage,
	canGoToPreviousPage,
	employeeNamesById,
	formatAssigneeNames,
	resolveAssigneeNames,
	resolveSubtaskAllDay,
	taskHasChildren,
	toggleExpandedTask,
} from '#/pages/planning/lib/task-list'

describe('taskHasChildren', () => {
	it('is false for a root task with no children', () => {
		expect(taskHasChildren({ child_count: 0 })).toBe(false)
	})

	it('is false when child_count is absent', () => {
		expect(taskHasChildren({})).toBe(false)
	})

	it('is false when child_count is null', () => {
		expect(taskHasChildren({ child_count: null })).toBe(false)
	})

	it('is true for a root task with children', () => {
		expect(taskHasChildren({ child_count: 3 })).toBe(true)
	})
})

describe('canGoToNextPage / canGoToPreviousPage', () => {
	it('has no next/previous page when pagination has not loaded', () => {
		expect(canGoToNextPage(null)).toBe(false)
		expect(canGoToPreviousPage(undefined)).toBe(false)
	})

	it('reflects next_page/prev_page from the metadata', () => {
		const pagination = {
			current_page: 2,
			first_page: 1,
			is_empty: false,
			last_page: 3,
			next_page: 3,
			per_page: 20,
			prev_page: 1,
			total: 45,
		}
		expect(canGoToNextPage(pagination)).toBe(true)
		expect(canGoToPreviousPage(pagination)).toBe(true)
	})

	it('is false on the last/first page', () => {
		const lastPage = {
			current_page: 3,
			first_page: 1,
			is_empty: false,
			last_page: 3,
			next_page: null,
			per_page: 20,
			prev_page: 2,
			total: 45,
		}
		expect(canGoToNextPage(lastPage)).toBe(false)

		const firstPage = {
			...lastPage,
			current_page: 1,
			next_page: 2,
			prev_page: null,
		}
		expect(canGoToPreviousPage(firstPage)).toBe(false)
	})
})

describe('toggleExpandedTask', () => {
	it('adds a task id absent from the expanded set', () => {
		expect(toggleExpandedTask([], 'task-1')).toEqual(['task-1'])
		expect(toggleExpandedTask(['task-1'], 'task-2')).toEqual([
			'task-1',
			'task-2',
		])
	})

	it('removes a task id already in the expanded set', () => {
		expect(toggleExpandedTask(['task-1', 'task-2'], 'task-1')).toEqual([
			'task-2',
		])
	})

	it('never mutates the input array', () => {
		const expanded = ['task-1']
		toggleExpandedTask(expanded, 'task-2')
		expect(expanded).toEqual(['task-1'])
	})
})

describe('employeeNamesById', () => {
	it('maps employee-kind resources by employee id', () => {
		const resources = [
			{ employee_id: 'employee-1', display_name: 'Alix Martin' },
			{ employee_id: 'employee-2', display_name: 'Marie Leroy' },
		]
		expect(employeeNamesById(resources)).toEqual({
			'employee-1': 'Alix Martin',
			'employee-2': 'Marie Leroy',
		})
	})

	it('skips member-kind resources with no employee id', () => {
		const resources = [
			{ employee_id: null, display_name: 'Jules Petit' },
			{ display_name: 'Sans id' },
		]
		expect(employeeNamesById(resources)).toEqual({})
	})
})

describe('resolveAssigneeNames', () => {
	it('resolves each id to its display name', () => {
		const namesById = { 'employee-1': 'Alix Martin' }
		expect(resolveAssigneeNames(['employee-1'], namesById)).toEqual([
			'Alix Martin',
		])
	})

	it('falls back to a placeholder for an id missing from the roster', () => {
		expect(resolveAssigneeNames(['employee-9'], {})).toEqual([
			'Assigné inconnu',
		])
	})
})

describe('formatAssigneeNames', () => {
	it('joins several names with a comma', () => {
		expect(formatAssigneeNames(['Alix Martin', 'Marie Leroy'])).toBe(
			'Alix Martin, Marie Leroy',
		)
	})

	it('shows a placeholder for an unassigned task rather than a blank cell', () => {
		expect(formatAssigneeNames([])).toBe('Personne assigné')
	})
})

describe('resolveSubtaskAllDay', () => {
	it("uses the subtask's own all_day when its window is its own", () => {
		expect(resolveSubtaskAllDay(false, true, false)).toBe(true)
		expect(resolveSubtaskAllDay(false, false, true)).toBe(false)
	})

	it("uses the root's all_day when the window shown is the root's, inherited", () => {
		expect(resolveSubtaskAllDay(true, false, true)).toBe(true)
		expect(resolveSubtaskAllDay(true, true, false)).toBe(false)
	})
})
