import { useSyncExternalStore } from 'react'
import { MOCK_WORKFLOW_FOLDERS } from '#/pages/automation/feature/mocks/workflow-folders'
import {
	archiveWorkflowMock,
	readArchived,
	readPlacements,
	rememberWorkflowNames,
	setPlacementMock,
	subscribePlacements,
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
	const data = useSyncExternalStore(subscribePlacements, readPlacements)
	return { data, isLoading: false }
}

export function useArchivedWorkflowIds(): ReadonlySet<string> {
	return useSyncExternalStore(subscribePlacements, readArchived)
}

export function useWorkflowPlacementActions() {
	return {
		rememberNames: rememberWorkflowNames,
		place: setPlacementMock,
		archive: archiveWorkflowMock,
	}
}
