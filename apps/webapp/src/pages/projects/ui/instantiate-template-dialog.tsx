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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { ProjectTemplateTask } from '#/hooks/use-project-templates'
import { dayOffsetLabel } from '#/pages/project-templates/lib/template-task-form'
import {
	customerIdOfSelect,
	customerSelectValue,
	NO_QUOTE,
	quoteIdOfSelect,
	quoteSelectValue,
} from '#/pages/projects/lib/project-form'
import { CLIENT_INTERNAL } from '#/pages/projects/lib/project-list'
import type { InstantiateTemplateFormValues } from '#/pages/projects/types'
import type { ProjectOption } from '#/pages/projects/ui/project-fields'

export interface InstantiateTemplateDialogProps {
	open: boolean
	templates: ProjectOption[]
	values: InstantiateTemplateFormValues
	previewTasks: ProjectTemplateTask[] | null
	customers: readonly ProjectOption[]
	quotes: readonly ProjectOption[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<InstantiateTemplateFormValues>) => void
	onSubmit: () => void
}

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
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<form
					noValidate
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit) onSubmit()
					}}
					className="flex min-h-0 flex-1 flex-col"
				>
					<DialogHeader className="px-6 pt-6 pb-4">
						<DialogTitle>Démarrer depuis un modèle</DialogTitle>
						<DialogDescription>
							Le modèle donne la forme des tâches ; le projet créé porte ses
							propres dates, calculées depuis le point de départ ci-dessous.
						</DialogDescription>
					</DialogHeader>
					<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
						<FormField
							id="instantiate-template"
							label="Modèle"
							className="sm:col-span-2"
						>
							<Select
								value={values.templateId}
								onValueChange={(templateId) => onChange({ templateId })}
							>
								<SelectTrigger id="instantiate-template" className="w-full">
									<SelectValue placeholder="Choisir un modèle…" />
								</SelectTrigger>
								<SelectContent>
									{templates.map((template) => (
										<SelectItem key={template.id} value={template.id}>
											{template.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</FormField>
						<FormField
							id="instantiate-name"
							label="Nom du projet"
							className="sm:col-span-2"
						>
							<Input
								id="instantiate-name"
								value={values.name}
								onChange={(event) => onChange({ name: event.target.value })}
								autoComplete="off"
								placeholder="Terrasse Dupont"
							/>
						</FormField>
						<FormField
							id="instantiate-start-date"
							label="Date de départ"
							hint="Chaque tâche du modèle se recale sur cette date."
							className="sm:col-span-2"
						>
							<Input
								id="instantiate-start-date"
								type="date"
								value={values.startDate}
								onChange={(event) =>
									onChange({ startDate: event.target.value })
								}
							/>
						</FormField>
						<FormField id="instantiate-customer" label="Client">
							<Select
								value={customerSelectValue(values.customerId)}
								onValueChange={(value) =>
									onChange({
										customerId: customerIdOfSelect(value),
										quoteId: '',
									})
								}
							>
								<SelectTrigger id="instantiate-customer" className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={CLIENT_INTERNAL}>
										Projet interne, sans client
									</SelectItem>
									{customers.map((customer) => (
										<SelectItem key={customer.id} value={customer.id}>
											{customer.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</FormField>
						<FormField id="instantiate-quote" label="Devis">
							<Select
								value={quoteSelectValue(values.quoteId)}
								onValueChange={(value) =>
									onChange({ quoteId: quoteIdOfSelect(value) })
								}
								disabled={values.customerId === ''}
							>
								<SelectTrigger id="instantiate-quote" className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={NO_QUOTE}>
										Aucun, pas de marge calculée
									</SelectItem>
									{quotes.map((quote) => (
										<SelectItem key={quote.id} value={quote.id}>
											{quote.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</FormField>
						{values.templateId !== '' ? (
							<div className="grid gap-1.5 sm:col-span-2">
								<span className="font-medium text-sm">
									Tâches qui seront créées
								</span>
								{previewTasks === null ? (
									<p className="text-[13px] text-muted-foreground">
										Chargement…
									</p>
								) : previewTasks.length === 0 ? (
									<p className="text-[13px] text-muted-foreground">
										Ce modèle ne porte aucune tâche.
									</p>
								) : (
									<ul className="grid gap-1 rounded-md border bg-muted/30 p-3 text-[13px]">
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
						{error ? (
							<p
								role="alert"
								className="text-[13px] text-state-alert sm:col-span-2"
							>
								{error}
							</p>
						) : null}
					</div>
					<div className="flex items-center justify-end gap-3 border-t px-6 py-3">
						<div className="flex gap-2">
							<DialogClose asChild>
								<Button type="button" variant="outline">
									Annuler
								</Button>
							</DialogClose>
							<RequirePermission permission="MANAGE_PLANNING">
								<Button type="submit" disabled={!canSubmit}>
									{isPending ? <Loader2 className="animate-spin" /> : null}
									Créer le projet
								</Button>
							</RequirePermission>
						</div>
					</div>
				</form>
			</DialogContent>
		</Dialog>
	)
}
