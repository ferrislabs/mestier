import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useReplayRun, useRun } from '#/hooks/use-automation'
import { groupRunSteps } from '#/pages/automation/lib/run-grouping'
import {
	RunInspectorUI,
	type RunSummary,
	type StepBlock,
	type StepRow,
} from '#/pages/automation/ui/run-inspector-ui'
import { canReplay } from '#/pages/settings/lib/automation'

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
		<RunInspector
			key={`${activeOrganization.id}:${runId}`}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
			runId={runId}
		/>
	)
}

interface RunInspectorProps {
	organizationId: string
	organizationName: string
	organizationSlug: string
	workflowId: string
	runId: string
}

function RunInspector({
	organizationId,
	organizationName,
	organizationSlug,
	workflowId,
	runId,
}: RunInspectorProps) {
	const runQuery = useRun(organizationId, runId)
	const replayRun = useReplayRun()

	const [replayingConnectorId, setReplayingConnectorId] = useState<
		string | null
	>(null)

	const run = runQuery.data?.data ?? null
	const steps = run?.steps ?? []

	const blocks: StepBlock[] = groupRunSteps(steps).map((block) =>
		block.kind === 'step'
			? { kind: 'step', step: toStepRow(block.step) }
			: {
					kind: 'iteration',
					path: block.group.path,
					label: block.group.label,
					steps: block.group.steps.map(toStepRow),
				},
	)

	const runSummary: RunSummary | null = run
		? {
				id: run.id,
				status: run.status,
				startedAt: run.started_at ?? null,
				finishedAt: run.finished_at ?? null,
				error: run.error ?? null,
			}
		: null

	const handleReplay = (step: StepRow) => {
		setReplayingConnectorId(step.connectorId)
		void replayRun
			.mutateAsync({
				path: { organization_id: organizationId, run_id: runId },
				body: { connector_id: step.connectorId },
			})
			.finally(() => setReplayingConnectorId(null))
	}

	return (
		<RunInspectorUI
			organizationName={organizationName}
			organizationSlug={organizationSlug}
			workflowId={workflowId}
			isLoading={runQuery.isLoading}
			error={runQuery.error?.message ?? replayRun.error?.message ?? null}
			run={runSummary}
			blocks={blocks}
			canReplayRun={run !== null && canReplay(run.status)}
			replayingConnectorId={replayingConnectorId}
			onReplay={handleReplay}
		/>
	)
}

function toStepRow(step: {
	id: string
	connector_id: string
	status: string
	attempts: number
	error?: string | null
	input?: unknown
	output?: unknown
	started_at?: string | null
	finished_at?: string | null
}): StepRow {
	return {
		id: step.id,
		connectorId: step.connector_id,
		status: step.status,
		attempts: step.attempts,
		error: step.error ?? null,
		input: step.input,
		output: step.output,
		startedAt: step.started_at ?? null,
		finishedAt: step.finished_at ?? null,
	}
}
