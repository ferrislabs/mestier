import { Link } from '@tanstack/react-router'
import { ArrowLeft, FileText, Loader2, Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { InvoiceKind, OperationNature } from '#/hooks/use-invoices'
import type { Project } from '#/hooks/use-projects'
import { buildOrgPath } from '#/modules/org-path'
import {
	customerDisplayName,
	formatMoney,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceKindLabel,
	invoiceLineTotalCents,
	operationNatureLabel,
} from '#/pages/invoices/types'
import { InvoiceLineEditor } from '#/pages/invoices/ui/invoice-line-editor'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'

const INVOICE_KIND_CHOICES: InvoiceKind[] = ['STANDARD', 'DEPOSIT', 'FINAL']
const OPERATION_NATURE_CHOICES: OperationNature[] = [
	'GOODS',
	'SERVICES',
	'BOTH',
]

interface InvoiceNewUIProps {
	organizationSlug: string
	values: InvoiceFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	projects: Project[]
	error: string | null
	isCreating: boolean
	isCustomerContextsLoading: boolean
	vatEnabled: boolean
	onChange: (patch: Partial<InvoiceFormValues>) => void
	onLineChange: (index: number, patch: Partial<InvoiceLineFormValues>) => void
	onAddLine: () => void
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
	onRemoveLine,
	onSubmit,
}: InvoiceNewUIProps) {
	const [openLineId, setOpenLineId] = useState<string | null>(
		values.lines[0]?.clientId ?? null,
	)

	const totalCents = values.lines.reduce(
		(sum, line) => sum + invoiceLineTotalCents(line),
		0,
	)

	const canSave =
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.every((line) => {
			const quantity = Number(line.quantity.replace(',', '.'))
			return (
				line.label.trim() &&
				Number.isFinite(quantity) &&
				quantity > 0 &&
				line.unitPrice.trim() &&
				eurosToCentsIsValid(line.unitPrice)
			)
		})

	return (
		<PageShell>
			<PageHeader
				eyebrow="Nouvelle facture"
				title="Composer une facture"
				description="Choisissez le client, la nature de la facture et ses lignes."
				actions={
					<Button asChild variant="outline">
						<Link to={buildOrgPath(organizationSlug, '/crm/invoices')}>
							<ArrowLeft />
							Retour
						</Link>
					</Button>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
				<div className="space-y-5">
					<SectionCard>
						<SectionHeader
							title="Informations"
							description="Type de document, client et adresse de facturation."
						/>
						<div className="grid gap-4 p-5 md:grid-cols-2">
							<FieldBlock label="Type de facture">
								<Select
									value={values.kind}
									onValueChange={(kind) =>
										onChange({ kind: kind as InvoiceKind })
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{INVOICE_KIND_CHOICES.map((kind) => (
											<SelectItem key={kind} value={kind}>
												{invoiceKindLabel(kind)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Projet (optionnel)">
								<Select
									value={values.projectId || 'none'}
									onValueChange={(projectId) =>
										onChange({
											projectId: projectId === 'none' ? '' : projectId,
										})
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue placeholder="Aucun projet" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="none">Aucun projet</SelectItem>
										{projects.map((project) => (
											<SelectItem key={project.id} value={project.id}>
												{project.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Client">
								<Select
									value={values.customerId}
									onValueChange={(customerId) => onChange({ customerId })}
								>
									<SelectTrigger className="w-full">
										<SelectValue placeholder="Sélectionner un client" />
									</SelectTrigger>
									<SelectContent>
										{customers.map((customer) => (
											<SelectItem key={customer.id} value={customer.id}>
												{customerDisplayName(customer)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Adresse de facturation">
								<BillingAddressField
									value={values.customerContextId}
									addresses={customerContexts}
									hasCustomer={Boolean(values.customerId)}
									isLoading={isCustomerContextsLoading}
									onChange={(customerContextId) =>
										onChange({ customerContextId })
									}
								/>
							</FieldBlock>
							<FieldBlock label="Échéance (optionnel)">
								<Input
									type="date"
									value={values.dueAt}
									onChange={(event) => onChange({ dueAt: event.target.value })}
								/>
							</FieldBlock>
							<FieldBlock label="Nature de l'opération (optionnel)">
								<Select
									value={values.operationNature || 'none'}
									onValueChange={(nature) =>
										onChange({
											operationNature:
												nature === 'none' ? '' : (nature as OperationNature),
										})
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue placeholder="Non renseignée" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="none">Non renseignée</SelectItem>
										{OPERATION_NATURE_CHOICES.map((nature) => (
											<SelectItem key={nature} value={nature}>
												{operationNatureLabel(nature)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Note (optionnel)">
								<Textarea
									rows={3}
									value={values.notes}
									onChange={(event) => onChange({ notes: event.target.value })}
									placeholder="Mention à faire apparaître sur la facture"
								/>
							</FieldBlock>
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader
							title={`Lignes (${values.lines.length})`}
							description="Chaque ligne facturée."
							actions={
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={onAddLine}
								>
									<Plus />
									Ajouter
								</Button>
							}
						/>
						<div className="divide-y">
							{values.lines.map((line, index) => (
								<InvoiceLineEditor
									key={line.clientId}
									index={index}
									line={line}
									isOpen={openLineId === line.clientId}
									canRemove={values.lines.length > 1}
									vatEnabled={vatEnabled}
									onOpenChange={(open) =>
										setOpenLineId(open ? line.clientId : null)
									}
									onChange={(patch) => onLineChange(index, patch)}
									onRemove={() => onRemoveLine(index)}
								/>
							))}
						</div>
					</SectionCard>
				</div>

				<aside className="h-fit rounded-lg border bg-card p-5 shadow-sm xl:sticky xl:top-5">
					<p className="text-sm font-semibold">Résumé</p>
					<div className="mt-4 space-y-3 text-sm">
						<SummaryRow label="Type" value={invoiceKindLabel(values.kind)} />
						<SummaryRow label="Lignes" value={String(values.lines.length)} />
						<SummaryRow
							label="Total HT (aperçu)"
							value={formatMoney(totalCents)}
							strong
						/>
					</div>
					<Button
						type="button"
						className="mt-5 w-full"
						disabled={!canSave || isCreating}
						onClick={onSubmit}
					>
						{isCreating ? <Loader2 className="animate-spin" /> : <FileText />}
						Créer le brouillon
					</Button>
				</aside>
			</div>
		</PageShell>
	)
}

function eurosToCentsIsValid(value: string): boolean {
	const normalized = value.replace(',', '.').trim()
	return (
		/^\d*\.?\d*$/.test(normalized) && normalized !== '.' && normalized !== ''
	)
}

function FieldBlock({
	label,
	children,
}: {
	label: string
	children: React.ReactNode
}) {
	return (
		<div className="flex min-w-0 flex-col gap-2">
			<Label>{label}</Label>
			{children}
		</div>
	)
}

function SummaryRow({
	label,
	value,
	strong,
}: {
	label: string
	value: string
	strong?: boolean
}) {
	return (
		<div className="flex items-center justify-between gap-4">
			<span className="text-muted-foreground">{label}</span>
			<span className={strong ? 'font-semibold' : 'truncate font-medium'}>
				{value}
			</span>
		</div>
	)
}
