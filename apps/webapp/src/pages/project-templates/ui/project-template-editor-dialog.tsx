import { Loader2 } from 'lucide-react'
import { TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Label } from '#/components/ui/label'
import { Textarea } from '#/components/ui/textarea'
import type { TemplateTaskDraft } from '#/pages/project-templates/lib/template-task-form'
import {
	validateTemplateTaskDraft,
	validateTemplateTaskHierarchy,
} from '#/pages/project-templates/lib/template-task-form'
import type { ProjectTemplateFormValues } from '#/pages/project-templates/types'
import { TemplateTaskShapeEditor } from '#/pages/project-templates/ui/template-task-shape-editor'

export interface ProjectTemplateEditorDialogProps {
	open: boolean
	/** `null` when creating, the template's name when editing. */
	editingName: string | null
	values: ProjectTemplateFormValues
	tasks: TemplateTaskDraft[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onValuesChange: (patch: Partial<ProjectTemplateFormValues>) => void
	onTasksChange: (tasks: TemplateTaskDraft[]) => void
	onSubmit: () => void
}

/**
 * Create and edit share one dialog, like `ProjectFormDialog` — the fields
 * (and the task-shape builder) are identical either way.
 */
export function ProjectTemplateEditorDialog({
	open,
	editingName,
	values,
	tasks,
	isPending,
	error,
	onOpenChange,
	onValuesChange,
	onTasksChange,
	onSubmit,
}: ProjectTemplateEditorDialogProps) {
	const taskErrors = [
		...tasks.flatMap((task) => validateTemplateTaskDraft(task)),
		...validateTemplateTaskHierarchy(tasks),
	]
	const canSubmit =
		values.name.trim().length > 0 && taskErrors.length === 0 && !isPending

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] max-w-2xl flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>
						{editingName ? `Modifier ${editingName}` : 'Nouveau modèle'}
					</DialogTitle>
					<DialogDescription>
						Un modèle est un ensemble de tâches types avec des décalages, pas un
						projet à copier. Aucun assigné ici : qui fait le travail change à
						chaque fois.
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-4 overflow-y-auto py-4">
					<TextField
						label="Nom"
						value={values.name}
						onChange={(name) => onValuesChange({ name })}
						placeholder="Pose de terrasse, Rénovation cuisine…"
					/>

					<div className="space-y-1">
						<Label htmlFor="project-template-description">Description</Label>
						<Textarea
							id="project-template-description"
							value={values.description}
							onChange={(event) =>
								onValuesChange({ description: event.target.value })
							}
							placeholder="Ce que ce modèle couvre, pour qui l'utilise sans avoir posé le chantier."
						/>
					</div>

					<div className="space-y-2">
						<Label>Tâches types</Label>
						<TemplateTaskShapeEditor tasks={tasks} onChange={onTasksChange} />
					</div>

					{error ? <p className="text-sm text-destructive">{error}</p> : null}
				</div>

				<DialogFooter className="border-t pt-4">
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<Button disabled={!canSubmit} onClick={onSubmit}>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						{editingName ? 'Enregistrer' : 'Créer'}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
