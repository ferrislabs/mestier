import { useCallback, useState } from 'react'
import { useUploadKnowledgeFile } from '#/hooks/use-knowledge'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	mimeTypeOf,
	refusalOf,
	titleFromFileName,
} from '#/pages/knowledge/lib/files'
import {
	clearFinished,
	hasActive,
	newItem,
	patchItem,
	runPool,
	UPLOAD_CONCURRENCY,
	type UploadItem,
} from '#/pages/photo-library/lib/uploads'

export function useKnowledgeUploads(organizationId: string) {
	const [items, setItems] = useState<UploadItem[]>([])
	const { mutateAsync } = useUploadKnowledgeFile(organizationId)

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
				const refusal = refusalOf(file)
				const mimeType = mimeTypeOf(file)
				if (refusal !== null || mimeType === null) {
					update(item.id, { status: 'rejected', message: refusal })
					return
				}
				update(item.id, { status: 'uploading' })
				try {
					await mutateAsync({
						file: file.slice(0, file.size, mimeType),
						mimeType,
						title: titleFromFileName(file.name),
					})
					update(item.id, { status: 'done', message: null })
				} catch (error) {
					update(item.id, {
						status: 'rejected',
						message: mutationErrorMessage(error),
					})
				}
			})
		},
		[mutateAsync],
	)

	return {
		items,
		isBusy: hasActive(items),
		enqueue,
		clear: () => setItems((current) => clearFinished(current)),
	}
}
