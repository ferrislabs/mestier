import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	useCreateAbsence,
	useDeleteAbsence,
	useUpdateAbsence,
} from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCheckAvailability,
	useMoveWorkOrder,
	usePlanning,
} from '#/hooks/use-planning'
import { employeeDisplayName } from '#/pages/hr/types'
import {
	type AbsenceFormValues,
	absenceToDraft,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft,
} from '#/pages/planning/lib/absences'
import {
	buildWarnings,
	conflictsForResource,
	type Warning,
} from '#/pages/planning/lib/warnings'
import { computeWindow } from '#/pages/planning/lib/window'
import {
	computeRemoveAssigneePatch,
	computeWorkOrderDropPatch,
} from '#/pages/planning/lib/work-order-drop'
import { type PlanningView, todayIsoDate } from '#/pages/planning/types'
import type {
	RemoveAssigneeEvent,
	WorkOrderDropEvent,
} from '#/pages/planning/ui/planning-grid'
import { PlanningTeamUI } from '#/pages/planning/ui/planning-team-ui'

export interface PlanningTeamFeatureProps {
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

/**
 * Mounts the team planning screen. Router-agnostic on purpose: `view`/`date`
 * and their change callbacks come in as props from the route (see
 * `_app.planning.team.tsx`), which is the only place that reads/writes the
 * URL — this component itself is a plain function of its props, so it tests
 * the same way `EmployeeWorkTimeFeature` does, without a router.
 */
export function PlanningTeamFeature({
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningTeamFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le planning nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<PlanningTeamScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			view={view}
			date={date}
			onViewChange={onViewChange}
			onDateChange={onDateChange}
		/>
	)
}

interface PlanningTeamScreenProps {
	organizationId: string
	organizationName: string
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

interface PendingDrop {
	workOrderId: string
	body: Schemas.UpdateWorkOrderRequest
	warnings: Warning[]
}

interface AbsenceSheetState {
	mode: 'create' | 'edit'
	absenceId: string | null
	draft: AbsenceFormValues
}

function PlanningTeamScreen({
	organizationId,
	organizationName,
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningTeamScreenProps) {
	// The `from`/`to` window is derived from `view`/`date`, never stored on its
	// own — see the planning design doc's "état de vue dans l'URL" section.
	const range = computeWindow(view, date)
	const planningQuery = usePlanning(organizationId, range)
	const data = planningQuery.data?.data ?? null

	const checkAvailability = useCheckAvailability()
	const moveWorkOrder = useMoveWorkOrder()
	const createAbsence = useCreateAbsence()
	const updateAbsence = useUpdateAbsence()
	const deleteAbsence = useDeleteAbsence()

	const [pendingDrop, setPendingDrop] = useState<PendingDrop | null>(null)
	const [dropError, setDropError] = useState<string | null>(null)
	const [absenceSheet, setAbsenceSheet] = useState<AbsenceSheetState | null>(
		null,
	)
	const [createdEmployeeNames, setCreatedEmployeeNames] = useState<string[]>([])

	async function applyWorkOrderPatch(
		workOrderId: string,
		body: Schemas.UpdateWorkOrderRequest,
	) {
		try {
			// `mutation()` resolves one envelope short of `InferResponseData`'s
			// declared type — see `use-work-time.ts`'s `.data?.data` precedent for
			// `useQuery`; the same extra `.data` applies here, undetected by
			// `tsc` since the generated type lies about it too.
			const response = await moveWorkOrder.mutateAsync({
				path: { organization_id: organizationId, work_order_id: workOrderId },
				body,
			})
			const result = response.data
			if (result.created_employees.length > 0) {
				setCreatedEmployeeNames(
					result.created_employees.map((employee) =>
						employeeDisplayName(employee),
					),
				)
			}
			setPendingDrop(null)
			setDropError(null)
		} catch (error) {
			// Kept open on failure — see `PlanningWarningDialog`'s `error` slot —
			// so the gesture can be retried instead of silently vanishing.
			setDropError(errorMessage(error))
		}
	}

	async function handleDropWorkOrder(event: WorkOrderDropEvent) {
		if (!data) return
		const entry = data.entries.find(
			(candidate) =>
				candidate.kind === 'work_order' && candidate.id === event.entryId,
		)
		if (!entry || entry.kind !== 'work_order') return

		const { changed, body } = computeWorkOrderDropPatch({
			source: {
				entryId: event.entryId,
				resourceId: event.sourceResourceId,
				date: event.sourceDate,
			},
			target: { resourceId: event.targetResourceId, date: event.targetDate },
			entry: {
				startsAt: entry.starts_at,
				endsAt: entry.ends_at,
				employeeIds: entry.employee_ids,
			},
			timeZone: data.timezone,
		})
		// A drop that lands back where it started writes nothing — see the
		// planning design doc's "Drag & drop" decision.
		if (!changed) return

		const targetResource = data.resources.find(
			(resource) => resource.resource_id === event.targetResourceId,
		)
		const windowStartsAt = body.starts_at ?? entry.starts_at
		const windowEndsAt = body.ends_at ?? entry.ends_at

		try {
			const availabilityResponse = await checkAvailability.mutateAsync({
				path: { organization_id: organizationId },
				query: {
					starts_at: windowStartsAt,
					ends_at: windowEndsAt,
					all_day: entry.all_day,
				},
			})
			const conflicts = conflictsForResource(
				availabilityResponse.data,
				event.targetResourceId,
			)
			const warnings = buildWarnings({
				conflicts,
				resourceKind: targetResource?.kind ?? 'employee',
			})

			if (warnings.length === 0) {
				await applyWorkOrderPatch(event.entryId, body)
				return
			}

			setDropError(null)
			setPendingDrop({ workOrderId: event.entryId, body, warnings })
		} catch (error) {
			console.error(
				'[planning] échec de la vérification de disponibilité',
				error,
			)
		}
	}

	function handleRemoveAssignee(event: RemoveAssigneeEvent) {
		if (!data) return
		const entry = data.entries.find(
			(candidate) =>
				candidate.kind === 'work_order' && candidate.id === event.entryId,
		)
		if (!entry || entry.kind !== 'work_order') return

		// Removing never introduces risk, so it skips the warnings dialog and
		// applies straight away — but through the exact same complete-list
		// `PATCH` path a move uses (see the planning design doc).
		const { changed, body } = computeRemoveAssigneePatch({
			employeeIds: entry.employee_ids,
			resourceId: event.resourceId,
		})
		if (!changed) return
		void applyWorkOrderPatch(event.entryId, body)
	}

	function handleConfirmDrop() {
		if (!pendingDrop) return
		void applyWorkOrderPatch(pendingDrop.workOrderId, pendingDrop.body)
	}

	function handleCancelDrop() {
		setPendingDrop(null)
		setDropError(null)
	}

	function handleCreateAbsence() {
		setAbsenceSheet({
			mode: 'create',
			absenceId: null,
			draft: emptyAbsenceDraft('', todayIsoDate()),
		})
	}

	function handleSelectAbsence(entryId: string) {
		if (!data) return
		const entry = data.entries.find(
			(candidate) => candidate.kind === 'absence' && candidate.id === entryId,
		)
		if (!entry || entry.kind !== 'absence') return
		setAbsenceSheet({
			mode: 'edit',
			absenceId: entryId,
			draft: absenceToDraft(entry, data.timezone),
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
		if (!absenceSheet || !data) return
		const timeZone = data.timezone

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

	async function handleDeleteAbsence() {
		if (!absenceSheet?.absenceId) return
		try {
			await deleteAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: absenceSheet.absenceId,
				},
			})
			setAbsenceSheet(null)
		} catch {
			// Surfaced reactively via `deleteAbsence.error`.
		}
	}

	const employeeOptions = (data?.resources ?? [])
		.filter((resource) => resource.employee_id)
		.map((resource) => ({
			employeeId: resource.employee_id as string,
			displayName: resource.display_name,
		}))

	const absenceDraft = absenceSheet?.draft ?? emptyAbsenceDraft('', date)
	const absenceErrors = absenceSheet
		? validateAbsenceDraft(absenceDraft, {
				requireEmployee: absenceSheet.mode === 'create',
			})
		: []
	const absenceSaveError =
		createAbsence.error?.message ??
		updateAbsence.error?.message ??
		deleteAbsence.error?.message ??
		null

	return (
		<PlanningTeamUI
			organizationName={organizationName}
			view={view}
			date={date}
			windowFrom={range.from}
			windowTo={range.to}
			onViewChange={onViewChange}
			onDateChange={onDateChange}
			isLoading={planningQuery.isLoading}
			error={planningQuery.error?.message ?? null}
			data={data}
			onDropWorkOrder={(event) => void handleDropWorkOrder(event)}
			onRemoveAssignee={handleRemoveAssignee}
			onSelectAbsence={handleSelectAbsence}
			onCreateAbsence={handleCreateAbsence}
			warningDialog={{
				open: pendingDrop !== null,
				warnings: pendingDrop?.warnings ?? [],
				isPending: moveWorkOrder.isPending,
				error: dropError,
				onConfirm: handleConfirmDrop,
				onCancel: handleCancelDrop,
			}}
			absenceSheet={{
				open: absenceSheet !== null,
				mode: absenceSheet?.mode ?? 'create',
				values: absenceDraft,
				employees: employeeOptions,
				errors: absenceErrors,
				isSaving: createAbsence.isPending || updateAbsence.isPending,
				isDeleting: deleteAbsence.isPending,
				saveError: absenceSaveError,
				onChange: handleAbsenceDraftChange,
				onSubmit: () => void handleSubmitAbsence(),
				onDelete:
					absenceSheet?.mode === 'edit'
						? () => void handleDeleteAbsence()
						: undefined,
				onOpenChange: handleAbsenceSheetOpenChange,
			}}
			createdEmployeeNames={createdEmployeeNames}
			onDismissCreatedEmployees={() => setCreatedEmployeeNames([])}
		/>
	)
}

function errorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	return 'La confirmation a échoué. Réessayez.'
}
