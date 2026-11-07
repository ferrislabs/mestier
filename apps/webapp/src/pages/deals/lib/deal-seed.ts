import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import type { Conversions, Deal, DealStage } from '#/pages/deals/lib/deal-types'

const STAGE_OF_CUSTOMER: Record<CustomerPipelineStage, DealStage> = {
	NEW: 'new',
	CONTACTED: 'visit',
	QUALIFIED: 'visit',
	QUOTE_SENT: 'quote_sent',
	WON: 'won',
	LOST: 'lost',
}

function dayOf(timestamp: string): string {
	return timestamp.slice(0, 10)
}

export function seedDeals(customers: readonly Customer[]): Deal[] {
	return customers
		.filter((customer) => customer.status !== 'ARCHIVED')
		.map((customer) => {
			const stage = STAGE_OF_CUSTOMER[customer.pipeline_stage]
			const changed = dayOf(customer.updated_at)
			return {
				id: `deal-${customer.id}`,
				customerId: customer.id,
				title: `Opportunité ${customer.name}`,
				amount: 0,
				stage,
				expectedCloseDate: null,
				ownerId: null,
				source: null,
				lostReason: '',
				notes: '',
				createdAt: dayOf(customer.created_at),
				stageChangedAt: changed,
				closedAt: stage === 'won' || stage === 'lost' ? changed : null,
			}
		})
}

export function seedConversions(customers: readonly Customer[]): Conversions {
	return Object.fromEntries(
		customers
			.filter(
				(customer) =>
					customer.status === 'CLIENT' && customer.pipeline_stage === 'WON',
			)
			.map((customer) => [customer.id, dayOf(customer.updated_at)]),
	)
}
