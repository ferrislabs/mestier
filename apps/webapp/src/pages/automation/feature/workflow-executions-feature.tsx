import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationRuns,
	useConnectorCatalogue,
	useReplayRun,
	useRunPolling,
	useWorkflow,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import { WorkflowTopBarFeature } from '#/pages/automation/feature/workflow-top-bar-feature'
import {
	connectorAppearance,
	TRIGGER_APPEARANCE,
} from '#/pages/automation/lib/connector-appearance'
import { shortRunId } from '#/pages/automation/lib/execution-log'
import { toExecutionStatus } from '#/pages/automation/lib/execution-status'
import {
	formatMilliseconds,
	formatStartedAt,
} from '#/pages/automation/lib/format'
import { readLayout } from '#/pages/automation/lib/graph'
import { mergedLayout } from '#/pages/automation/lib/layout'
import {
	nodeNeighbours,
	parameterEntries,
} from '#/pages/automation/lib/run-node'
import {
	buildRunView,
	type RunNodeView,
	type RunStepView,
} from '#/pages/automation/lib/run-view'
import {
	canReplay,
	runDurationMs,
	runTriggerLabel,
} from '#/pages/automation/lib/workflow-runs'
import type { ExecutionFilter } from '#/pages/automation/ui/execution-list'
import type { ExecutionStepEntry } from '#/pages/automation/ui/execution-steps'
import type { NdvNavItem } from '#/pages/automation/ui/ndv-node-nav'
import { RunNodeDetails } from '#/pages/automation/ui/run-node-details'
import {
	type ExecutionDetailData,
	WorkflowExecutions,
} from '#/pages/automation/ui/workflow-executions'

export interface WorkflowExecutionsFeatureProps {
	workflowId: string
	runId?: string
}

export function WorkflowExecutionsFeature({
	workflowId,
	runId,
}: WorkflowExecutionsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowExecutionsWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
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

function toStepEntry(step: RunStepView): ExecutionStepEntry {
	const appearance = connectorAppearance(step.family)
	return {
		id: step.stepId,
		nodeId: step.nodeId,
		label: step.label,
		icon: appearance.icon,
		hue: appearance.hue,
		status: step.status,
		detail: step.detail,
	}
}

function WorkflowExecutionsWorkspace({
	organizationId,
	organizationSlug,
	workflowId,
	runId,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
	runId?: string
}) {
	const navigate = useNavigate()
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const runsQuery = useAutomationRuns(organizationId)
	const replayRun = useReplayRun()

	const [filter, setFilter] = useState<ExecutionFilter>('all')
	const [openNodeId, setOpenNodeId] = useState<string | null>(null)
	const [replayError, setReplayError] = useState<string | null>(null)

	const runs = useMemo(
		() =>
			(runsQuery.data?.data ?? [])
				.filter((run) => run.workflow_id === workflowId)
				.sort((a, b) => (a.created_at < b.created_at ? 1 : -1)),
		[runsQuery.data, workflowId],
	)
	const selectedId = runs.some((run) => run.id === runId)
		? (runId ?? null)
		: (runs[0]?.id ?? null)
	const runQuery = useRunPolling(organizationId, selectedId)
	const run = runQuery.data?.data

	const entries = runs
		.filter((item) => {
			if (filter === 'all') return true
			return toExecutionStatus(item.status) === filter
		})
		.map((item) => {
			const duration = runDurationMs(item)
			return {
				id: item.id,
				shortId: shortRunId(item.id),
				status: toExecutionStatus(item.status),
				startedAt: formatStartedAt(item.started_at ?? item.created_at),
				trigger: runTriggerLabel(item),
				duration: duration === null ? null : formatMilliseconds(duration),
			}
		})

	const descriptors = useMemo(
		() => descriptorsByKind(catalogueQuery.data?.data.connectors ?? []),
		[catalogueQuery.data],
	)

	const view = useMemo(
		() => (run ? buildRunView(run, descriptors) : null),
		[run, descriptors],
	)

	const layout = useMemo(() => {
		if (!run?.graph) return new Map()
		return mergedLayout(run.graph, readLayout(workflowQuery.data?.data.layout))
	}, [run, workflowQuery.data])

	const detail: ExecutionDetailData | null =
		run && view
			? {
					header: {
						shortId: shortRunId(run.id),
						startedAt: formatStartedAt(run.started_at ?? run.created_at),
						trigger: runTriggerLabel(run),
						status: view.status,
						duration:
							view.durationMs === null
								? undefined
								: formatMilliseconds(view.durationMs),
						versionLabel: `Version ${shortRunId(run.workflow_version_id)}`,
						isCurrentVersion:
							run.workflow_version_id ===
							workflowQuery.data?.data.current_version?.id,
					},
					graph: run.graph ?? null,
					layout,
					descriptors,
					view,
					steps: view.steps.map(toStepEntry),
					pending: view.pending.map(toStepEntry),
					failure: view.failed
						? {
								label: view.failed.label,
								message: view.failed.detail,
								nodeId: view.failed.nodeId,
							}
						: null,
				}
			: null

	async function handleReplay(connectorId: string) {
		if (!selectedId) return
		setReplayError(null)
		try {
			await replayRun.mutateAsync({
				path: { organization_id: organizationId, run_id: selectedId },
				body: { connector_id: connectorId },
			})
			await runQuery.refetch()
		} catch (error) {
			setReplayError(
				error instanceof Error && error.message
					? error.message
					: 'La relance a échoué.',
			)
		}
	}

	function navItem(
		id: string,
		branch: string | undefined,
		nodes: RunNodeView[],
	): NdvNavItem {
		const node = nodes.find((candidate) => candidate.id === id)
		const appearance =
			node?.kind === 'trigger'
				? TRIGGER_APPEARANCE
				: connectorAppearance(node?.family)
		return {
			id,
			label: node?.label ?? id,
			icon: appearance.icon,
			hue: appearance.hue,
			branch,
		}
	}

	const openNode =
		openNodeId && view && run
			? (view.nodes.find((node) => node.id === openNodeId) ?? null)
			: null

	const nodeDialog =
		openNode && view && run ? (
			<RunNodeDetails
				key={`${run.id}:${openNode.id}`}
				runShortId={shortRunId(run.id)}
				versionLabel={`Version ${shortRunId(run.workflow_version_id)}`}
				node={openNode}
				typeLabel={
					openNode.kind === 'trigger'
						? 'Déclencheur'
						: (descriptors.get(openNode.connector?.kind ?? '')?.label ??
							openNode.label)
				}
				icon={
					(openNode.kind === 'trigger'
						? TRIGGER_APPEARANCE
						: connectorAppearance(openNode.family)
					).icon
				}
				hue={
					(openNode.kind === 'trigger'
						? TRIGGER_APPEARANCE
						: connectorAppearance(openNode.family)
					).hue
				}
				parameters={parameterEntries(
					openNode.connector,
					descriptors.get(openNode.connector?.kind ?? ''),
				)}
				triggerPayload={run.trigger_payload}
				sources={
					run.graph
						? nodeNeighbours(run.graph, openNode.id).previous.map(
								(neighbour) =>
									`« ${view.nodes.find((n) => n.id === neighbour.id)?.label ?? neighbour.id} »`,
							)
						: []
				}
				previous={
					run.graph
						? nodeNeighbours(run.graph, openNode.id).previous.map((n) =>
								navItem(n.id, n.branch, view.nodes),
							)
						: []
				}
				next={
					run.graph
						? nodeNeighbours(run.graph, openNode.id).next.map((n) =>
								navItem(n.id, n.branch, view.nodes),
							)
						: []
				}
				replay={{
					available: canReplay(run.status),
					isReplaying: replayRun.isPending,
					error: replayError,
					onReplay: () => void handleReplay(openNode.id),
				}}
				credentialsHref={buildOrgPath(
					organizationSlug,
					'/automatisation/credentials',
				)}
				onNavigate={setOpenNodeId}
				onClose={() => {
					setOpenNodeId(null)
					setReplayError(null)
				}}
			/>
		) : null

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowTopBarFeature workflowId={workflowId} view="executions" />
			<div className="relative min-h-0 flex-1">
				<WorkflowExecutions
					entries={entries}
					total={runs.length}
					selectedId={selectedId}
					filter={filter}
					isLoading={runsQuery.isLoading || workflowQuery.isLoading}
					error={runsQuery.error?.message ?? null}
					detail={detail}
					detailLoading={runQuery.isLoading}
					detailError={runQuery.error?.message ?? null}
					onSelect={(id) => {
						setOpenNodeId(null)
						void navigate({
							to: buildOrgPath(
								organizationSlug,
								`/automatisation/${workflowId}/executions/${id}`,
							),
						})
					}}
					onFilterChange={setFilter}
					onOpenNode={setOpenNodeId}
					nodeDialog={nodeDialog}
				/>
			</div>
		</div>
	)
}
