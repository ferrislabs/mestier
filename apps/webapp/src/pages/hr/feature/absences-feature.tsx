import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import {
	useAbsences,
	useDeleteAbsence,
	useUpdateAbsence,
} from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	isRealAbsenceId,
	realAbsenceId,
} from '#/pages/hr/feature/mocks/rh-binding'
import {
	type AbsenceFormValues,
	absenceToDraft,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft,
} from '#/pages/hr/lib/absences'
import {
	cancelAbsence,
	decideAbsence,
	findMember,
	pendingAbsences,
} from '#/pages/hr/lib/rh/absences'
import {
	type HistoryFilters,
	sortPending,
} from '#/pages/hr/lib/rh/absences-view'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Absence, RhState } from '#/pages/hr/lib/rh/types'
import { browserTimeZone } from '#/pages/hr/types'
import type { AbsenceFormInitial } from '#/pages/hr/ui/rh/absences/absence-form-dialog'
import {
	AbsencesPage,
	type AbsencesTab,
} from '#/pages/hr/ui/rh/absences/absences-page'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'
import { RhScreen } from './rh-screen'
import { useSubmitAbsence } from './use-absence-submit'

export type { AbsencesTab } from '#/pages/hr/ui/rh/absences/absences-page'

export interface AbsencesFeatureProps {
	historyStatus: HistoryFilters['status']
	historyMemberId: string
	onHistoryFiltersChange: (patch: {
		status: HistoryFilters['status']
		memberId: string
	}) => void
	tab: AbsencesTab
	onTabChange: (tab: AbsencesTab) => void
	creating: boolean
	initial?: AbsenceFormInitial
	onCreatingChange: (open: boolean, initial?: AbsenceFormInitial) => void
}

export function AbsencesFeature(props: AbsencesFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement des absences…">
			<AbsencesScreen {...props} />
		</RhScreen>
	)
}

interface RejectState {
	absence: Absence | null
	open: boolean
}

interface EditState {
	absenceId: string
	draft: AbsenceFormValues
}

function AbsencesScreen({
	historyStatus,
	historyMemberId,
	onHistoryFiltersChange,
	tab,
	onTabChange,
	creating,
	initial,
	onCreatingChange,
}: AbsencesFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const state = useRhState()
	const updateRh = useRhUpdate()
	const absenceSubmit = useSubmitAbsence(organizationId)
	const realAbsences = useAbsences(organizationId)
	const updateAbsence = useUpdateAbsence()
	const deleteAbsence = useDeleteAbsence()
	const [rejecting, setRejecting] = useState<RejectState>({
		absence: null,
		open: false,
	})
	const [editing, setEditing] = useState<EditState | null>(null)
	const [deleting, setDeleting] = useState<Absence | null>(null)
	const [notice, setNotice] = useState('')
	const timeZone = browserTimeZone()
	const pending = sortPending(pendingAbsences(state))

	function announce(absence: Absence, message: string) {
		const member = findMember(state, absence.memberId)
		setNotice(`Demande${member ? ` de ${fullName(member)}` : ''} ${message}.`)
	}

	function approve(absence: Absence) {
		void updateRh((current, context) =>
			decideAbsence(current, context, absence.id, 'approved', ''),
		)
		announce(absence, 'validée')
	}

	function reject(absence: Absence, comment: string) {
		void updateRh((current, context) =>
			decideAbsence(current, context, absence.id, 'rejected', comment),
		)
		setRejecting((current) => ({ ...current, open: false }))
		announce(absence, 'refusée')
	}

	function cancel(absence: Absence) {
		void updateRh((current, context) =>
			cancelAbsence(current, context, absence.id),
		)
		announce(absence, 'annulée')
	}

	function edit(absence: Absence) {
		if (!isRealAbsenceId(absence.id)) return
		const id = realAbsenceId(absence.id)
		const source = (realAbsences.data?.data ?? []).find(
			(candidate) => candidate.id === id,
		)
		if (!source) return
		setEditing({
			absenceId: id,
			draft: absenceToDraft({ ...source, absence_kind: source.kind }, timeZone),
		})
	}

	async function submitEdit() {
		if (!editing) return
		const request = draftToUpdateAbsenceRequest(editing.draft, timeZone)
		if (!request) return
		try {
			await updateAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: editing.absenceId,
				},
				body: request,
			})
			setEditing(null)
		} catch {
			return
		}
	}

	async function confirmDelete() {
		if (!deleting) return
		try {
			await deleteAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: realAbsenceId(deleting.id),
				},
			})
		} catch {
			return
		} finally {
			setDeleting(null)
		}
	}

	const sheetDraft = editing?.draft ?? emptyAbsenceDraft('', todayIso())
	const saveError =
		absenceSubmit.error ??
		updateAbsence.error?.message ??
		deleteAbsence.error?.message ??
		null

	return (
		<>
			<AbsencesPage
				state={state}
				tab={tab}
				onTabChange={onTabChange}
				pending={pending}
				historyStatus={historyStatus}
				historyMemberId={historyMemberId}
				onHistoryFiltersChange={onHistoryFiltersChange}
				onApprove={approve}
				onAskReject={(absence) => setRejecting({ absence, open: true })}
				onCancel={cancel}
				onEdit={edit}
				onDelete={setDeleting}
				rejecting={rejecting}
				onRejectOpenChange={(open) =>
					setRejecting((current) => ({ ...current, open }))
				}
				onRejectConfirm={reject}
				creating={creating}
				initial={initial}
				onCreatingChange={onCreatingChange}
				onSubmitAbsence={(draft) => {
					void absenceSubmit.submit(draft)
					onCreatingChange(false)
				}}
				absenceSheet={{
					open: editing !== null,
					mode: 'edit',
					values: sheetDraft,
					members: state.members.map((member) => ({
						memberId: member.id,
						displayName: fullName(member),
					})),
					errors: validateAbsenceDraft(sheetDraft, { requireMember: false }),
					isSaving: updateAbsence.isPending,
					isDeleting: deleteAbsence.isPending,
					saveError,
					onChange: (patch) =>
						setEditing((current) =>
							current
								? { ...current, draft: { ...current.draft, ...patch } }
								: current,
						),
					onSubmit: () => void submitEdit(),
					onOpenChange: (open) => {
						if (!open) setEditing(null)
					},
				}}
				saveError={saveError}
				notice={notice}
			/>
			<ConfirmDialog
				open={deleting !== null}
				onOpenChange={(open) => {
					if (!open) setDeleting(null)
				}}
				title={`Supprimer l’absence${deleting ? ` de ${deletingName(state, deleting)}` : ''} ?`}
				description="Cette absence sera définitivement supprimée. Cette action est irréversible."
				confirmLabel="Supprimer"
				destructive
				disabled={deleteAbsence.isPending}
				onConfirm={() => void confirmDelete()}
			/>
		</>
	)
}

function deletingName(state: RhState, absence: Absence): string {
	const member = findMember(state, absence.memberId)
	return member ? fullName(member) : 'cette personne'
}
