import { PEOPLE } from '#/pages/customers/feature/mocks/people'
import { centsToInput, parseAmount } from '#/pages/customers/lib/format'
import { LEAD_SOURCES } from '#/pages/customers/types'
import type { Deal, DealDraft } from '#/pages/deals/lib/deal-types'
import { validateDealDraft } from '#/pages/deals/lib/pipeline'

export type DealForm = {
	customerId: string
	title: string
	amount: string
	expectedCloseDate: string
	ownerId: string
	source: string
	notes: string
}

export type DealFormField =
	| 'customerId'
	| 'title'
	| 'amount'
	| 'expectedCloseDate'

export type DealFormErrors = Partial<Record<DealFormField, string>>

export type DealFormResult = {
	draft: DealDraft | null
	errors: DealFormErrors
}

export const AMOUNT_ERROR = 'Saisis un montant valide, par exemple 12 500,00.'

export const DATE_ERROR = 'Saisis une date valide.'

export function isIsoDate(value: string): boolean {
	if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false
	const parsed = new Date(`${value}T00:00:00Z`)
	return (
		!Number.isNaN(parsed.getTime()) &&
		parsed.toISOString().slice(0, 10) === value
	)
}

export function emptyForm(ownerId: string): DealForm {
	return {
		customerId: '',
		title: '',
		amount: '',
		expectedCloseDate: '',
		ownerId,
		source: '',
		notes: '',
	}
}

export function formFromDeal(deal: Deal): DealForm {
	return {
		customerId: deal.customerId,
		title: deal.title,
		amount: centsToInput(deal.amount),
		expectedCloseDate: deal.expectedCloseDate ?? '',
		ownerId: deal.ownerId ?? '',
		source: deal.source ?? '',
		notes: deal.notes,
	}
}

export function isFormChanged(form: DealForm, deal: Deal): boolean {
	const original = formFromDeal(deal)
	return (
		form.customerId !== original.customerId ||
		form.title !== original.title ||
		form.amount.trim() !== original.amount ||
		form.expectedCloseDate !== original.expectedCloseDate ||
		form.ownerId !== original.ownerId ||
		form.source !== original.source ||
		form.notes !== original.notes
	)
}

function fieldOf(message: string): DealFormField {
	if (message.includes('intitulé')) return 'title'
	if (message.includes('montant')) return 'amount'
	return 'customerId'
}

export function buildDraft(form: DealForm): DealFormResult {
	const errors: DealFormErrors = {}
	const rawAmount = form.amount.trim()
	const parsed = rawAmount === '' ? 0 : parseAmount(rawAmount)
	if (parsed === null) errors.amount = AMOUNT_ERROR
	const expectedCloseDate =
		form.expectedCloseDate === '' ? null : form.expectedCloseDate
	if (expectedCloseDate !== null && !isIsoDate(expectedCloseDate))
		errors.expectedCloseDate = DATE_ERROR
	const draft: DealDraft = {
		customerId: form.customerId,
		title: form.title.trim(),
		amount: parsed ?? 0,
		expectedCloseDate,
		ownerId: PEOPLE.find((person) => person.id === form.ownerId)?.id ?? null,
		source: LEAD_SOURCES.find((source) => source === form.source) ?? null,
		notes: form.notes.trim(),
	}
	for (const message of validateDealDraft(draft)) {
		const field = fieldOf(message)
		if (errors[field] === undefined) errors[field] = message
	}
	return { draft: Object.keys(errors).length === 0 ? draft : null, errors }
}
