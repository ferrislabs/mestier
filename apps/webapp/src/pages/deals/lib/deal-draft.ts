import { parseAmount } from '#/pages/customers/lib/format'
import type { Cents, DealDraft, LeadSource } from '#/pages/deals/lib/deal-types'
import { validateDealDraft } from '#/pages/deals/lib/pipeline'

export type DealForm = {
	title: string
	amount: string
	expectedCloseDate: string
}

export type DealFormErrors = { title?: string; amount?: string }

export type DealCustomerPrefill = {
	id: string
	ownerId: string | null
	source: LeadSource | null
}

export const EMPTY_DEAL_FORM: DealForm = {
	title: '',
	amount: '',
	expectedCloseDate: '',
}

export function amountOf(input: string): Cents | null {
	return input.trim() === '' ? 0 : parseAmount(input)
}

export function dealDraftOf(
	customer: DealCustomerPrefill,
	form: DealForm,
): DealDraft {
	return {
		customerId: customer.id,
		title: form.title.trim(),
		amount: amountOf(form.amount) ?? Number.NaN,
		expectedCloseDate:
			form.expectedCloseDate === '' ? null : form.expectedCloseDate,
		ownerId: customer.ownerId,
		source: customer.source,
		notes: '',
	}
}

export function validateDealForm(
	customer: DealCustomerPrefill,
	form: DealForm,
): DealFormErrors {
	const errors: DealFormErrors = {}
	if (amountOf(form.amount) === null)
		errors.amount = 'Saisis un montant valide, par exemple 1 250,50.'
	for (const message of validateDealDraft(dealDraftOf(customer, form))) {
		if (message.startsWith('L’intitulé')) errors.title ??= message
		else if (message.startsWith('Le montant')) errors.amount ??= message
	}
	return errors
}
