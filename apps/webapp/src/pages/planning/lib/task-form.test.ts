import { describe, expect, it } from 'vitest'
import {
	buildCreateTaskPayload,
	buildFollowUpPatchPayload,
	buildPatchTaskPayload,
	calendarSelectionToDateRange,
	dateRangeToCalendarSelection,
	emptyTaskDraft,
	expensesToCents,
	formatDateRangeFr,
	needsFollowUpPatch,
	shiftEndTimeForNewStartTime,
	taskToDraft,
	timeOptionsWith,
	validateTaskDraft,
} from '#/pages/planning/lib/task-form'

const TIME_ZONE = 'Europe/Paris'

function rootDraft(overrides: Partial<ReturnType<typeof emptyTaskDraft>> = {}) {
	return {
		...emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' }),
		title: 'Réunion de projet',
		startDate: '2026-08-10',
		endDate: '2026-08-10',
		startTime: '09:00',
		endTime: '10:00',
		...overrides,
	}
}

describe('emptyTaskDraft', () => {
	it('defaults blocksAvailability to true — never inferred, always shown', () => {
		const draft = emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' })
		expect(draft.blocksAvailability).toBe(true)
	})

	it('seeds a root draft with today as both dates', () => {
		const draft = emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' })
		expect(draft.startDate).toBe('2026-08-10')
		expect(draft.endDate).toBe('2026-08-10')
	})

	it('seeds a subtask draft with no dates — the placeholder-shows-inheritance case', () => {
		const draft = emptyTaskDraft({
			parentTaskId: 'task-1',
			today: '2026-08-10',
		})
		expect(draft.startDate).toBe('')
		expect(draft.endDate).toBe('')
	})

	it('seeds an empty assignees and labelIds list', () => {
		const draft = emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' })
		expect(draft.assignees).toEqual([])
		expect(draft.labelIds).toEqual([])
	})
})

describe('validateTaskDraft', () => {
	it('rejects a blank title', () => {
		const errors = validateTaskDraft(rootDraft({ title: '   ' }), {
			isSubtask: false,
		})
		expect(errors).toContain('Titre requis')
	})

	it('rejects a root task without dates', () => {
		const errors = validateTaskDraft(
			rootDraft({ startDate: '', endDate: '' }),
			{ isSubtask: false },
		)
		expect(errors.length).toBeGreaterThan(0)
	})

	it('accepts a subtask without dates — it inherits the parent window', () => {
		const errors = validateTaskDraft(
			rootDraft({ startDate: '', endDate: '' }),
			{ isSubtask: true },
		)
		expect(errors).toEqual([])
	})

	it('accepts a subtask with its own dates', () => {
		const errors = validateTaskDraft(rootDraft(), { isSubtask: true })
		expect(errors).toEqual([])
	})

	it('accepts a customer without a customer context', () => {
		const errors = validateTaskDraft(
			rootDraft({ customerId: 'cust-1', customerContextId: '' }),
			{ isSubtask: false },
		)
		expect(errors).toEqual([])
	})

	it('rejects a customer context without a customer', () => {
		const errors = validateTaskDraft(
			rootDraft({ customerId: '', customerContextId: 'ctx-1' }),
			{ isSubtask: false },
		)
		expect(errors.length).toBeGreaterThan(0)
	})

	it('rejects an end before the start when not all-day', () => {
		const errors = validateTaskDraft(
			rootDraft({ startTime: '11:00', endTime: '10:00' }),
			{ isSubtask: false },
		)
		expect(errors.length).toBeGreaterThan(0)
	})

	it('accepts a valid root draft with a client', () => {
		const errors = validateTaskDraft(
			rootDraft({ customerId: 'cust-1', customerContextId: 'ctx-1' }),
			{ isSubtask: false },
		)
		expect(errors).toEqual([])
	})
})

describe('buildCreateTaskPayload — devis', () => {
	/**
	 * The link the profitability report reads to state a margin. Asserted on the
	 * payload rather than on the form, because the field can render perfectly and
	 * still not be transmitted, which is exactly what happened before this test
	 * existed.
	 */
	it('carries the chosen quote', () => {
		const payload = buildCreateTaskPayload(rootDraft({ quoteId: 'quote-1' }), {
			parentTaskId: null,
			timeZone: TIME_ZONE,
		})

		expect(payload?.quote_id).toBe('quote-1')
	})

	it('sends null rather than an empty string when none was chosen', () => {
		const payload = buildCreateTaskPayload(rootDraft({ quoteId: '' }), {
			parentTaskId: null,
			timeZone: TIME_ZONE,
		})

		expect(payload?.quote_id).toBeNull()
	})
})

describe('buildCreateTaskPayload — without a customer', () => {
	it('sends null customer_id/customer_context_id for a task with no client', () => {
		const payload = buildCreateTaskPayload(rootDraft(), {
			parentTaskId: null,
			timeZone: TIME_ZONE,
		})

		expect(payload).not.toBeNull()
		expect(payload?.customer_id).toBeNull()
		expect(payload?.customer_context_id).toBeNull()
	})

	it('sends the trimmed title and description', () => {
		const payload = buildCreateTaskPayload(
			rootDraft({ title: '  Réunion  ', description: '  ordre du jour  ' }),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)

		expect(payload?.title).toBe('Réunion')
		expect(payload?.description).toBe('ordre du jour')
	})

	it('sends null description when blank', () => {
		const payload = buildCreateTaskPayload(rootDraft({ description: '   ' }), {
			parentTaskId: null,
			timeZone: TIME_ZONE,
		})

		expect(payload?.description).toBeNull()
	})

	it('carries blocks_availability verbatim, never inferred', () => {
		const blocking = buildCreateTaskPayload(
			rootDraft({ blocksAvailability: true }),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)
		const nonBlocking = buildCreateTaskPayload(
			rootDraft({ blocksAvailability: false }),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)

		expect(blocking?.blocks_availability).toBe(true)
		expect(nonBlocking?.blocks_availability).toBe(false)
	})
})

describe('buildCreateTaskPayload — with a customer', () => {
	it('sends both customer_id and customer_context_id for a projet', () => {
		const payload = buildCreateTaskPayload(
			rootDraft({ customerId: 'cust-1', customerContextId: 'ctx-1' }),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)

		expect(payload?.customer_id).toBe('cust-1')
		expect(payload?.customer_context_id).toBe('ctx-1')
	})

	it('sends customer_id with a null customer_context_id when no context was picked', () => {
		const payload = buildCreateTaskPayload(
			rootDraft({ customerId: 'cust-1', customerContextId: '' }),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)

		expect(payload?.customer_id).toBe('cust-1')
		expect(payload?.customer_context_id).toBeNull()
	})
})

describe('buildCreateTaskPayload — racine', () => {
	it('returns null when the root has no dates', () => {
		const payload = buildCreateTaskPayload(
			rootDraft({ startDate: '', endDate: '' }),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)

		expect(payload).toBeNull()
	})

	it('resolves starts_at/ends_at from the local date and time', () => {
		const payload = buildCreateTaskPayload(rootDraft(), {
			parentTaskId: null,
			timeZone: TIME_ZONE,
		})

		expect(payload?.starts_at).toBe(
			new Date('2026-08-10T07:00:00.000Z').toISOString(),
		)
		expect(payload?.ends_at).toBe(
			new Date('2026-08-10T08:00:00.000Z').toISOString(),
		)
	})

	it('never carries a parent_task_id', () => {
		const payload = buildCreateTaskPayload(rootDraft(), {
			parentTaskId: null,
			timeZone: TIME_ZONE,
		})

		expect(payload?.parent_task_id ?? null).toBeNull()
	})
})

describe('buildCreateTaskPayload — subtask', () => {
	it('sends null starts_at/ends_at when the subtask has no dates of its own', () => {
		const payload = buildCreateTaskPayload(
			rootDraft({ startDate: '', endDate: '', startTime: '', endTime: '' }),
			{ parentTaskId: 'parent-1', timeZone: TIME_ZONE },
		)

		expect(payload).not.toBeNull()
		expect(payload?.starts_at).toBeNull()
		expect(payload?.ends_at).toBeNull()
		expect(payload?.parent_task_id).toBe('parent-1')
	})

	it('sends its own starts_at/ends_at when the subtask sets them', () => {
		const payload = buildCreateTaskPayload(rootDraft(), {
			parentTaskId: 'parent-1',
			timeZone: TIME_ZONE,
		})

		expect(payload?.starts_at).not.toBeNull()
		expect(payload?.parent_task_id).toBe('parent-1')
	})
})

describe('buildCreateTaskPayload — full day', () => {
	it('resolves an all-day window to local midnight boundaries, end exclusive', () => {
		const payload = buildCreateTaskPayload(
			rootDraft({
				allDay: true,
				startDate: '2026-08-10',
				endDate: '2026-08-10',
			}),
			{ parentTaskId: null, timeZone: TIME_ZONE },
		)

		expect(payload?.starts_at).toBe(
			new Date('2026-08-09T22:00:00.000Z').toISOString(),
		)
		expect(payload?.ends_at).toBe(
			new Date('2026-08-10T22:00:00.000Z').toISOString(),
		)
	})
})

describe('needsFollowUpPatch', () => {
	it('is false when no assignees and no labels were picked', () => {
		expect(needsFollowUpPatch({ assignees: [], labelIds: [] })).toBe(false)
	})

	it('is true when at least one assignee was picked', () => {
		expect(
			needsFollowUpPatch({
				assignees: [{ member_id: 'member-1' }],
				labelIds: [],
			}),
		).toBe(true)
	})

	it('is true when at least one label was picked', () => {
		expect(needsFollowUpPatch({ assignees: [], labelIds: ['label-1'] })).toBe(
			true,
		)
	})
})

describe('buildFollowUpPatchPayload', () => {
	it('carries the complete assignees and label_ids lists', () => {
		const payload = buildFollowUpPatchPayload({
			assignees: [{ member_id: 'member-1' }],
			labelIds: ['label-1', 'label-2'],
		})

		expect(payload.assignees).toEqual([{ member_id: 'member-1' }])
		expect(payload.label_ids).toEqual(['label-1', 'label-2'])
	})
})

describe('buildPatchTaskPayload', () => {
	it('sends label_ids as the complete list, never a delta', () => {
		const payload = buildPatchTaskPayload(
			rootDraft({ labelIds: ['label-1', 'label-2'] }),
			{ isSubtask: false, timeZone: TIME_ZONE },
		)

		expect(payload?.label_ids).toEqual(['label-1', 'label-2'])
	})

	it('sends an empty label_ids array to remove every label', () => {
		const payload = buildPatchTaskPayload(rootDraft({ labelIds: [] }), {
			isSubtask: false,
			timeZone: TIME_ZONE,
		})

		expect(payload?.label_ids).toEqual([])
	})

	it('sends assignees as the complete list, never a delta', () => {
		const payload = buildPatchTaskPayload(
			rootDraft({
				assignees: [{ member_id: 'member-1' }, { member_id: 'member-2' }],
			}),
			{ isSubtask: false, timeZone: TIME_ZONE },
		)

		expect(payload?.assignees).toEqual([
			{ member_id: 'member-1' },
			{ member_id: 'member-2' },
		])
	})

	it('sends an empty assignees array to unassign everyone', () => {
		const payload = buildPatchTaskPayload(rootDraft({ assignees: [] }), {
			isSubtask: false,
			timeZone: TIME_ZONE,
		})

		expect(payload?.assignees).toEqual([])
	})

	it('clears a subtask back to inheriting when its dates are emptied', () => {
		const payload = buildPatchTaskPayload(
			rootDraft({ startDate: '', endDate: '', startTime: '', endTime: '' }),
			{ isSubtask: true, timeZone: TIME_ZONE },
		)

		expect(payload?.starts_at).toBeNull()
		expect(payload?.ends_at).toBeNull()
	})

	it('never carries customer_id/customer_context_id — PATCH cannot change the client', () => {
		const payload = buildPatchTaskPayload(
			rootDraft({ customerId: 'cust-1', customerContextId: 'ctx-1' }),
			{ isSubtask: false, timeZone: TIME_ZONE },
		)

		expect(payload).not.toHaveProperty('customer_id')
		expect(payload).not.toHaveProperty('customer_context_id')
	})

	it('returns null for an invalid draft', () => {
		const payload = buildPatchTaskPayload(rootDraft({ title: '   ' }), {
			isSubtask: false,
			timeZone: TIME_ZONE,
		})

		expect(payload).toBeNull()
	})
})

describe('taskToDraft', () => {
	it('maps a non-all-day task’s ISO dates to local date/time fields', () => {
		const draft = taskToDraft(
			{
				title: 'Réunion',
				description: null,
				all_day: false,
				starts_at: '2026-08-10T07:00:00.000Z',
				ends_at: '2026-08-10T08:00:00.000Z',
				blocks_availability: true,
				labels: [],
				member_ids: [],
			},
			TIME_ZONE,
		)

		expect(draft.startDate).toBe('2026-08-10')
		expect(draft.startTime).toBe('09:00')
		expect(draft.endDate).toBe('2026-08-10')
		expect(draft.endTime).toBe('10:00')
		expect(draft.allDay).toBe(false)
	})

	it('maps an all-day task’s exclusive end back to the inclusive last day', () => {
		const draft = taskToDraft(
			{
				title: 'Formation',
				description: null,
				all_day: true,
				starts_at: '2026-08-09T22:00:00.000Z',
				ends_at: '2026-08-11T22:00:00.000Z',
				blocks_availability: true,
				labels: [],
				member_ids: [],
			},
			TIME_ZONE,
		)

		expect(draft.startDate).toBe('2026-08-10')
		expect(draft.endDate).toBe('2026-08-11')
	})

	it('maps labels to labelIds and member_ids to assignees', () => {
		const draft = taskToDraft(
			{
				title: 'Projet',
				description: 'Toiture',
				all_day: false,
				starts_at: '2026-08-10T07:00:00.000Z',
				ends_at: '2026-08-10T08:00:00.000Z',
				blocks_availability: true,
				labels: [{ id: 'l1' }, { id: 'l2' }],
				member_ids: ['member-1'],
			},
			TIME_ZONE,
		)

		expect(draft.labelIds).toEqual(['l1', 'l2'])
		expect(draft.assignees).toEqual([{ member_id: 'member-1' }])
		expect(draft.description).toBe('Toiture')
	})

	it('leaves the date fields empty for a dateless subtask — inherits the parent window', () => {
		const draft = taskToDraft(
			{
				title: 'Sous-tâche',
				description: null,
				all_day: false,
				starts_at: null,
				ends_at: null,
				blocks_availability: true,
				labels: [],
				member_ids: [],
			},
			TIME_ZONE,
		)

		expect(draft.startDate).toBe('')
		expect(draft.endDate).toBe('')
	})

	it('never carries a customerId/customerContextId — the edit form never lets them be touched', () => {
		const draft = taskToDraft(
			{
				title: 'Projet',
				description: null,
				all_day: false,
				starts_at: '2026-08-10T07:00:00.000Z',
				ends_at: '2026-08-10T08:00:00.000Z',
				blocks_availability: true,
				labels: [],
				member_ids: [],
			},
			TIME_ZONE,
		)

		expect(draft.customerId).toBe('')
		expect(draft.customerContextId).toBe('')
	})
})

describe('timeOptionsWith', () => {
	it('offers every half hour of the day', () => {
		const options = timeOptionsWith('09:00')
		expect(options[0]).toBe('00:00')
		expect(options.at(-1)).toBe('23:30')
		expect(options).toContain('09:00')
		expect(options).toContain('14:30')
	})

	it('adds an odd value not already on the half-hour grid, without duplicating one that is', () => {
		expect(timeOptionsWith('09:07')).toContain('09:07')
		expect(
			timeOptionsWith('09:00').filter((time) => time === '09:00'),
		).toHaveLength(1)
	})
})

describe('dateRangeToCalendarSelection / calendarSelectionToDateRange', () => {
	it('round-trips a single-day range', () => {
		const selection = dateRangeToCalendarSelection('2026-08-10', '2026-08-10')
		expect(calendarSelectionToDateRange(selection)).toEqual({
			startDate: '2026-08-10',
			endDate: '2026-08-10',
		})
	})

	it('round-trips a multi-day range', () => {
		const selection = dateRangeToCalendarSelection('2026-08-10', '2026-08-14')
		expect(calendarSelectionToDateRange(selection)).toEqual({
			startDate: '2026-08-10',
			endDate: '2026-08-14',
		})
	})

	it('is undefined when either date is blank — a subtask inheriting its parent window', () => {
		expect(dateRangeToCalendarSelection('', '')).toBeUndefined()
		expect(dateRangeToCalendarSelection('2026-08-10', '')).toBeUndefined()
	})

	it('folds a from-only click (no `to` yet) into a one-day range', () => {
		expect(
			calendarSelectionToDateRange({ from: new Date('2026-08-10T00:00:00Z') }),
		).toEqual({ startDate: '2026-08-10', endDate: '2026-08-10' })
	})

	it('is null with no selection at all', () => {
		expect(calendarSelectionToDateRange(undefined)).toBeNull()
	})
})

describe('formatDateRangeFr', () => {
	it('shows one date for a single day', () => {
		expect(formatDateRangeFr('2026-08-10', '2026-08-10')).toBe('10/08/2026')
	})

	it('shows a "from – to" range for a multi-day span', () => {
		expect(formatDateRangeFr('2026-08-10', '2026-08-14')).toBe(
			'10/08/2026 – 14/08/2026',
		)
	})
})

describe('shiftEndTimeForNewStartTime', () => {
	it('preserves the duration on a single-day window', () => {
		const shifted = shiftEndTimeForNewStartTime(
			{
				startDate: '2026-08-10',
				endDate: '2026-08-10',
				startTime: '09:00',
				endTime: '10:00',
			},
			'11:00',
		)
		expect(shifted).toBe('12:00')
	})

	it('leaves the end time untouched on a multi-day window', () => {
		const shifted = shiftEndTimeForNewStartTime(
			{
				startDate: '2026-08-10',
				endDate: '2026-08-12',
				startTime: '09:00',
				endTime: '10:00',
			},
			'11:00',
		)
		expect(shifted).toBe('10:00')
	})

	it('leaves the end time untouched when the current window is already inverted', () => {
		const shifted = shiftEndTimeForNewStartTime(
			{
				startDate: '2026-08-10',
				endDate: '2026-08-10',
				startTime: '10:00',
				endTime: '09:00',
			},
			'11:00',
		)
		expect(shifted).toBe('09:00')
	})

	it('clamps to the last slot of the day instead of rolling into the next day', () => {
		const shifted = shiftEndTimeForNewStartTime(
			{
				startDate: '2026-08-10',
				endDate: '2026-08-10',
				startTime: '22:00',
				endTime: '23:00',
			},
			'23:30',
		)
		expect(shifted).toBe('23:45')
	})
})

describe('expensesToCents', () => {
	/** An empty field is "nothing to declare", which is the common case. */
	it('reads an empty field as zero', () => {
		expect(expensesToCents('')).toBe(0)
		expect(expensesToCents('   ')).toBe(0)
	})

	it('accepts a comma as the decimal separator', () => {
		expect(expensesToCents('45,50')).toBe(4550)
		expect(expensesToCents('45.50')).toBe(4550)
	})

	it('refuses what is not a positive number', () => {
		expect(expensesToCents('quarante-cinq')).toBeNull()
		expect(expensesToCents('-1')).toBeNull()
	})
})

describe('expenses on the task payloads', () => {
	const draft = () => ({
		...emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' }),
		title: 'Mission Clermont',
	})

	it('refuses an amount with no reason', () => {
		const errors = validateTaskDraft(
			{ ...draft(), expensesEuros: '45' },
			{ isSubtask: false },
		)

		expect(errors).toContain('Un montant de frais doit être justifié')
	})

	it('accepts an amount once it is justified', () => {
		const errors = validateTaskDraft(
			{ ...draft(), expensesEuros: '45', expensesLabel: 'Déplacement' },
			{ isSubtask: false },
		)

		expect(errors).toEqual([])
	})

	it('carries the amount, its label and the project on create', () => {
		const payload = buildCreateTaskPayload(
			{
				...draft(),
				projectId: 'project-1',
				expensesEuros: '45',
				expensesLabel: 'Déplacement',
			},
			{ parentTaskId: null, timeZone: 'Europe/Paris' },
		)

		expect(payload?.project_id).toBe('project-1')
		expect(payload?.expenses_cents).toBe(4500)
		expect(payload?.expenses_label).toBe('Déplacement')
	})

	/**
	 * Unlike the customer and the quote, the project is editable after
	 * creation — `PATCH /tasks/{id}` is the single way a task joins or leaves
	 * one.
	 */
	it('carries the project on patch, and detaches with null', () => {
		const attached = buildPatchTaskPayload(
			{ ...draft(), projectId: 'project-1' },
			{ isSubtask: false, timeZone: 'Europe/Paris' },
		)
		expect(attached?.project_id).toBe('project-1')

		const detached = buildPatchTaskPayload(draft(), {
			isSubtask: false,
			timeZone: 'Europe/Paris',
		})
		expect(detached?.project_id).toBeNull()
	})

	it('drops the label when the amount goes back to zero', () => {
		const payload = buildPatchTaskPayload(
			{ ...draft(), expensesEuros: '', expensesLabel: 'Déplacement' },
			{ isSubtask: false, timeZone: 'Europe/Paris' },
		)

		expect(payload?.expenses_cents).toBe(0)
		expect(payload?.expenses_label).toBeNull()
	})
})
