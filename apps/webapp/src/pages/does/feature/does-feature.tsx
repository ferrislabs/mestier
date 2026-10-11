import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type DoeSummary, useDoes } from '#/hooks/use-doe'
import { mutationErrorMessage } from '#/lib/api-error'
import { DeleteDoeFeature } from '#/pages/doe/feature/delete-doe-feature'
import { NewDoeDialogFeature } from '#/pages/doe/feature/new-doe-dialog-feature'
import { DoesList } from '#/pages/does/ui/does-list'

export function DoesFeature() {
	const { activeOrganization } = useActiveOrganization()
	const does = useDoes(activeOrganization.id)
	const [dialogOpen, setDialogOpen] = useState(false)
	const [toDelete, setToDelete] = useState<DoeSummary | null>(null)

	return (
		<>
			<DoesList
				organizationName={activeOrganization.name}
				organizationSlug={activeOrganization.slug}
				does={does.data?.data ?? []}
				isLoading={does.isLoading}
				error={mutationErrorMessage(does.error)}
				onNewDoe={() => setDialogOpen(true)}
				onDelete={setToDelete}
			/>
			<DeleteDoeFeature
				open={toDelete !== null}
				organizationId={activeOrganization.id}
				doeId={toDelete?.id ?? ''}
				title={toDelete?.title ?? ''}
				onOpenChange={(open) => {
					if (!open) setToDelete(null)
				}}
				onDeleted={() => setToDelete(null)}
			/>
			<NewDoeDialogFeature open={dialogOpen} onOpenChange={setDialogOpen} />
		</>
	)
}
