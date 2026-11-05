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

const FROM_STEP_STATUS: Record<string, ExecutionStatus> = {
	pending: 'waiting',
	in_flight: 'running',
	succeeded: 'success',
	failed: 'error',
	dead: 'error',
	skipped: 'skipped',
}

export function toExecutionStatus(runStatus: string): ExecutionStatus {
	return FROM_RUN_STATUS[runStatus] ?? 'waiting'
}

export function toStepExecutionStatus(stepStatus: string): ExecutionStatus {
	return FROM_STEP_STATUS[stepStatus] ?? 'waiting'
}

export function toAnyExecutionStatus(status: string): ExecutionStatus {
	return FROM_STEP_STATUS[status] ?? FROM_RUN_STATUS[status] ?? 'waiting'
}

export function executionFromRunStatus(
	runStatus: string | null,
): ExecutionStatus | null {
	if (runStatus === 'succeeded') return 'success'
	if (runStatus === 'failed' || runStatus === 'dead') return 'error'
	if (runStatus === 'running' || runStatus === 'in_flight') return 'running'
	return null
}
