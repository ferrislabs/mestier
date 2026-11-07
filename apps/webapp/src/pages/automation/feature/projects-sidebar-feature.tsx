import { useParams } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useAutomationWorkflows } from '#/hooks/use-automation'
import { useWorkflowFolders } from '#/pages/automation/feature/use-workflow-folders'
import {
	childFolders,
	countWorkflowsInSubtree,
	folderPath,
} from '#/pages/automation/lib/folders'
import { ProjectsSidebar } from '#/pages/automation/ui/projects-sidebar'

export function ProjectsSidebarFeature() {
	const { activeOrganization } = useActiveOrganization()
	const workflows = useAutomationWorkflows(activeOrganization.id)
	const folders = useWorkflowFolders(activeOrganization.id)
	const projectId = useParams({
		strict: false,
		select: (params) => params.projectId,
	})

	const foldered = (workflows.data?.data ?? []).map((workflow) => ({
		folderId: workflow.folder_id ?? null,
	}))
	const activeRootId = projectId
		? folderPath(folders.data, projectId)[0]?.id
		: undefined

	return (
		<ProjectsSidebar
			organizationSlug={activeOrganization.slug}
			entries={childFolders(folders.data, null).map((folder) => ({
				folder,
				workflowCount: countWorkflowsInSubtree(
					folders.data,
					foldered,
					folder.id,
				),
				active: folder.id === activeRootId,
			}))}
		/>
	)
}
