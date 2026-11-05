import { Loader2 } from 'lucide-react'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import type { TemplateTaskDraft } from '#/pages/project-templates/lib/template-task-form'
import {
	validateTemplateTaskDraft,
	validateTemplateTaskHierarchy,
} from '#/pages/project-templates/lib/template-task-form'
import type { ProjectTemplateFormValues } from '#/pages/project-templates/types'
import { TemplateTaskShapeEditor } from '#/pages/project-templates/ui/template-task-shape-editor'
import { ProjectTextarea } from '#/pages/projects/ui/project-textarea'

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
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<form
					noValidate
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit) onSubmit()
					}}
					className="flex min-h-0 flex-1 flex-col"
				>
					<DialogHeader className="px-6 pt-6 pb-4">
						<DialogTitle>
							{editingName ? `Modifier ${editingName}` : 'Nouveau modèle'}
						</DialogTitle>
						<DialogDescription>
							Un modèle est un ensemble de tâches types avec des décalages, pas
							un projet à copier. Aucun assigné ici : qui fait le travail change
							à chaque fois.
						</DialogDescription>
					</DialogHeader>
					<div className="grid min-h-0 flex-1 content-start gap-4 overflow-y-auto border-t px-6 py-5">
						<FormField id="project-template-name" label="Nom" required>
							<Input
								id="project-template-name"
								value={values.name}
								onChange={(event) =>
									onValuesChange({ name: event.target.value })
								}
								autoComplete="off"
								placeholder="Pose de terrasse, Rénovation cuisine…"
							/>
						</FormField>
						<FormField id="project-template-description" label="Description">
							<ProjectTextarea
								id="project-template-description"
								value={values.description}
								onChange={(event) =>
									onValuesChange({ description: event.target.value })
								}
								placeholder="Ce que ce modèle couvre, pour qui l'utilise sans avoir posé le chantier."
							/>
						</FormField>
						<div className="grid gap-1.5">
							<span className="font-medium text-sm">Tâches types</span>
							<TemplateTaskShapeEditor tasks={tasks} onChange={onTasksChange} />
						</div>
						{error ? (
							<p role="alert" className="text-[13px] text-state-alert">
								{error}
							</p>
						) : null}
					</div>
					<div className="flex items-center justify-between gap-3 border-t px-6 py-3">
						<p className="text-[12px] text-muted-foreground">
							Le nom du modèle est obligatoire.
						</p>
						<div className="flex gap-2">
							<DialogClose asChild>
								<Button type="button" variant="outline">
									Annuler
								</Button>
							</DialogClose>
							<RequirePermission permission="MANAGE_PLANNING">
								<Button type="submit" disabled={!canSubmit}>
									{isPending ? <Loader2 className="animate-spin" /> : null}
									{editingName ? 'Enregistrer' : 'Créer'}
								</Button>
							</RequirePermission>
						</div>
					</div>
				</form>
			</DialogContent>
		</Dialog>
	)
}
