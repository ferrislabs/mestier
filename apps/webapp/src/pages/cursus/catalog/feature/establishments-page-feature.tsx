import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { useCatalogFilters } from '#/pages/cursus/catalog/feature/use-catalog-filters'
import {
	useArchiveEstablishment,
	useCreateEstablishment,
	useEstablishments,
	useUpdateEstablishment,
} from '#/pages/cursus/catalog/feature/use-cursus-catalog'
import {
	type EditorState,
	toEstablishmentRequest,
} from '#/pages/cursus/catalog/lib/catalog'
import { EstablishmentsPage } from '#/pages/cursus/catalog/ui/establishments-page'
import { countOf } from '#/pages/cursus/nav/lib/count-of'

export function EstablishmentsPageFeature() {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [editor, setEditor] = useState<EditorState>(null)
	const { filters, toolbar } = useCatalogFilters()
	const list = useEstablishments(activeOrganizationId, filters)
	const create = useCreateEstablishment()
	const update = useUpdateEstablishment()
	const archive = useArchiveEstablishment()
	const close = () => setEditor(null)

	return (
		<EstablishmentsPage
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
