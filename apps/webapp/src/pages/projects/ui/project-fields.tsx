import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { CustomerContext } from '#/hooks/use-customers'
import type { Person } from '#/pages/projects/lib/model'
import {
	customerIdOfSelect,
	customerSelectValue,
	NO_QUOTE,
	type ProjectField,
	type ProjectFullForm,
	quoteIdOfSelect,
	quoteSelectValue,
} from '#/pages/projects/lib/project-form'
import { CLIENT_INTERNAL } from '#/pages/projects/lib/project-list'
import type { FieldBinding } from '#/pages/projects/lib/use-form-binding'
import { PersonSelect } from '#/pages/projects/ui/person-select'
import { ProjectTextarea } from '#/pages/projects/ui/project-textarea'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'

export interface ProjectOption {
	id: string
	label: string
}

export function ProjectFields({
	form,
	onChange,
	bind,
	baseId,
	customers,
	quotes,
	customerContexts,
	isCustomerContextsLoading,
	people,
}: {
	form: ProjectFullForm
	onChange: (patch: Partial<ProjectFullForm>) => void
	bind: (field: ProjectField) => FieldBinding
	baseId: string
	customers: readonly ProjectOption[]
	quotes: readonly ProjectOption[]
	customerContexts: CustomerContext[]
	isCustomerContextsLoading: boolean
	people: readonly Person[]
}) {
	const name = bind('name')
	const startDate = bind('startDate')
	const endDate = bind('endDate')
	const sorted = [...customers].sort((a, b) =>
		a.label.localeCompare(b.label, 'fr'),
	)
	const hasCustomer = form.customerId !== ''
	return (
		<>
			<FormField
				id={name.id}
				label="Nom du projet"
				required
				error={name.error}
				className="sm:col-span-2"
			>
				<Input
					{...name.input}
					value={form.name}
					onChange={(event) => onChange({ name: event.target.value })}
					autoComplete="off"
					placeholder="Réaménagement de l’avenue, pose de 14 fenêtres…"
				/>
			</FormField>
			<FormField
				id={`${baseId}-customer`}
				label="Client"
				hint="Sans client, le projet est un projet interne."
				className="sm:col-span-2"
			>
				<Select
					value={customerSelectValue(form.customerId)}
					onValueChange={(value) =>
						onChange({
							customerId: customerIdOfSelect(value),
							quoteId: '',
							customerContextId: '',
						})
					}
				>
					<SelectTrigger id={`${baseId}-customer`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={CLIENT_INTERNAL}>
							Projet interne, sans client
						</SelectItem>
						{sorted.map((customer) => (
							<SelectItem key={customer.id} value={customer.id}>
								{customer.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</FormField>
			<FormField
				id={`${baseId}-address`}
				label="Adresse de facturation"
				hint="Requise pour émettre un acompte ou un solde depuis ce projet ; une facture à montant libre peut toujours en choisir une autre."
				className="sm:col-span-2"
			>
				<BillingAddressField
					id={`${baseId}-address`}
					value={form.customerContextId}
					addresses={customerContexts}
					hasCustomer={hasCustomer}
					isLoading={isCustomerContextsLoading}
					onChange={(customerContextId) => onChange({ customerContextId })}
				/>
			</FormField>
			<FormField
				id={`${baseId}-quote`}
				label="Devis"
				hint={
					hasCustomer
						? 'La marge est le devis moins le coût planifié.'
						: 'Un devis suppose un client.'
				}
				className="sm:col-span-2"
			>
				<Select
					value={quoteSelectValue(form.quoteId)}
					onValueChange={(value) =>
						onChange({ quoteId: quoteIdOfSelect(value) })
					}
					disabled={!hasCustomer}
				>
					<SelectTrigger id={`${baseId}-quote`} className="w-full">
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
			<FormField
				id={`${baseId}-place`}
				label="Lieu d’intervention"
				className="sm:col-span-2"
			>
				<Input
					id={`${baseId}-place`}
					value={form.place}
					onChange={(event) => onChange({ place: event.target.value })}
					autoComplete="off"
					placeholder="Adresse du chantier"
				/>
			</FormField>
			<FormField
				id={`${baseId}-description`}
				label="Description"
				className="sm:col-span-2"
			>
				<ProjectTextarea
					id={`${baseId}-description`}
					value={form.description}
					onChange={(event) => onChange({ description: event.target.value })}
					placeholder="Ce que couvre le projet, en quelques lignes."
				/>
			</FormField>
			<FormField id={startDate.id} label="Début" error={startDate.error}>
				<Input
					{...startDate.input}
					type="date"
					value={form.startDate}
					onChange={(event) => onChange({ startDate: event.target.value })}
				/>
			</FormField>
			<FormField id={endDate.id} label="Fin" error={endDate.error}>
				<Input
					{...endDate.input}
					type="date"
					value={form.endDate}
					onChange={(event) => onChange({ endDate: event.target.value })}
				/>
			</FormField>
			<FormField
				id={`${baseId}-owner`}
				label="Responsable"
				className="sm:col-span-2"
			>
				<PersonSelect
					id={`${baseId}-owner`}
					value={form.ownerId}
					people={people}
					onChange={(ownerId) => onChange({ ownerId })}
				/>
			</FormField>
		</>
	)
}
