export interface WorkflowFormValues {
	name: string
	description: string
}

export const EMPTY_WORKFLOW_FORM: WorkflowFormValues = {
	name: '',
	description: '',
}

export interface CredentialFormValues {
	kind: string
	name: string
	description: string
	origin: 'supplied' | 'generated'
	data: Record<string, string>
}
