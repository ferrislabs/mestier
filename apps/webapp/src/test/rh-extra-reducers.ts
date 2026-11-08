import {
	findSurcharge,
	isActiveExtra,
	validateExtra,
	validateSurcharge,
} from '#/pages/hr/lib/rh/extra'
import type {
	ActionContext,
	ExtraDraft,
	ExtraPeriod,
	RhState,
	SurchargeDraft,
} from '#/pages/hr/lib/rh/types'

export function createExtra(
	state: RhState,
	context: ActionContext,
	draft: ExtraDraft,
	approve = false,
): RhState {
	if (validateExtra(state, draft).length > 0) return state
	const extra: ExtraPeriod = {
		...draft,
		reason: draft.reason.trim(),
		id: context.newId(),
		status: approve ? 'approved' : 'pending',
		createdAt: context.now,
		createdById: context.actorId,
		decidedById: approve ? context.actorId : null,
		decidedAt: approve ? context.now : null,
		decisionComment: '',
	}
	return { ...state, extras: [extra, ...state.extras] }
}

export function decideExtra(
	state: RhState,
	context: ActionContext,
	extraId: string,
	decision: 'approved' | 'rejected',
	comment: string,
): RhState {
	return {
		...state,
		extras: state.extras.map((extra) =>
			extra.id === extraId && extra.status === 'pending'
				? {
						...extra,
						status: decision,
						decidedById: context.actorId,
						decidedAt: context.now,
						decisionComment: comment.trim(),
					}
				: extra,
		),
	}
}

export function cancelExtra(
	state: RhState,
	context: ActionContext,
	extraId: string,
): RhState {
	return {
		...state,
		extras: state.extras.map((extra) =>
			extra.id === extraId && isActiveExtra(extra)
				? {
						...extra,
						status: 'cancelled',
						decidedById: context.actorId,
						decidedAt: context.now,
					}
				: extra,
		),
	}
}

export function createSurcharge(
	state: RhState,
	context: ActionContext,
	draft: SurchargeDraft,
): RhState {
	if (validateSurcharge(state, draft).length > 0) return state
	return {
		...state,
		surcharges: [
			...state.surcharges,
			{
				...draft,
				name: draft.name.trim(),
				id: context.newId(),
				archived: false,
			},
		],
	}
}

export function updateSurcharge(
	state: RhState,
	surchargeId: string,
	patch: Partial<SurchargeDraft>,
): RhState {
	const current = findSurcharge(state, surchargeId)
	if (!current) return state
	const next = { ...current, ...patch }
	if (validateSurcharge(state, next, surchargeId).length > 0) return state
	return {
		...state,
		surcharges: state.surcharges.map((surcharge) =>
			surcharge.id === surchargeId
				? { ...next, name: next.name.trim() }
				: surcharge,
		),
	}
}

export function setSurchargeArchived(
	state: RhState,
	surchargeId: string,
	archived: boolean,
): RhState {
	return {
		...state,
		surcharges: state.surcharges.map((surcharge) =>
			surcharge.id === surchargeId ? { ...surcharge, archived } : surcharge,
		),
	}
}
