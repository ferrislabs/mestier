import { Loader2 } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Textarea } from '#/components/ui/textarea'
import { TagsEditor } from '#/pages/knowledge/ui/tags-editor'

export interface NoteDraft {
	title: string
	body: string
	tags: string[]
	tagInput: string
}

export const EMPTY_NOTE: NoteDraft = {
	title: '',
	body: '',
	tags: [],
	tagInput: '',
}

interface NoteDialogProps {
	open: boolean
	draft: NoteDraft
	isSaving: boolean
	error: string | null
	onDraftChange: (patch: Partial<NoteDraft>) => void
	onAddTags: () => void
	onRemoveTag: (tag: string) => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}

export function NoteDialog({
	open,
	draft,
	isSaving,
	error,
	onDraftChange,
	onAddTags,
	onRemoveTag,
	onSubmit,
	onOpenChange,
}: NoteDialogProps) {
	const baseId = useId()
	const canSubmit = draft.title.trim() !== '' && draft.body.trim() !== ''
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
				<DialogHeader>
					<DialogTitle>Nouvelle note</DialogTitle>
					<DialogDescription>
						Une information à garder : conditions, tarifs, habitudes de pose.
					</DialogDescription>
				</DialogHeader>
				<form
					noValidate
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit && !isSaving) onSubmit()
					}}
				>
					<div className="grid gap-1">
						<Label htmlFor={`${baseId}-title`}>Titre</Label>
						<Input
							id={`${baseId}-title`}
							value={draft.title}
							maxLength={200}
							autoComplete="off"
							onChange={(event) => onDraftChange({ title: event.target.value })}
						/>
					</div>
					<div className="grid gap-1">
						<Label htmlFor={`${baseId}-body`}>Texte</Label>
						<Textarea
							id={`${baseId}-body`}
							rows={8}
							value={draft.body}
							onChange={(event) => onDraftChange({ body: event.target.value })}
						/>
					</div>
					<TagsEditor
						tags={draft.tags}
						input={draft.tagInput}
						onInputChange={(tagInput) => onDraftChange({ tagInput })}
						onAdd={onAddTags}
						onRemove={onRemoveTag}
					/>
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={!canSubmit || isSaving}>
							{isSaving ? <Loader2 className="animate-spin" /> : null}
							Créer la note
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
