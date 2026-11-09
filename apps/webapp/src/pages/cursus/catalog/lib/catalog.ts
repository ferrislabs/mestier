import type { Schemas } from '#/api/api.client'
import { emptyToNull } from '#/pages/cursus/participants/lib/forms'

export interface EstablishmentValues {
	name: string
	uai: string
	siret: string
	address: string
	isMainSite: boolean
}

export const EMPTY_ESTABLISHMENT: EstablishmentValues = {
	name: '',
	uai: '',
	siret: '',
	address: '',
	isMainSite: false,
}

export function toEstablishmentValues(
	establishment: Schemas.EstablishmentResponse,
): EstablishmentValues {
	return {
		name: establishment.name,
		uai: establishment.uai ?? '',
		siret: establishment.siret ?? '',
		address: establishment.address ?? '',
		isMainSite: establishment.is_main_site,
	}
}

export function validateEstablishment(
	values: EstablishmentValues,
): string | null {
	return values.name.trim() === ''
		? "Le nom de l'établissement est obligatoire."
		: null
}

export function toEstablishmentRequest(
	values: EstablishmentValues,
): Schemas.CreateEstablishmentRequest {
	return {
		name: values.name.trim(),
		uai: emptyToNull(values.uai),
		siret: emptyToNull(values.siret),
		address: emptyToNull(values.address),
		is_main_site: values.isMainSite,
	}
}

export interface TrainingValues {
	establishmentId: string | null
	name: string
	rncpCode: string
	rncpLabel: string
	level: string
	degreeType: string
}

export const EMPTY_TRAINING: TrainingValues = {
	establishmentId: null,
	name: '',
	rncpCode: '',
	rncpLabel: '',
	level: '',
	degreeType: '',
}

export function toTrainingValues(
	training: Schemas.TrainingResponse,
): TrainingValues {
	return {
		establishmentId: training.establishment_id ?? null,
		name: training.name,
		rncpCode: training.rncp_code ?? '',
		rncpLabel: training.rncp_label ?? '',
		level: training.level ?? '',
		degreeType: training.degree_type ?? '',
	}
}

export function validateTraining(values: TrainingValues): string | null {
	if (values.name.trim() === '')
		return 'Le nom de la formation est obligatoire.'
	return null
}

export function toTrainingRequest(
	values: TrainingValues,
): Schemas.CreateTrainingRequest {
	return {
		...(values.establishmentId === null
			? {}
			: { establishment_id: values.establishmentId }),
		name: values.name.trim(),
		rncp_code: emptyToNull(values.rncpCode),
		rncp_label: emptyToNull(values.rncpLabel),
		level: emptyToNull(values.level),
		degree_type: emptyToNull(values.degreeType),
	}
}

export interface ClosureReasonValues {
	label: string
}

export const EMPTY_CLOSURE_REASON: ClosureReasonValues = { label: '' }

export function validateClosureReason(
	values: ClosureReasonValues,
): string | null {
	return values.label.trim() === '' ? 'Le libellé est obligatoire.' : null
}

export function toClosureReasonRequest(
	values: ClosureReasonValues,
): Schemas.CreateClosureReasonRequest {
	return { label: values.label.trim() }
}

export type EditorState = { id: string | null } | null
