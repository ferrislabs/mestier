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
import type { CustomerContext } from '#/hooks/use-customers'
import type { ProjectFormValues } from '#/pages/projects/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'

export interface ProjectOption {
	id: string
	label: string
}

export interface ProjectFormDialogProps {
	open: boolean
	/** `null` when creating, the project's name when editing. */
	editingName: string | null
	values: ProjectFormValues
	customers: ProjectOption[]
	/** Quotes already narrowed to the picked customer by the feature. */
	quotes: ProjectOption[]
	/** Addresses already narrowed to the picked customer by the feature. */
	customerContexts: CustomerContext[]
	isCustomerContextsLoading: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<ProjectFormValues>) => void
	onSubmit: () => void
}

/**
 * Create and edit share one dialog: the fields are identical, and two dialogs
 * would be two places to keep the customer/quote coupling right.
 *
 * Native `<select>` rather than the styled combobox: three fields, no search,
 * and the picker is capped at 100 customers upstream. A searchable picker is
 * noted as out of scope rather than half-built here.
 */
export function ProjectFormDialog({
	open,
	editingName,
	values,
	customers,
	quotes,
	customerContexts,
	isCustomerContextsLoading,
	isPending,
	error,
	onOpenChange,
	onChange,
	onSubmit,
}: ProjectFormDialogProps) {
	const canSubmit = values.name.trim().length > 0 && !isPending

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>
						{editingName ? `Modifier ${editingName}` : 'Nouveau projet'}
					</DialogTitle>
					<DialogDescription>
						Laisser le client vide crée un projet interne. Son coût est calculé
						comme celui d'un chantier, il n'a simplement pas de marge.
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-4 overflow-y-auto py-4">
					<TextField
						label="Nom"
						value={values.name}
						onChange={(name) => onChange({ name })}
						placeholder="Réunion hebdo, Jardin Duval…"
					/>

					<div className="space-y-1">
						<Label htmlFor="project-customer">Client</Label>
						<select
							id="project-customer"
							className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
							value={values.customerId}
							onChange={(event) =>
								// Changing the customer drops the quote and the pinned
								// address: both belong to one customer, and keeping either
								// would let it disagree with the new one.
								onChange({
									customerId: event.target.value,
									quoteId: '',
									customerContextId: '',
								})
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
						<Label htmlFor="project-billing-address">
							Adresse de facturation
						</Label>
						<BillingAddressField
							value={values.customerContextId}
							addresses={customerContexts}
							hasCustomer={Boolean(values.customerId)}
							isLoading={isCustomerContextsLoading}
							onChange={(customerContextId) => onChange({ customerContextId })}
						/>
						<p className="text-xs text-muted-foreground">
							Requise pour émettre un acompte ou un solde depuis ce projet ; une
							facture à montant libre peut toujours en choisir une autre.
						</p>
					</div>

					<div className="space-y-1">
						<Label htmlFor="project-quote">Devis</Label>
						<select
							id="project-quote"
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
						<p className="text-xs text-muted-foreground">
							{values.customerId === ''
								? 'Un devis suppose un client.'
								: 'La marge est le devis moins le coût planifié.'}
						</p>
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
