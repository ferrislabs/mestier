import { useLocation } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useDoes } from '#/hooks/use-doe'
import { useDoeModels } from '#/hooks/use-doe-models'
import { useProjects } from '#/hooks/use-projects'
import { splitOrgPath } from '#/modules/org-path'
import { modelCounts } from '#/pages/doe-models/lib/models-filter'
import { useAllMissions } from '#/pages/projects/feature/mocks/missions'
import { useSidebarGroups } from '#/pages/projet/feature/use-sidebar-groups'
import {
	activeSidebarEntries,
	projectIdOf,
} from '#/pages/projet/lib/project-path'
import { ProjetSidebar } from '#/pages/projet/ui/projet-sidebar'

const COLLAPSED_KEY = 'mestier:projet:sidebar-collapsed'

export function ProjetSidebarFeature() {
	const { activeOrganization } = useActiveOrganization()
	const [isCollapsed, toggleGroup] = useSidebarGroups(COLLAPSED_KEY)
	const pathname = useLocation({ select: (location) => location.pathname })
	const modulePath = splitOrgPath(pathname).path
	const projectId = projectIdOf(modulePath)

	const projects = useProjects(activeOrganization.id, {
		includeArchived: true,
	})
	const models = useDoeModels(activeOrganization.id, true)
	const does = useDoes(activeOrganization.id)
	const missions = useAllMissions()

	const rows = projects.data?.data ?? []
	const selectedProject = projectId
		? rows.find((project) => project.id === projectId)
		: undefined
	const doe = projectId
		? (does.data?.data ?? []).find((item) => item.project_id === projectId)
		: undefined

	return (
		<ProjetSidebar
			organizationSlug={activeOrganization.slug}
			active={activeSidebarEntries(modulePath)}
			projectsCount={rows.filter((project) => !project.archived_at).length}
			modelsCount={modelCounts(models.data?.data ?? []).active}
			selected={
				selectedProject
					? {
							id: selectedProject.id,
							missionCount: missions.filter(
								(mission) => mission.projectId === selectedProject.id,
							).length,
							doeNote: doe ? `indice ${doe.revision}` : 'à créer',
						}
					: null
			}
			isCollapsed={isCollapsed}
			onToggleGroup={toggleGroup}
		/>
	)
}
