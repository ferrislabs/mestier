import { createContext, useContext } from 'react'
import type { ExitHandle } from '#/pages/survey/lib/survey-graph'

export interface SurveyCanvasActions {
	openNode: (nodeId: string) => void
	addAfter: (nodeId: string, handle: ExitHandle) => void
}

const IGNORE = () => undefined

export const SurveyCanvasActionsContext = createContext<SurveyCanvasActions>({
	openNode: IGNORE,
	addAfter: IGNORE,
})

export function useSurveyCanvasActions(): SurveyCanvasActions {
	return useContext(SurveyCanvasActionsContext)
}
