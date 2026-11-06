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
import { workflowExecutionsLocation } from '#/pages/automation/lib/automation-paths'
import { triggerEventNames } from '#/pages/automation/lib/trigger-events'
import { RunNowDialog } from '#/pages/automation/ui/run-now-dialog'

export interface RunNowControl {
	open: boolean
	onOpenChange: (open: boolean) => void
}

export interface RunNowFeatureProps {
	workflowId: string
	controlled?: RunNowControl
}

export function RunNowFeature({ workflowId, controlled }: RunNowFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<RunNowWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
			controlled={controlled}
		/>
	)
}

function subscribedEventNames(graph: Schemas.GraphDto | null | undefined) {
	if (!graph) return []

	const names = new Set<string>()
	for (const trigger of graph.triggers) {
		for (const name of triggerEventNames(trigger)) names.add(name)
	}

	return [...names]
}

function RunNowWorkspace({
	organizationId,
	organizationSlug,
	workflowId,
	controlled,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
	controlled?: RunNowControl
}) {
	const eventsQuery = useAutomationEvents(organizationId)
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const startRun = useStartRun()
	const navigate = useNavigate()

	const [innerOpen, setInnerOpen] = useState(false)
	const open = controlled ? controlled.open : innerOpen
	const setOpen = controlled ? controlled.onOpenChange : setInnerOpen
	const [startError, setStartError] = useState<string | null>(null)

	async function handleConfirm(payload: unknown) {
		setStartError(null)
		try {
			await startRun.mutateAsync({
				path: { organization_id: organizationId, workflow_id: workflowId },
				body: { trigger_payload: payload },
			})
			setOpen(false)
		} catch (error) {
			setStartError(
				error instanceof Error && error.message
					? error.message
					: 'Le démarrage a échoué.',
			)
		}
	}

	function handleOpenHistory() {
		const location = workflowExecutionsLocation(workflowId)
		void navigate({
			to: buildOrgPath(organizationSlug, location.to),
			search: location.search,
		})
	}

	const dialog = (
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
	)

	if (controlled) return dialog

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
			{dialog}
		</div>
	)
}
