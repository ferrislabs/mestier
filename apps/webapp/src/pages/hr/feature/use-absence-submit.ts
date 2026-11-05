import { useCreateAbsence } from '#/hooks/use-absences'
import { kindOfType } from '#/pages/hr/feature/mocks/rh-binding'
import {
	type AbsenceFormValues,
	draftToCreateAbsenceRequest,
} from '#/pages/hr/lib/absences'
import { createAbsence, findType } from '#/pages/hr/lib/rh/absences'
import type { AbsenceDraft } from '#/pages/hr/lib/rh/types'
import { browserTimeZone } from '#/pages/hr/types'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

function toRealValues(draft: AbsenceDraft): AbsenceFormValues {
	const allDay = draft.startPart === 'morning' && draft.endPart === 'afternoon'
	return {
		memberId: draft.memberId,
		kind: kindOfType(draft.typeId),
		allDay,
		range: { from: draft.startDate, to: draft.endDate },
		startTime: draft.startPart === 'morning' ? '08:00' : '13:00',
		endTime: draft.endPart === 'morning' ? '12:00' : '18:00',
		note: draft.comment,
	}
}

export function useSubmitAbsence(organizationId: string) {
	const state = useRhState()
	const updateRh = useRhUpdate()
	const createRealAbsence = useCreateAbsence()

	async function submit(draft: AbsenceDraft) {
		const type = findType(state, draft.typeId)
		if (type && !type.needsApproval) {
			const request = draftToCreateAbsenceRequest(
				toRealValues(draft),
				browserTimeZone(),
			)
			if (!request) return
			try {
				await createRealAbsence.mutateAsync({
					path: { organization_id: organizationId },
					body: request,
				})
			} catch {
				return
			}
			return
		}
		await updateRh((current, context) => createAbsence(current, context, draft))
	}

	return { submit, error: createRealAbsence.error?.message ?? null }
}
