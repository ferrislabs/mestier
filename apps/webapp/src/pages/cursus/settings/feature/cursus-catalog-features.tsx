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
} from '#/pages/cursus/settings/feature/use-cursus-catalog'
import {
	type EditorState,
	toClosureReasonRequest,
	toEstablishmentRequest,
	toTrainingRequest,
} from '#/pages/cursus/settings/lib/catalog'
import {
	ClosureReasonsBlock,
	EstablishmentsBlock,
	TrainingsBlock,
} from '#/pages/cursus/settings/ui/catalog-blocks'

export function EstablishmentsFeature() {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const list = useEstablishments(activeOrganizationId)
	const create = useCreateEstablishment()
	const update = useUpdateEstablishment()
	const archive = useArchiveEstablishment()
	const close = () => setEditor(null)

	return (
		<EstablishmentsBlock
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

export function TrainingsFeature() {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const list = useTrainings(activeOrganizationId)
	const create = useCreateTraining()
	const update = useUpdateTraining()
	const archive = useArchiveTraining()
	const close = () => setEditor(null)

	return (
		<TrainingsBlock
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
