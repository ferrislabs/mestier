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
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { ProjectTemplateTask } from '#/hooks/use-project-templates'
import { dayOffsetLabel } from '#/pages/project-templates/lib/template-task-form'
import type { InstantiateTemplateFormValues } from '#/pages/projects/types'
import type { ProjectOption } from '#/pages/projects/ui/project-form-dialog'

export interface InstantiateTemplateDialogProps {
	open: boolean
	templates: ProjectOption[]
	values: InstantiateTemplateFormValues
	/** The picked template's task shapes, once loaded — `null` while nothing is picked or still loading. */
	previewTasks: ProjectTemplateTask[] | null
	customers: ProjectOption[]
	quotes: ProjectOption[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<InstantiateTemplateFormValues>) => void
	onSubmit: () => void
}

/**
 * Picks a template, a name, a start date and optionally a customer and a
 * quote, and shows the tasks it will produce before creating anything — a
 * template that produces nineteen tasks silently is a template nobody dares
 * use twice.
 */
export function InstantiateTemplateDialog({
	open,
	templates,
	values,
	previewTasks,
	customers,
	quotes,
	isPending,
	error,
	onOpenChange,
	onChange,
	onSubmit,
}: InstantiateTemplateDialogProps) {
	const canSubmit =
		values.templateId !== '' &&
		values.name.trim().length > 0 &&
		values.startDate !== '' &&
		!isPending

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] max-w-xl flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>Démarrer depuis un modèle</DialogTitle>
					<DialogDescription>
						Le modèle donne la forme des tâches ; le projet créé porte ses
						propres dates, calculées depuis le point de départ ci-dessous.
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-4 overflow-y-auto py-4">
					<div className="space-y-1">
						<Label htmlFor="instantiate-template">Modèle</Label>
						<select
							id="instantiate-template"
							className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
							value={values.templateId}
							onChange={(event) => onChange({ templateId: event.target.value })}
						>
							<option value="">Choisir un modèle…</option>
							{templates.map((template) => (
								<option key={template.id} value={template.id}>
									{template.label}
								</option>
							))}
						</select>
					</div>

					<TextField
						label="Nom du projet"
						value={values.name}
						onChange={(name) => onChange({ name })}
						placeholder="Terrasse Dupont"
					/>

					<div className="space-y-1">
						<Label htmlFor="instantiate-start-date">Date de départ</Label>
						<Input
							id="instantiate-start-date"
							type="date"
							value={values.startDate}
							onChange={(event) => onChange({ startDate: event.target.value })}
						/>
						<p className="text-xs text-muted-foreground">
							Chaque tâche du modèle se recale sur cette date.
						</p>
					</div>

					<div className="space-y-1">
						<Label htmlFor="instantiate-customer">Client</Label>
						<select
							id="instantiate-customer"
							className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
							value={values.customerId}
							onChange={(event) =>
								onChange({ customerId: event.target.value, quoteId: '' })
							}
						>
							<option value="">Aucun — projet interne</option>
							{customers.map((customer) => (
								<option key={customer.id} value={customer.id}>
									{customer.label}
								</option>
							))}
						</select>
					</div>

					<div className="space-y-1">
						<Label htmlFor="instantiate-quote">Devis</Label>
						<select
							id="instantiate-quote"
							className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm disabled:opacity-50"
							value={values.quoteId}
							disabled={values.customerId === ''}
							onChange={(event) => onChange({ quoteId: event.target.value })}
						>
							<option value="">Aucun — pas de marge calculée</option>
							{quotes.map((quote) => (
								<option key={quote.id} value={quote.id}>
									{quote.label}
								</option>
							))}
						</select>
					</div>

					{values.templateId !== '' ? (
						<div className="space-y-2">
							<Label>Tâches qui seront créées</Label>
							{previewTasks === null ? (
								<p className="text-sm text-muted-foreground">Chargement…</p>
							) : previewTasks.length === 0 ? (
								<p className="text-sm text-muted-foreground">
									Ce modèle ne porte aucune tâche.
								</p>
							) : (
								<ul className="space-y-1 rounded-lg border bg-muted/30 p-3 text-sm">
									{previewTasks.map((task) => (
										<li key={task.id} className="flex items-center gap-2">
											<span className="text-muted-foreground">
												{dayOffsetLabel(task.day_offset)}
											</span>
											{task.parent_index !== null &&
											task.parent_index !== undefined ? (
												<span className="text-muted-foreground">↳</span>
											) : null}
											<span>{task.title}</span>
										</li>
									))}
								</ul>
							)}
						</div>
					) : null}

					{error ? <p className="text-sm text-destructive">{error}</p> : null}
				</div>

				<DialogFooter className="border-t pt-4">
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<Button disabled={!canSubmit} onClick={onSubmit}>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						Créer le projet
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
