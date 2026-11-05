import { describe, expect, it } from 'vitest'
import {
	applyStatus,
	computeColumnShift,
	computeDrop,
	groupTicketsByStatus,
	SUPPORT_COLUMNS,
	supportStatusDescription,
	supportStatusDot,
	supportStatusLabel,
	supportStatusTone,
} from '#/pages/support/lib/columns'
import type { SupportStatus } from '#/pages/support/types'

interface TicketStub {
	id: string
	status: SupportStatus
}

function ticket(id: string, status: SupportStatus): TicketStub {
	return { id, status }
}

const TICKETS: TicketStub[] = [
	ticket('a', 'new'),
	ticket('b', 'in_progress'),
	ticket('c', 'new'),
	ticket('d', 'resolved'),
	ticket('e', 'new'),
]

describe('SUPPORT_COLUMNS', () => {
	it('lists the four statuses in workflow order', () => {
		expect(SUPPORT_COLUMNS.map((column) => column.status)).toEqual([
			'new',
			'in_progress',
			'waiting_customer',
			'resolved',
		])
	})

	it('labels them in French', () => {
		expect(SUPPORT_COLUMNS.map((column) => column.label)).toEqual([
			'Backlog',
			'Ouvert',
			'En attente client',
			'Fermé',
		])
	})
})

describe('supportStatusLabel and supportStatusTone', () => {
	it('reads the label of every status', () => {
		expect(supportStatusLabel('new')).toBe('Backlog')
		expect(supportStatusLabel('in_progress')).toBe('Ouvert')
		expect(supportStatusLabel('waiting_customer')).toBe('En attente client')
		expect(supportStatusLabel('resolved')).toBe('Fermé')
	})

	it('maps each status to its tone', () => {
		expect(supportStatusTone('new')).toBe('neutral')
		expect(supportStatusTone('in_progress')).toBe('brand')
		expect(supportStatusTone('waiting_customer')).toBe('warning')
		expect(supportStatusTone('resolved')).toBe('success')
	})
})

describe('supportStatusDot and supportStatusDescription', () => {
	it('maps each status to its dot tone', () => {
		expect(supportStatusDot('new')).toBe('neutral')
		expect(supportStatusDot('in_progress')).toBe('progress')
		expect(supportStatusDot('waiting_customer')).toBe('customer')
		expect(supportStatusDot('resolved')).toBe('done')
	})

	it('explains each status', () => {
		expect(supportStatusDescription('new')).toBe(
			'Reçu, pas encore pris en charge.',
		)
		expect(supportStatusDescription('resolved')).toBe('Problème résolu.')
	})
})

describe('groupTicketsByStatus', () => {
	it('keeps all four keys even when there is nothing to group', () => {
		expect(groupTicketsByStatus([])).toEqual({
			new: [],
			in_progress: [],
			waiting_customer: [],
			resolved: [],
		})
	})

	it('keeps the input order inside each column', () => {
		const grouped = groupTicketsByStatus(TICKETS)

		expect(grouped.new.map((entry) => entry.id)).toEqual(['a', 'c', 'e'])
		expect(grouped.in_progress.map((entry) => entry.id)).toEqual(['b'])
		expect(grouped.waiting_customer).toEqual([])
		expect(grouped.resolved.map((entry) => entry.id)).toEqual(['d'])
	})

	it('hands back the very same ticket objects', () => {
		const grouped = groupTicketsByStatus(TICKETS)

		expect(grouped.in_progress[0]).toBe(TICKETS[1])
	})
})

describe('computeDrop', () => {
	it('is unchanged when dropped on the column it already sits in', () => {
		expect(computeDrop(TICKETS, 'a', 'new')).toEqual({ changed: false })
	})

	it('reports the target status when dropped on another column', () => {
		expect(computeDrop(TICKETS, 'a', 'waiting_customer')).toEqual({
			changed: true,
			status: 'waiting_customer',
		})
	})

	it('is unchanged for a ticket that is not in the list', () => {
		expect(computeDrop(TICKETS, 'ghost', 'resolved')).toEqual({
			changed: false,
		})
	})
})

describe('computeColumnShift', () => {
	it('moves one column forward', () => {
		expect(computeColumnShift(TICKETS, 'a', 1)).toEqual({
			changed: true,
			status: 'in_progress',
		})
		expect(computeColumnShift(TICKETS, 'b', 1)).toEqual({
			changed: true,
			status: 'waiting_customer',
		})
	})

	it('moves one column back', () => {
		expect(computeColumnShift(TICKETS, 'd', -1)).toEqual({
			changed: true,
			status: 'waiting_customer',
		})
	})

	it('clamps at the first column', () => {
		expect(computeColumnShift(TICKETS, 'a', -1)).toEqual({ changed: false })
	})

	it('clamps at the last column', () => {
		expect(computeColumnShift(TICKETS, 'd', 1)).toEqual({ changed: false })
	})

	it('is unchanged for a ticket that is not in the list', () => {
		expect(computeColumnShift(TICKETS, 'ghost', 1)).toEqual({ changed: false })
	})
})

describe('applyStatus', () => {
	it('changes only the status of the targeted ticket and keeps the order', () => {
		const next = applyStatus(TICKETS, 'c', 'resolved')

		expect(next.map((entry) => entry.id)).toEqual(['a', 'b', 'c', 'd', 'e'])
		expect(next.map((entry) => entry.status)).toEqual([
			'new',
			'in_progress',
			'resolved',
			'resolved',
			'new',
		])
	})

	it('leaves the other tickets referentially equal and the input untouched', () => {
		const next = applyStatus(TICKETS, 'c', 'resolved')

		expect(next).not.toBe(TICKETS)
		expect(next[0]).toBe(TICKETS[0])
		expect(next[1]).toBe(TICKETS[1])
		expect(next[3]).toBe(TICKETS[3])
		expect(next[4]).toBe(TICKETS[4])
		expect(next[2]).not.toBe(TICKETS[2])
		expect(TICKETS[2].status).toBe('new')
	})

	it('leaves every ticket untouched when the id is unknown', () => {
		const next = applyStatus(TICKETS, 'ghost', 'resolved')

		expect(next).toEqual(TICKETS)
		next.forEach((entry, index) => {
			expect(entry).toBe(TICKETS[index])
		})
	})
})
