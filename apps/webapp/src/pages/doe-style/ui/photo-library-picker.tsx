import { Loader2, Search } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import type { LibraryPhoto } from '#/hooks/use-photo-library'
import { PhotoGrid } from '#/pages/photo-library/ui/photo-grid'

interface PhotoLibraryPickerProps {
	open: boolean
	photos: readonly LibraryPhoto[]
	thumbnailUrls: Record<string, string | undefined>
	search: string
	loading: boolean
	error: string | null
	pendingId: string | null
	onSearchChange: (value: string) => void
	onPick: (photo: LibraryPhoto) => void
	onClose: () => void
}

export function PhotoLibraryPicker({
	open,
	photos,
	thumbnailUrls,
	search,
	loading,
	error,
	pendingId,
	onSearchChange,
	onPick,
	onClose,
}: PhotoLibraryPickerProps) {
	return (
		<Dialog open={open} onOpenChange={(next) => (next ? undefined : onClose())}>
			<DialogContent className="max-h-[85dvh] grid-rows-[auto_auto_minmax(0,1fr)] sm:max-w-3xl">
				<div className="grid gap-1">
					<DialogTitle>Photothèque</DialogTitle>
					<DialogDescription>
						Cliquez sur une photo pour l'ajouter. Elle est copiée : la supprimer
						ensuite de la photothèque ne change pas le style.
					</DialogDescription>
				</div>
				<div className="relative">
					<Search
						aria-hidden
						className="pointer-events-none absolute top-2.5 left-2.5 size-4 text-muted-foreground"
					/>
					<Input
						type="search"
						aria-label="Rechercher une photo"
						placeholder="Rechercher par légende ou étiquette"
						className="pl-9"
						value={search}
						onChange={(event) => onSearchChange(event.target.value)}
					/>
				</div>
				<div className="min-h-0 overflow-y-auto" aria-busy={loading}>
					{error ? (
						<p role="alert" className="text-sm text-destructive">
							{error}
						</p>
					) : null}
					{loading && photos.length === 0 ? (
						<Loader2
							aria-label="Chargement de la photothèque"
							className="mx-auto my-8 size-5 animate-spin text-muted-foreground"
						/>
					) : null}
					{!loading && !error && photos.length === 0 ? (
						<p className="py-8 text-center text-sm text-muted-foreground">
							Aucune photo.
						</p>
					) : null}
					<div
						className={pendingId ? 'pointer-events-none opacity-70' : undefined}
					>
						<PhotoGrid
							photos={photos}
							thumbnailUrls={thumbnailUrls}
							selectedIds={pendingId ? new Set([pendingId]) : undefined}
							onSelect={onPick}
						/>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}
