import { ImagePlus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { FilePreview } from '#/hooks/use-file-url'

interface PhotoStripProps {
	photos: FilePreview[]
	isUploading?: boolean
	onAdd: (file: File) => void
	onRemove: (key: string) => void
}

/**
 * Thumbnails for a line's photos, plus the control that adds one.
 *
 * Presentational by contract: the urls are resolved by the feature layer and
 * handed down, so this component never fetches. A photo whose link has not
 * arrived yet renders as a placeholder rather than a broken image, because a
 * signed url is a network round trip and a flash of a broken icon reads as
 * data loss.
 */
export function PhotoStrip({
	photos,
	isUploading,
	onAdd,
	onRemove,
}: PhotoStripProps) {
	return (
		<div className="flex flex-wrap items-center gap-2">
			{photos.map((photo) => (
				<div
					key={photo.key}
					className="group relative size-16 overflow-hidden rounded-md border bg-muted"
				>
					{photo.url ? (
						<img
							src={photo.url}
							alt=""
							loading="lazy"
							className="size-full object-cover"
						/>
					) : (
						<div className="size-full animate-pulse bg-muted" />
					)}
					<Button
						type="button"
						variant="secondary"
						size="icon-sm"
						className="absolute right-0.5 top-0.5 size-5 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
						onClick={() => onRemove(photo.key)}
					>
						<X className="size-3" />
						<span className="sr-only">Retirer la photo</span>
					</Button>
				</div>
			))}

			<label className="flex size-16 cursor-pointer flex-col items-center justify-center gap-1 rounded-md border border-dashed text-muted-foreground transition-colors hover:border-primary hover:text-primary">
				<ImagePlus className="size-4" />
				<span className="text-[10px] font-medium">Ajouter</span>
				<input
					type="file"
					accept="image/*"
					className="sr-only"
					disabled={isUploading}
					onChange={(event) => {
						const file = event.target.files?.[0]
						if (file) onAdd(file)
						event.target.value = ''
					}}
				/>
			</label>
		</div>
	)
}
