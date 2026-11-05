import { ImageIcon, X } from 'lucide-react'
import { useId } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { photoSummary } from '#/pages/doe/lib/photos'
import type { DoeFile, PhotosPerPage } from '#/pages/doe/types'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'
import { IconButton } from '#/pages/doe/ui/icon-button'
import { SectionFrame } from '#/pages/doe/ui/section-frame'

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
	{ value: 'one', label: '1 par page' },
	{ value: 'two', label: '2 par page' },
	{ value: 'four', label: '4 par page' },
]

function isPerPage(value: string): value is PhotosPerPage {
	return OPTIONS.some((option) => option.value === value)
}

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
	const perPageId = useId()
	return (
		<SectionFrame
			number={sectionNumber}
			title={title}
			actions={
				<>
					<Label htmlFor={perPageId} className="text-muted-foreground">
						Photos par page
					</Label>
					<Select
						value={perPage}
						onValueChange={(value) => {
							if (isPerPage(value)) onPerPageChange(value)
						}}
					>
						<SelectTrigger id={perPageId} size="sm" className="w-36">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{OPTIONS.map((option) => (
								<SelectItem key={option.value} value={option.value}>
									{option.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</>
			}
		>
			<div className="grid gap-4 p-5">
				<RequirePermission permission="MANAGE_PLANNING">
					<FileDropZone
						accept="image/*"
						label="Dépose des photos ici"
						hint="Ou clique pour choisir des images"
						isUploading={isUploading}
						onFiles={onUpload}
					/>
				</RequirePermission>
				{photos.length === 0 ? (
					<p className="py-4 text-center text-muted-foreground text-sm">
						Aucune photo dans ce thème.
					</p>
				) : (
					<>
						<output className="block text-muted-foreground text-sm">
							{photoSummary(photos.length, perPage)}
						</output>
						<ul className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4">
							{photos.map((photo, index) => {
								const url = fileUrls[photo.key]
								const position = index + 1
								return (
									<li key={photo.key} className="grid content-start gap-2">
										{url ? (
											<img
												src={url}
												alt={photo.caption?.trim() || `Photo ${position}`}
												className="aspect-[4/3] w-full rounded-md border object-cover"
											/>
										) : (
											<div
												aria-hidden
												className="flex aspect-[4/3] w-full items-center justify-center rounded-md border bg-muted text-muted-foreground"
											>
												<ImageIcon className="size-6" />
											</div>
										)}
										<Input
											value={photo.caption ?? ''}
											placeholder="Légende"
											aria-label={`Légende de la photo ${position}`}
											autoComplete="off"
											className="h-8"
											onChange={(event) =>
												onCaptionChange(index, event.target.value)
											}
										/>
										<RequirePermission permission="MANAGE_PLANNING">
											<div className="flex items-center gap-0.5">
												<span className="flex-1" />
												<IconButton
													label={`Retirer la photo ${position}`}
													icon={X}
													onClick={() => onRemove(index)}
												/>
											</div>
										</RequirePermission>
									</li>
								)
							})}
						</ul>
					</>
				)}
			</div>
		</SectionFrame>
	)
}
