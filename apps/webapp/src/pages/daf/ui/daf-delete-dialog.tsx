import { Loader2, Trash2 } from 'lucide-react'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'

export function DafDeleteDialog({
	open,
	label,
	isPending,
	error,
	onOpenChange,
	onConfirm,
}: {
	open: boolean
	label: string
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onConfirm: () => void
}) {
	return (
		<AlertDialog open={open} onOpenChange={onOpenChange}>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Supprimer cette DAF ?</AlertDialogTitle>
					<AlertDialogDescription>
						{`La ${label} sera supprimée. `}
						La suppression est définitive.
					</AlertDialogDescription>
				</AlertDialogHeader>
				{error ? (
					<p role="alert" className="text-sm text-destructive">
						{error}
					</p>
				) : null}
				<AlertDialogFooter>
					<AlertDialogCancel disabled={isPending}>Annuler</AlertDialogCancel>
					<AlertDialogAction
						disabled={isPending}
						className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
						onClick={(event) => {
							event.preventDefault()
							onConfirm()
						}}
					>
						{isPending ? <Loader2 className="animate-spin" /> : <Trash2 />}
						Supprimer définitivement
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
