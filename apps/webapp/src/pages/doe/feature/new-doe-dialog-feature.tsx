import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCreateDoe, useDoes } from '#/hooks/use-doe'
import {
	useCreateDoeModelFromStarter,
	useDoeModels,
} from '#/hooks/use-doe-models'
import { useProjects } from '#/hooks/use-projects'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { activeModels, chooseCreationMode } from '#/pages/doe/lib/empty-state'
import { projectsWithoutDoe } from '#/pages/doe/lib/new-doe'
import { NewDoeDialog } from '#/pages/doe/ui/new-doe-dialog'

interface NewDoeDialogFeatureProps {
	open: boolean
	presetModelId?: string | null
	onOpenChange: (open: boolean) => void
}

export function NewDoeDialogFeature({
	open,
	presetModelId = null,
	onOpenChange,
}: NewDoeDialogFeatureProps) {
	if (!open) return null
	return (
		<NewDoeDialogContent
			presetModelId={presetModelId}
			onOpenChange={onOpenChange}
		/>
	)
}

function NewDoeDialogContent({
	presetModelId,
	onOpenChange,
}: {
	presetModelId: string | null
	onOpenChange: (open: boolean) => void
}) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const projects = useProjects(organizationId, { includeArchived: false })
	const does = useDoes(organizationId)
	const models = useDoeModels(organizationId, false)
	const createDoe = useCreateDoe()
	const fromStarter = useCreateDoeModelFromStarter()
	const [projectId, setProjectId] = useState('')
	const [chosenModelId, setChosenModelId] = useState(presetModelId ?? '')

	const projectRows = projectsWithoutDoe(
		projects.data?.data ?? [],
		does.data?.data ?? [],
	)
	const modelRows = activeModels(models.data?.data ?? [])
	const selectedModelId = modelRows.some((model) => model.id === chosenModelId)
		? chosenModelId
		: (modelRows[0]?.id ?? '')

	return (
		<NewDoeDialog
			open
			mode={chooseCreationMode(modelRows)}
			projects={projectRows}
			models={modelRows}
			selectedProjectId={projectId}
			selectedModelId={selectedModelId}
			isLoading={projects.isLoading || does.isLoading || models.isLoading}
			isPending={createDoe.isPending || fromStarter.isPending}
			error={
				mutationErrorMessage(createDoe.error) ??
				mutationErrorMessage(fromStarter.error) ??
				mutationErrorMessage(projects.error) ??
				mutationErrorMessage(does.error) ??
				mutationErrorMessage(models.error)
			}
			onOpenChange={onOpenChange}
			onSelectProject={setProjectId}
			onSelectModel={setChosenModelId}
			onCreate={() => {
				void createDoe
					.mutateAsync({
						path: { organization_id: organizationId, project_id: projectId },
						body: { model_id: selectedModelId },
					})
					.then(() => {
						onOpenChange(false)
						return navigate({
							to: buildOrgPath(
								activeOrganization.slug,
								'/planification/projects/$projectId/doe',
							),
							params: { projectId },
						})
					})
					.catch(() => {})
			}}
			onCreateStarter={() => {
				void fromStarter
					.mutateAsync({
						path: { organization_id: organizationId },
						body: { starter: 'landscaping' },
					})
					.then((created) => setChosenModelId(created.data.id))
					.catch(() => {})
			}}
		/>
	)
}
