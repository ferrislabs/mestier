import { AlertCircle, Loader2 } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useConnectorCatalogue,
	useReplayRun,
	useRunPolling,
	useWorkflow,
} from '#/hooks/use-automation'
import { readLayout } from '#/pages/automation/lib/graph'
import { mergedLayout } from '#/pages/automation/lib/layout'
import {
	aggregateConnectorStatuses,
	groupRunSteps,
} from '#/pages/automation/lib/workflow-runs'
import { RunInspector } from '#/pages/automation/ui/run-inspector'

export interface RunInspectorFeatureProps {
	workflowId: string
	runId: string
}

export function RunInspectorFeature({
	workflowId,
	runId,
}: RunInspectorFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<RunInspectorWorkspace
			key={`${activeOrganization.id}:${workflowId}:${runId}`}
			organizationId={activeOrganization.id}
			workflowId={workflowId}
			runId={runId}
		/>
	)
}

function descriptorsByKind(
	connectors: Schemas.ConnectorDescriptorResponse[],
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(connectors.map((connector) => [connector.kind, connector]))
}

function RunInspectorWorkspace({
	organizationId,
	workflowId,
	runId,
}: {
	organizationId: string
	workflowId: string
	runId: string
}) {
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const runQuery = useRunPolling(organizationId, runId)
	const replayRun = useReplayRun()

	const [replayError, setReplayError] = useState<string | null>(null)
	const [replayingConnectorId, setReplayingConnectorId] = useState<
		string | null
	>(null)

	if (
		workflowQuery.isLoading ||
		catalogueQuery.isLoading ||
		runQuery.isLoading
	) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement de l’exécution…
				</SectionCard>
			</PageShell>
		)
	}

	if (runQuery.isError || !runQuery.data?.data) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
					<AlertCircle className="size-6 text-destructive" />
					<div>
						<p className="font-semibold">
							Impossible de charger cette exécution
						</p>
						<p className="mt-1 text-sm text-muted-foreground">
							{runQuery.error?.message ?? 'Réessayez dans quelques instants.'}
						</p>
					</div>
				</SectionCard>
			</PageShell>
		)
	}

	const run = runQuery.data.data
	const descriptors = descriptorsByKind(
		catalogueQuery.data?.data.connectors ?? [],
	)
	const storedLayout = readLayout(workflowQuery.data?.data.layout)
	const layout = run.graph ? mergedLayout(run.graph, storedLayout) : new Map()
	const connectorStatuses = aggregateConnectorStatuses(run.steps)
	const stepTree = groupRunSteps(run.steps)

	async function handleReplay(connectorId: string) {
		setReplayError(null)
		setReplayingConnectorId(connectorId)
		try {
			await replayRun.mutateAsync({
				path: { organization_id: organizationId, run_id: runId },
				body: { connector_id: connectorId },
			})
			await runQuery.refetch()
		} catch (error) {
			setReplayError(
				error instanceof Error && error.message
					? error.message
					: 'La relance a échoué.',
			)
		} finally {
			setReplayingConnectorId(null)
		}
	}

	return (
		<RunInspector
			run={run}
			graph={run.graph ?? null}
			layout={layout}
			descriptors={descriptors}
			connectorStatuses={connectorStatuses}
			stepTree={stepTree}
			onReplay={(connectorId) => void handleReplay(connectorId)}
			isReplaying={replayRun.isPending}
			replayingConnectorId={replayingConnectorId}
			replayError={replayError}
		/>
	)
}
