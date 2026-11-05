// MOCK: workflow placements — no backend endpoint yet. Replace with `folder_id` and `tags` on the workflow response.
export interface WorkflowPlacement {
	folderId: string | null
	tags: readonly string[]
}

export const MOCK_WORKFLOW_PLACEMENTS: Readonly<
	Record<string, WorkflowPlacement>
> = {}

export const NO_PLACEMENT: WorkflowPlacement = { folderId: null, tags: [] }
