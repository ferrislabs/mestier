import { useEffect, useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { usePlaceDafPhoto } from '#/hooks/use-daf'
import { useFileUrls } from '#/hooks/use-file-url'
import { type LibraryPhoto, usePhotoLibrary } from '#/hooks/use-photo-library'
import { mutationErrorMessage } from '#/lib/api-error'
import { DoePhotoLibraryPickerDialog } from '#/pages/doe/ui/doe-photo-library-picker-dialog'
import { EMPTY_FILTERS } from '#/pages/photo-library/lib/filters'

export function DafPhotoPickerFeature({
	open,
	organizationId,
	dafId,
	onPlaced,
	onClose,
}: {
	open: boolean
	organizationId: string
	dafId: string
	onPlaced: (daf: Schemas.DafResponse) => void
	onClose: () => void
}) {
	const [search, setSearch] = useState('')
	const [page, setPage] = useState(1)
	const [selected, setSelected] = useState<LibraryPhoto | null>(null)
	const place = usePlaceDafPhoto()
	const { reset: resetPlace } = place

	useEffect(() => {
		if (open) return
		setSearch('')
		setPage(1)
		setSelected(null)
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
		if (!selected) return
		try {
			const response = await place.mutateAsync({
				path: { organization_id: organizationId, daf_id: dafId },
				body: { photo_id: selected.id },
			})
			onPlaced(response.data)
			onClose()
		} catch {}
	}

	return (
		<DoePhotoLibraryPickerDialog
			open={open}
			destination="cette DAF"
			photos={photos}
			thumbnailUrls={thumbnailUrls}
			selectedIds={new Set(selected ? [selected.id] : [])}
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
			failures={[]}
			onSearchChange={(next) => {
				setSearch(next)
				setPage(1)
			}}
			onToggle={(photo) =>
				setSelected((current) => (current?.id === photo.id ? null : photo))
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
