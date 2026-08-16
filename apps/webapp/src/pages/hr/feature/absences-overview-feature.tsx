import { useState } from 'react'
import {
	useAbsences,
	useCreateAbsence,
	useDeleteAbsence,
	useUpdateAbsence,
} from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import {
	type AbsenceFormValues,
	absenceToDraft,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft,
} from '#/pages/hr/lib/absences'
import { browserTimeZone, todayIsoDate } from '#/pages/hr/types'
import type { AbsenceMemberOption } from '#/pages/hr/ui/absence-form-sheet'
import {
	type AbsenceOverviewRow,
	AbsencesOverviewUI,
} from '#/pages/hr/ui/absences-overview-ui'

interface AbsenceSheetState {
	mode: 'create' | 'edit'
	absenceId: string | null
	draft: AbsenceFormValues
}

export function AbsencesOverviewFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<AbsencesOverviewScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
		/>
	)
}

interface AbsencesOverviewScreenProps {
	organizationId: string
	organizationName: string
}

function AbsencesOverviewScreen({
	organizationId,
	organizationName,
}: AbsencesOverviewScreenProps) {
	const catalog = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const absencesQuery = useAbsences(organizationId)
	const createAbsence = useCreateAbsence()
	const updateAbsence = useUpdateAbsence()
	const deleteAbsence = useDeleteAbsence()

	const [today] = useState(() => todayIsoDate())
	const [absenceSheet, setAbsenceSheet] = useState<AbsenceSheetState | null>(
		null,
	)

	// See `browserTimeZone`'s doc for why this isn't `organizations.timezone`.
	const timeZone = browserTimeZone()

	const members = catalog.members.data?.data ?? []
	const memberOptions: AbsenceMemberOption[] = members.map((member) => ({
		memberId: member.id,
		displayName: member.display_name,
	}))
	const memberNameById = new Map(
		members.map((member) => [member.id, member.display_name]),
	)

	const rows: AbsenceOverviewRow[] = (absencesQuery.data?.data ?? [])
		.map((absence) => ({
			id: absence.id,
			memberDisplayName:
				memberNameById.get(absence.member_id) ?? 'Personne inconnue',
			...absenceToDraft({ ...absence, absence_kind: absence.kind }, timeZone),
		}))
		.sort((a, b) => a.range.from.localeCompare(b.range.from))

	const isLoading = catalog.members.isLoading || absencesQuery.isLoading
	const loadError =
		catalog.members.error?.message ?? absencesQuery.error?.message ?? null

	function handleCreateAbsence() {
		setAbsenceSheet({
			mode: 'create',
			absenceId: null,
			draft: emptyAbsenceDraft('', today),
		})
	}

	function handleEditAbsence(absenceId: string) {
		const absence = (absencesQuery.data?.data ?? []).find(
			(candidate) => candidate.id === absenceId,
		)
		if (!absence) return
		setAbsenceSheet({
			mode: 'edit',
			absenceId,
			draft: absenceToDraft(
				{ ...absence, absence_kind: absence.kind },
				timeZone,
			),
		})
	}

	function handleAbsenceSheetOpenChange(open: boolean) {
		if (!open) setAbsenceSheet(null)
	}

	function handleAbsenceDraftChange(patch: Partial<AbsenceFormValues>) {
		setAbsenceSheet((current) =>
			current ? { ...current, draft: { ...current.draft, ...patch } } : current,
		)
	}

	async function handleSubmitAbsence() {
		if (!absenceSheet) return

		if (absenceSheet.mode === 'create') {
			const request = draftToCreateAbsenceRequest(absenceSheet.draft, timeZone)
			if (!request) return
			try {
				await createAbsence.mutateAsync({
					path: { organization_id: organizationId },
					body: request,
				})
				setAbsenceSheet(null)
			} catch {
				// Surfaced reactively via `createAbsence.error` — the draft is kept.
			}
			return
		}

		if (!absenceSheet.absenceId) return
		const request = draftToUpdateAbsenceRequest(absenceSheet.draft, timeZone)
		if (!request) return
		try {
			await updateAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: absenceSheet.absenceId,
				},
				body: request,
			})
			setAbsenceSheet(null)
		} catch {
			// Surfaced reactively via `updateAbsence.error`.
		}
	}

	async function handleDeleteSheetAbsence() {
		if (!absenceSheet?.absenceId) return
		await handleDeleteAbsence(absenceSheet.absenceId)
		setAbsenceSheet(null)
	}

	/** Row-level delete, straight from the table — doesn't need the form open first (see `RowActions` in `AbsencesOverviewUI`). */
	async function handleDeleteAbsence(absenceId: string) {
		try {
			await deleteAbsence.mutateAsync({
				path: { organization_id: organizationId, absence_id: absenceId },
			})
		} catch {
			// Surfaced reactively via `deleteAbsence.error`.
		}
	}

	const absenceDraft = absenceSheet?.draft ?? emptyAbsenceDraft('', today)
	const absenceErrors = absenceSheet
		? validateAbsenceDraft(absenceDraft, {
				requireMember: absenceSheet.mode === 'create',
			})
		: []
	const absenceSaveError =
		createAbsence.error?.message ??
		updateAbsence.error?.message ??
		deleteAbsence.error?.message ??
		null

	return (
		<AbsencesOverviewUI
			organizationName={organizationName}
			isLoading={isLoading}
			error={loadError}
			absences={rows}
			onCreate={handleCreateAbsence}
			onEdit={handleEditAbsence}
			onDelete={handleDeleteAbsence}
			absenceSheet={{
				open: absenceSheet !== null,
				mode: absenceSheet?.mode ?? 'create',
				values: absenceDraft,
				members: memberOptions,
				errors: absenceErrors,
				isSaving: createAbsence.isPending || updateAbsence.isPending,
				isDeleting: deleteAbsence.isPending,
				saveError: absenceSaveError,
				onChange: handleAbsenceDraftChange,
				onSubmit: () => void handleSubmitAbsence(),
				onDelete:
					absenceSheet?.mode === 'edit'
						? () => void handleDeleteSheetAbsence()
						: undefined,
				onOpenChange: handleAbsenceSheetOpenChange,
			}}
		/>
	)
}
