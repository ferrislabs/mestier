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
	origin: 'supplied' | 'generated'
	/** Keyed by the chosen auth scheme's field name. Ignored when
	 * `origin === 'generated'` — the backend fabricates the secret itself
	 * and never reads this. */
	data: Record<string, string>
}
