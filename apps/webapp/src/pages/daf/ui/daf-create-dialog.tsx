import { Loader2, PackageSearch } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

export function DafCreateDialog({
	open,
	designation,
	isPending,
	error,
	onDesignationChange,
	onPickProduct,
	onSubmit,
	onOpenChange,
}: {
	open: boolean
	designation: string
	isPending: boolean
	error: string | null
	onDesignationChange: (designation: string) => void
	onPickProduct: () => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}) {
	const baseId = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<DialogHeader>
					<DialogTitle>Nouvelle DAF</DialogTitle>
					<DialogDescription>
						Saisis une désignation, ou pars d’un produit du catalogue.
					</DialogDescription>
				</DialogHeader>
				<form
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						onSubmit()
					}}
				>
					<div className="grid gap-1.5">
						<Label htmlFor={`${baseId}-designation`}>Désignation</Label>
						<Input
							id={`${baseId}-designation`}
							value={designation}
							maxLength={200}
							autoComplete="off"
							onChange={(event) => onDesignationChange(event.target.value)}
						/>
					</div>
					{error ? (
						<p role="alert" className="text-[13px] text-destructive">
							{error}
						</p>
					) : null}
					<DialogFooter className="gap-2 sm:justify-between">
						<Button
							type="button"
							variant="outline"
							disabled={isPending}
							onClick={onPickProduct}
						>
							<PackageSearch aria-hidden />
							Choisir un produit
						</Button>
						<Button
							type="submit"
							disabled={isPending || designation.trim() === ''}
						>
							{isPending ? <Loader2 className="animate-spin" /> : null}
							Créer la DAF
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
