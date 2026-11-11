import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import {
	downloadKnowledgeFile,
	type KnowledgeArticle,
	useDeleteKnowledgeArticle,
	useKnowledgeArticle,
	useLinkKnowledgeArticle,
	usePatchKnowledgeArticle,
	useUnlinkKnowledgeArticle,
} from '#/hooks/use-knowledge'
import { mutationErrorMessage } from '#/lib/api-error'
import { useResourceOptions } from '#/pages/knowledge/feature/use-resource-options'
import {
	type ArticleDraft,
	KnowledgeDetailDialog,
} from '#/pages/knowledge/ui/knowledge-detail-dialog'
import {
	addTags,
	MAX_TAG_LENGTH,
	MAX_TAGS,
	removeTag,
} from '#/pages/photo-library/lib/tags'

interface KnowledgeDetailFeatureProps {
	organizationId: string
	articleId: string
	canManage: boolean
	onClose: () => void
}

function draftOf(article: KnowledgeArticle): ArticleDraft {
	return {
		title: article.title,
		tags: article.tags,
		tagInput: '',
	}
}

function sameTags(a: readonly string[], b: readonly string[]): boolean {
	return a.length === b.length && a.every((tag, index) => tag === b[index])
}

function tagsProblem(tags: readonly string[]): string | null {
	if (tags.length > MAX_TAGS) {
		return `Un élément porte ${MAX_TAGS} étiquettes au plus.`
	}
	if (tags.some((tag) => tag.length > MAX_TAG_LENGTH)) {
		return `Une étiquette fait ${MAX_TAG_LENGTH} caractères au plus.`
	}
	return null
}

export function KnowledgeDetailFeature({
	organizationId,
	articleId,
	canManage,
	onClose,
}: KnowledgeDetailFeatureProps) {
	const query = useKnowledgeArticle(organizationId, articleId)
	if (!query.data) return null
	return (
		<KnowledgeDetailLoaded
			key={query.data.id}
			organizationId={organizationId}
			article={query.data}
			canManage={canManage}
			onClose={onClose}
		/>
	)
}

function KnowledgeDetailLoaded({
	organizationId,
	article,
	canManage,
	onClose,
}: Omit<KnowledgeDetailFeatureProps, 'articleId'> & {
	article: KnowledgeArticle
}) {
	const [draft, setDraft] = useState<ArticleDraft>(draftOf(article))
	const [confirmDelete, setConfirmDelete] = useState(false)
	const [localError, setLocalError] = useState<string | null>(null)
	const [isDownloading, setIsDownloading] = useState(false)
	const patch = usePatchKnowledgeArticle(organizationId)
	const remove = useDeleteKnowledgeArticle(organizationId)
	const link = useLinkKnowledgeArticle(organizationId)
	const unlink = useUnlinkKnowledgeArticle(organizationId)
	const resources = useResourceOptions(organizationId)

	const pendingTags = addTags(draft.tags, draft.tagInput)
	const isDirty =
		draft.title.trim() !== article.title || !sameTags(pendingTags, article.tags)

	const save = async () => {
		const problem = tagsProblem(pendingTags)
		if (problem) {
			setLocalError(problem)
			return
		}
		setLocalError(null)
		try {
			const saved = await patch.mutateAsync({
				id: article.id,
				patch: {
					title: draft.title.trim(),
					tags: pendingTags,
				},
			})
			setDraft(draftOf(saved))
		} catch {}
	}

	const confirmedDelete = async () => {
		try {
			await remove.mutateAsync(article.id)
			setConfirmDelete(false)
			onClose()
		} catch {}
	}

	const download = async () => {
		setIsDownloading(true)
		setLocalError(null)
		try {
			await downloadKnowledgeFile(organizationId, article)
		} catch (error) {
			setLocalError(mutationErrorMessage(error))
		} finally {
			setIsDownloading(false)
		}
	}

	return (
		<>
			<KnowledgeDetailDialog
				open
				article={article}
				canManage={canManage}
				draft={draft}
				isDirty={isDirty}
				isSaving={patch.isPending}
				isDownloading={isDownloading}
				isLinking={link.isPending || unlink.isPending}
				error={
					localError ??
					mutationErrorMessage(patch.error ?? link.error ?? unlink.error)
				}
				resources={resources}
				onDraftChange={(change) =>
					setDraft((current) => ({ ...current, ...change }))
				}
				onAddTags={() =>
					setDraft((current) => ({
						...current,
						tags: addTags(current.tags, current.tagInput),
						tagInput: '',
					}))
				}
				onRemoveTag={(tag) =>
					setDraft((current) => ({
						...current,
						tags: removeTag(current.tags, tag),
					}))
				}
				onSave={() => {
					void save()
				}}
				onDelete={() => setConfirmDelete(true)}
				onDownload={() => {
					void download()
				}}
				onLink={(resourceType, resourceId) =>
					link.mutate({ articleId: article.id, resourceType, resourceId })
				}
				onUnlink={(target) =>
					unlink.mutate({
						articleId: article.id,
						resourceType: target.resource_type,
						resourceId: target.resource_id,
					})
				}
				onOpenChange={(open) => {
					if (!open) onClose()
				}}
			/>
			<ConfirmDialog
				open={confirmDelete}
				onOpenChange={setConfirmDelete}
				title="Supprimer cet élément ?"
				description="Il disparaît de la base de connaissances, avec son fichier. Les projets, clients et devis auxquels il était rattaché ne changent pas."
				confirmLabel="Supprimer"
				destructive
				disabled={remove.isPending}
				onConfirm={() => {
					void confirmedDelete()
				}}
			>
				{remove.error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{mutationErrorMessage(remove.error)}
					</p>
				) : null}
			</ConfirmDialog>
		</>
	)
}
