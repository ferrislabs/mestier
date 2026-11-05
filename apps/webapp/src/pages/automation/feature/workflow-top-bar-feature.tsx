import { useNavigate } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useUpdateWorkflow, useWorkflow } from '#/hooks/use-automation'
import { useHasPermission } from '#/hooks/use-permissions'
import { buildOrgPath } from '#/modules/org-path'
import { RunNowFeature } from '#/pages/automation/feature/run-now-feature'
import { useWorkflowPlacements } from '#/pages/automation/feature/use-workflow-folders'
import {
	EditorTopBar,
	type EditorView,
} from '#/pages/automation/ui/editor-top-bar'

export interface WorkflowTopBarFeatureProps {
	workflowId: string
	view: EditorView
	save?: { dirty: boolean; isSaving: boolean; onSave: () => void }
}

export function WorkflowTopBarFeature({
	workflowId,
	view,
	save,
}: WorkflowTopBarFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const workflowQuery = useWorkflow(activeOrganization.id, workflowId)
	const updateWorkflow = useUpdateWorkflow()
	const placements = useWorkflowPlacements()
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_AUTOMATION')

	const workflow = workflowQuery.data?.data

	function patch(body: { name?: string; enabled?: boolean }) {
		void updateWorkflow.mutateAsync({
			path: {
				organization_id: activeOrganization.id,
				workflow_id: workflowId,
			},
			body,
		})
	}

	function changeView(next: EditorView) {
		if (next === view) return
		void navigate({
			to: buildOrgPath(
				activeOrganization.slug,
				next === 'editor'
					? `/automatisation/${workflowId}`
					: `/automatisation/${workflowId}/executions`,
			),
		})
	}

	return (
		<EditorTopBar
			organizationSlug={activeOrganization.slug}
			name={workflow?.name ?? ''}
			canRename={canManage}
			onNameCommit={(name) => patch({ name })}
			tags={placements.data[workflowId]?.tags ?? []}
			view={view}
			onViewChange={changeView}
			enabled={workflow?.enabled ?? false}
			onEnabledChange={(enabled) => patch({ enabled })}
			actions={<RunNowFeature workflowId={workflowId} bare />}
			save={save}
		/>
	)
}
