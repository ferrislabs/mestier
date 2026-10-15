import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type DoeModel,
	useArchiveDoeModel,
	useCreateDoeModel,
	useCreateDoeModelFromStarter,
	useDoeModelStarters,
	useDoeModels,
	useDoeSettings,
	useRestoreDoeModel,
	useUpdateDoeModel,
} from '#/hooks/use-doe-models'
import { mutationErrorMessage } from '#/lib/api-error'
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
import { DoeStyleEditorFeature } from '#/pages/doe-style/feature/doe-style-editor-feature'
import { completeStyle } from '#/pages/doe-style/lib/style'

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
			{...props}
		/>
	)
}

function DoeModelsWorkspace({
	organizationId,
	includeArchived,
	onIncludeArchivedChange,
}: DoeModelsFeatureProps & {
	organizationId: string
}) {
	const [draft, setDraft] = useState<ModelDraft | null>(null)
	const [startersOpen, setStartersOpen] = useState(false)

	const models = useDoeModels(organizationId, includeArchived)
	const starters = useDoeModelStarters(organizationId, startersOpen)
	const createModel = useCreateDoeModel()
	const updateModel = useUpdateDoeModel()
	const archiveModel = useArchiveDoeModel()
	const restoreModel = useRestoreDoeModel()
	const fromStarter = useCreateDoeModelFromStarter()
	const settings = useDoeSettings(organizationId)

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
				styleEditor={
					draft?.style ? (
						<DoeStyleEditorFeature
							organizationId={organizationId}
							value={draft.style}
							onChange={(patch) =>
								setDraft((current) =>
									current?.style
										? { ...current, style: { ...current.style, ...patch } }
										: current,
								)
							}
						/>
					) : null
				}
				onStyleToggle={(enabled) =>
					setDraft((current) =>
						current
							? {
									...current,
									style: enabled
										? completeStyle(settings.data?.data?.style)
										: null,
								}
							: current,
					)
				}
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
