import { useCallback, useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationCredentials,
	useAutomationEvents,
	useAutomationRuns,
	useConnectorCatalogue,
	useCreateCredential,
	useEvaluateExpression,
	useRunPolling,
} from '#/hooks/use-automation'
import {
	type LastRunData,
	WorkflowCanvas,
} from '#/pages/automation/feature/editor-canvas-feature'
import { RunNowFeature } from '#/pages/automation/feature/run-now-feature'
import {
	type EditorSession,
	WorkflowTopBarFeature,
} from '#/pages/automation/feature/workflow-top-bar-feature'
import { NEW_WORKFLOW_ID } from '#/pages/automation/lib/automation-paths'
import { typographicEventLabels } from '#/pages/automation/lib/event-search'
import {
	aggregateConnectorStatuses,
	connectorOutputsFromSteps,
	isTerminalRunStatus,
	latestRunId,
	representativeStepByConnector,
} from '#/pages/automation/lib/workflow-runs'
import { WorkflowLoading } from '#/pages/automation/ui/workflow-not-found'

export interface WorkflowCanvasFeatureProps {
	session: EditorSession
}

function descriptorsByKind(
	connectors: Schemas.ConnectorDescriptorResponse[],
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(connectors.map((connector) => [connector.kind, connector]))
}

export function WorkflowCanvasFeature({ session }: WorkflowCanvasFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const workflowId = session.workflowId
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const credentialsQuery = useAutomationCredentials(organizationId)
	const eventsQuery = useAutomationEvents(organizationId)
	const runsQuery = useAutomationRuns(organizationId)
	const latestId = workflowId
		? latestRunId(runsQuery.data?.data ?? [], workflowId)
		: null
	const runDetailQuery = useRunPolling(organizationId, latestId)
	const createCredential = useCreateCredential(organizationId)
	const { mutateAsync: evaluate } = useEvaluateExpression()
	const [runDialogOpen, setRunDialogOpen] = useState(false)
	const connectors = catalogueQuery.data?.data.connectors
	const descriptors = useMemo(
		() => descriptorsByKind(connectors ?? []),
		[connectors],
	)
	const events = useMemo(
		() => typographicEventLabels(eventsQuery.data?.data ?? []),
		[eventsQuery.data],
	)
	const runDetail = runDetailQuery.data?.data
	const lastRun = useMemo<LastRunData | null>(
		() =>
			runDetail
				? {
						triggerPayload: runDetail.trigger_payload ?? null,
						connectorOutputs: connectorOutputsFromSteps(runDetail.steps),
					}
				: null,
		[runDetail],
	)
	const runStatuses = useMemo(
		() =>
			runDetail
				? aggregateConnectorStatuses(runDetail.steps)
				: new Map<string, string>(),
		[runDetail],
	)
	const runSteps = useMemo(
		() =>
			runDetail
				? representativeStepByConnector(runDetail.steps)
				: new Map<string, Schemas.RunStepResponse>(),
		[runDetail],
	)
	const [initial] = useState(() => session.snapshot())

	const handleCreateCredential = async (
		body: Schemas.CreateCredentialRequest,
	) => {
		const created = await createCredential.mutateAsync({
			path: { organization_id: organizationId },
			body,
		})
		return created.data as Schemas.CredentialResponse & { secret: unknown }
	}

	const handleEvaluateExpression = useCallback(
		async (template: unknown, context: Schemas.EvaluateContextBody) => {
			const result = await evaluate({
				path: { organization_id: organizationId },
				body: { template, context },
			})
			return (result.data as { value: unknown }).value
		},
		[evaluate, organizationId],
	)

	const topBarId = workflowId ?? NEW_WORKFLOW_ID
	const { validation, saveError } = session

	if (
		catalogueQuery.isLoading ||
		credentialsQuery.isLoading ||
		eventsQuery.isLoading
	) {
		return (
			<div className="flex min-h-0 flex-1 flex-col">
				<WorkflowTopBarFeature workflowId={topBarId} view="editor" />
				<WorkflowLoading />
			</div>
		)
	}

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowTopBarFeature workflowId={topBarId} view="editor" />
			{validation.graphErrors.length > 0 || saveError ? (
				<div
					role="alert"
					className="border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive"
				>
					<ul>
						{saveError ? <li>{saveError}</li> : null}
						{validation.graphErrors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				</div>
			) : null}
			<WorkflowCanvas
				graph={initial.graph}
				layout={initial.layout}
				organizationSlug={activeOrganization.slug}
				descriptors={descriptors}
				connectorErrors={validation.connectorErrors}
				events={events}
				lastRun={lastRun}
				runStatuses={runStatuses}
				runSteps={runSteps}
				onEvaluateExpression={handleEvaluateExpression}
				credentials={credentialsQuery.data?.data ?? []}
				authSchemes={catalogueQuery.data?.data.auth_schemes ?? []}
				onCreateCredential={handleCreateCredential}
				canTest={workflowId !== null}
				testRun={{
					running:
						runDetail !== undefined && !isTerminalRunStatus(runDetail.status),
					start: () => {
						if (workflowId) setRunDialogOpen(true)
					},
				}}
				onChange={session.onGraphChange}
			/>
			{workflowId ? (
				<RunNowFeature
					workflowId={workflowId}
					controlled={{ open: runDialogOpen, onOpenChange: setRunDialogOpen }}
				/>
			) : null}
		</div>
	)
}
