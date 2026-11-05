import { useParams } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useAutomationWorkflows } from '#/hooks/use-automation'
import {
	useArchivedWorkflowIds,
	useWorkflowFolders,
	useWorkflowPlacementActions,
	useWorkflowPlacements,
} from '#/pages/automation/feature/use-workflow-folders'
import {
	childFolders,
	countWorkflowsInSubtree,
	folderPath,
} from '#/pages/automation/lib/folders'
import { ProjectsSidebar } from '#/pages/automation/ui/projects-sidebar'

export function ProjectsSidebarFeature() {
	const { activeOrganization } = useActiveOrganization()
	const workflows = useAutomationWorkflows(activeOrganization.id)
	const folders = useWorkflowFolders()
	const placements = useWorkflowPlacements()
	const archived = useArchivedWorkflowIds()
	const { rememberNames } = useWorkflowPlacementActions()
	const items = workflows.data?.data
	useEffect(() => {
		if (items) rememberNames(items)
	}, [items, rememberNames])
	const projectId = useParams({
		strict: false,
		select: (params) => params.projectId,
	})

	const foldered = (items ?? [])
		.filter((workflow) => !archived.has(workflow.id))
		.map((workflow) => ({
			folderId: placements.data[workflow.id]?.folderId ?? null,
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
