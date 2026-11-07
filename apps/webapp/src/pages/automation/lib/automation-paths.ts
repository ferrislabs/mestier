const BASE = '/automatisation'

export const NEW_WORKFLOW_ID = 'nouveau'

export type WorkflowExecutionsSearch = {
	vue: 'executions'
	execution?: number
}

const EXECUTION_DIGITS = /^\d+$/

export function parseExecutionNumber(value: unknown): number | undefined {
	const number =
		typeof value === 'string' && EXECUTION_DIGITS.test(value)
			? Number(value)
			: value
	return typeof number === 'number' &&
		Number.isSafeInteger(number) &&
		number > 0
		? number
		: undefined
}

export type WorkflowExecutionsLocation = {
	to: string
	search: WorkflowExecutionsSearch
}

export function automationBasePath(): string {
	return BASE
}

export function workflowsPath(): string {
	return `${BASE}/workflows`
}

export function workflowPath(workflowId: string): string {
	return `${workflowsPath()}/${workflowId}`
}

export function newWorkflowPath(): string {
	return workflowPath(NEW_WORKFLOW_ID)
}

export function workflowExecutionsSearch(
	executionNumber?: number,
): WorkflowExecutionsSearch {
	const execution = parseExecutionNumber(executionNumber)
	return execution === undefined
		? { vue: 'executions' }
		: { vue: 'executions', execution }
}

export function workflowExecutionsLocation(
	workflowId: string,
	executionNumber?: number,
): WorkflowExecutionsLocation {
	return {
		to: workflowPath(workflowId),
		search: workflowExecutionsSearch(executionNumber),
	}
}

export function legacyExecutionLocation(
	workflowId: string,
	runId: string,
): WorkflowExecutionsLocation {
	return workflowExecutionsLocation(workflowId, parseExecutionNumber(runId))
}

export function executionsPath(): string {
	return `${BASE}/executions`
}

export function collectionsPath(): string {
	return `${BASE}/collections`
}

export function collectionPath(collectionId: string): string {
	return `${collectionsPath()}/${collectionId}`
}

export function identificationsPath(): string {
	return `${BASE}/identifications`
}

export function projectPath(projectId: string): string {
	return `${BASE}/projets/${projectId}`
}

export type WorkflowSearch = {
	vue?: 'executions'
	execution?: number
}

export function parseWorkflowSearch(
	raw: Record<string, unknown>,
): WorkflowSearch {
	const execution = parseExecutionNumber(raw.execution)
	return {
		...(raw.vue === 'executions' ? { vue: 'executions' as const } : {}),
		...(execution === undefined ? {} : { execution }),
	}
}
