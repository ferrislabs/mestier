import { createContext, useContext } from 'react'

export type EditorActions = {
	openNode: (nodeId: string) => void
	chooseEvent: (nodeId: string) => void
	addAfter: (nodeId: string, outputId: string | null) => void
}

const IGNORE_ACTION = () => undefined

export const EditorActionsContext = createContext<EditorActions>({
	openNode: IGNORE_ACTION,
	chooseEvent: IGNORE_ACTION,
	addAfter: IGNORE_ACTION,
})

export function useEditorActions(): EditorActions {
	return useContext(EditorActionsContext)
}
