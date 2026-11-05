import { useNavigate } from '@tanstack/react-router'
import { useCallback, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useCreateDoe, useDoes } from '#/hooks/use-doe'
import {
	type DoeModel,
	useArchiveDoeModel,
	useDoeModels,
	useDoeSettings,
	useRestoreDoeModel,
	useUpdateDoeModel,
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
	buildModelRequest,
	modelToDraft,
	sameModelDraft,
} from '#/pages/doe-models/lib/model-form'
import { themeCounts, themesLabel } from '#/pages/doe-models/lib/models-filter'
import type { ModelDraft } from '#/pages/doe-models/types'
import { CreateDoeDialog } from '#/pages/doe-models/ui/create-doe-dialog'
import {
	DoeModelNotFound,
	DoeModelPage,
} from '#/pages/doe-models/ui/doe-model-page'
import { DoeStyleEditorFeature } from '#/pages/doe-style/feature/doe-style-editor-feature'
import { completeStyle } from '#/pages/doe-style/lib/style'
import { useSyncedDraft } from '#/pages/projects/lib/use-synced-draft'
import { PageLoading } from '#/pages/projects/ui/page-loading'
import { UnsavedGuard } from '#/pages/projects/ui/unsaved-guard'

export function DoeModelPageFeature({ modelId }: { modelId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const models = useDoeModels(activeOrganization.id, true)

	if (models.isLoading) return <PageLoading label="Chargement du modèle…" />
	const model = models.data?.data.find((entry) => entry.id === modelId)
	if (!model)
		return <DoeModelNotFound organizationSlug={activeOrganization.slug} />

	return (
		<DoeModelWorkspace
			key={model.id}
			model={model}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function DoeModelWorkspace({
	model,
	organizationId,
	organizationSlug,
}: {
	model: DoeModel
	organizationId: string
	organizationSlug: string
}) {
	const navigate = useNavigate()
	const stored = useMemo(
		() => modelToDraft(model, (index) => `section-${index}`),
		[model],
	)
	const [draft, setDraft] = useSyncedDraft(stored, sameModelDraft)
	const [dirty, setDirty] = useState(false)
	const [creatingDoe, setCreatingDoe] = useState(false)
	const [notice, setNotice] = useState('')
	const onDirtyChange = useCallback((value: boolean) => setDirty(value), [])
	const archived = Boolean(model.archived_at)

	const updateModel = useUpdateDoeModel()
	const archiveModel = useArchiveDoeModel()
	const restoreModel = useRestoreDoeModel()
	const settings = useDoeSettings(organizationId)
	const createDoe = useCreateDoe()
	const projects = useProjects(organizationId, { includeArchived: false })
	const does = useDoes(organizationId)
	const customers = useCustomers(organizationId)

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

	const path = { organization_id: organizationId, doe_model_id: model.id }

	return (
		<>
			<DoeModelPage
				organizationSlug={organizationSlug}
				name={model.name}
				themesSummary={themesLabel(themeCounts(model))}
				archived={archived}
				draft={draft}
				stored={stored}
				notice={notice}
				isSaving={updateModel.isPending}
				error={
					mutationErrorMessage(updateModel.error) ??
					mutationErrorMessage(archiveModel.error) ??
					mutationErrorMessage(restoreModel.error)
				}
				styleEditor={
					draft.style ? (
						<DoeStyleEditorFeature
							organizationId={organizationId}
							value={draft.style}
							onChange={(patch) =>
								setDraft((current) =>
									current.style
										? { ...current, style: { ...current.style, ...patch } }
										: current,
								)
							}
						/>
					) : null
				}
				onChange={(patch: Partial<ModelDraft>) =>
					setDraft((current) => ({ ...current, ...patch }))
				}
				onStyleToggle={(enabled) =>
					setDraft((current) => ({
						...current,
						style: enabled ? completeStyle(settings.data?.data?.style) : null,
					}))
				}
				onSubmit={async () => {
					await updateModel.mutateAsync({
						path,
						body: buildModelRequest(draft),
					})
				}}
				onToggleArchive={() => {
					const run = archived
						? restoreModel.mutateAsync({ path })
						: archiveModel.mutateAsync({ path })
					void run
						.then(() =>
							setNotice(
								archived
									? 'Modèle restauré.'
									: 'Modèle archivé. Les DOE déjà créés ne changent pas.',
							),
						)
						.catch(() => {})
				}}
				onCreateDoe={() => setCreatingDoe(true)}
				onDirtyChange={onDirtyChange}
			/>
			<CreateDoeDialog
				organizationSlug={organizationSlug}
				modelName={model.name}
				open={creatingDoe && !archived}
				targets={targets.eligible.map((project) => ({
					id: project.id,
					label: projectOptionLabel(project),
				}))}
				noTargetMessage={noTargetMessage(targets.all)}
				isPending={createDoe.isPending}
				error={mutationErrorMessage(createDoe.error)}
				onOpenChange={(open) => {
					if (!open) setCreatingDoe(false)
				}}
				onCreate={(projectId) => {
					void createDoe
						.mutateAsync({
							path: { organization_id: organizationId, project_id: projectId },
							body: { model_id: model.id },
						})
						.then(() => {
							setCreatingDoe(false)
							return navigate({
								to: buildOrgPath(organizationSlug, '/projet/$projectId/doe'),
								params: { projectId },
							})
						})
						.catch(() => {})
				}}
			/>
			<UnsavedGuard dirty={dirty} />
		</>
	)
}
