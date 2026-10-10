import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useDoes } from '#/hooks/use-doe'
import { mutationErrorMessage } from '#/lib/api-error'
import { NewDoeDialogFeature } from '#/pages/doe/feature/new-doe-dialog-feature'
import { DoesList } from '#/pages/does/ui/does-list'

export function DoesFeature() {
	const { activeOrganization } = useActiveOrganization()
	const does = useDoes(activeOrganization.id)
	const [dialogOpen, setDialogOpen] = useState(false)

	return (
		<>
			<DoesList
				organizationName={activeOrganization.name}
				organizationSlug={activeOrganization.slug}
				does={does.data?.data ?? []}
				isLoading={does.isLoading}
				error={mutationErrorMessage(does.error)}
				onNewDoe={() => setDialogOpen(true)}
			/>
			<NewDoeDialogFeature open={dialogOpen} onOpenChange={setDialogOpen} />
		</>
	)
}
