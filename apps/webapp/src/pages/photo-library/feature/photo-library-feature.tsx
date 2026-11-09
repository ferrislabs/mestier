import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useFileUrls } from '#/hooks/use-file-url'
import { type LibraryPhoto, usePhotoLibrary } from '#/hooks/use-photo-library'
import { useProjects } from '#/hooks/use-projects'
import { mutationErrorMessage } from '#/lib/api-error'
import { PhotoDetailFeature } from '#/pages/photo-library/feature/photo-detail-feature'
import { useLibraryUploads } from '#/pages/photo-library/feature/use-library-uploads'
import {
	filtersFromInputs,
	hasFilters,
} from '#/pages/photo-library/lib/filters'
import { PhotoLibraryPage } from '#/pages/photo-library/ui/photo-library-page'

const SEARCH_DELAY_MS = 300

function useDebounced<T>(value: T, delay: number): T {
	const [debounced, setDebounced] = useState(value)
	useEffect(() => {
		const timer = setTimeout(() => setDebounced(value), delay)
		return () => clearTimeout(timer)
	}, [value, delay])
	return debounced
}

export function PhotoLibraryFeature() {
	const { activeOrganization } = useActiveOrganization()
	return (
		<PhotoLibraryWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function PhotoLibraryWorkspace({ organizationId }: { organizationId: string }) {
	const [q, setQ] = useState('')
	const [tagInput, setTagInput] = useState('')
	const [projectId, setProjectId] = useState<string | null>(null)
	const [toSort, setToSort] = useState(false)
	const [page, setPage] = useState(1)
	const [openId, setOpenId] = useState<string | null>(null)

	const debouncedQ = useDebounced(q, SEARCH_DELAY_MS)
	const debouncedTags = useDebounced(tagInput, SEARCH_DELAY_MS)
	const filters = useMemo(
		() =>
			filtersFromInputs({
				q: debouncedQ,
				tagInput: debouncedTags,
				projectId,
				toSort,
			}),
		[debouncedQ, debouncedTags, projectId, toSort],
	)

	const library = usePhotoLibrary(organizationId, filters, page)
	const projects = useProjects(organizationId, { includeArchived: false })
	const projectOptions = useMemo(
		() =>
			(projects.data?.data ?? []).map((project) => ({
				id: project.id,
				name: project.name,
			})),
		[projects.data],
	)
	const uploads = useLibraryUploads(organizationId, projectId)

	const photos = library.data?.photos ?? []
	const pagination = library.data?.pagination ?? null
	const previews = useFileUrls(photos.map((photo) => photo.thumbnail_key))
	const thumbnailUrls = useMemo(() => {
		const urls: Record<string, string | undefined> = {}
		for (const preview of previews) urls[preview.key] = preview.url
		return urls
	}, [previews])
	const openPhoto: LibraryPhoto | null =
		photos.find((photo) => photo.id === openId) ?? null

	return (
		<>
			<PhotoLibraryPage
				photos={photos}
				thumbnailUrls={thumbnailUrls}
				uploads={uploads.items}
				isUploading={uploads.isBusy}
				isLoading={library.isLoading}
				error={mutationErrorMessage(library.error)}
				hasFilters={hasFilters(filters)}
				page={page}
				hasNext={
					pagination?.next_page != null ||
					(pagination?.last_page != null && page < pagination.last_page)
				}
				total={pagination?.total ?? null}
				q={q}
				tagInput={tagInput}
				projectId={projectId}
				toSort={toSort}
				projects={projectOptions}
				onFiles={(files) => {
					void uploads.enqueue(files)
				}}
				onClearUploads={uploads.clear}
				onQChange={(value) => {
					setQ(value)
					setPage(1)
				}}
				onTagInputChange={(value) => {
					setTagInput(value)
					setPage(1)
				}}
				onProjectChange={(value) => {
					setProjectId(value)
					setPage(1)
				}}
				onToSortChange={(value) => {
					setToSort(value)
					setPage(1)
				}}
				onPageChange={setPage}
				onOpenPhoto={(photo) => setOpenId(photo.id)}
			/>
			<PhotoDetailFeature
				key={openId}
				organizationId={organizationId}
				photo={openPhoto}
				projects={projectOptions}
				onClose={() => setOpenId(null)}
			/>
		</>
	)
}
