import { useDeleteDoe } from '#/hooks/use-doe'
import { mutationErrorMessage } from '#/lib/api-error'
import { DeleteDoeDialog } from '#/pages/doe/ui/delete-doe-dialog'

interface DeleteDoeFeatureProps {
	open: boolean
	organizationId: string
	doeId: string
	title: string
	onOpenChange: (open: boolean) => void
	onDeleted: () => void
}

export function DeleteDoeFeature({
	open,
	organizationId,
	doeId,
	title,
	onOpenChange,
	onDeleted,
}: DeleteDoeFeatureProps) {
	const deleteDoe = useDeleteDoe()

	return (
		<DeleteDoeDialog
			open={open}
			title={title}
			isPending={deleteDoe.isPending}
			error={mutationErrorMessage(deleteDoe.error)}
			onOpenChange={(next) => {
				if (!next) deleteDoe.reset()
				onOpenChange(next)
			}}
			onConfirm={() => {
				void deleteDoe
					.mutateAsync({
						path: { organization_id: organizationId, doe_id: doeId },
					})
					.then(() => {
						onOpenChange(false)
						onDeleted()
					})
					.catch(() => {})
			}}
		/>
	)
}
