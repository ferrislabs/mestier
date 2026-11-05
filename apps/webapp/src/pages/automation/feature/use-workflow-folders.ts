import { MOCK_WORKFLOW_FOLDERS } from '#/pages/automation/feature/mocks/workflow-folders'
import {
	MOCK_WORKFLOW_PLACEMENTS,
	type WorkflowPlacement,
} from '#/pages/automation/feature/mocks/workflow-placements'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'

export function useWorkflowFolders(): {
	data: readonly WorkflowFolder[]
	isLoading: boolean
} {
	return { data: MOCK_WORKFLOW_FOLDERS, isLoading: false }
}

export function useWorkflowPlacements(): {
	data: Readonly<Record<string, WorkflowPlacement>>
	isLoading: boolean
} {
	return { data: MOCK_WORKFLOW_PLACEMENTS, isLoading: false }
}
