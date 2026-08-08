import { Loader2, Pencil, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Textarea } from '#/components/ui/textarea'
import { isOwnComment, type TaskComment } from '#/pages/planning/lib/comments'

export interface CommentThreadProps {
	comments: TaskComment[]
	/** The display name of the person looking at the thread — see `isOwnComment`'s own doc on why this is a heuristic, not a proof of identity. */
	currentDisplayName: string | null
	isLoading: boolean
	error: string | null
	canLoadMore: boolean
	canLoadOlder: boolean
	isLoadingPage?: boolean
	draftBody: string
	onDraftChange: (body: string) => void
	onSubmit: () => void
	isSubmitting: boolean
	editingCommentId: string | null
	editingBody: string
	onStartEdit: (comment: TaskComment) => void
	onEditBodyChange: (body: string) => void
	onConfirmEdit: () => void
	onCancelEdit: () => void
	onDelete: (comment: TaskComment) => void
	deletingCommentId?: string | null
	onLoadOlder: () => void
	onLoadMore: () => void
}

function formatTimestamp(iso: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
	}).format(new Date(iso))
}

/**
 * A task's discussion thread — pure presentation. Paginated one page at a
 * time (`comments` is always a single page, never the accumulated
 * history — see the planning remodel design doc: the thread must not load
 * everything up front). Edit/delete only render on a comment whose author
 * matches `currentDisplayName` (`isOwnComment`); the backend's own `403`
 * remains the real guard, this only avoids offering an action doomed to
 * fail.
 */
export function CommentThread({
	comments,
	currentDisplayName,
	isLoading,
	error,
	canLoadMore,
	canLoadOlder,
	draftBody,
	onDraftChange,
	onSubmit,
	isSubmitting,
	editingCommentId,
	editingBody,
	onStartEdit,
	onEditBodyChange,
	onConfirmEdit,
	onCancelEdit,
	onDelete,
	deletingCommentId,
	onLoadOlder,
	onLoadMore,
}: CommentThreadProps) {
	return (
		<div className="flex flex-col gap-3">
			{canLoadOlder ? (
				<Button
					type="button"
					variant="ghost"
					size="sm"
					className="self-center"
					onClick={onLoadOlder}
				>
					Charger les messages plus anciens
				</Button>
			) : null}

			{isLoading ? (
				<p className="py-6 text-center text-sm text-muted-foreground">
					Chargement du fil…
				</p>
			) : error ? (
				<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</p>
			) : comments.length === 0 ? (
				<p className="py-6 text-center text-sm text-muted-foreground">
					Aucun commentaire pour l’instant.
				</p>
			) : (
				<ul className="flex flex-col gap-3">
					{comments.map((comment) => {
						const own = isOwnComment(comment, currentDisplayName)
						const isEditing = editingCommentId === comment.id

						return (
							<li
								key={comment.id}
								data-testid={`comment-${comment.id}`}
								className="group rounded-lg border p-3"
							>
								<div className="flex items-center justify-between gap-2">
									<div className="flex items-baseline gap-2 text-sm">
										<span className="font-medium">
											{comment.author.display_name}
										</span>
										<span className="text-xs text-muted-foreground">
											{formatTimestamp(comment.created_at)}
											{comment.updated_at !== comment.created_at
												? ' (modifié)'
												: ''}
										</span>
									</div>
									{own && !isEditing ? (
										<div className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
											<Button
												type="button"
												variant="ghost"
												size="icon-sm"
												aria-label="Modifier"
												onClick={() => onStartEdit(comment)}
											>
												<Pencil className="size-3.5" />
											</Button>
											<Button
												type="button"
												variant="ghost"
												size="icon-sm"
												aria-label="Supprimer"
												disabled={deletingCommentId === comment.id}
												className="text-destructive hover:text-destructive"
												onClick={() => onDelete(comment)}
											>
												{deletingCommentId === comment.id ? (
													<Loader2 className="size-3.5 animate-spin" />
												) : (
													<Trash2 className="size-3.5" />
												)}
											</Button>
										</div>
									) : null}
								</div>

								{isEditing ? (
									<div className="mt-2 flex flex-col gap-2">
										<Textarea
											aria-label="Modifier le commentaire"
											value={editingBody}
											onChange={(event) => onEditBodyChange(event.target.value)}
										/>
										<div className="flex justify-end gap-2">
											<Button
												type="button"
												variant="ghost"
												size="sm"
												onClick={onCancelEdit}
											>
												Annuler
											</Button>
											<Button
												type="button"
												size="sm"
												disabled={!editingBody.trim()}
												onClick={onConfirmEdit}
											>
												Enregistrer
											</Button>
										</div>
									</div>
								) : (
									<p
										data-testid="comment-body"
										className="mt-1 whitespace-pre-wrap text-sm"
									>
										{comment.body}
									</p>
								)}
							</li>
						)
					})}
				</ul>
			)}

			{canLoadMore ? (
				<Button
					type="button"
					variant="ghost"
					size="sm"
					className="self-center"
					onClick={onLoadMore}
				>
					Charger les messages plus récents
				</Button>
			) : null}

			<form
				className="flex flex-col gap-2 border-t pt-3"
				onSubmit={(event) => {
					event.preventDefault()
					if (draftBody.trim() && !isSubmitting) onSubmit()
				}}
			>
				<Textarea
					aria-label="Nouveau commentaire"
					placeholder="Écrire un commentaire…"
					value={draftBody}
					onChange={(event) => onDraftChange(event.target.value)}
				/>
				<Button
					type="submit"
					size="sm"
					className="self-end"
					disabled={!draftBody.trim() || isSubmitting}
				>
					{isSubmitting ? <Loader2 className="animate-spin" /> : null}
					Envoyer
				</Button>
			</form>
		</div>
	)
}
