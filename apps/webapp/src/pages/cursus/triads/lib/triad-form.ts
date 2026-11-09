import type { Schemas } from '#/api/api.client'

export interface TriadFormValues {
	promotionId: string
	apprenticeId: string
	pedagogicTutorMemberId: string | null
}

export interface TriadFormErrors {
	promotionId?: string
	apprenticeId?: string
}

export function triadFormErrors(values: TriadFormValues): TriadFormErrors {
	const errors: TriadFormErrors = {}
	if (!values.promotionId) errors.promotionId = 'Choisissez une promotion.'
	if (!values.apprenticeId) errors.apprenticeId = 'Choisissez un apprenti.'
	return errors
}

export function toTriadRequest(
	values: TriadFormValues,
): Schemas.CreateTriadRequest {
	return {
		apprentice_id: values.apprenticeId,
		promotion_id: values.promotionId,
		pedagogic_tutor_member_id: values.pedagogicTutorMemberId,
	}
}

export function isClosed(entity: { closed_at?: string | null }): boolean {
	return Boolean(entity.closed_at)
}
