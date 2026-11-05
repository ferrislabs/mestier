import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useCreateDoe, useDoes } from '#/hooks/use-doe'
import {
	type DoeModel,
	useArchiveDoeModel,
	useCreateDoeModel,
	useCreateDoeModelFromStarter,
	useDoeModelStarters,
	useDoeModels,
	useRestoreDoeModel,
} from '#/hooks/use-doe-models'
import { useProjects } from '#/hooks/use-projects'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import {
	eligibleProjects,
	noTargetMessage,
	projectOptionLabel,
	type TargetProject,
} from '#/pages/doe-models/lib/create-doe-targets'
import {
	filterModels,
	type ModelStatus,
	modelCounts,
} from '#/pages/doe-models/lib/models-filter'
import { CreateDoeDialog } from '#/pages/doe-models/ui/create-doe-dialog'
import { DoeModelStartersDialog } from '#/pages/doe-models/ui/doe-model-starters-dialog'
import { DoeModelsPage } from '#/pages/doe-models/ui/doe-models-page'
import { NewModelDialog } from '#/pages/doe-models/ui/new-model-dialog'

interface DoeModelsPageFeatureProps {
	embedded?: boolean
	status: ModelStatus
	onStatusChange: (status: ModelStatus) => void
}

export function DoeModelsPageFeature(props: DoeModelsPageFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<DoeModelsPageWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function DoeModelsPageWorkspace({
	organizationId,
	organizationSlug,
	embedded,
	status,
	onStatusChange,
}: DoeModelsPageFeatureProps & {
	organizationId: string
	organizationSlug: string
}) {
	const navigate = useNavigate()
	const [creating, setCreating] = useState(false)
	const [startersOpen, setStartersOpen] = useState(false)
	const [creatingDoeFor, setCreatingDoeFor] = useState<DoeModel | null>(null)
	const [notice, setNotice] = useState('')

	const models = useDoeModels(organizationId, true)
	const starters = useDoeModelStarters(organizationId, startersOpen)
	const createModel = useCreateDoeModel()
	const archiveModel = useArchiveDoeModel()
	const restoreModel = useRestoreDoeModel()
	const fromStarter = useCreateDoeModelFromStarter()
	const createDoe = useCreateDoe()
	const projects = useProjects(organizationId, { includeArchived: false })
	const does = useDoes(organizationId)
	const customers = useCustomers(organizationId)

	const rows: DoeModel[] = models.data?.data ?? []
	const listed = filterModels(rows, status)

	const targets = useMemo(() => {
		const names = new Map(
			(customers.data?.data ?? []).map((customer) => [
				customer.id,
				customer.name,
			]),
		)
		const all = (projects.data?.data ?? []).map(
			(project): TargetProject => ({
				id: project.id,
				name: project.name,
				customerName: project.customer_id
					? (names.get(project.customer_id) ?? 'Client')
					: null,
				archived: Boolean(project.archived_at),
			}),
		)
		const withDoe = new Set(
			(does.data?.data ?? []).map((doe) => doe.project_id),
		)
		return { all, eligible: eligibleProjects(all, withDoe) }
	}, [projects.data, does.data, customers.data])

	const pathFor = (model: DoeModel) => ({
		organization_id: organizationId,
		doe_model_id: model.id,
	})

	const openModel = (modelId: string) =>
		navigate({
			to: buildOrgPath(organizationSlug, '/projet/doe-models/$modelId'),
			params: { modelId },
		})

	return (
		<>
			<DoeModelsPage
				embedded={embedded}
				organizationSlug={organizationSlug}
				status={status}
				counts={modelCounts(rows)}
				models={listed}
				notice={notice}
				isLoading={models.isLoading}
				error={
					mutationErrorMessage(models.error) ??
					mutationErrorMessage(archiveModel.error) ??
					mutationErrorMessage(restoreModel.error)
				}
				onStatusChange={onStatusChange}
				onNew={() => setCreating(true)}
				onStarters={() => setStartersOpen(true)}
				onArchive={(model) => {
					void archiveModel
						.mutateAsync({ path: pathFor(model) })
						.then(() =>
							setNotice(
								`Modèle « ${model.name} » archivé. Les DOE déjà créés ne changent pas.`,
							),
						)
						.catch(() => {})
				}}
				onRestore={(model) => {
					void restoreModel
						.mutateAsync({ path: pathFor(model) })
						.then(() => setNotice(`Modèle « ${model.name} » restauré.`))
						.catch(() => {})
				}}
				onCreateDoe={setCreatingDoeFor}
			/>

			<NewModelDialog
				open={creating}
				isPending={createModel.isPending}
				error={mutationErrorMessage(createModel.error)}
				onOpenChange={setCreating}
				onSubmit={({ name, description }) => {
					void createModel
						.mutateAsync({
							path: { organization_id: organizationId },
							body: {
								name: name.trim(),
								description: description.trim() || null,
								sections: [],
								style: null,
							},
						})
						.then((created) => {
							setCreating(false)
							return openModel(created.data.id)
						})
						.catch(() => {})
				}}
			/>

			<DoeModelStartersDialog
				open={startersOpen}
				starters={starters.data?.data ?? []}
				isLoading={starters.isLoading}
				isPending={fromStarter.isPending}
				error={
					mutationErrorMessage(starters.error) ??
					mutationErrorMessage(fromStarter.error)
				}
				onOpenChange={setStartersOpen}
				onCreate={(starter) => {
					void fromStarter
						.mutateAsync({
							path: { organization_id: organizationId },
							body: { starter },
						})
						.then((created) => {
							setStartersOpen(false)
							return openModel(created.data.id)
						})
						.catch(() => {})
				}}
			/>

			<CreateDoeDialog
				organizationSlug={organizationSlug}
				modelName={creatingDoeFor?.name ?? ''}
				open={creatingDoeFor !== null}
				targets={targets.eligible.map((project) => ({
					id: project.id,
					label: projectOptionLabel(project),
				}))}
				noTargetMessage={noTargetMessage(targets.all)}
				isPending={createDoe.isPending}
				error={mutationErrorMessage(createDoe.error)}
				onOpenChange={(open) => {
					if (!open) setCreatingDoeFor(null)
				}}
				onCreate={(projectId) => {
					if (!creatingDoeFor) return
					void createDoe
						.mutateAsync({
							path: { organization_id: organizationId, project_id: projectId },
							body: { model_id: creatingDoeFor.id },
						})
						.then(() => {
							setCreatingDoeFor(null)
							return navigate({
								to: buildOrgPath(organizationSlug, '/projet/$projectId/doe'),
								params: { projectId },
							})
						})
						.catch(() => {})
				}}
			/>
		</>
	)
}
