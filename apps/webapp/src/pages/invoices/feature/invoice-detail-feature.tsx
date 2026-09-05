import { Link, useNavigate } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	Ban,
	Download,
	FileText,
	Loader2,
	Plus,
	Receipt,
	Send,
	Trash2,
} from 'lucide-react'
import type * as React from 'react'
import { useEffect, useRef, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from '#/components/ui/alert-dialog'
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
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	useCustomerContexts,
	useCustomers,
} from '#/hooks/use-customers'
import {
	type Invoice,
	type InvoicePayment,
	useCancelInvoice,
	useDeletePayment,
	useInvoice,
	useInvoiceBalance,
	useInvoiceCreditNotes,
	useInvoicePayments,
	useIssueCreditNote,
	useIssueInvoice,
	useRecordPayment,
	useUpdateInvoice,
} from '#/hooks/use-invoices'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { buildOrgPath } from '#/modules/org-path'
import {
	centsToEuros,
	customerDisplayName,
	emptyInvoiceLine,
	eurosToCents,
	formatDate,
	formatMoney,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceKindLabel,
	invoiceLineTotalCents,
	invoiceStatusLabel,
} from '#/pages/invoices/types'
import { InvoiceLineEditor } from '#/pages/invoices/ui/invoice-line-editor'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'

interface InvoiceDetailFeatureProps {
	invoiceId: string
}

export function InvoiceDetailFeature({ invoiceId }: InvoiceDetailFeatureProps) {
	const invoice = useInvoice(invoiceId)

	if (invoice.isLoading) return <InvoiceDetailLoading />

	if (invoice.isError) {
		return (
			<InvoiceDetailError
				title="Impossible de charger la facture"
				message={invoice.error.message}
				onRetry={() => void invoice.refetch()}
			/>
		)
	}

	if (!invoice.data?.data) {
		return (
			<InvoiceDetailError
				title="Facture introuvable"
				message="Aucune facture ne correspond à cet identifiant."
			/>
		)
	}

	return <InvoiceDetailWorkspace invoice={invoice.data.data} />
}

function InvoiceDetailWorkspace({ invoice }: { invoice: Invoice }) {
	const navigate = useNavigate()
	const { activeOrganization } = useActiveOrganization()
	const isDraft = invoice.status === 'DRAFT'
	const canCancel = invoice.status === 'DRAFT' || invoice.status === 'ISSUED'
	const canRecordPayment =
		invoice.status === 'ISSUED' || invoice.status === 'PARTIALLY_PAID'
	const canIssueCreditNote =
		invoice.status === 'ISSUED' ||
		invoice.status === 'PARTIALLY_PAID' ||
		invoice.status === 'PAID'

	const customers = useCustomers(invoice.organization_id)
	const [values, setValues] = useState<InvoiceFormValues>(() =>
		invoiceToForm(invoice),
	)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)
	const updateInvoice = useUpdateInvoice()
	const issueInvoice = useIssueInvoice()
	const cancelInvoice = useCancelInvoice()
	const balance = useInvoiceBalance(invoice.id)
	const payments = useInvoicePayments(invoice.id)
	const creditNotes = useInvoiceCreditNotes(invoice.id)
	const recordPayment = useRecordPayment()
	const deletePayment = useDeletePayment(invoice.id, invoice.organization_id)
	const issueCreditNote = useIssueCreditNote()
	const [isDownloading, setIsDownloading] = useState(false)
	const [downloadError, setDownloadError] = useState<string | null>(null)
	// Only one draft line open at a time, same reason `QuoteLineEditor` does
	// it: a long invoice stays readable.
	const [openLineId, setOpenLineId] = useState<string | null>(
		values.lines[0]?.clientId ?? null,
	)

	// Reloads the form when the server's invoice changes, same guard
	// `QuoteEditFeature` uses: without it, a catalogue/customer-contexts
	// fetch resolving after the invoice would overwrite what the user typed.
	const loadedVersion = useRef<string | null>(null)
	useEffect(() => {
		const version = `${invoice.id}:${invoice.updated_at}`
		if (loadedVersion.current === version) return
		loadedVersion.current = version
		setValues(invoiceToForm(invoice))
	}, [invoice])

	const updateValues = (patch: Partial<InvoiceFormValues>) => {
		setValues((current) => ({ ...current, ...patch }))
	}

	const updateLine = (index: number, patch: Partial<InvoiceLineFormValues>) => {
		setValues((current) => {
			const lines = [...current.lines]
			const line = lines[index]
			if (!line) return current
			lines[index] = { ...line, ...patch }
			return { ...current, lines }
		})
	}

	const addLine = () => {
		setValues((current) => ({
			...current,
			lines: [
				...current.lines,
				emptyInvoiceLine(`line-${Date.now()}-${current.lines.length}`),
			],
		}))
	}

	const removeLine = (index: number) => {
		setValues((current) => {
			const lines = current.lines.filter((_line, i) => i !== index)
			return { ...current, lines: lines.length ? lines : [emptyInvoiceLine()] }
		})
	}

	const totalCents = values.lines.reduce(
		(sum, line) => sum + invoiceLineTotalCents(line),
		0,
	)

	const canSave =
		isDraft &&
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.every((line) => {
			const quantity = Number(line.quantity.replace(',', '.'))
			return (
				line.label.trim() &&
				Number.isFinite(quantity) &&
				quantity > 0 &&
				line.unitPrice.trim()
			)
		})

	const saveInvoice = async () => {
		if (!canSave) return
		await updateInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
			body: {
				customer_id: values.customerId,
				customer_context_id: values.customerContextId,
				project_id: invoice.project_id ?? null,
				due_at: values.dueAt ? new Date(values.dueAt).toISOString() : null,
				notes: values.notes.trim() || null,
				operation_nature: values.operationNature || null,
				lines: values.lines.map((line) => ({
					label: line.label.trim(),
					quantity: line.quantity.replace(',', '.').trim(),
					unit_price_cents: eurosToCents(line.unitPrice),
					vat_rate_basis_points:
						line.vatRateBp === '' ? null : Number(line.vatRateBp),
				})),
			},
		})
	}

	const doIssue = () =>
		issueInvoice.mutateAsync({ path: { invoice_id: invoice.id }, body: {} })

	const doCancel = () =>
		cancelInvoice.mutateAsync({ path: { invoice_id: invoice.id } })

	const downloadPdf = async () => {
		setDownloadError(null)
		setIsDownloading(true)
		try {
			await downloadAuthenticatedFile(
				`/api/v1/invoices/${invoice.id}/pdf`,
				`${invoice.number ?? invoice.id}.pdf`,
			)
		} catch (err) {
			setDownloadError(
				err instanceof Error ? err.message : 'Téléchargement impossible',
			)
		} finally {
			setIsDownloading(false)
		}
	}

	const error =
		customers.error?.message ??
		customerContexts.error?.message ??
		updateInvoice.error?.message ??
		issueInvoice.error?.message ??
		cancelInvoice.error?.message ??
		balance.error?.message ??
		payments.error?.message ??
		creditNotes.error?.message ??
		recordPayment.error?.message ??
		deletePayment.error?.message ??
		issueCreditNote.error?.message ??
		downloadError

	return (
		<PageShell>
			<PageHeader
				eyebrow={invoice.number ?? 'Numéro attribué à l’émission'}
				title={invoiceKindLabel(invoice.kind)}
				description="Visualisez la facture, ses paiements et ses avoirs."
				actions={
					<div className="flex flex-col gap-2 sm:flex-row">
						<Button asChild variant="outline">
							<Link to={buildOrgPath(activeOrganization.slug, '/crm/invoices')}>
								<ArrowLeft />
								Retour
							</Link>
						</Button>
						<Button
							type="button"
							variant="outline"
							disabled={isDownloading}
							onClick={() => void downloadPdf()}
						>
							{isDownloading ? (
								<Loader2 className="animate-spin" />
							) : (
								<Download />
							)}
							PDF
						</Button>
						{canCancel ? (
							<RequirePermission permission="MANAGE_INVOICES">
								<AlertDialog>
									<AlertDialogTrigger asChild>
										<Button
											type="button"
											variant="destructive"
											disabled={cancelInvoice.isPending}
										>
											{cancelInvoice.isPending ? (
												<Loader2 className="animate-spin" />
											) : (
												<Ban />
											)}
											Annuler
										</Button>
									</AlertDialogTrigger>
									<AlertDialogContent>
										<AlertDialogHeader>
											<AlertDialogTitle>
												Annuler cette facture ?
											</AlertDialogTitle>
											<AlertDialogDescription>
												La facture {invoice.number ?? invoice.id} sera annulée.
												Cette action est irréversible.
											</AlertDialogDescription>
										</AlertDialogHeader>
										<AlertDialogFooter>
											<AlertDialogCancel disabled={cancelInvoice.isPending}>
												Retour
											</AlertDialogCancel>
											<AlertDialogAction
												disabled={cancelInvoice.isPending}
												className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
												onClick={(event) => {
													event.preventDefault()
													void doCancel()
												}}
											>
												{cancelInvoice.isPending ? (
													<Loader2 className="animate-spin" />
												) : (
													<Ban />
												)}
												Annuler la facture
											</AlertDialogAction>
										</AlertDialogFooter>
									</AlertDialogContent>
								</AlertDialog>
							</RequirePermission>
						) : null}
						{isDraft ? (
							<RequirePermission permission="MANAGE_INVOICES">
								<Button
									type="button"
									disabled={!canSave || updateInvoice.isPending}
									onClick={() => void saveInvoice()}
								>
									{updateInvoice.isPending ? (
										<Loader2 className="animate-spin" />
									) : (
										<FileText />
									)}
									Enregistrer
								</Button>
								<Button
									type="button"
									variant="default"
									disabled={!canSave || issueInvoice.isPending}
									onClick={() => void doIssue()}
								>
									{issueInvoice.isPending ? (
										<Loader2 className="animate-spin" />
									) : (
										<Send />
									)}
									Émettre
								</Button>
							</RequirePermission>
						) : null}
					</div>
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
							description={
								isDraft
									? 'Modifiable tant que la facture est un brouillon.'
									: 'Facture émise : les informations ne peuvent plus être modifiées.'
							}
						/>
						<div className="grid gap-4 p-5 md:grid-cols-2">
							<FieldBlock label="Statut">
								<StatusBadge tone={statusTone(invoice.status)}>
									{invoiceStatusLabel(invoice.status)}
								</StatusBadge>
							</FieldBlock>
							<FieldBlock label="Type">
								<p className="text-sm">{invoiceKindLabel(invoice.kind)}</p>
							</FieldBlock>
							<FieldBlock label="Client">
								{isDraft ? (
									<Select
										value={values.customerId}
										onValueChange={(customerId) =>
											updateValues({ customerId, customerContextId: '' })
										}
									>
										<SelectTrigger className="w-full">
											<SelectValue placeholder="Sélectionner un client" />
										</SelectTrigger>
										<SelectContent>
											{(customers.data?.data ?? []).map((customer) => (
												<SelectItem key={customer.id} value={customer.id}>
													{customerDisplayName(customer)}
												</SelectItem>
											))}
										</SelectContent>
									</Select>
								) : (
									<p className="text-sm">
										{customerName(
											customers.data?.data ?? [],
											invoice.customer_id,
										)}
									</p>
								)}
							</FieldBlock>
							<FieldBlock label="Adresse de facturation">
								{isDraft ? (
									<BillingAddressField
										value={values.customerContextId}
										addresses={customerContexts.data?.data ?? []}
										hasCustomer={Boolean(values.customerId)}
										isLoading={customerContexts.isLoading}
										onChange={(customerContextId) =>
											updateValues({ customerContextId })
										}
									/>
								) : (
									<p className="text-sm text-muted-foreground">
										{customerContexts.data?.data.find(
											(context) => context.id === invoice.customer_context_id,
										)?.label ?? 'Adresse enregistrée sur la facture'}
									</p>
								)}
							</FieldBlock>
							<FieldBlock label="Échéance">
								{isDraft ? (
									<Input
										type="date"
										value={values.dueAt}
										onChange={(event) =>
											updateValues({ dueAt: event.target.value })
										}
									/>
								) : (
									<p className="text-sm">
										{invoice.due_at
											? formatDate(invoice.due_at)
											: 'Sans échéance'}
									</p>
								)}
							</FieldBlock>
							<FieldBlock label="Note">
								{isDraft ? (
									<Textarea
										rows={3}
										value={values.notes}
										onChange={(event) =>
											updateValues({ notes: event.target.value })
										}
									/>
								) : (
									<p className="text-sm text-muted-foreground">
										{invoice.notes ?? 'Aucune note'}
									</p>
								)}
							</FieldBlock>
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader
							title={`Lignes (${isDraft ? values.lines.length : invoice.lines.length})`}
							description={
								isDraft ? 'Chaque ligne peut être modifiée.' : undefined
							}
							actions={
								isDraft ? (
									<Button
										type="button"
										variant="outline"
										size="sm"
										onClick={addLine}
									>
										<Plus />
										Ajouter
									</Button>
								) : undefined
							}
						/>
						{isDraft ? (
							<div className="divide-y">
								{values.lines.map((line, index) => (
									<InvoiceLineEditor
										key={line.clientId}
										index={index}
										line={line}
										isOpen={openLineId === line.clientId}
										canRemove={values.lines.length > 1}
										vatEnabled={
											activeOrganization.vat_status?.type === 'subject'
										}
										onOpenChange={(open) =>
											setOpenLineId(open ? line.clientId : null)
										}
										onChange={(patch) => updateLine(index, patch)}
										onRemove={() => removeLine(index)}
									/>
								))}
							</div>
						) : (
							<ul className="divide-y">
								{invoice.lines.map((line) => (
									<li
										key={line.id}
										className="flex items-center justify-between gap-4 px-5 py-3 text-sm"
									>
										<div className="min-w-0">
											<p className="truncate font-medium">{line.label}</p>
											<p className="text-xs text-muted-foreground">
												{line.quantity} × {formatMoney(line.unit_price_cents)}
											</p>
										</div>
										<p className="shrink-0 font-semibold tabular-nums">
											{formatMoney(
												invoiceLineTotalCents({
													clientId: line.id,
													label: line.label,
													quantity: line.quantity,
													unitPrice: centsToEuros(line.unit_price_cents),
													vatRateBp: '',
												}),
											)}
										</p>
									</li>
								))}
							</ul>
						)}
					</SectionCard>

					<PaymentsSection
						invoiceId={invoice.id}
						canRecordPayment={canRecordPayment}
						payments={payments.data?.data ?? []}
						isLoading={payments.isLoading}
						recordPayment={recordPayment}
						deletePayment={deletePayment}
					/>

					<CreditNotesSection
						organizationSlug={activeOrganization.slug}
						invoiceId={invoice.id}
						canIssueCreditNote={canIssueCreditNote}
						creditNotes={creditNotes.data?.data ?? []}
						isLoading={creditNotes.isLoading}
						issueCreditNote={issueCreditNote}
						navigate={navigate}
					/>
				</div>

				<aside className="h-fit space-y-5 xl:sticky xl:top-5">
					<div className="rounded-lg border bg-card p-5 shadow-sm">
						<p className="text-sm font-semibold">Totaux</p>
						<div className="mt-4 space-y-2 text-sm">
							<SummaryRow
								label={isDraft ? 'Total HT (aperçu)' : 'Total HT'}
								value={formatMoney(isDraft ? totalCents : invoice.net_cents)}
							/>
							{invoice.vat_breakdown.map((breakdown) => (
								<SummaryRow
									key={breakdown.rate_bp}
									label={`TVA ${(breakdown.rate_bp / 100).toFixed(2)} %`}
									value={formatMoney(breakdown.vat_cents)}
								/>
							))}
							<SummaryRow
								label="Total TTC"
								value={formatMoney(invoice.gross_cents)}
								strong
							/>
						</div>
					</div>

					<div className="rounded-lg bg-muted p-5 text-sm">
						<p className="text-xs font-medium text-muted-foreground">
							Ce qu'il reste à encaisser
						</p>
						{balance.data ? (
							<div className="mt-3 space-y-2">
								<SummaryRow
									label="Avoirs"
									value={formatMoney(Number(balance.data.data.credited_cents))}
								/>
								<SummaryRow
									label="Payé"
									value={formatMoney(Number(balance.data.data.paid_cents))}
								/>
								<SummaryRow
									label="Reste dû"
									value={formatMoney(Number(balance.data.data.remaining_cents))}
									strong
								/>
							</div>
						) : (
							<p className="mt-3 text-xs text-muted-foreground">Chargement…</p>
						)}
					</div>
				</aside>
			</div>
		</PageShell>
	)
}

function PaymentsSection({
	invoiceId,
	canRecordPayment,
	payments,
	isLoading,
	recordPayment,
	deletePayment,
}: {
	invoiceId: string
	canRecordPayment: boolean
	payments: InvoicePayment[]
	isLoading: boolean
	recordPayment: ReturnType<typeof useRecordPayment>
	deletePayment: ReturnType<typeof useDeletePayment>
}) {
	const [amount, setAmount] = useState('')
	const [paidOn, setPaidOn] = useState(() =>
		new Date().toISOString().slice(0, 10),
	)
	const [method, setMethod] = useState('')
	const [reference, setReference] = useState('')

	const canSubmit =
		Boolean(amount.trim()) && Boolean(paidOn) && Boolean(method.trim())

	const submit = async () => {
		if (!canSubmit) return
		await recordPayment.mutateAsync({
			path: { invoice_id: invoiceId },
			body: {
				amount_cents: eurosToCents(amount),
				paid_on: paidOn,
				method: method.trim(),
				reference: reference.trim() || null,
			},
		})
		setAmount('')
		setMethod('')
		setReference('')
	}

	return (
		<SectionCard>
			<SectionHeader
				title={`Paiements (${payments.length})`}
				description="Historique des encaissements enregistrés sur cette facture."
			/>
			{canRecordPayment ? (
				<div className="grid gap-3 border-b p-5 sm:grid-cols-[1fr_1fr_1fr_1fr_auto]">
					<Input
						inputMode="decimal"
						placeholder="Montant (€)"
						value={amount}
						onChange={(event) => setAmount(event.target.value)}
					/>
					<Input
						type="date"
						value={paidOn}
						onChange={(event) => setPaidOn(event.target.value)}
					/>
					<Input
						placeholder="Moyen (virement, chèque…)"
						value={method}
						onChange={(event) => setMethod(event.target.value)}
					/>
					<Input
						placeholder="Référence (optionnel)"
						value={reference}
						onChange={(event) => setReference(event.target.value)}
					/>
					<RequirePermission permission="MANAGE_INVOICES">
						<Button
							type="button"
							disabled={!canSubmit || recordPayment.isPending}
							onClick={() => void submit()}
						>
							{recordPayment.isPending ? (
								<Loader2 className="animate-spin" />
							) : (
								<Receipt />
							)}
							Enregistrer
						</Button>
					</RequirePermission>
				</div>
			) : null}
			{isLoading ? (
				<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
			) : payments.length === 0 ? (
				<p className="p-5 text-sm text-muted-foreground">
					Aucun paiement enregistré.
				</p>
			) : (
				<ul className="divide-y">
					{payments.map((payment) => (
						<li
							key={payment.id}
							className="flex items-center justify-between gap-4 px-5 py-3 text-sm"
						>
							<div className="min-w-0">
								<p className="font-medium">
									{formatMoney(payment.amount_cents)}
								</p>
								<p className="text-xs text-muted-foreground">
									{formatDate(payment.paid_on)} · {payment.method}
									{payment.reference ? ` · ${payment.reference}` : ''}
								</p>
							</div>
							<RequirePermission permission="MANAGE_INVOICES">
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									disabled={deletePayment.isPending}
									onClick={() =>
										void deletePayment.mutateAsync({
											path: { invoice_payment_id: payment.id },
										})
									}
								>
									<Trash2 />
									<span className="sr-only">Supprimer le paiement</span>
								</Button>
							</RequirePermission>
						</li>
					))}
				</ul>
			)}
		</SectionCard>
	)
}

function CreditNotesSection({
	organizationSlug,
	invoiceId,
	canIssueCreditNote,
	creditNotes,
	isLoading,
	issueCreditNote,
	navigate,
}: {
	organizationSlug: string
	invoiceId: string
	canIssueCreditNote: boolean
	creditNotes: Invoice[]
	isLoading: boolean
	issueCreditNote: ReturnType<typeof useIssueCreditNote>
	navigate: ReturnType<typeof useNavigate>
}) {
	const [label, setLabel] = useState('')
	const [amount, setAmount] = useState('')

	const canSubmit = Boolean(label.trim()) && Boolean(amount.trim())

	const submit = async () => {
		if (!canSubmit) return
		// A credit note's own lines carry a positive `unit_price_cents` —
		// `validate_line` in `domain::invoice::service` refuses a negative one
		// outright. The credit is expressed by the document's `kind`
		// (`CREDIT_NOTE`), which is what `sum_already_issued_cents` and this
		// page's own balance read subtract, not by the sign of the line.
		const created = await issueCreditNote.mutateAsync({
			path: { invoice_id: invoiceId },
			body: {
				lines: [
					{
						label: label.trim(),
						quantity: '1',
						unit_price_cents: eurosToCents(amount),
					},
				],
			},
		})
		setLabel('')
		setAmount('')
		await navigate({
			to: buildOrgPath(organizationSlug, '/crm/invoices/$invoiceId'),
			params: { invoiceId: created.data.id },
		})
	}

	return (
		<SectionCard>
			<SectionHeader
				title={`Avoirs (${creditNotes.length})`}
				description="Corrections émises contre cette facture."
			/>
			{canIssueCreditNote ? (
				<div className="grid gap-3 border-b p-5 sm:grid-cols-[1fr_1fr_auto]">
					<Input
						placeholder="Motif de l'avoir"
						value={label}
						onChange={(event) => setLabel(event.target.value)}
					/>
					<Input
						inputMode="decimal"
						placeholder="Montant à créditer (€)"
						value={amount}
						onChange={(event) => setAmount(event.target.value)}
					/>
					<RequirePermission permission="MANAGE_INVOICES">
						<Button
							type="button"
							disabled={!canSubmit || issueCreditNote.isPending}
							onClick={() => void submit()}
						>
							{issueCreditNote.isPending ? (
								<Loader2 className="animate-spin" />
							) : (
								<FileText />
							)}
							Émettre l'avoir
						</Button>
					</RequirePermission>
				</div>
			) : null}
			{isLoading ? (
				<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
			) : creditNotes.length === 0 ? (
				<p className="p-5 text-sm text-muted-foreground">Aucun avoir émis.</p>
			) : (
				<ul className="divide-y">
					{creditNotes.map((creditNote) => (
						<li key={creditNote.id}>
							<Link
								to={buildOrgPath(organizationSlug, '/crm/invoices/$invoiceId')}
								params={{ invoiceId: creditNote.id }}
								className="flex items-center justify-between gap-4 px-5 py-3 text-sm transition hover:bg-muted/40"
							>
								<div className="min-w-0">
									<p className="font-medium">
										{creditNote.number ?? 'Numéro attribué à l’émission'}
									</p>
									<p className="text-xs text-muted-foreground">
										{formatDate(creditNote.created_at)}
									</p>
								</div>
								<p className="font-semibold tabular-nums">
									{formatMoney(creditNote.gross_cents)}
								</p>
							</Link>
						</li>
					))}
				</ul>
			)}
		</SectionCard>
	)
}

function invoiceToForm(invoice: Invoice): InvoiceFormValues {
	const lines: InvoiceLineFormValues[] = invoice.lines.map((line, index) => ({
		clientId: line.id || `line-${index + 1}`,
		label: line.label,
		quantity: line.quantity,
		unitPrice: centsToEuros(line.unit_price_cents),
		vatRateBp:
			line.vat_rate_basis_points != null
				? String(line.vat_rate_basis_points)
				: '',
	}))

	return {
		kind: invoice.kind,
		projectId: invoice.project_id ?? '',
		customerId: invoice.customer_id,
		customerContextId: invoice.customer_context_id,
		dueAt: invoice.due_at ? invoice.due_at.slice(0, 10) : '',
		notes: invoice.notes ?? '',
		operationNature: invoice.operation_nature ?? '',
		lines: lines.length ? lines : [emptyInvoiceLine()],
	}
}

function customerName(customers: Customer[], customerId: string): string {
	const customer = customers.find((item) => item.id === customerId)
	return customer ? customerDisplayName(customer) : 'Client inconnu'
}

function statusTone(status: Invoice['status']) {
	if (status === 'PAID') return 'success'
	if (status === 'PARTIALLY_PAID') return 'warning'
	if (status === 'ISSUED') return 'brand'
	if (status === 'CANCELLED') return 'error'
	return 'neutral'
}

function InvoiceDetailLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement de la facture…
			</SectionCard>
		</PageShell>
	)
}

function InvoiceDetailError({
	title,
	message,
	onRetry,
}: {
	title: string
	message: string
	onRetry?: () => void
}) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<PageShell>
			<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
				<AlertCircle className="size-6 text-destructive" />
				<div>
					<p className="font-semibold">{title}</p>
					<p className="mt-1 text-sm text-muted-foreground">{message}</p>
				</div>
				<div className="flex gap-2">
					<Button asChild variant="outline">
						<Link to={buildOrgPath(activeOrganization.slug, '/crm/invoices')}>
							Retour aux factures
						</Link>
					</Button>
					{onRetry ? (
						<Button type="button" onClick={onRetry}>
							Réessayer
						</Button>
					) : null}
				</div>
			</SectionCard>
		</PageShell>
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
