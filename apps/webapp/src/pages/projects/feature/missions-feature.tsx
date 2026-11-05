import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useProject } from '#/hooks/use-projects'
import {
	createMission,
	deleteMission,
	moveMission,
	setMissionStatus,
	updateMission,
	useAllMissions,
} from '#/pages/projects/feature/mocks/missions'
import { useProjectExtras } from '#/pages/projects/feature/mocks/project-extras'
import { usePeople } from '#/pages/projects/feature/use-people'
import type { MissionFilter } from '#/pages/projects/lib/mission-list'
import { sortedMissions } from '#/pages/projects/lib/missions'
import { statusOf } from '#/pages/projects/lib/project-list'
import { MissionsPage } from '#/pages/projects/ui/missions-page'
import { PageLoading } from '#/pages/projects/ui/page-loading'
import { ProjectNotFound } from '#/pages/projects/ui/project-not-found'

export interface MissionsFeatureProps {
	projectId: string
	filter: MissionFilter
	onFilterChange: (filter: MissionFilter) => void
}

export function MissionsFeature({
	projectId,
	filter,
	onFilterChange,
}: MissionsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)
	const extras = useProjectExtras(projectId).data
	const people = usePeople(activeOrganization.id)
	const missions = useAllMissions()
	const ordered = useMemo(
		() =>
			sortedMissions(
				missions.filter((mission) => mission.projectId === projectId),
			),
		[missions, projectId],
	)

	if (project.isLoading) return <PageLoading label="Chargement des missions…" />
	if (!project.data?.data)
		return <ProjectNotFound organizationSlug={activeOrganization.slug} />

	return (
		<MissionsPage
			organizationSlug={activeOrganization.slug}
			projectId={projectId}
			projectName={project.data.data.name}
			projectStatus={statusOf(project.data.data)}
			ownerId={extras.ownerId}
			people={people.data}
			ordered={ordered}
			filter={filter}
			onFilterChange={onFilterChange}
			onCreate={(draft) => {
				void createMission(projectId, draft)
			}}
			onUpdate={(mission, draft) => {
				void updateMission(mission.id, draft)
			}}
			onDelete={(mission) => {
				void deleteMission(mission.id)
			}}
			onStatusChange={(mission, status) => {
				void setMissionStatus(mission.id, status)
			}}
			onMove={(mission, delta) => {
				void moveMission(projectId, mission.id, delta)
			}}
		/>
	)
}
