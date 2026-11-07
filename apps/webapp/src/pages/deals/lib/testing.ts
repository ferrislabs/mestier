import type { Customer } from '#/hooks/use-customers'
import type { Deal, DealQuote } from '#/pages/deals/lib/deal-types'

export const TODAY = '2026-11-05'

export function deal(overrides: Partial<Deal> = {}): Deal {
	return {
		id: 'd1',
		customerId: 'c1',
		title: 'Remplacement de 4 fenêtres',
		amount: 150000,
		stage: 'new',
		expectedCloseDate: null,
		ownerId: null,
		source: null,
		lostReason: '',
		notes: '',
		createdAt: '2026-10-01',
		stageChangedAt: '2026-10-01',
		closedAt: null,
		...overrides,
	}
}

export function quote(overrides: Partial<DealQuote> = {}): DealQuote {
	return {
		id: 'q1',
		dealId: 'd1',
		number: 'DEV-001',
		title: 'Devis fenêtres',
		status: 'draft',
		subtotal: 150000,
		createdAt: '2026-10-02',
		...overrides,
	}
}

export function customer(overrides: Partial<Customer> = {}): Customer {
	return {
		id: 'c1',
		organization_id: 'org',
		name: 'Famille Dupont',
		email: null,
		phone: null,
		status: 'PROSPECT',
		pipeline_stage: 'NEW',
		kind: 'PROFESSIONAL',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

export function ctxAt(date: string, ids: readonly string[] = ['t-1']) {
	let index = 0
	return {
		today: date,
		newId: () => ids[index++] ?? `t-${index}`,
	}
}
