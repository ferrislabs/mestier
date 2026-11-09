import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	type LibraryPhoto,
	useDeleteLibraryPhoto,
	usePatchLibraryPhoto,
} from '#/hooks/use-photo-library'
import { mutationErrorMessage } from '#/lib/api-error'
import { addTags, removeTag, tagsError } from '#/pages/photo-library/lib/tags'
import {
	PhotoDetailDialog,
	type PhotoDraft,
} from '#/pages/photo-library/ui/photo-detail-dialog'
import type { ProjectOption } from '#/pages/photo-library/ui/photo-filters'

interface PhotoDetailFeatureProps {
	organizationId: string
	photo: LibraryPhoto | null
	projects: readonly ProjectOption[]
	onClose: () => void
}

function draftOf(photo: LibraryPhoto): PhotoDraft {
	return {
		caption: photo.caption ?? '',
		tags: photo.tags,
		tagInput: '',
		projectId: photo.project_id,
	}
}

function sameTags(a: readonly string[], b: readonly string[]): boolean {
	return a.length === b.length && a.every((tag, index) => tag === b[index])
}

export function PhotoDetailFeature({
	organizationId,
	photo,
	projects,
	onClose,
}: PhotoDetailFeatureProps) {
	const [draft, setDraft] = useState<PhotoDraft | null>(
		photo ? draftOf(photo) : null,
	)
	const [confirmDelete, setConfirmDelete] = useState(false)
	const [localError, setLocalError] = useState<string | null>(null)
	const patch = usePatchLibraryPhoto(organizationId)
	const remove = useDeleteLibraryPhoto(organizationId)

	const [preview] = useFileUrls(photo ? [photo.file_key] : [])

	if (!photo || !draft) return null

	const pendingTags = addTags(draft.tags, draft.tagInput)
	const isDirty =
		draft.caption.trim() !== (photo.caption ?? '') ||
		!sameTags(pendingTags, photo.tags) ||
		draft.projectId !== photo.project_id

	const save = async () => {
		const error = tagsError(pendingTags)
		if (error) {
			setLocalError(error)
			return
		}
		setLocalError(null)
		try {
			const saved = await patch.mutateAsync({
				id: photo.id,
				patch: {
					caption: draft.caption.trim(),
					tags: pendingTags,
					project_id: draft.projectId,
				},
			})
			setDraft(draftOf(saved))
		} catch {}
	}

	const confirmedDelete = async () => {
		try {
			await remove.mutateAsync(photo.id)
			setConfirmDelete(false)
			onClose()
		} catch {}
	}

	return (
		<>
			<PhotoDetailDialog
				open
				imageUrl={preview?.url}
				draft={draft}
				projects={projects}
				isDirty={isDirty}
				isSaving={patch.isPending}
				error={localError ?? mutationErrorMessage(patch.error)}
				onDraftChange={(change) =>
					setDraft((current) => (current ? { ...current, ...change } : current))
				}
				onAddTags={() =>
					setDraft((current) =>
						current
							? {
									...current,
									tags: addTags(current.tags, current.tagInput),
									tagInput: '',
								}
							: current,
					)
				}
				onRemoveTag={(tag) =>
					setDraft((current) =>
						current
							? { ...current, tags: removeTag(current.tags, tag) }
							: current,
					)
				}
				onSave={() => {
					void save()
				}}
				onDelete={() => setConfirmDelete(true)}
				onOpenChange={(open) => {
					if (!open) onClose()
				}}
			/>
			<ConfirmDialog
				open={confirmDelete}
				onOpenChange={setConfirmDelete}
				title="Supprimer cette photo ?"
				description="La photo est retirée de la photothèque. Les DOE où elle a déjà été placée ne changent pas."
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
