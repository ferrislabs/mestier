import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationEvents,
	useStartRun,
	useWorkflow,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import { RunNowDialog } from '#/pages/automation/ui/run-now-dialog'

export interface RunNowFeatureProps {
	workflowId: string
}

export function RunNowFeature({ workflowId }: RunNowFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<RunNowWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
		/>
	)
}

function subscribedEventNames(graph: Schemas.GraphDto | null | undefined) {
	if (!graph) return []

	const names = new Set<string>()
	for (const trigger of graph.triggers) {
		if (trigger.kind === 'Manual') continue
		for (const name of trigger.kind.Events) names.add(name)
	}

	return [...names]
}

function RunNowWorkspace({
	organizationId,
	organizationSlug,
	workflowId,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
}) {
	const eventsQuery = useAutomationEvents(organizationId)
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const startRun = useStartRun()
	const navigate = useNavigate()

	const [open, setOpen] = useState(false)
	const [startError, setStartError] = useState<string | null>(null)

	async function handleConfirm(payload: unknown) {
		setStartError(null)
		try {
			const result = await startRun.mutateAsync({
				path: { organization_id: organizationId, workflow_id: workflowId },
				body: { trigger_payload: payload },
			})
			setOpen(false)
			await navigate({
				to: buildOrgPath(
					organizationSlug,
					'/automatisation/$workflowId/executions/$runId',
				),
				params: {
					workflowId,
					runId: (result.data as { run_id: string }).run_id,
				},
			})
		} catch (error) {
			setStartError(
				error instanceof Error && error.message
					? error.message
					: 'Le démarrage a échoué.',
			)
		}
	}

	function handleOpenHistory() {
		void navigate({
			to: buildOrgPath(
				organizationSlug,
				'/automatisation/$workflowId/executions',
			),
			params: { workflowId },
		})
	}

	return (
		<div className="flex items-center justify-between gap-2 border-b px-4 py-2">
			<Button variant="ghost" size="sm" onClick={handleOpenHistory}>
				Historique d’exécution
			</Button>
			<RequirePermission permission="MANAGE_AUTOMATION">
				<Button size="sm" onClick={() => setOpen(true)}>
					Exécuter maintenant
				</Button>
			</RequirePermission>

			<RunNowDialog
				open={open}
				events={eventsQuery.data?.data ?? []}
				triggerEventNames={subscribedEventNames(
					workflowQuery.data?.data.current_version?.graph,
				)}
				isStarting={startRun.isPending}
				startError={startError}
				onOpenChange={setOpen}
				onConfirm={(payload) => void handleConfirm(payload)}
			/>
		</div>
	)
}
