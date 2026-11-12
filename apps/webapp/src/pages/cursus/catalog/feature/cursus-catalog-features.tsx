import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useArchiveClosureReason,
	useClosureReasons,
	useCreateClosureReason,
	useUpdateClosureReason,
} from '#/pages/cursus/catalog/feature/use-cursus-catalog'
import {
	type EditorState,
	toClosureReasonRequest,
} from '#/pages/cursus/catalog/lib/catalog'
import { ClosureReasonsBlock } from '#/pages/cursus/catalog/ui/catalog-blocks'

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
