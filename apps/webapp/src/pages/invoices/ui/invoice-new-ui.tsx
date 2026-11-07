import { Link } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { InvoiceKind, OperationNature } from '#/hooks/use-invoices'
import type { Project } from '#/hooks/use-projects'
import { buildOrgPath } from '#/modules/org-path'
import { MAX_PAYMENT_TERMS_DAYS } from '#/pages/invoices/lib/invoice-new'
import {
	firstInvalidInvoiceFieldId,
	NEW_INVOICE_FIELD_IDS,
	type NewInvoiceFormValues,
	validateNewInvoice,
} from '#/pages/invoices/lib/new-invoice-form'
import {
	type InvoiceLineFormValues,
	invoiceLineTotalCents,
	operationNatureLabel,
} from '#/pages/invoices/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { CustomerPicker } from '#/pages/quotes/ui/doc/customer-picker'
import { BackLink } from '#/pages/quotes/ui/doc/detail-shell'
import { LinesEditor } from '#/pages/quotes/ui/doc/lines-editor'
import {
	AutoTextarea,
	LockBanner,
	Money,
	OptionCard,
} from '#/pages/quotes/ui/doc/primitives'

const KIND_CHOICES: {
	kind: InvoiceKind
	title: string
	description: string
}[] = [
	{
		kind: 'STANDARD',
		title: 'Facture',
		description: 'Une facture classique, avec ses propres lignes.',
	},
	{
		kind: 'DEPOSIT',
		title: 'Facture d’acompte',
		description: 'Demande un acompte avant le début des travaux.',
	},
	{
		kind: 'FINAL',
		title: 'Facture de solde',
		description: 'Solde le projet une fois les travaux terminés.',
	},
]

const OPERATION_NATURE_CHOICES: OperationNature[] = [
	'GOODS',
	'SERVICES',
	'BOTH',
]

const NO_VALUE = 'none'

interface InvoiceNewUIProps {
	organizationSlug: string
	values: NewInvoiceFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	projects: Project[]
	error: string | null
	isCreating: boolean
	isCustomerContextsLoading: boolean
	vatEnabled: boolean
	onChange: (patch: Partial<NewInvoiceFormValues>) => void
	onLineChange: (index: number, patch: Partial<InvoiceLineFormValues>) => void
	onAddLine: () => string
	onMoveLine: (index: number, delta: -1 | 1) => void
	onDuplicateLine: (index: number) => string
	onRemoveLine: (index: number) => void
	onSubmit: () => void
}

export function InvoiceNewUI({
	organizationSlug,
	values,
	customers,
	customerContexts,
	projects,
	error,
	isCreating,
	isCustomerContextsLoading,
	vatEnabled,
	onChange,
	onLineChange,
	onAddLine,
	onMoveLine,
	onDuplicateLine,
	onRemoveLine,
	onSubmit,
}: InvoiceNewUIProps) {
	const baseId = useId()
	const [submitted, setSubmitted] = useState(false)
	const errors = submitted ? validateNewInvoice(values) : {}
	const hasCustomer = values.customerId !== ''
	const noAddress =
		hasCustomer && !isCustomerContextsLoading && customerContexts.length === 0
	const totalCents = values.lines.reduce(
		(sum, line) => sum + invoiceLineTotalCents(line),
		0,
	)

	const submit = (event: FormEvent) => {
		event.preventDefault()
		setSubmitted(true)
		const firstId = firstInvalidInvoiceFieldId(validateNewInvoice(values))
		if (firstId) {
			document.getElementById(firstId)?.focus()
			return
		}
		onSubmit()
	}

	return (
		<PageShell className="max-w-5xl gap-0">
			<div className="mb-3">
				<BackLink to="/crm/factures" label="Factures" />
			</div>
			<PageHeader
				className="mb-6"
				title="Nouvelle facture"
				description="Choisis le client, le type de facture et ses lignes."
			/>
			<form noValidate onSubmit={submit} className="grid gap-5">
				<div className="grid gap-5 rounded-lg border bg-card p-5">
					<fieldset className="grid gap-3">
						<legend className="mb-1 font-medium text-sm">
							Type de facture
						</legend>
						<div className="grid gap-3 sm:grid-cols-3">
							{KIND_CHOICES.map((choice) => (
								<OptionCard
									key={choice.kind}
									name={`${baseId}-kind`}
									value={choice.kind}
									checked={values.kind === choice.kind}
									onSelect={() => onChange({ kind: choice.kind })}
									title={choice.title}
									description={choice.description}
								/>
							))}
						</div>
					</fieldset>
					<FormField
						id={NEW_INVOICE_FIELD_IDS.customerId}
						label="Client"
						required
						error={errors.customerId}
					>
						<CustomerPicker
							id={NEW_INVOICE_FIELD_IDS.customerId}
							customers={customers}
							value={hasCustomer ? values.customerId : null}
							invalid={errors.customerId !== undefined}
							onChange={(customerId) => onChange({ customerId })}
						/>
					</FormField>
					{noAddress && (
						<LockBanner
							tone="warn"
							title="La fiche de ce client est incomplète"
						>
							<p>
								Il n’a pas d’adresse de facturation : la facture ne pourra pas
								être émise.
							</p>
						</LockBanner>
					)}
					<FormField
						id={NEW_INVOICE_FIELD_IDS.customerContextId}
						label="Adresse de facturation"
						required
						error={errors.customerContextId}
					>
						<BillingAddressField
							id={NEW_INVOICE_FIELD_IDS.customerContextId}
							value={values.customerContextId}
							addresses={customerContexts}
							hasCustomer={hasCustomer}
							isLoading={isCustomerContextsLoading}
							onChange={(customerContextId) => onChange({ customerContextId })}
						/>
					</FormField>
					<div className="@container grid gap-5">
						<div className="grid gap-5 @md:grid-cols-3">
							<FormField id={`${baseId}-project`} label="Projet">
								<Select
									value={values.projectId || NO_VALUE}
									onValueChange={(projectId) =>
										onChange({
											projectId: projectId === NO_VALUE ? '' : projectId,
										})
									}
								>
									<SelectTrigger id={`${baseId}-project`} className="w-full">
										<SelectValue placeholder="Aucun projet" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value={NO_VALUE}>Aucun projet</SelectItem>
										{projects.map((project) => (
											<SelectItem key={project.id} value={project.id}>
												{project.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FormField>
							<FormField id={`${baseId}-title`} label="Objet">
								<Input
									id={`${baseId}-title`}
									value={values.title}
									onChange={(event) => onChange({ title: event.target.value })}
								/>
							</FormField>
							<FormField
								id={NEW_INVOICE_FIELD_IDS.terms}
								label="Délai de paiement (jours)"
								error={errors.terms}
								hint={`De 0 à ${MAX_PAYMENT_TERMS_DAYS} jours : 0 pour un paiement à réception. Vide, le délai du client ou des réglages s’applique.`}
							>
								<Input
									id={NEW_INVOICE_FIELD_IDS.terms}
									inputMode="numeric"
									autoComplete="off"
									placeholder="Réglages"
									value={values.termsDays}
									aria-invalid={errors.terms !== undefined || undefined}
									onChange={(event) =>
										onChange({ termsDays: event.target.value })
									}
									className="w-28 text-right tabular-nums"
								/>
							</FormField>
							<FormField id={`${baseId}-nature`} label="Nature de l’opération">
								<Select
									value={values.operationNature || NO_VALUE}
									onValueChange={(nature) =>
										onChange({
											operationNature:
												nature === NO_VALUE ? '' : (nature as OperationNature),
										})
									}
								>
									<SelectTrigger id={`${baseId}-nature`} className="w-full">
										<SelectValue placeholder="Non renseignée" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value={NO_VALUE}>Non renseignée</SelectItem>
										{OPERATION_NATURE_CHOICES.map((nature) => (
											<SelectItem key={nature} value={nature}>
												{operationNatureLabel(nature)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FormField>
						</div>
					</div>
					<FormField
						id={`${baseId}-notes`}
						label="Notes"
						hint="Affichées sur la facture, sous le détail des lignes."
					>
						<AutoTextarea
							id={`${baseId}-notes`}
							value={values.notes}
							placeholder="Mention à faire apparaître sur la facture"
							onChange={(event) => onChange({ notes: event.target.value })}
							className="min-h-20"
						/>
					</FormField>
				</div>

				<section
					id={NEW_INVOICE_FIELD_IDS.lines}
					aria-labelledby={`${baseId}-lines-title`}
					tabIndex={-1}
					className="grid gap-3 outline-none"
				>
					<div>
						<h2
							id={`${baseId}-lines-title`}
							className="font-semibold text-base"
						>
							Lignes
						</h2>
						<p className="mt-0.5 text-muted-foreground text-sm">
							Les montants sont en euros hors taxes.
						</p>
					</div>
					<LinesEditor
						lines={values.lines}
						idPrefix="new-invoice"
						vatEnabled={vatEnabled}
						totalOf={invoiceLineTotalCents}
						onPatch={onLineChange}
						onMove={onMoveLine}
						onDuplicate={onDuplicateLine}
						onRemove={onRemoveLine}
						onAdd={onAddLine}
					/>
					{errors.lines && (
						<p role="alert" className="text-[12px] text-state-alert">
							{errors.lines}
						</p>
					)}
					<dl className="ml-auto grid w-full max-w-sm gap-2 rounded-lg border bg-card p-4 text-sm">
						<div className="flex items-baseline justify-between gap-4">
							<dt className="text-muted-foreground">Total HT (aperçu)</dt>
							<dd className="font-semibold text-xl">
								<Money cents={totalCents} />
							</dd>
						</div>
						<p className="text-right text-muted-foreground text-xs italic">
							La TVA et le total TTC sont calculés à l’enregistrement.
						</p>
					</dl>
				</section>

				{error && (
					<p role="alert" className="text-sm text-state-alert">
						{error}
					</p>
				)}
				<div className="flex flex-wrap justify-end gap-2 border-t pt-4">
					<Button variant="outline" asChild>
						<Link to={buildOrgPath(organizationSlug, '/crm/factures')}>
							Annuler
						</Link>
					</Button>
					<RequirePermission permission="MANAGE_INVOICES">
						<Button type="submit" disabled={isCreating}>
							{isCreating ? <Loader2 className="animate-spin" /> : null}
							Créer le brouillon
						</Button>
					</RequirePermission>
				</div>
			</form>
		</PageShell>
	)
}
