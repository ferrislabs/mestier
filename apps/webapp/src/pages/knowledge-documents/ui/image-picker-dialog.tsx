import { Loader2, Search } from 'lucide-react'
import { useId } from 'react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type { LibraryPhoto } from '#/hooks/use-photo-library'
import type { Choice } from '#/pages/knowledge-documents/ui/choice-dialog'
import { PhotoGrid } from '#/pages/photo-library/ui/photo-grid'

export type ImagePickerTab = 'drive' | 'photo' | 'upload'

interface ImagePickerDialogProps {
	open: boolean
	tab: ImagePickerTab
	query: string
	driveImages: readonly Choice[]
	photos: readonly LibraryPhoto[]
	thumbnailUrls: Record<string, string | undefined>
	isLoading: boolean
	isUploading: boolean
	error: string | null
	folderName: string
	onTabChange: (tab: ImagePickerTab) => void
	onQueryChange: (query: string) => void
	onPickDrive: (articleId: string) => void
	onPickPhoto: (photo: LibraryPhoto) => void
	onUpload: (file: File) => void
	onOpenChange: (open: boolean) => void
}

export function ImagePickerDialog({
	open,
	tab,
	query,
	driveImages,
	photos,
	thumbnailUrls,
	isLoading,
	isUploading,
	error,
	folderName,
	onTabChange,
	onQueryChange,
	onPickDrive,
	onPickPhoto,
	onUpload,
	onOpenChange,
}: ImagePickerDialogProps) {
	const baseId = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-3xl">
				<DialogHeader>
					<DialogTitle>Choisir une image</DialogTitle>
					<DialogDescription>
						Une image du Drive, une photo de la photothèque, ou une nouvelle
						image.
					</DialogDescription>
				</DialogHeader>
				<Tabs
					value={tab}
					onValueChange={(value) => onTabChange(value as ImagePickerTab)}
				>
					<TabsList>
						<TabsTrigger value="drive">Drive</TabsTrigger>
						<TabsTrigger value="photo">Photothèque</TabsTrigger>
						<TabsTrigger value="upload">Nouvelle image</TabsTrigger>
					</TabsList>
					{tab !== 'upload' ? (
						<div className="grid gap-1">
							<Label htmlFor={`${baseId}-q`} className="text-muted-foreground">
								Recherche
							</Label>
							<div className="relative">
								<Search
									aria-hidden
									className="absolute top-2.5 left-2.5 size-4 text-muted-foreground"
								/>
								<Input
									id={`${baseId}-q`}
									type="search"
									value={query}
									autoComplete="off"
									className="pl-8"
									onChange={(event) => onQueryChange(event.target.value)}
								/>
							</div>
						</div>
					) : null}
					<TabsContent value="drive">
						{isLoading ? (
							<Loading />
						) : driveImages.length === 0 ? (
							<p className="text-[13px] text-muted-foreground">
								Aucune image dans le Drive.
							</p>
						) : (
							<ul className="max-h-80 divide-y overflow-y-auto rounded-md border">
								{driveImages.map((image) => (
									<li key={image.id}>
										<button
											type="button"
											onClick={() => onPickDrive(image.id)}
											className="w-full truncate px-3 py-2 text-left text-sm outline-none hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50"
										>
											{image.label}
										</button>
									</li>
								))}
							</ul>
						)}
					</TabsContent>
					<TabsContent value="photo">
						{isLoading ? (
							<Loading />
						) : photos.length === 0 ? (
							<p className="text-[13px] text-muted-foreground">
								Aucune photo dans la photothèque.
							</p>
						) : (
							<div className="max-h-80 overflow-y-auto">
								<PhotoGrid
									photos={photos}
									thumbnailUrls={thumbnailUrls}
									onSelect={onPickPhoto}
								/>
							</div>
						)}
					</TabsContent>
					<TabsContent value="upload" className="grid gap-2">
						<Label htmlFor={`${baseId}-file`}>
							Image à déposer dans « {folderName} »
						</Label>
						<Input
							id={`${baseId}-file`}
							type="file"
							accept="image/jpeg,image/png,image/webp"
							disabled={isUploading}
							onChange={(event) => {
								const file = event.target.files?.[0]
								if (file) onUpload(file)
								event.target.value = ''
							}}
						/>
						{isUploading ? <Loading label="Envoi…" /> : null}
					</TabsContent>
				</Tabs>
				{error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				) : null}
			</DialogContent>
		</Dialog>
	)
}

function Loading({ label = 'Chargement…' }: { label?: string }) {
	return (
		<div className="flex items-center gap-2 text-muted-foreground text-sm">
			<Loader2 aria-hidden className="size-4 animate-spin" />
			{label}
		</div>
	)
}
