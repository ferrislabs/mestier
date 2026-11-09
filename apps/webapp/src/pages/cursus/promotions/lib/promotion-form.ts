import type { Schemas } from '#/api/api.client'

export interface PromotionFormValues {
	trainingId: string
	label: string
	startYear: string
	endYear: string
}

export interface PromotionFormErrors {
	trainingId?: string
	label?: string
	startYear?: string
	endYear?: string
}

export const EMPTY_PROMOTION_FORM: PromotionFormValues = {
	trainingId: '',
	label: '',
	startYear: '',
	endYear: '',
}

export function promotionToForm(
	promotion: Schemas.PromotionResponse,
): PromotionFormValues {
	return {
		trainingId: promotion.training_id,
		label: promotion.label,
		startYear: String(promotion.start_year),
		endYear: String(promotion.end_year),
	}
}

function isYear(value: string): boolean {
	return /^\d{4}$/.test(value.trim())
}

export function promotionFormErrors(
	values: PromotionFormValues,
): PromotionFormErrors {
	const errors: PromotionFormErrors = {}
	if (!values.trainingId) errors.trainingId = 'Choisissez une formation.'
	if (!values.label.trim()) errors.label = 'Le libellé est requis.'
	if (!isYear(values.startYear)) errors.startYear = 'Année invalide.'
	if (!isYear(values.endYear)) errors.endYear = 'Année invalide.'
	if (
		!errors.startYear &&
		!errors.endYear &&
		Number(values.endYear) < Number(values.startYear)
	) {
		errors.endYear = 'La fin doit suivre le début.'
	}
	return errors
}

export function toPromotionRequest(
	values: PromotionFormValues,
): Schemas.CreatePromotionRequest {
	return {
		training_id: values.trainingId,
		label: values.label.trim(),
		start_year: Number(values.startYear),
		end_year: Number(values.endYear),
	}
}
