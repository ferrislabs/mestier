import { Loader2, X } from 'lucide-react'
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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Textarea } from '#/components/ui/textarea'
import type { ProjectOption } from '#/pages/photo-library/ui/photo-filters'

export interface PhotoDraft {
	caption: string
	tags: string[]
	tagInput: string
	projectId: string | null
}

interface PhotoDetailDialogProps {
	open: boolean
	imageUrl: string | undefined
	draft: PhotoDraft
	projects: readonly ProjectOption[]
	isDirty: boolean
	isSaving: boolean
	error: string | null
	onDraftChange: (patch: Partial<PhotoDraft>) => void
	onAddTags: () => void
	onRemoveTag: (tag: string) => void
	onSave: () => void
	onDelete: () => void
	onOpenChange: (open: boolean) => void
}

const NO_PROJECT = '__none__'

export function PhotoDetailDialog({
	open,
	imageUrl,
	draft,
	projects,
	isDirty,
	isSaving,
	error,
	onDraftChange,
	onAddTags,
	onRemoveTag,
	onSave,
	onDelete,
	onOpenChange,
}: PhotoDetailDialogProps) {
	const baseId = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
				<DialogHeader>
					<DialogTitle>Photo</DialogTitle>
					<DialogDescription>
						Légende, étiquettes et projet de la photo. Les DOE qui l’utilisent
						déjà n’en sont pas modifiés.
					</DialogDescription>
				</DialogHeader>
				<form
					noValidate
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						if (isDirty && !isSaving) onSave()
					}}
				>
					{imageUrl ? (
						<img
							src={imageUrl}
							alt={draft.caption.trim() || 'Photo sans légende'}
							className="max-h-80 w-full rounded-md border bg-muted object-contain"
						/>
					) : (
						<div
							aria-hidden
							className="h-48 w-full animate-pulse rounded-md border bg-muted"
						/>
					)}
					<div className="grid gap-1">
						<Label htmlFor={`${baseId}-caption`}>Légende</Label>
						<Textarea
							id={`${baseId}-caption`}
							rows={2}
							value={draft.caption}
							onChange={(event) =>
								onDraftChange({ caption: event.target.value })
							}
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${baseId}-tags`}>Étiquettes</Label>
						{draft.tags.length > 0 ? (
							<ul className="flex flex-wrap gap-1.5">
								{draft.tags.map((tag) => (
									<li
										key={tag}
										className="flex items-center gap-1 rounded-full bg-muted py-0.5 pr-1 pl-2.5 text-sm"
									>
										{tag}
										<button
											type="button"
											aria-label={`Retirer l’étiquette ${tag}`}
											onClick={() => onRemoveTag(tag)}
											className="rounded-full p-0.5 outline-none hover:bg-background focus-visible:ring-[3px] focus-visible:ring-ring/50"
										>
											<X aria-hidden className="size-3" />
										</button>
									</li>
								))}
							</ul>
						) : null}
						<div className="flex gap-2">
							<Input
								id={`${baseId}-tags`}
								value={draft.tagInput}
								placeholder="Ajouter une étiquette"
								autoComplete="off"
								onChange={(event) =>
									onDraftChange({ tagInput: event.target.value })
								}
								onKeyDown={(event) => {
									if (event.key === 'Enter' || event.key === ',') {
										event.preventDefault()
										onAddTags()
									}
								}}
							/>
							<Button
								type="button"
								variant="outline"
								disabled={draft.tagInput.trim() === ''}
								onClick={onAddTags}
							>
								Ajouter
							</Button>
						</div>
					</div>
					<div className="grid gap-1">
						<Label htmlFor={`${baseId}-project`}>Projet</Label>
						<Select
							value={draft.projectId ?? NO_PROJECT}
							onValueChange={(value) =>
								onDraftChange({
									projectId: value === NO_PROJECT ? null : value,
								})
							}
						>
							<SelectTrigger id={`${baseId}-project`}>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={NO_PROJECT}>Aucun projet</SelectItem>
								{projects.map((project) => (
									<SelectItem key={project.id} value={project.id}>
										{project.name}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
					<DialogFooter className="sm:justify-between">
						<Button type="button" variant="destructive" onClick={onDelete}>
							Supprimer
						</Button>
						<div className="flex gap-2">
							<Button
								type="button"
								variant="outline"
								onClick={() => onOpenChange(false)}
							>
								Fermer
							</Button>
							<Button type="submit" disabled={!isDirty || isSaving}>
								{isSaving ? <Loader2 className="animate-spin" /> : null}
								Enregistrer
							</Button>
						</div>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
