import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type DoeModel,
	useArchiveDoeModel,
	useCreateDoeModel,
	useCreateDoeModelFromStarter,
	useDoeModelStarters,
	useDoeModels,
	useRestoreDoeModel,
	useUpdateDoeModel,
} from '#/hooks/use-doe-models'
import { mutationErrorMessage } from '#/lib/api-error'
import { NewDoeDialogFeature } from '#/pages/doe/feature/new-doe-dialog-feature'
import {
	buildModelRequest,
	emptyModelDraft,
	modelToDraft,
	newModelSectionDraft,
} from '#/pages/doe-models/lib/model-form'
import type { ModelDraft } from '#/pages/doe-models/types'
import { DoeModelEditorDialog } from '#/pages/doe-models/ui/doe-model-editor-dialog'
import { DoeModelStartersDialog } from '#/pages/doe-models/ui/doe-model-starters-dialog'
import { DoeModelsList } from '#/pages/doe-models/ui/doe-models-list'

interface DoeModelsFeatureProps {
	includeArchived: boolean
	onIncludeArchivedChange: (includeArchived: boolean) => void
}

export function DoeModelsFeature(props: DoeModelsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<DoeModelsWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			{...props}
		/>
	)
}

function DoeModelsWorkspace({
	organizationId,
	organizationName,
	includeArchived,
	onIncludeArchivedChange,
}: DoeModelsFeatureProps & {
	organizationId: string
	organizationName: string
}) {
	const [draft, setDraft] = useState<ModelDraft | null>(null)
	const [startersOpen, setStartersOpen] = useState(false)
	const [doeModelId, setDoeModelId] = useState<string | null>(null)

	const models = useDoeModels(organizationId, includeArchived)
	const starters = useDoeModelStarters(organizationId, startersOpen)
	const createModel = useCreateDoeModel()
	const updateModel = useUpdateDoeModel()
	const archiveModel = useArchiveDoeModel()
	const restoreModel = useRestoreDoeModel()
	const fromStarter = useCreateDoeModelFromStarter()

	const rows: DoeModel[] = models.data?.data ?? []

	const submit = async () => {
		if (!draft) return
		const body = buildModelRequest(draft)
		if (draft.id === null) {
			await createModel.mutateAsync({
				path: { organization_id: organizationId },
				body,
			})
		} else {
			await updateModel.mutateAsync({
				path: { organization_id: organizationId, doe_model_id: draft.id },
				body,
			})
		}
		setDraft(null)
	}

	const pathFor = (model: DoeModel) => ({
		organization_id: organizationId,
		doe_model_id: model.id,
	})

	return (
		<>
			<DoeModelsList
				organizationName={organizationName}
				models={rows}
				includeArchived={includeArchived}
				isLoading={models.isLoading}
				error={
					mutationErrorMessage(models.error) ??
					mutationErrorMessage(archiveModel.error) ??
					mutationErrorMessage(restoreModel.error)
				}
				onIncludeArchivedChange={onIncludeArchivedChange}
				onCreate={() => setDraft(emptyModelDraft())}
				onOpenStarters={() => setStartersOpen(true)}
				onCreateDoe={(model) => setDoeModelId(model.id)}
				onEdit={(model) =>
					setDraft(modelToDraft(model, () => crypto.randomUUID()))
				}
				onArchive={(model) => {
					void archiveModel.mutateAsync({ path: pathFor(model) })
				}}
				onRestore={(model) => {
					void restoreModel.mutateAsync({ path: pathFor(model) })
				}}
			/>

			<DoeModelEditorDialog
				open={draft !== null}
				draft={draft ?? emptyModelDraft()}
				isPending={createModel.isPending || updateModel.isPending}
				error={
					mutationErrorMessage(createModel.error) ??
					mutationErrorMessage(updateModel.error)
				}
				onOpenChange={(open) => {
					if (!open) setDraft(null)
				}}
				onChange={(patch) =>
					setDraft((current) => (current ? { ...current, ...patch } : current))
				}
				onAddSection={(title, kind) =>
					setDraft((current) =>
						current
							? {
									...current,
									sections: [
										...current.sections,
										newModelSectionDraft(
											crypto.randomUUID(),
											title.trim(),
											kind,
										),
									],
								}
							: current,
					)
				}
				onSubmit={() => {
					void submit().catch(() => {})
				}}
			/>

			<NewDoeDialogFeature
				open={doeModelId !== null}
				presetModelId={doeModelId}
				onOpenChange={(open) => {
					if (!open) setDoeModelId(null)
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
						.then(() => setStartersOpen(false))
						.catch(() => {})
				}}
			/>
		</>
	)
}
