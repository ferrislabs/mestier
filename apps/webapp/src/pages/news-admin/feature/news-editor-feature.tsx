import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useUploadFile } from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import { useCreateNews, useDeleteNews, useUpdateNews } from '#/hooks/use-news'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	addItem,
	draftFromNews,
	emptyDraft,
	moveItem,
	type NewsDraft,
	type NewsDraftItem,
	removeItem,
	toRequest,
	validateDraft,
} from '#/pages/news-admin/lib/news-form'
import { NewsEditor } from '#/pages/news-admin/ui/news-editor'

interface NewsEditorFeatureProps {
	news: Schemas.NewsResponse | null
	onBack: () => void
}

export function NewsEditorFeature({ news, onBack }: NewsEditorFeatureProps) {
	const [draft, setDraft] = useState<NewsDraft>(() =>
		news ? draftFromNews(news, new Date()) : emptyDraft(),
	)
	const [showErrors, setShowErrors] = useState(false)
	const [deleteOpen, setDeleteOpen] = useState(false)
	const create = useCreateNews()
	const update = useUpdateNews()
	const remove = useDeleteNews()
	const upload = useUploadFile()
	const [image] = useFileUrls(draft.imageKey ? [draft.imageKey] : [])

	const errors = showErrors ? validateDraft(draft, new Date()) : {}
	const error =
		mutationErrorMessage(create.error) ??
		mutationErrorMessage(update.error) ??
		mutationErrorMessage(remove.error) ??
		mutationErrorMessage(upload.error)

	const patch = (next: Partial<NewsDraft>) =>
		setDraft((current) => ({ ...current, ...next }))

	return (
		<NewsEditor
			draft={draft}
			errors={errors}
			isNew={news === null}
			imageUrl={image?.url}
			uploading={upload.isPending}
			saving={create.isPending || update.isPending}
			deleting={remove.isPending}
			error={error}
			deleteOpen={deleteOpen}
			onDeleteOpenChange={setDeleteOpen}
			onChange={patch}
			onItemChange={(index, change: Partial<NewsDraftItem>) =>
				setDraft((current) => ({
					...current,
					items: current.items.map((item, position) =>
						position === index ? { ...item, ...change } : item,
					),
				}))
			}
			onItemAdd={() =>
				setDraft((current) => ({ ...current, items: addItem(current.items) }))
			}
			onItemRemove={(index) =>
				setDraft((current) => ({
					...current,
					items: removeItem(current.items, index),
				}))
			}
			onItemMove={(index, delta) =>
				setDraft((current) => ({
					...current,
					items: moveItem(current.items, index, delta),
				}))
			}
			onImageUpload={(file) => {
				void upload
					.mutateAsync(file)
					.then((response) => patch({ imageKey: response.data.key }))
					.catch(() => {})
			}}
			onImageRemove={() => patch({ imageKey: null })}
			onSave={() => {
				const now = new Date()
				if (Object.keys(validateDraft(draft, now)).length > 0) {
					setShowErrors(true)
					return
				}
				const body = toRequest(draft, now)
				const saving = news
					? update.mutateAsync({ path: { news_id: news.id }, body })
					: create.mutateAsync({ body })
				void saving.then(onBack).catch(() => {})
			}}
			onDelete={() => {
				if (!news) return
				void remove
					.mutateAsync({ path: { news_id: news.id } })
					.then(() => {
						setDeleteOpen(false)
						onBack()
					})
					.catch(() => setDeleteOpen(false))
			}}
			onBack={onBack}
		/>
	)
}
