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
