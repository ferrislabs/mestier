import { X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { cn } from '#/lib/utils'
import { photoSummary } from '#/pages/doe/lib/photos'
import type { DoeFile, PhotosPerPage } from '#/pages/doe/types'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'

interface DoePhotosTabProps {
	sectionNumber: number
	title: string
	perPage: PhotosPerPage
	photos: DoeFile[]
	fileUrls: Record<string, string | undefined>
	isUploading: boolean
	onPerPageChange: (perPage: PhotosPerPage) => void
	onUpload: (files: File[]) => void
	onCaptionChange: (index: number, caption: string) => void
	onRemove: (index: number) => void
}

const OPTIONS: Array<{ value: PhotosPerPage; label: string }> = [
	{ value: 'one', label: '1' },
	{ value: 'two', label: '2' },
	{ value: 'four', label: '4' },
]

export function DoePhotosTab({
	sectionNumber,
	title,
	perPage,
	photos,
	fileUrls,
	isUploading,
	onPerPageChange,
	onUpload,
	onCaptionChange,
	onRemove,
}: DoePhotosTabProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`${sectionNumber}. ${title}`}
				actions={
					<div
						role="radiogroup"
						aria-label="Photos par page"
						className="flex items-center gap-2 text-sm"
					>
						<span className="text-muted-foreground">Photos par page</span>
						{OPTIONS.map((option) => (
							<Button
								key={option.value}
								type="button"
								role="radio"
								aria-checked={perPage === option.value}
								size="sm"
								variant={perPage === option.value ? 'default' : 'outline'}
								onClick={() => onPerPageChange(option.value)}
							>
								{option.label}
							</Button>
						))}
					</div>
				}
			/>
			<div className="space-y-4 p-5">
				<RequirePermission permission="MANAGE_PLANNING">
					<FileDropZone
						accept="image/*"
						label="Déposez des photos ici"
						hint="Ou cliquez pour choisir des images"
						isUploading={isUploading}
						onFiles={onUpload}
					/>
				</RequirePermission>
				<p className="text-sm text-muted-foreground">
					{photoSummary(photos.length, perPage)}
				</p>
				{photos.length > 0 ? (
					<ul className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4">
						{photos.map((photo, index) => {
							const url = fileUrls[photo.key]
							return (
								<li key={photo.key} className="space-y-2">
									<div className="relative">
										{url ? (
											<img
												src={url}
												alt={photo.caption ?? `Photo ${index + 1}`}
												className="aspect-[4/3] w-full rounded-md border object-cover"
											/>
										) : (
											<div
												className={cn(
													'aspect-[4/3] w-full rounded-md border bg-muted',
												)}
											/>
										)}
										<RequirePermission permission="MANAGE_PLANNING">
											<Button
												type="button"
												variant="secondary"
												size="icon-xs"
												className="absolute top-1 right-1"
												onClick={() => onRemove(index)}
											>
												<X />
												<span className="sr-only">
													Retirer la photo {index + 1}
												</span>
											</Button>
										</RequirePermission>
									</div>
									<input
										className="w-full rounded-md border bg-card px-2 py-1 text-sm"
										value={photo.caption ?? ''}
										placeholder="Légende"
										aria-label={`Légende de la photo ${index + 1}`}
										onChange={(event) =>
											onCaptionChange(index, event.target.value)
										}
									/>
								</li>
							)
						})}
					</ul>
				) : null}
			</div>
		</SectionCard>
	)
}
