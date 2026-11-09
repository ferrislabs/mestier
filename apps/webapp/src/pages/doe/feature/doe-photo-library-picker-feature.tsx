import { useEffect, useMemo, useState } from 'react'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	type LibraryPhoto,
	type PlacementResponse,
	usePhotoLibrary,
	usePlaceDoePhotos,
} from '#/hooks/use-photo-library'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	buildPlacements,
	failedResults,
	failureLabel,
	type LibraryTarget,
	placedIds,
} from '#/pages/doe/lib/library-placement'
import {
	DoePhotoLibraryPickerDialog,
	type PickerFailure,
} from '#/pages/doe/ui/doe-photo-library-picker-dialog'
import { EMPTY_FILTERS } from '#/pages/photo-library/lib/filters'

interface DoePhotoLibraryPickerFeatureProps {
	open: boolean
	organizationId: string
	doeId: string
	target: LibraryTarget | null
	onPlaced: (response: PlacementResponse) => void
	onClose: () => void
}

export function DoePhotoLibraryPickerFeature({
	open,
	organizationId,
	doeId,
	target,
	onPlaced,
	onClose,
}: DoePhotoLibraryPickerFeatureProps) {
	const [search, setSearch] = useState('')
	const [page, setPage] = useState(1)
	const [selected, setSelected] = useState<ReadonlyMap<string, LibraryPhoto>>(
		new Map(),
	)
	const [failures, setFailures] = useState<PickerFailure[]>([])
	const place = usePlaceDoePhotos(organizationId)
	const { reset: resetPlace } = place

	useEffect(() => {
		if (open) return
		setSearch('')
		setPage(1)
		setSelected(new Map())
		setFailures([])
		resetPlace()
	}, [open, resetPlace])

	const filters = useMemo(
		() => ({ ...EMPTY_FILTERS, q: search.trim() }),
		[search],
	)
	const library = usePhotoLibrary(organizationId, filters, page, {
		enabled: open,
	})
	const photos = library.data?.photos ?? []
	const pagination = library.data?.pagination ?? null
	const previews = useFileUrls(photos.map((photo) => photo.thumbnail_key))
	const thumbnailUrls = useMemo(() => {
		const urls: Record<string, string | undefined> = {}
		for (const preview of previews) urls[preview.key] = preview.url
		return urls
	}, [previews])

	const confirm = async () => {
		if (!target || selected.size === 0) return
		const labels = new Map(
			[...selected.values()].map((photo) => [
				photo.id,
				photo.caption?.trim() || 'Photo sans légende',
			]),
		)
		try {
			const response = await place.mutateAsync({
				doeId,
				placements: buildPlacements([...selected.keys()], target),
			})
			const failed = failedResults(response.results)
			const done = placedIds(response.results)
			setFailures(
				failed.map((result) => ({
					photoId: result.photo_id,
					label: labels.get(result.photo_id) ?? 'Photo',
					message: failureLabel(result),
				})),
			)
			setSelected((current) => {
				const next = new Map(current)
				for (const id of done) next.delete(id)
				return next
			})
			if (done.size > 0) onPlaced(response)
			if (failed.length === 0) onClose()
		} catch {}
	}

	return (
		<DoePhotoLibraryPickerDialog
			open={open}
			photos={photos}
			thumbnailUrls={thumbnailUrls}
			selectedIds={new Set(selected.keys())}
			search={search}
			isLoading={library.isLoading}
			isPlacing={place.isPending}
			hasPrevious={page > 1}
			hasNext={
				pagination?.next_page != null ||
				(pagination?.last_page != null && page < pagination.last_page)
			}
			error={
				mutationErrorMessage(place.error) ?? mutationErrorMessage(library.error)
			}
			failures={failures}
			onSearchChange={(next) => {
				setSearch(next)
				setPage(1)
			}}
			onToggle={(photo) =>
				setSelected((current) => {
					const next = new Map(current)
					if (next.has(photo.id)) next.delete(photo.id)
					else next.set(photo.id, photo)
					return next
				})
			}
			onPrevious={() => setPage((current) => Math.max(1, current - 1))}
			onNext={() => setPage((current) => current + 1)}
			onConfirm={() => {
				void confirm()
			}}
			onOpenChange={(next) => {
				if (!next) onClose()
			}}
		/>
	)
}
