import {
	type BillingFrequency,
	isBillingFrequency,
} from '#/pages/quotes/lib/billing-frequency'

export interface QuoteLineExtras {
	discountPercent: number
	frequency: BillingFrequency
}

export interface QuoteLink {
	id: string
	label: string
}

export interface QuoteExtraEvent {
	id: string
	kind: 'revised' | 'reminder'
	detail: string
	at: string
}

export interface QuoteExtras {
	ownerId: string | null
	validUntil: string | null
	depositPercent: number
	notes: string
	lines: QuoteLineExtras[]
	signer: string
	declineReason: string
	respondedAt: string | null
	revisionOf: QuoteLink | null
	supersededBy: QuoteLink | null
	dealId: string | null
	events: QuoteExtraEvent[]
}

export const EMPTY_QUOTE_EXTRAS: QuoteExtras = {
	ownerId: null,
	validUntil: null,
	depositPercent: 0,
	notes: '',
	lines: [],
	signer: '',
	declineReason: '',
	respondedAt: null,
	revisionOf: null,
	supersededBy: null,
	dealId: null,
	events: [],
}

const DEFAULT_LINE_EXTRAS: QuoteLineExtras = {
	discountPercent: 0,
	frequency: 'one_shot',
}

export function mergeQuoteExtras(
	base: QuoteExtras | undefined,
	patch: Partial<QuoteExtras>,
): QuoteExtras {
	return { ...(base ?? EMPTY_QUOTE_EXTRAS), ...patch }
}

export function acceptedPatch(
	signer: string,
	today: string,
): Partial<QuoteExtras> {
	return { signer: signer.trim(), respondedAt: today, declineReason: '' }
}

export function declinedPatch(
	reason: string,
	today: string,
): Partial<QuoteExtras> {
	return { declineReason: reason.trim(), respondedAt: today, signer: '' }
}

export function revisionPatches(input: {
	previous: QuoteLink
	previousExtras: QuoteExtras
	next: QuoteLink
	at: string
}): { previous: Partial<QuoteExtras>; next: Partial<QuoteExtras> } {
	return {
		previous: {
			supersededBy: input.next,
			events: [
				...input.previousExtras.events,
				{
					id: `revised-${input.next.id}`,
					kind: 'revised',
					detail: `Remplacé par ${input.next.label}`,
					at: input.at,
				},
			],
		},
		next: { revisionOf: input.previous },
	}
}

export function lineExtrasAt(
	extras: Pick<QuoteExtras, 'lines'>,
	index: number,
): QuoteLineExtras {
	return extras.lines[index] ?? DEFAULT_LINE_EXTRAS
}

function asRecord(value: unknown): Record<string, unknown> | null {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: null
}

function asString(value: unknown): string {
	return typeof value === 'string' ? value : ''
}

function asNullableString(value: unknown): string | null {
	return typeof value === 'string' && value !== '' ? value : null
}

function asNumber(value: unknown): number {
	return typeof value === 'number' && Number.isFinite(value) ? value : 0
}

function asLink(value: unknown): QuoteLink | null {
	const record = asRecord(value)
	if (!record || typeof record.id !== 'string') return null
	return { id: record.id, label: asString(record.label) }
}

function asLine(value: unknown): QuoteLineExtras {
	const record = asRecord(value)
	return {
		discountPercent: asNumber(record?.discountPercent),
		frequency: isBillingFrequency(record?.frequency)
			? record.frequency
			: 'one_shot',
	}
}

function asEvent(value: unknown): QuoteExtraEvent | null {
	const record = asRecord(value)
	if (!record) return null
	const kind = record.kind
	if (kind !== 'revised' && kind !== 'reminder') return null
	return {
		id: asString(record.id),
		kind,
		detail: asString(record.detail),
		at: asString(record.at),
	}
}

export function parseQuoteExtras(raw: string | null): QuoteExtras {
	if (raw === null) return EMPTY_QUOTE_EXTRAS
	let parsed: unknown
	try {
		parsed = JSON.parse(raw)
	} catch {
		return EMPTY_QUOTE_EXTRAS
	}
	const record = asRecord(parsed)
	if (!record) return EMPTY_QUOTE_EXTRAS
	return {
		ownerId: asNullableString(record.ownerId),
		validUntil: asNullableString(record.validUntil),
		depositPercent: asNumber(record.depositPercent),
		notes: asString(record.notes),
		lines: Array.isArray(record.lines) ? record.lines.map(asLine) : [],
		signer: asString(record.signer),
		declineReason: asString(record.declineReason),
		respondedAt: asNullableString(record.respondedAt),
		revisionOf: asLink(record.revisionOf),
		supersededBy: asLink(record.supersededBy),
		dealId: asNullableString(record.dealId),
		events: Array.isArray(record.events)
			? record.events.flatMap((event) => asEvent(event) ?? [])
			: [],
	}
}
