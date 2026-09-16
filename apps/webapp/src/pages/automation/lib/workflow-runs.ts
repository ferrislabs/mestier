import type { Schemas } from '#/api/api.client'
import type { Run } from '#/hooks/use-automation'

export interface LatestWorkflowRun {
	status: string
	at: string
}

export function latestRunByWorkflow(
	runs: Run[],
): Map<string, LatestWorkflowRun> {
	const latest = new Map<string, LatestWorkflowRun>()

	for (const run of runs) {
		const current = latest.get(run.workflow_id)
		if (!current || run.created_at > current.at) {
			latest.set(run.workflow_id, { status: run.status, at: run.created_at })
		}
	}

	return latest
}

export function latestRunId(runs: Run[], workflowId: string): string | null {
	let latest: Run | null = null

	for (const run of runs) {
		if (run.workflow_id !== workflowId) continue
		if (!latest || run.created_at > latest.created_at) latest = run
	}

	return latest?.id ?? null
}

export function connectorOutputsFromSteps(
	steps: Schemas.RunStepResponse[],
): Record<string, unknown> {
	const chosen = new Map<string, Schemas.RunStepResponse>()

	for (const step of steps) {
		if (step.output === undefined) continue
		const existing = chosen.get(step.connector_id)
		if (!existing || (existing.iteration_path !== '' && step.iteration_path === '')) {
			chosen.set(step.connector_id, step)
		}
	}

	return Object.fromEntries(
		[...chosen].map(([id, step]) => [id, step.output]),
	)
}
