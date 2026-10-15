import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useMaintenanceGrids,
	useProposedInterventions,
	useRejectIntervention,
	useValidateIntervention,
} from '#/hooks/use-maintenance-validation'
import { usePlanning } from '#/hooks/use-planning'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	buildQueueRows,
	interventionActionError,
	seasonsOf,
	visitLineIndex,
} from '#/pages/maintenance/lib/validation-lines'
import { ValidationUI } from '#/pages/maintenance/ui/validation-ui'
import { memberNamesById } from '#/pages/planning/lib/member-roster'
import { computeWindow } from '#/pages/planning/lib/window'
import { todayIsoDate } from '#/pages/planning/types'

/** Mounts the queue of visits waiting for the manager. */
export function ValidationFeature() {
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
						Les interventions à valider nécessitent une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<ValidationScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
		/>
	)
}

function ValidationScreen({
	organizationId,
	organizationName,
}: {
	organizationId: string
	organizationName: string
}) {
	const queue = useProposedInterventions(organizationId)
	const visits = queue.data?.data ?? []
	const grids = useMaintenanceGrids(organizationId, seasonsOf(visits))
	const validate = useValidateIntervention()
	const reject = useRejectIntervention()

	// Fetched only for the roster: a visit carries the id of whoever proposed
	// it. The same shortcut as the reported gaps list, see
	// `AssignmentReportListFeature`.
	const planning = usePlanning(
		organizationId,
		computeWindow('day', todayIsoDate()),
	)
	const names = memberNamesById(planning.data?.data.resources ?? [])

	const [pendingId, setPendingId] = useState<string | null>(null)
	const [rejectingId, setRejectingId] = useState<string | null>(null)
	const [rejectNote, setRejectNote] = useState('')

	const rows = buildQueueRows(
		visits,
		visitLineIndex(grids.map((grid) => grid.data?.data)),
		(memberId) => names[memberId] ?? 'Membre inconnu',
	)

	function closeRejectForm() {
		setRejectingId(null)
		setRejectNote('')
	}

	async function run(id: string, action: () => Promise<unknown>) {
		setPendingId(id)
		validate.reset()
		reject.reset()
		try {
			await action()
			closeRejectForm()
		} catch {
			// Shown through `validate.error` / `reject.error`; the visit stays in
			// the queue, or leaves it on the refetch if someone else resolved it.
		} finally {
			setPendingId(null)
		}
	}

	const path = (id: string) => ({
		organization_id: organizationId,
		intervention_id: id,
	})

	return (
		<ValidationUI
			organizationName={organizationName}
			rows={rows}
			totalCount={queue.data?.pagination?.total ?? rows.length}
			isLoading={queue.isLoading}
			loadError={mutationErrorMessage(queue.error)}
			onRetry={() => void queue.refetch()}
			actionError={interventionActionError(validate.error ?? reject.error)}
			pendingId={pendingId}
			rejectingId={rejectingId}
			rejectNote={rejectNote}
			onValidate={(id) =>
				void run(id, () => validate.mutateAsync({ path: path(id) } as never))
			}
			onStartReject={(id) => {
				setRejectingId(id)
				setRejectNote('')
			}}
			onCancelReject={closeRejectForm}
			onRejectNoteChange={setRejectNote}
			onConfirmReject={() => {
				if (!rejectingId) return
				const note = rejectNote.trim()
				void run(rejectingId, () =>
					reject.mutateAsync({
						path: path(rejectingId),
						body: { note: note === '' ? null : note },
					} as never),
				)
			}}
		/>
	)
}
