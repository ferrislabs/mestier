import { Loader2, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import {
	CommentThread,
	type CommentThreadProps,
} from '#/pages/planning/ui/comment-thread'
import {
	SubtaskList,
	type SubtaskListProps,
} from '#/pages/planning/ui/subtask-list'
import {
	TaskFormFields,
	type TaskFormFieldsProps,
} from '#/pages/planning/ui/task-form-fields'

export interface TaskSheetProps {
	open: boolean
	mode: 'create' | 'edit'
	title: string
	fields: TaskFormFieldsProps
	isSaving: boolean
	isDeleting?: boolean
	/** `false` when the task has children — a task with subtasks is not casually removed, see this workstream's report. Ignored (no delete button at all) in create mode. */
	canDelete?: boolean
	saveError: string | null
	onSubmit: () => void
	onDelete?: () => void
	onOpenChange: (open: boolean) => void
	/** Present only in edit mode — a task not yet created has no subtasks and no comment thread to show. */
	subtasksTab?: SubtaskListProps
	commentsTab?: CommentThreadProps
}

/**
 * The task create/edit surface — a `Sheet` with an optional tab strip.
 * Create mode never offers the "Sous-tâches"/"Commentaires" tabs: both
 * need a saved `task_id` the draft doesn't have yet (see the planning
 * remodel design doc). Pure presentation — all state and mutations live in
 * `feature/task-sheet-feature.tsx`.
 */
export function TaskSheet({
	open,
	mode,
	title,
	fields,
	isSaving,
	isDeleting,
	canDelete,
	saveError,
	onSubmit,
	onDelete,
	onOpenChange,
	subtasksTab,
	commentsTab,
}: TaskSheetProps) {
	const hasTabs = mode === 'edit' && subtasksTab && commentsTab

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-xl">
				{/*
				 * Deliberately not a <form>: the "Commentaires" tab renders
				 * `CommentThread`, which owns its own composer <form> so its
				 * "Envoyer" button submits independently of this sheet's
				 * save action. Nesting that inside a sheet-wide <form> is
				 * invalid HTML (a browser either hoists the inner form's
				 * fields to the outer one or silently drops the tag,
				 * behavior that differs across browsers) and risks a stray
				 * click on "Envoyer" also saving the task fields. The save
				 * button below is a plain button wired to `onSubmit`
				 * instead.
				 */}
				<div className="flex min-h-0 flex-1 flex-col">
					<SheetHeader className="border-b">
						<SheetTitle>{title}</SheetTitle>
						<SheetDescription>
							Titre, dates, labels et assignés — la question de
							l’indisponibilité est toujours posée, jamais devinée.
						</SheetDescription>
					</SheetHeader>

					<div className="flex-1 overflow-y-auto p-4">
						{hasTabs ? (
							<Tabs defaultValue="details">
								<TabsList>
									<TabsTrigger value="details">Détails</TabsTrigger>
									<TabsTrigger value="subtasks">Sous-tâches</TabsTrigger>
									<TabsTrigger value="comments">Commentaires</TabsTrigger>
								</TabsList>
								<TabsContent value="details" className="pt-4">
									<TaskFormFields {...fields} />
								</TabsContent>
								<TabsContent value="subtasks" className="pt-4">
									<SubtaskList {...subtasksTab} />
								</TabsContent>
								<TabsContent value="comments" className="pt-4">
									<CommentThread {...commentsTab} />
								</TabsContent>
							</Tabs>
						) : (
							<TaskFormFields {...fields} />
						)}

						{saveError ? (
							<p className="mt-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{saveError}
							</p>
						) : null}
					</div>

					<SheetFooter className="border-t bg-background sm:flex-row sm:justify-between">
						{mode === 'edit' && canDelete && onDelete ? (
							<Button
								type="button"
								variant="ghost"
								className="text-destructive hover:text-destructive"
								disabled={isDeleting}
								onClick={onDelete}
							>
								{isDeleting ? <Loader2 className="animate-spin" /> : <Trash2 />}
								Supprimer
							</Button>
						) : (
							<span />
						)}
						<div className="flex gap-2">
							<Button
								type="button"
								variant="ghost"
								onClick={() => onOpenChange(false)}
							>
								Annuler
							</Button>
							<Button type="button" disabled={isSaving} onClick={onSubmit}>
								{isSaving ? <Loader2 className="animate-spin" /> : null}
								{mode === 'create' ? 'Créer' : 'Enregistrer'}
							</Button>
						</div>
					</SheetFooter>
				</div>
			</SheetContent>
		</Sheet>
	)
}
