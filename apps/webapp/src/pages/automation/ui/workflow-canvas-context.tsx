import { createContext, useContext } from 'react'
import type { Schemas } from '#/api/api.client'

export interface WorkflowCanvasActions {
	catalogue: Schemas.ConnectorDescriptorResponse[]
	onAddNode: (
		sourceId: string,
		branch: Schemas.BranchDto | null,
		descriptor: Schemas.ConnectorDescriptorResponse,
	) => void
	onRequestDelete: (connectorId: string) => void
}

export const WorkflowCanvasActionsContext =
	createContext<WorkflowCanvasActions | null>(null)

export function useWorkflowCanvasActions(): WorkflowCanvasActions {
	const actions = useContext(WorkflowCanvasActionsContext)
	if (!actions)
		throw new Error('WorkflowCanvasActionsContext missing a provider')
	return actions
}
