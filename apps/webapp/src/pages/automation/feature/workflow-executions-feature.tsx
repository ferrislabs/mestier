import { useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Run,
	useAutomationEvents,
	useAutomationRuns,
	useConnectorCatalogue,
	useReplayRun,
	useRunGraphs,
	useRunPolling,
	useWorkflow,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import { RunNodeDetailsFeature } from '#/pages/automation/feature/run-node-details-feature'
import { RunNowFeature } from '#/pages/automation/feature/run-now-feature'
import { WorkflowTopBarFeature } from '#/pages/automation/feature/workflow-top-bar-feature'
import { workflowExecutionsLocation } from '#/pages/automation/lib/automation-paths'
import { typographicEventLabels } from '#/pages/automation/lib/event-search'
import {
	filterStatusOf,
	runResumeAt,
	toRunDisplayStatus,
} from '#/pages/automation/lib/execution-status'
import {
	formatMilliseconds,
	formatResumeNote,
	formatStartedAt,
	formatVersionLabel,
} from '#/pages/automation/lib/format'
import { readLayout } from '#/pages/automation/lib/graph'
import { mergedLayout } from '#/pages/automation/lib/layout'
import { runNodeAppearance } from '#/pages/automation/lib/run-appearance'
import {
	buildRunView,
	graphNodeNames,
	type RunNodeView,
	type RunStepGroup,
	type RunStepView,
} from '#/pages/automation/lib/run-view'
import {
	canReplay,
	isTerminalRunStatus,
	runElapsedMs,
	runTriggerLabel,
	triggerRunProbes,
} from '#/pages/automation/lib/workflow-runs'
import type { ExecutionFilter } from '#/pages/automation/ui/execution-list'
import type {
	ExecutionStepEntry,
	ExecutionStepItem,
} from '#/pages/automation/ui/execution-steps'
import {
	type ExecutionDetailData,
	WorkflowExecutions,
} from '#/pages/automation/ui/workflow-executions'
import { WorkflowNotFound } from '#/pages/automation/ui/workflow-not-found'

const LIVE_POLL_MS = 3000

const LIVE_TICK_MS = 200

function isNotFoundError(error: unknown): boolean {
	return (
		typeof error === 'object' &&
		error !== null &&
		'status' in error &&
		error.status === 404
	)
}

export interface WorkflowExecutionsFeatureProps {
	workflowId: string
	executionNumber?: number
}

export function WorkflowExecutionsFeature({
	workflowId,
	executionNumber,
}: WorkflowExecutionsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowExecutionsWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
			executionNumber={executionNumber}
		/>
	)
}

function descriptorsByKind(
	connectors: Schemas.ConnectorDescriptorResponse[],
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(connectors.map((connector) => [connector.kind, connector]))
}

function toStepEntry(
	step: RunStepView,
	nodes: ReadonlyMap<string, RunNodeView>,
	plain = false,
): ExecutionStepEntry {
	const appearance = runNodeAppearance(nodes.get(step.nodeId))
	return {
		id: step.stepId,
		nodeId: step.nodeId,
		label: plain ? step.plainLabel : step.label,
		icon: appearance.icon,
		hue: appearance.hue,
		status: step.status,
		detail: step.detail,
	}
}

function toStepItems(
	groups: readonly RunStepGroup[],
	nodes: ReadonlyMap<string, RunNodeView>,
	insideLoop = false,
): ExecutionStepItem[] {
	return groups.map((group): ExecutionStepItem => {
		if (group.kind === 'step')
			return { kind: 'step', step: toStepEntry(group.step, nodes, insideLoop) }
		return {
			kind: 'loop',
			id: group.id,
			label: group.label,
			iterations: group.iterations.map((iteration) => ({
				index: iteration.index,
				path: iteration.path,
				children: toStepItems(iteration.children, nodes, true),
			})),
		}
	})
}

function WorkflowExecutionsWorkspace({
	organizationId,
	organizationSlug,
	workflowId,
	executionNumber,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
	executionNumber?: number
}) {
	const navigate = useNavigate()
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const eventsQuery = useAutomationEvents(organizationId)
	const runsQuery = useAutomationRuns(organizationId)
	const replayRun = useReplayRun()

	const [filter, setFilter] = useState<ExecutionFilter>('all')
	const [openNodeId, setOpenNodeId] = useState<string | null>(null)
	const [replayError, setReplayError] = useState<string | null>(null)
	const [runNowOpen, setRunNowOpen] = useState(false)

	const runs = useMemo(
		() =>
			(runsQuery.data?.data ?? [])
				.filter((run) => run.workflow_id === workflowId)
				.sort((a, b) => b.number - a.number),
		[runsQuery.data, workflowId],
	)
	const probes = useMemo(() => triggerRunProbes(runs), [runs])
	const graphs = useRunGraphs(organizationId, probes)
	const selectedRun =
		runs.find((item) => item.number === executionNumber) ?? runs[0]
	const selectedId = selectedRun?.id ?? null
	const runQuery = useRunPolling(organizationId, selectedId)
	const run = runQuery.data?.data
	const { refetch: refetchRuns } = runsQuery
	const runStatus = run?.status
	const hasLiveRun = runs.some((item) => !isTerminalRunStatus(item.status))
	const isTicking =
		runStatus === 'running' || runs.some((item) => item.status === 'running')
	const [now, setNow] = useState(() => Date.now())

	useEffect(() => {
		if (runStatus !== undefined && isTerminalRunStatus(runStatus))
			void refetchRuns()
	}, [runStatus, refetchRuns])

	useEffect(() => {
		if (!hasLiveRun) return
		const timer = window.setInterval(() => void refetchRuns(), LIVE_POLL_MS)
		return () => window.clearInterval(timer)
	}, [hasLiveRun, refetchRuns])

	useEffect(() => {
		if (!isTicking) return
		setNow(Date.now())
		const timer = window.setInterval(() => setNow(Date.now()), LIVE_TICK_MS)
		return () => window.clearInterval(timer)
	}, [isTicking])

	const events = useMemo(
		() => typographicEventLabels(eventsQuery.data?.data ?? []),
		[eventsQuery.data],
	)
	const eventLabels = useMemo(
		() => new Map(events.map((event) => [event.name, event.label])),
		[events],
	)
	const descriptors = useMemo(
		() => descriptorsByKind(catalogueQuery.data?.data.connectors ?? []),
		[catalogueQuery.data],
	)
	const currentVersion = workflowQuery.data?.data.current_version
	const currentNodeNames = useMemo(
		() =>
			currentVersion
				? graphNodeNames(currentVersion.graph, descriptors, events)
				: undefined,
		[currentVersion, descriptors, events],
	)
	const ownNodeNames = useMemo(
		() =>
			run?.graph ? graphNodeNames(run.graph, descriptors, events) : undefined,
		[run?.graph, descriptors, events],
	)
	const versionNodeNames = useMemo(
		() =>
			new Map(
				[...graphs].map(([versionId, graph]) => [
					versionId,
					graph ? graphNodeNames(graph, descriptors, events) : new Map(),
				]),
			),
		[graphs, descriptors, events],
	)
	const triggerOf = (item: Run) =>
		runTriggerLabel(item, {
			events: eventLabels,
			nodeNames:
				item.id === run?.id
					? ownNodeNames
					: item.workflow_version_id === currentVersion?.id
						? currentNodeNames
						: versionNodeNames.get(item.workflow_version_id),
		})

	const entries = runs
		.filter((item) => {
			if (filter === 'all') return true
			return filterStatusOf(toRunDisplayStatus(item, now)) === filter
		})
		.map((item) => {
			const resumeAt = runResumeAt(item, now)
			const duration = resumeAt === null ? runElapsedMs(item, now) : null
			return {
				number: item.number,
				status: toRunDisplayStatus(item, now),
				startedAt: formatStartedAt(item.started_at ?? item.created_at),
				trigger: triggerOf(item),
				duration: duration === null ? null : formatMilliseconds(duration),
				resumeNote: formatResumeNote(resumeAt),
			}
		})

	const viewNow = runStatus === 'running' ? now : undefined
	const view = useMemo(
		() => (run ? buildRunView(run, descriptors, viewNow, events) : null),
		[run, descriptors, viewNow, events],
	)
	const nodesById = useMemo(
		() => new Map((view?.nodes ?? []).map((node) => [node.id, node])),
		[view],
	)

	const layout = useMemo(() => {
		if (!run?.graph) return new Map()
		return mergedLayout(run.graph, readLayout(workflowQuery.data?.data.layout))
	}, [run, workflowQuery.data])

	const detail: ExecutionDetailData | null =
		run && view
			? {
					header: {
						runId: run.id,
						number: run.number,
						startedAt: formatStartedAt(run.started_at ?? run.created_at),
						trigger: triggerOf(run),
						status: view.status,
						duration:
							view.status === 'paused'
								? (formatResumeNote(view.resumeAt) ?? undefined)
								: view.durationMs === null
									? undefined
									: formatMilliseconds(view.durationMs),
						versionLabel: formatVersionLabel(run.workflow_version_created_at),
						isCurrentVersion: run.workflow_version_id === currentVersion?.id,
					},
					graph: run.graph ?? null,
					layout,
					descriptors,
					view,
					steps: toStepItems(view.stepGroups, nodesById),
					pending: view.pending.map((step) => toStepEntry(step, nodesById)),
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

	const openNode =
		openNodeId && view && run
			? (view.nodes.find((node) => node.id === openNodeId) ?? null)
			: null

	const nodeDialog =
		openNode && view && run ? (
			<RunNodeDetailsFeature
				key={`${run.id}:${openNode.id}`}
				organizationId={organizationId}
				organizationSlug={organizationSlug}
				run={run}
				view={view}
				descriptors={descriptors}
				node={openNode}
				triggerLabel={triggerOf(run)}
				versionLabel={formatVersionLabel(run.workflow_version_created_at)}
				replay={{
					available: canReplay(run.status),
					isReplaying: replayRun.isPending,
					error: replayError,
					onReplay: () => void handleReplay(openNode.id),
				}}
				onNavigate={setOpenNodeId}
				onClose={() => {
					setOpenNodeId(null)
					setReplayError(null)
				}}
			/>
		) : null

	if (isNotFoundError(workflowQuery.error))
		return (
			<WorkflowNotFound
				organizationSlug={organizationSlug}
				workflowId={workflowId}
			/>
		)

	const loadError = workflowQuery.error
		? `Impossible de charger le workflow : ${workflowQuery.error.message}`
		: (runsQuery.error?.message ?? null)

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowTopBarFeature workflowId={workflowId} view="executions" />
			<div className="relative min-h-0 flex-1">
				<WorkflowExecutions
					entries={entries}
					total={runs.length}
					selectedNumber={selectedRun?.number ?? null}
					filter={filter}
					isLoading={runsQuery.isLoading || workflowQuery.isLoading}
					error={loadError}
					detail={detail}
					detailLoading={runQuery.isLoading}
					detailError={runQuery.error?.message ?? null}
					onSelect={(number) => {
						setOpenNodeId(null)
						const location = workflowExecutionsLocation(workflowId, number)
						void navigate({
							to: buildOrgPath(organizationSlug, location.to),
							search: location.search,
						})
					}}
					onFilterChange={setFilter}
					onOpenNode={setOpenNodeId}
					onRunNow={() => setRunNowOpen(true)}
					nodeDialog={nodeDialog}
				/>
			</div>
			<RunNowFeature
				workflowId={workflowId}
				controlled={{ open: runNowOpen, onOpenChange: setRunNowOpen }}
			/>
		</div>
	)
}
