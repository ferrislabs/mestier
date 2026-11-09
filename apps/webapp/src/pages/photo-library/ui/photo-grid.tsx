import { Check, ImageIcon } from 'lucide-react'
import type { LibraryPhoto } from '#/hooks/use-photo-library'
import { cn } from '#/lib/utils'

interface PhotoGridProps {
	photos: readonly LibraryPhoto[]
	thumbnailUrls: Record<string, string | undefined>
	selectedIds?: ReadonlySet<string>
	onSelect: (photo: LibraryPhoto) => void
}

export function photoLabel(photo: LibraryPhoto): string {
	return photo.caption?.trim() || 'Photo sans légende'
}

export function PhotoGrid({
	photos,
	thumbnailUrls,
	selectedIds,
	onSelect,
}: PhotoGridProps) {
	return (
		<ul className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6">
			{photos.map((photo) => {
				const url = thumbnailUrls[photo.thumbnail_key]
				const isSelected = selectedIds?.has(photo.id) ?? false
				return (
					<li key={photo.id}>
						<button
							type="button"
							aria-pressed={selectedIds ? isSelected : undefined}
							aria-label={photoLabel(photo)}
							onClick={() => onSelect(photo)}
							className={cn(
								'relative block aspect-square w-full overflow-hidden rounded-md border bg-muted outline-none transition focus-visible:ring-[3px] focus-visible:ring-ring/50',
								isSelected
									? 'border-primary ring-2 ring-primary'
									: 'hover:border-muted-foreground/60',
							)}
						>
							{url ? (
								<img
									src={url}
									alt=""
									loading="lazy"
									className="size-full object-cover"
								/>
							) : (
								<span
									aria-hidden
									className="flex size-full items-center justify-center text-muted-foreground"
								>
									<ImageIcon className="size-6" />
								</span>
							)}
							{isSelected ? (
								<span
									aria-hidden
									className="absolute top-1.5 right-1.5 flex size-5 items-center justify-center rounded-full bg-primary text-primary-foreground"
								>
									<Check className="size-3" />
								</span>
							) : null}
							{photo.caption ? (
								<span className="absolute inset-x-0 bottom-0 truncate bg-background/80 px-2 py-1 text-left text-xs">
									{photo.caption}
								</span>
							) : null}
						</button>
					</li>
				)
			})}
		</ul>
	)
}
