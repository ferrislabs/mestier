import { useCallback, useState } from 'react'
import { useUploadLibraryPhoto } from '#/hooks/use-photo-library'
import { mutationErrorMessage } from '#/lib/api-error'
import { convertHeicToJpeg, prepareFile } from '#/pages/photo-library/lib/files'
import {
	clearFinished,
	hasActive,
	newItem,
	patchItem,
	runPool,
	UPLOAD_CONCURRENCY,
	type UploadItem,
} from '#/pages/photo-library/lib/uploads'

export function useLibraryUploads(
	organizationId: string,
	projectId: string | null,
) {
	const [items, setItems] = useState<UploadItem[]>([])
	const upload = useUploadLibraryPhoto(organizationId)
	const { mutateAsync } = upload

	const enqueue = useCallback(
		async (files: File[]) => {
			const entries = files.map((file) => ({
				file,
				item: newItem(crypto.randomUUID(), file.name),
			}))
			setItems((current) => [...current, ...entries.map((entry) => entry.item)])
			const update = (id: string, patch: Partial<Omit<UploadItem, 'id'>>) =>
				setItems((current) => patchItem(current, id, patch))

			await runPool(entries, UPLOAD_CONCURRENCY, async ({ file, item }) => {
				update(item.id, { status: 'converting' })
				const prepared = await prepareFile(file, convertHeicToJpeg)
				if (prepared.kind === 'rejected') {
					update(item.id, { status: 'rejected', message: prepared.message })
					return
				}
				update(item.id, { status: 'uploading' })
				try {
					const outcome = await mutateAsync({ file: prepared.file, projectId })
					update(item.id, {
						status: outcome.duplicate ? 'duplicate' : 'done',
						message: null,
					})
				} catch (error) {
					update(item.id, {
						status: 'rejected',
						message: mutationErrorMessage(error),
					})
				}
			})
		},
		[mutateAsync, projectId],
	)

	return {
		items,
		isBusy: hasActive(items),
		enqueue,
		clear: () => setItems((current) => clearFinished(current)),
	}
}
