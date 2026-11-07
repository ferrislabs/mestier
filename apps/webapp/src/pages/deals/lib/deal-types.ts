import type { LeadSource } from '#/hooks/use-customers'

export type { LeadSource }

export type IsoDate = string

export type Cents = number

export type DealStage =
	| 'new'
	| 'visit'
	| 'quote_sent'
	| 'negotiation'
	| 'won'
	| 'lost'

export type Deal = {
	id: string
	customerId: string
	title: string
	amount: Cents
	stage: DealStage
	expectedCloseDate: IsoDate | null
	ownerId: string | null
	source: LeadSource | null
	lostReason: string
	notes: string
	quoteCount: number
	createdAt: IsoDate
	stageChangedAt: IsoDate
	closedAt: IsoDate | null
}

export type DealDraft = Pick<
	Deal,
	| 'customerId'
	| 'title'
	| 'amount'
	| 'expectedCloseDate'
	| 'ownerId'
	| 'source'
	| 'notes'
>

export type DealQuoteStatus =
	| 'draft'
	| 'sent'
	| 'accepted'
	| 'declined'
	| 'cancelled'
	| 'superseded'

export type DealQuote = {
	id: string
	dealId: string
	number: string | null
	title: string
	status: DealQuoteStatus
	subtotal: Cents
	createdAt: IsoDate
}
