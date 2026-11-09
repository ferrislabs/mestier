import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import type { LibraryPhoto } from '#/hooks/use-photo-library'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'
import type { UploadItem } from '#/pages/photo-library/lib/uploads'
import {
	PhotoFilters,
	type ProjectOption,
} from '#/pages/photo-library/ui/photo-filters'
import { PhotoGrid } from '#/pages/photo-library/ui/photo-grid'
import { UploadList } from '#/pages/photo-library/ui/upload-list'
import { Page, PageHeader } from '#/pages/projects/ui/page'

interface PhotoLibraryPageProps {
	photos: readonly LibraryPhoto[]
	thumbnailUrls: Record<string, string | undefined>
	uploads: readonly UploadItem[]
	isUploading: boolean
	isLoading: boolean
	error: string | null
	hasFilters: boolean
	page: number
	hasNext: boolean
	total: number | null
	q: string
	tagInput: string
	projectId: string | null
	toSort: boolean
	projects: readonly ProjectOption[]
	onFiles: (files: File[]) => void
	onClearUploads: () => void
	onQChange: (q: string) => void
	onTagInputChange: (tagInput: string) => void
	onProjectChange: (projectId: string | null) => void
	onToSortChange: (toSort: boolean) => void
	onPageChange: (page: number) => void
	onOpenPhoto: (photo: LibraryPhoto) => void
}

export function PhotoLibraryPage({
	photos,
	thumbnailUrls,
	uploads,
	isUploading,
	isLoading,
	error,
	hasFilters,
	page,
	hasNext,
	total,
	q,
	tagInput,
	projectId,
	toSort,
	projects,
	onFiles,
	onClearUploads,
	onQChange,
	onTagInputChange,
	onProjectChange,
	onToSortChange,
	onPageChange,
	onOpenPhoto,
}: PhotoLibraryPageProps) {
	return (
		<Page>
			<PageHeader
				title="Photothèque"
				description="Les photos de chantier de l’organisation, à légender et à placer dans les DOE."
			/>
			<div className="grid gap-5">
				<FileDropZone
					accept="image/*,.heic,.heif"
					label="Dépose des photos ici"
					hint="JPEG, PNG, WebP ou HEIC, plusieurs à la fois"
					isUploading={isUploading}
					onFiles={onFiles}
				/>
				<UploadList
					items={uploads}
					isBusy={isUploading}
					onClear={onClearUploads}
				/>
				<PhotoFilters
					q={q}
					tagInput={tagInput}
					projectId={projectId}
					toSort={toSort}
					projects={projects}
					onQChange={onQChange}
					onTagInputChange={onTagInputChange}
					onProjectChange={onProjectChange}
					onToSortChange={onToSortChange}
				/>
				{error ? (
					<div
						role="alert"
						className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-destructive text-sm"
					>
						{error}
					</div>
				) : null}
				{isLoading ? (
					<div className="flex items-center justify-center gap-2 py-10 text-muted-foreground text-sm">
						<Loader2 aria-hidden className="size-4 animate-spin" />
						Chargement de la photothèque…
					</div>
				) : photos.length === 0 ? (
					<EmptyState
						title={hasFilters ? 'Aucune photo ne correspond' : 'Aucune photo'}
						hint={
							hasFilters
								? 'Modifie ou retire un filtre pour voir plus de photos.'
								: 'Dépose tes premières photos de chantier ci-dessus.'
						}
					/>
				) : (
					<>
						<PhotoGrid
							photos={photos}
							thumbnailUrls={thumbnailUrls}
							onSelect={onOpenPhoto}
						/>
						<nav
							aria-label="Pagination"
							className="flex items-center justify-between gap-2"
						>
							<p className="text-muted-foreground text-sm">
								{total === null
									? `Page ${page}`
									: `${total} photos, page ${page}`}
							</p>
							<div className="flex gap-2">
								<Button
									type="button"
									variant="outline"
									size="sm"
									disabled={page <= 1}
									onClick={() => onPageChange(page - 1)}
								>
									<ChevronLeft aria-hidden />
									Précédente
								</Button>
								<Button
									type="button"
									variant="outline"
									size="sm"
									disabled={!hasNext}
									onClick={() => onPageChange(page + 1)}
								>
									Suivante
									<ChevronRight aria-hidden />
								</Button>
							</div>
						</nav>
					</>
				)}
			</div>
		</Page>
	)
}
