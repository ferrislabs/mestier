import { Loader2 } from 'lucide-react'
import { TextField } from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
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
import type { WorkflowFormValues } from '#/pages/automation/types'

export interface WorkflowEditorDialogProps {
	open: boolean
	editingName: string | null
	values: WorkflowFormValues
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onValuesChange: (patch: Partial<WorkflowFormValues>) => void
	onSubmit: () => void
}

export function WorkflowEditorDialog({
	open,
	editingName,
	values,
	isPending,
	error,
	onOpenChange,
	onValuesChange,
	onSubmit,
}: WorkflowEditorDialogProps) {
	const canSubmit = values.name.trim().length > 0 && !isPending

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>
						{editingName ? `Modifier ${editingName}` : 'Nouveau workflow'}
					</DialogTitle>
					<DialogDescription>
						Un workflow automatise une suite d’actions déclenchées par un
						événement de l’organisation. Le déclencheur se choisit une fois le
						workflow créé.
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-4 overflow-y-auto py-4">
					<TextField
						label="Nom"
						value={values.name}
						onChange={(name) => onValuesChange({ name })}
						placeholder="Créer une facture Odoo, Notifier le client…"
					/>

					<div className="space-y-1">
						<Label htmlFor="workflow-description">Description</Label>
						<Textarea
							id="workflow-description"
							value={values.description}
							onChange={(event) =>
								onValuesChange({ description: event.target.value })
							}
							placeholder="Ce que déclenche ce workflow, pour qui le retrouve dans la liste."
						/>
					</div>

					{error ? <p className="text-sm text-destructive">{error}</p> : null}
				</div>

				<DialogFooter className="border-t pt-4">
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<RequirePermission permission="MANAGE_AUTOMATION">
						<Button disabled={!canSubmit} onClick={onSubmit}>
							{isPending ? <Loader2 className="animate-spin" /> : null}
							{editingName ? 'Enregistrer' : 'Créer'}
						</Button>
					</RequirePermission>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
