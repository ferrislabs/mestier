import { Loader2 } from 'lucide-react'
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
import type { LibraryPhoto } from '#/hooks/use-photo-library'
import { PhotoGrid } from '#/pages/photo-library/ui/photo-grid'

export interface PickerFailure {
	photoId: string
	label: string
	message: string
}

interface DoePhotoLibraryPickerDialogProps {
	open: boolean
	photos: readonly LibraryPhoto[]
	thumbnailUrls: Record<string, string | undefined>
	selectedIds: ReadonlySet<string>
	search: string
	isLoading: boolean
	isPlacing: boolean
	hasPrevious: boolean
	hasNext: boolean
	error: string | null
	failures: readonly PickerFailure[]
	onSearchChange: (search: string) => void
	onToggle: (photo: LibraryPhoto) => void
	onPrevious: () => void
	onNext: () => void
	onConfirm: () => void
	onOpenChange: (open: boolean) => void
}

export function DoePhotoLibraryPickerDialog({
	open,
	photos,
	thumbnailUrls,
	selectedIds,
	search,
	isLoading,
	isPlacing,
	hasPrevious,
	hasNext,
	error,
	failures,
	onSearchChange,
	onToggle,
	onPrevious,
	onNext,
	onConfirm,
	onOpenChange,
}: DoePhotoLibraryPickerDialogProps) {
	const baseId = useId()
	const count = selectedIds.size
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[90dvh] flex-col gap-0 p-0 sm:max-w-3xl">
				<DialogHeader className="border-b px-6 py-4 pr-12">
					<DialogTitle>Choisir dans la photothèque</DialogTitle>
					<DialogDescription>
						Les photos sélectionnées sont copiées dans ce DOE. La photothèque
						n’est pas modifiée.
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-2 px-6 py-4">
					<Label htmlFor={`${baseId}-search`} className="sr-only">
						Rechercher une photo
					</Label>
					<Input
						id={`${baseId}-search`}
						type="search"
						value={search}
						placeholder="Rechercher par légende ou étiquette"
						autoComplete="off"
						onChange={(event) => onSearchChange(event.target.value)}
					/>
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
					{failures.length > 0 ? (
						<div
							role="alert"
							className="grid gap-1 text-[13px] text-state-alert"
						>
							<p className="font-medium">
								Ces photos n’ont pas pu être placées
							</p>
							<ul className="list-disc pl-5">
								{failures.map((failure) => (
									<li key={failure.photoId}>
										{failure.label} : {failure.message}
									</li>
								))}
							</ul>
						</div>
					) : null}
				</div>
				<div className="min-h-0 flex-1 overflow-y-auto border-t px-6 py-4">
					{isLoading ? (
						<div className="flex items-center justify-center gap-2 py-10 text-muted-foreground text-sm">
							<Loader2 className="size-4 animate-spin" aria-hidden />
							Chargement de la photothèque…
						</div>
					) : photos.length === 0 ? (
						<p className="py-10 text-center text-muted-foreground text-sm">
							Aucune photo ne correspond.
						</p>
					) : (
						<PhotoGrid
							photos={photos}
							thumbnailUrls={thumbnailUrls}
							selectedIds={selectedIds}
							onSelect={onToggle}
						/>
					)}
				</div>
				<DialogFooter className="items-center border-t px-6 py-4 sm:justify-between">
					<div className="flex gap-2">
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={!hasPrevious}
							onClick={onPrevious}
						>
							Précédente
						</Button>
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={!hasNext}
							onClick={onNext}
						>
							Suivante
						</Button>
					</div>
					<div className="flex gap-2">
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Fermer
						</Button>
						<Button
							type="button"
							disabled={count === 0 || isPlacing}
							onClick={onConfirm}
						>
							{isPlacing ? <Loader2 className="animate-spin" /> : null}
							{count === 0
								? 'Placer'
								: `Placer ${count} photo${count > 1 ? 's' : ''}`}
						</Button>
					</div>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
