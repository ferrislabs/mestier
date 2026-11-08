import { validateAdjustment } from '#/pages/hr/lib/rh/balances'
import type {
	ActionContext,
	AdjustmentDraft,
	LeaveAccount,
	RhState,
} from '#/pages/hr/lib/rh/types'

export function addAdjustment(
	state: RhState,
	context: ActionContext,
	draft: AdjustmentDraft,
): RhState {
	if (validateAdjustment(state, draft).length > 0) return state
	return {
		...state,
		adjustments: [
			...state.adjustments,
			{
				...draft,
				reason: draft.reason.trim(),
				id: context.newId(),
				authorId: context.actorId,
			},
		],
	}
}

export function updateAccount(
	state: RhState,
	accountId: string,
	patch: Partial<Omit<LeaveAccount, 'id'>>,
): RhState {
	return {
		...state,
		accounts: state.accounts.map((account) =>
			account.id === accountId ? { ...account, ...patch } : account,
		),
	}
}
