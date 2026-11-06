import { useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationEvents,
	useAutomationRuns,
	useAutomationWorkflows,
	useConnectorCatalogue,
	useRunGraphs,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import { workflowExecutionsLocation } from '#/pages/automation/lib/automation-paths'
import { typographicEventLabels } from '#/pages/automation/lib/event-search'
import {
	ALL,
	buildRow,
	DEFAULT_FILTERS,
	distinctTriggers,
	type ExecutionFilters,
	type ExecutionRow,
	type ExecutionSortKey,
	filterRows,
	hasActiveFilters,
	type SortDirection,
	sortRows,
	statusCounts,
	summarize,
} from '#/pages/automation/lib/execution-log'
import { graphNodeNames } from '#/pages/automation/lib/run-view'
import {
	failedRunProbes,
	isTerminalRunStatus,
} from '#/pages/automation/lib/workflow-runs'
import { AutomationExecutionsLog } from '#/pages/automation/ui/automation-executions-log'

const LIVE_POLL_MS = 3000

const LIVE_TICK_MS = 200

const DELETED_WORKFLOW = 'Workflow supprimé'

export type ExecutionsLogFilters = Omit<ExecutionFilters, 'query'>

export interface ExecutionsLogFeatureProps {
	filters: ExecutionsLogFilters
	onFiltersChange: (patch: Partial<ExecutionsLogFilters>) => void
}

export function ExecutionsLogFeature(props: ExecutionsLogFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<ExecutionsLogWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function ExecutionsLogWorkspace({
	organizationId,
	organizationSlug,
	filters,
	onFiltersChange,
}: ExecutionsLogFeatureProps & {
	organizationId: string
	organizationSlug: string
}) {
	const navigate = useNavigate()
	const [query, setQuery] = useState('')
	const [sortKey, setSortKey] = useState<ExecutionSortKey>('started')
	const [direction, setDirection] = useState<SortDirection>('desc')
	const runsQuery = useAutomationRuns(organizationId)
	const workflowsQuery = useAutomationWorkflows(organizationId)
	const archivedQuery = useAutomationWorkflows(organizationId, true)
	const eventsQuery = useAutomationEvents(organizationId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const { refetch: refetchRuns } = runsQuery
	const runs = runsQuery.data?.data
	const probes = useMemo(() => failedRunProbes(runs ?? []), [runs])
	const graphs = useRunGraphs(organizationId, probes)
	const hasLiveRun = (runs ?? []).some(
		(run) => !isTerminalRunStatus(run.status),
	)
	const hasRunningRun = (runs ?? []).some((run) => run.status === 'running')
	const [now, setNow] = useState(() => Date.now())

	useEffect(() => {
		if (!hasLiveRun) return
		const timer = window.setInterval(() => void refetchRuns(), LIVE_POLL_MS)
		return () => window.clearInterval(timer)
	}, [hasLiveRun, refetchRuns])

	useEffect(() => {
		if (!hasRunningRun) return
		setNow(Date.now())
		const timer = window.setInterval(() => setNow(Date.now()), LIVE_TICK_MS)
		return () => window.clearInterval(timer)
	}, [hasRunningRun])

	const allWorkflows = useMemo(
		() => [
			...(workflowsQuery.data?.data ?? []),
			...(archivedQuery.data?.data ?? []),
		],
		[workflowsQuery.data, archivedQuery.data],
	)
	const names = useMemo(
		() => new Map(allWorkflows.map((workflow) => [workflow.id, workflow.name])),
		[allWorkflows],
	)
	const workflowList = useMemo(() => {
		const withRuns = new Set((runs ?? []).map((run) => run.workflow_id))
		return [
			...allWorkflows
				.filter((workflow) => withRuns.has(workflow.id))
				.map((workflow) => ({ id: workflow.id, name: workflow.name })),
			...[...withRuns]
				.filter((id) => !names.has(id))
				.map((id) => ({ id, name: DELETED_WORKFLOW })),
		]
	}, [runs, allWorkflows, names])
	const events = useMemo(
		() => typographicEventLabels(eventsQuery.data?.data ?? []),
		[eventsQuery.data],
	)
	const eventLabels = useMemo(
		() => new Map(events.map((event) => [event.name, event.label])),
		[events],
	)
	const descriptors = useMemo(
		() =>
			new Map(
				(catalogueQuery.data?.data.connectors ?? []).map((connector) => [
					connector.kind,
					connector,
				]),
			),
		[catalogueQuery.data],
	)
	const nodeNamesByVersion = useMemo(
		() =>
			new Map(
				[...graphs].map(([versionId, graph]) => [
					versionId,
					graph ? graphNodeNames(graph, descriptors, events) : new Map(),
				]),
			),
		[graphs, descriptors, events],
	)
	const rows = useMemo(
		() =>
			(runs ?? []).map((run) => {
				const failedId = run.failed_connector_id
				const nodeNames = nodeNamesByVersion.get(run.workflow_version_id)
				return buildRow(
					run,
					names.get(run.workflow_id) ?? DELETED_WORKFLOW,
					now,
					{
						trigger: { events: eventLabels },
						failedNode:
							failedId && nodeNames
								? (nodeNames.get(failedId) ?? failedId)
								: null,
					},
				)
			}),
		[runs, names, now, eventLabels, nodeNamesByVersion],
	)

	const active: ExecutionFilters = { ...filters, query }
	const visible = sortRows(filterRows(rows, active), sortKey, direction)
	const counts = statusCounts(filterRows(rows, active, undefined, 'status'))

	function reset() {
		setQuery('')
		onFiltersChange({
			status: DEFAULT_FILTERS.status,
			workflow: ALL,
			trigger: ALL,
			period: DEFAULT_FILTERS.period,
		})
	}

	function sortBy(key: ExecutionSortKey) {
		if (key === sortKey) setDirection(direction === 'asc' ? 'desc' : 'asc')
		else {
			setSortKey(key)
			setDirection(key === 'workflow' ? 'asc' : 'desc')
		}
	}

	function open(row: ExecutionRow) {
		const location = workflowExecutionsLocation(row.workflowId, row.number)
		void navigate({
			to: buildOrgPath(organizationSlug, location.to),
			search: location.search,
		})
	}

	return (
		<AutomationExecutionsLog
			organizationSlug={organizationSlug}
			rows={visible}
			summary={summarize(visible)}
			filters={active}
			counts={counts}
			workflows={workflowList}
			triggers={distinctTriggers(rows)}
			canReset={hasActiveFilters(active)}
			isLoading={runsQuery.isLoading}
			error={runsQuery.error?.message ?? workflowsQuery.error?.message ?? null}
			sortKey={sortKey}
			direction={direction}
			onFiltersChange={(patch) => {
				const { query: nextQuery, ...rest } = patch
				if (nextQuery !== undefined) setQuery(nextQuery)
				if (Object.keys(rest).length > 0) onFiltersChange(rest)
			}}
			onSort={sortBy}
			onOpen={open}
			onReset={reset}
		/>
	)
}
