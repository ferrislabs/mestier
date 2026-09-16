import type { Schemas } from '#/api/api.client'
import { StatusBadge } from '#/components/ui/surface'
import type { NodePosition } from '#/pages/automation/lib/graph'
import {
	formatRunDuration,
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
	type RunStepTreeNode,
	runDurationMs,
	runTriggerLabel,
} from '#/pages/automation/lib/workflow-runs'
import { RunGraphCanvas } from '#/pages/automation/ui/run-graph-canvas'
import { RunStepList } from '#/pages/automation/ui/run-step-list'

export interface RunInspectorProps {
	run: Schemas.RunResponse
	graph: Schemas.GraphDto | null
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorStatuses: Map<string, string>
	stepTree: RunStepTreeNode[]
	onReplay: (connectorId: string) => void
	isReplaying: boolean
	replayingConnectorId: string | null
	replayError: string | null
}

export function RunInspector({
	run,
	graph,
	layout,
	descriptors,
	connectorStatuses,
	stepTree,
	onReplay,
	isReplaying,
	replayingConnectorId,
	replayError,
}: RunInspectorProps) {
	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<div className="flex flex-wrap items-center gap-3 border-b px-4 py-3 text-sm">
				<StatusBadge tone={RUN_STATUS_TONE[run.status] ?? 'neutral'}>
					{RUN_STATUS_LABEL[run.status] ?? run.status}
				</StatusBadge>
				<span className="text-muted-foreground">{runTriggerLabel(run)}</span>
				<span className="text-muted-foreground">
					Durée : {formatRunDuration(runDurationMs(run))}
				</span>
				{run.error ? (
					<span className="text-destructive">{run.error}</span>
				) : null}
			</div>

			<div className="flex min-h-0 flex-1">
				{graph ? (
					<RunGraphCanvas
						graph={graph}
						layout={layout}
						descriptors={descriptors}
						connectorStatuses={connectorStatuses}
					/>
				) : (
					<div className="flex flex-1 items-center justify-center p-8 text-center text-sm text-muted-foreground">
						La version de ce run a été supprimée depuis — son graphe n’est plus
						disponible, mais ses étapes restent consultables.
					</div>
				)}

				<div className="flex w-[420px] shrink-0 flex-col gap-3 overflow-y-auto border-l p-4">
					<RunStepList
						stepTree={stepTree}
						runStatus={run.status}
						onReplay={onReplay}
						isReplaying={isReplaying}
						replayingConnectorId={replayingConnectorId}
						replayError={replayError}
					/>
				</div>
			</div>
		</div>
	)
}
