const BASE = '/automatisation'

export const NEW_WORKFLOW_ID = 'nouveau'

export type WorkflowExecutionsSearch = {
	vue: 'executions'
	execution?: string
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
	executionId?: string,
): WorkflowExecutionsSearch {
	return executionId
		? { vue: 'executions', execution: executionId }
		: { vue: 'executions' }
}

export function workflowExecutionsLocation(
	workflowId: string,
	executionId?: string,
): WorkflowExecutionsLocation {
	return {
		to: workflowPath(workflowId),
		search: workflowExecutionsSearch(executionId),
	}
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
	execution?: string
}

export function parseWorkflowSearch(
	raw: Record<string, unknown>,
): WorkflowSearch {
	return {
		...(raw.vue === 'executions' ? { vue: 'executions' as const } : {}),
		...(typeof raw.execution === 'string' && raw.execution !== ''
			? { execution: raw.execution }
			: {}),
	}
}
