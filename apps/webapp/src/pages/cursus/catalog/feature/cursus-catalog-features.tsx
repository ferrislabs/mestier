import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useArchiveClosureReason,
	useArchiveEstablishment,
	useArchiveTraining,
	useClosureReasons,
	useCreateClosureReason,
	useCreateEstablishment,
	useCreateTraining,
	useEstablishments,
	useTrainings,
	useUpdateClosureReason,
	useUpdateEstablishment,
	useUpdateTraining,
} from '#/pages/cursus/catalog/feature/use-cursus-catalog'
import {
	type EditorState,
	toClosureReasonRequest,
	toEstablishmentRequest,
	toTrainingRequest,
} from '#/pages/cursus/catalog/lib/catalog'
import {
	ClosureReasonsBlock,
	EstablishmentsBlock,
	TrainingsBlock,
} from '#/pages/cursus/catalog/ui/catalog-blocks'

export interface CatalogFeatureProps {
	pageDescription?: string
}

export function EstablishmentsFeature({
	pageDescription,
}: CatalogFeatureProps) {
	const [archived, setArchived] = useState(false)
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const list = useEstablishments(activeOrganizationId, archived)
	const create = useCreateEstablishment()
	const update = useUpdateEstablishment()
	const archive = useArchiveEstablishment()
	const close = () => setEditor(null)

	return (
		<EstablishmentsBlock
			page={
				pageDescription === undefined
					? undefined
					: {
							description: pageDescription,
							archived,
							onArchivedChange: setArchived,
						}
			}
			items={list.data?.data ?? []}
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
								body: toEstablishmentRequest(values),
							},
							{ onSuccess: close },
						)
					: update.mutate(
							{
								path: {
									organization_id: activeOrganizationId,
									establishment_id: id,
								},
								body: toEstablishmentRequest(values),
							},
							{ onSuccess: close },
						)
			}
			onArchive={(id) =>
				archive.mutate({
					path: {
						organization_id: activeOrganizationId,
						establishment_id: id,
					},
				})
			}
		/>
	)
}

export function TrainingsFeature({ pageDescription }: CatalogFeatureProps) {
	const [archived, setArchived] = useState(false)
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const list = useTrainings(activeOrganizationId, archived)
	const create = useCreateTraining()
	const update = useUpdateTraining()
	const archive = useArchiveTraining()
	const close = () => setEditor(null)

	return (
		<TrainingsBlock
			page={
				pageDescription === undefined
					? undefined
					: {
							description: pageDescription,
							archived,
							onArchivedChange: setArchived,
						}
			}
			items={list.data?.data ?? []}
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

export function ClosureReasonsFeature() {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const list = useClosureReasons(activeOrganizationId)
	const create = useCreateClosureReason()
	const update = useUpdateClosureReason()
	const archive = useArchiveClosureReason()
	const close = () => setEditor(null)

	return (
		<ClosureReasonsBlock
			items={list.data?.data ?? []}
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
								body: toClosureReasonRequest(values),
							},
							{ onSuccess: close },
						)
					: update.mutate(
							{
								path: {
									organization_id: activeOrganizationId,
									closure_reason_id: id,
								},
								body: toClosureReasonRequest(values),
							},
							{ onSuccess: close },
						)
			}
			onArchive={(id) =>
				archive.mutate({
					path: {
						organization_id: activeOrganizationId,
						closure_reason_id: id,
					},
				})
			}
		/>
	)
}
