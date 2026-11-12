import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { useCatalogFilters } from '#/pages/cursus/catalog/feature/use-catalog-filters'
import {
	useArchiveTraining,
	useCreateTraining,
	useTrainings,
	useUpdateTraining,
} from '#/pages/cursus/catalog/feature/use-cursus-catalog'
import {
	type EditorState,
	toTrainingRequest,
} from '#/pages/cursus/catalog/lib/catalog'
import { TrainingsPage } from '#/pages/cursus/catalog/ui/trainings-page'
import { countOf } from '#/pages/cursus/nav/lib/count-of'

export function TrainingsPageFeature() {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const { filters, toolbar } = useCatalogFilters()
	const list = useTrainings(activeOrganizationId, filters)
	const create = useCreateTraining()
	const update = useUpdateTraining()
	const archive = useArchiveTraining()
	const close = () => setEditor(null)

	return (
		<TrainingsPage
			{...toolbar}
			items={list.data?.data ?? []}
			total={countOf(list.data)}
			isLoading={list.isLoading}
			error={mutationErrorMessage(list.error ?? archive.error)}
			canManage={canManage}
			editor={editor}
			isSaving={create.isPending || update.isPending}
			saveError={mutationErrorMessage(create.error ?? update.error)}
			onOpenEditor={(id) => setEditor({ id })}
			onCloseEditor={close}
			onSave={(values, id) =>
				id === null
					? create.mutate(
							{
								path: { organization_id: activeOrganizationId },
								body: toTrainingRequest(values),
							},
							{ onSuccess: close },
						)
					: update.mutate(
							{
								path: {
									organization_id: activeOrganizationId,
									training_id: id,
								},
								body: toTrainingRequest(values),
							},
							{ onSuccess: close },
						)
			}
			onArchive={(id) =>
				archive.mutate({
					path: { organization_id: activeOrganizationId, training_id: id },
				})
			}
		/>
	)
}
