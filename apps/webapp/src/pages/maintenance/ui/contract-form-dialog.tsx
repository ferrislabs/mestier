import { Loader2, Plus, Trash2 } from 'lucide-react'
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
import type {
	ContractDraft,
	ContractIssues,
	LineDraft,
} from '#/pages/maintenance/lib/contract-form'
import { CadenceField } from '#/pages/maintenance/ui/cadence-field'

export interface Option {
	id: string
	label: string
}

export interface ContractFormDialogProps {
	/** `null` keeps the dialog closed. */
	draft: ContractDraft | null
	customers: Option[]
	sites: Option[]
	isLoadingSites: boolean
	managers: Option[]
	issues: ContractIssues
	/** Issues are only shown once a save was attempted. */
	showIssues: boolean
	isPending: boolean
	error: string | null
	onChange: (patch: Partial<ContractDraft>) => void
	onLineChange: (key: string, patch: Partial<LineDraft>) => void
	onAddLine: () => void
	onRemoveLine: (key: string) => void
	onSubmit: () => void
	onClose: () => void
}

const SELECT =
	'h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm disabled:opacity-60'

function FieldError({ message }: { message?: string }) {
	return message ? (
		<p role="alert" className="text-sm text-destructive">
			{message}
		</p>
	) : null
}

/**
 * Contract and its lines, in one form. A `PUT` replaces the whole contract,
 * so the lines are edited here together and saved together.
 */
export function ContractFormDialog({
	draft,
	customers,
	sites,
	isLoadingSites,
	managers,
	issues,
	showIssues,
	isPending,
	error,
	onChange,
	onLineChange,
	onAddLine,
	onRemoveLine,
	onSubmit,
	onClose,
}: ContractFormDialogProps) {
	const editing = draft?.id != null
	const shown: ContractIssues = showIssues ? issues : { lines: {} }

	return (
		<Dialog open={draft !== null} onOpenChange={(open) => !open && onClose()}>
			<DialogContent className="flex max-h-[90vh] flex-col gap-0 overflow-hidden sm:max-w-3xl">
				{draft ? (
					<form
						className="flex min-h-0 flex-1 flex-col"
						onSubmit={(event) => {
							event.preventDefault()
							onSubmit()
						}}
					>
						<DialogHeader className="border-b pb-4">
							<DialogTitle>
								{editing
									? "Modifier le contrat d'entretien"
									: "Nouveau contrat d'entretien"}
							</DialogTitle>
							<DialogDescription>
								Un contrat couvre une saison. Le client, le site et l'année ne
								changent plus une fois le contrat créé.
							</DialogDescription>
						</DialogHeader>

						<div className="flex-1 space-y-5 overflow-y-auto py-4">
							<div className="grid gap-4 sm:grid-cols-2">
								<div className="space-y-1">
									<Label htmlFor="contract-customer">Client</Label>
									<select
										id="contract-customer"
										className={SELECT}
										disabled={editing}
										value={draft.customerId}
										onChange={(event) =>
											onChange({
												customerId: event.target.value,
												customerContextId: '',
											})
										}
									>
										<option value="">Choisir un client</option>
										{customers.map((customer) => (
											<option key={customer.id} value={customer.id}>
												{customer.label}
											</option>
										))}
									</select>
									<FieldError message={shown.customer} />
								</div>

								<div className="space-y-1">
									<Label htmlFor="contract-site">Site</Label>
									<select
										id="contract-site"
										className={SELECT}
										disabled={editing || !draft.customerId || isLoadingSites}
										value={draft.customerContextId}
										onChange={(event) =>
											onChange({ customerContextId: event.target.value })
										}
									>
										<option value="">
											{draft.customerId
												? 'Choisir un site'
												: "Choisir d'abord un client"}
										</option>
										{sites.map((site) => (
											<option key={site.id} value={site.id}>
												{site.label}
											</option>
										))}
									</select>
									<FieldError message={shown.site} />
								</div>

								<div className="space-y-1">
									<Label htmlFor="contract-reference">Code affaire</Label>
									<Input
										id="contract-reference"
										placeholder="C25029"
										value={draft.reference}
										onChange={(event) =>
											onChange({ reference: event.target.value })
										}
									/>
								</div>

								<div className="space-y-1">
									<Label htmlFor="contract-manager">
										Conducteur de travaux
									</Label>
									<select
										id="contract-manager"
										className={SELECT}
										value={draft.managerMemberId}
										onChange={(event) =>
											onChange({ managerMemberId: event.target.value })
										}
									>
										<option value="">Aucun</option>
										{managers.map((manager) => (
											<option key={manager.id} value={manager.id}>
												{manager.label}
											</option>
										))}
									</select>
								</div>
							</div>

							<fieldset className="space-y-1">
								<legend className="text-sm font-medium">Saison</legend>
								<div className="flex flex-wrap items-center gap-2 text-sm">
									<Label htmlFor="contract-year" className="sr-only">
										Année
									</Label>
									<Input
										id="contract-year"
										inputMode="numeric"
										className="w-24"
										disabled={editing}
										value={draft.year}
										onChange={(event) => onChange({ year: event.target.value })}
									/>
									<span>de la semaine</span>
									<Input
										aria-label="Première semaine"
										inputMode="numeric"
										className="w-16"
										value={draft.firstWeek}
										onChange={(event) =>
											onChange({ firstWeek: event.target.value })
										}
									/>
									<span>à la semaine</span>
									<Input
										aria-label="Dernière semaine"
										inputMode="numeric"
										className="w-16"
										value={draft.lastWeek}
										onChange={(event) =>
											onChange({ lastWeek: event.target.value })
										}
									/>
								</div>
								<FieldError message={shown.season} />
							</fieldset>

							<div className="grid gap-4 sm:grid-cols-2">
								<div className="space-y-1">
									<Label htmlFor="contract-amount">Montant (euros)</Label>
									<Input
										id="contract-amount"
										inputMode="decimal"
										placeholder="1 250,00"
										value={draft.amount}
										onChange={(event) =>
											onChange({ amount: event.target.value })
										}
									/>
									<FieldError message={shown.amount} />
								</div>
								<div className="space-y-1 sm:col-span-2">
									<Label htmlFor="contract-notes">Notes</Label>
									<Textarea
										id="contract-notes"
										value={draft.notes}
										onChange={(event) =>
											onChange({ notes: event.target.value })
										}
									/>
								</div>
							</div>

							<section aria-label="Prestations" className="space-y-3">
								<h3 className="text-sm font-semibold">Prestations</h3>
								{draft.lines.map((line, index) => (
									<div key={line.key} className="space-y-3 border p-3">
										<div className="flex items-end gap-2">
											<div className="min-w-0 flex-1 space-y-1">
												<Label htmlFor={`${line.key}-label`}>
													Prestation {index + 1}
												</Label>
												<Input
													id={`${line.key}-label`}
													placeholder="Tonte avec ramassage"
													value={line.label}
													onChange={(event) =>
														onLineChange(line.key, {
															label: event.target.value,
														})
													}
												/>
											</div>
											<div className="w-32 space-y-1">
												<Label htmlFor={`${line.key}-minutes`}>
													Minutes par passage
												</Label>
												<Input
													id={`${line.key}-minutes`}
													inputMode="numeric"
													value={line.minutes}
													onChange={(event) =>
														onLineChange(line.key, {
															minutes: event.target.value,
														})
													}
												/>
											</div>
											<Button
												type="button"
												variant="ghost"
												size="icon"
												aria-label={`Retirer la prestation ${index + 1}`}
												onClick={() => onRemoveLine(line.key)}
											>
												<Trash2 />
											</Button>
										</div>
										<CadenceField
											idPrefix={line.key}
											value={line.cadence}
											onChange={(cadence) =>
												onLineChange(line.key, { cadence })
											}
										/>
										<FieldError message={shown.lines[line.key]} />
									</div>
								))}
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={onAddLine}
								>
									<Plus />
									Ajouter une prestation
								</Button>
							</section>

							<FieldError message={error ?? undefined} />
						</div>

						<DialogFooter className="border-t pt-4">
							<Button type="button" variant="outline" onClick={onClose}>
								Annuler
							</Button>
							<Button type="submit" disabled={isPending}>
								{isPending ? <Loader2 className="animate-spin" /> : null}
								Enregistrer
							</Button>
						</DialogFooter>
					</form>
				) : null}
			</DialogContent>
		</Dialog>
	)
}
