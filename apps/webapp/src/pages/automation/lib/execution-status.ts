export type ExecutionStatus =
	| 'success'
	| 'error'
	| 'running'
	| 'waiting'
	| 'skipped'
	| 'cancelled'

const FROM_RUN_STATUS: Record<string, ExecutionStatus> = {
	succeeded: 'success',
	failed: 'error',
	running: 'running',
	pending: 'waiting',
	cancelled: 'cancelled',
}

export function toExecutionStatus(runStatus: string): ExecutionStatus {
	return FROM_RUN_STATUS[runStatus] ?? 'waiting'
}

export function executionFromRunStatus(
	runStatus: string | null,
): ExecutionStatus | null {
	if (runStatus === 'succeeded') return 'success'
	if (runStatus === 'failed') return 'error'
	if (runStatus === 'running') return 'running'
	return null
}
