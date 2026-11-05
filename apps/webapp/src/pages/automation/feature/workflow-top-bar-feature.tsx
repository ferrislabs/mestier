import { useNavigate } from '@tanstack/react-router'
import { createContext, useContext } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useWorkflow } from '#/hooks/use-automation'
import { useHasPermission } from '#/hooks/use-permissions'
import { buildOrgPath } from '#/modules/org-path'
import { useWorkflowTags } from '#/pages/automation/feature/use-workflow-folders'
import type { GraphChange } from '#/pages/automation/feature/use-workflow-graph'
import {
	workflowExecutionsLocation,
	workflowPath,
} from '#/pages/automation/lib/automation-paths'
import type { GraphValidation } from '#/pages/automation/lib/validation'
import {
	EditorTopBar,
	type EditorView,
} from '#/pages/automation/ui/editor-top-bar'

export interface EditorSession {
	workflowId: string | null
	name: string
	enabled: boolean
	dirty: boolean
	isSaving: boolean
	saveError: string | null
	validation: GraphValidation
	snapshot: () => GraphChange
	onNameChange: (name: string) => void
	onNameBlur: () => void
	onEnabledChange: (enabled: boolean) => void
	onGraphChange: (change: GraphChange) => void
	onSave: () => void
}

export const EditorSessionContext = createContext<EditorSession | null>(null)

export function useEditorSession(): EditorSession | null {
	return useContext(EditorSessionContext)
}

export interface WorkflowTopBarFeatureProps {
	workflowId: string
	view: EditorView
}

function useViewSwitch(workflowId: string, view: EditorView) {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()

	return (next: EditorView) => {
		if (next === view) return
		const location =
			next === 'editor'
				? { to: workflowPath(workflowId), search: {} }
				: workflowExecutionsLocation(workflowId)
		void navigate({
			to: buildOrgPath(activeOrganization.slug, location.to),
			search: location.search,
		})
	}
}

function SessionTopBar({
	session,
	workflowId,
	view,
}: WorkflowTopBarFeatureProps & { session: EditorSession }) {
	const { activeOrganization } = useActiveOrganization()
	const tags = useWorkflowTags(session.workflowId)
	const canManage = useHasPermission('MANAGE_AUTOMATION')
	const changeView = useViewSwitch(workflowId, view)

	return (
		<EditorTopBar
			organizationSlug={activeOrganization.slug}
			name={session.name}
			canRename={canManage}
			onNameChange={session.onNameChange}
			onNameBlur={session.onNameBlur}
			tags={tags}
			view={view}
			onViewChange={changeView}
			enabled={session.enabled}
			onEnabledChange={session.onEnabledChange}
			save={{
				dirty: session.dirty,
				isSaving: session.isSaving,
				onSave: session.onSave,
			}}
		/>
	)
}

function StandaloneTopBar({ workflowId, view }: WorkflowTopBarFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const workflowQuery = useWorkflow(activeOrganization.id, workflowId)
	const tags = useWorkflowTags(workflowId)
	const changeView = useViewSwitch(workflowId, view)
	const workflow = workflowQuery.data?.data

	return (
		<EditorTopBar
			organizationSlug={activeOrganization.slug}
			name={workflow?.name ?? ''}
			canRename={false}
			onNameChange={() => undefined}
			onNameBlur={() => undefined}
			tags={tags}
			view={view}
			onViewChange={changeView}
			enabled={workflow?.enabled ?? false}
			save={{
				dirty: false,
				isSaving: false,
				disabled: true,
				onSave: () => undefined,
			}}
		/>
	)
}

export function WorkflowTopBarFeature({
	workflowId,
	view,
}: WorkflowTopBarFeatureProps) {
	const session = useEditorSession()

	return session ? (
		<SessionTopBar session={session} workflowId={workflowId} view={view} />
	) : (
		<StandaloneTopBar workflowId={workflowId} view={view} />
	)
}
