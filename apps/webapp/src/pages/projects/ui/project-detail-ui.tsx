import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	Building2,
	Clock,
	Loader2,
	Receipt,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { Textarea } from '#/components/ui/textarea'
import type { CustomerContext } from '#/hooks/use-customers'
import type { Invoice, ProjectBillingSummary } from '#/hooks/use-invoices'
import type { Project } from '#/hooks/use-projects'
import type { Period, ProjectProfitability } from '#/hooks/use-reporting'
import { buildOrgPath } from '#/modules/org-path'
import {
	eurosToCents,
	formatDate,
	formatMoney,
	invoiceKindLabel,
	invoiceStatusLabel,
} from '#/pages/invoices/types'
import type { InvoicerMode } from '#/pages/projects/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { incompleteReason, plannedCostCents } from '#/pages/reporting/types'

export interface ProjectDetailUIProps {
	organizationSlug: string
	project: Project
	customerName: string | null

	billingSummary: ProjectBillingSummary | null
	isBillingSummaryLoading: boolean

	period: Period
	onPeriodChange: (period: Period) => void
	costRow: ProjectProfitability | null
	isCostLoading: boolean

	invoices: Invoice[]
	isInvoicesLoading: boolean

	hasCustomer: boolean
	hasPinnedAddress: boolean
	hasQuote: boolean

	mode: InvoicerMode
	onModeChange: (mode: InvoicerMode) => void

	percentageInput: string
	onPercentageInputChange: (value: string) => void
	percentagePreviewCents: number | null

	freeAmountInput: string
	onFreeAmountInputChange: (value: string) => void
	freeLabel: string
	onFreeLabelChange: (value: string) => void
	freeCustomerContexts: CustomerContext[]
	isFreeCustomerContextsLoading: boolean
	freeCustomerContextId: string
	onFreeCustomerContextIdChange: (value: string) => void

	dueAt: string
	onDueAtChange: (value: string) => void
	notes: string
	onNotesChange: (value: string) => void

	isIssuing: boolean
	error: string | null
	onConfirm: () => void
}

/**
 * A project's own page: what it was quoted, what has been billed, what
 * remains, what it costs, its invoices, and the Invoicer that bills it.
 *
 * Presentational only — every figure was computed by the backend (the
 * billing summary, the profitability report), this file only formats and
 * lays them out.
 */
export function ProjectDetailUI({
	organizationSlug,
	project,
	customerName,
	billingSummary,
	isBillingSummaryLoading,
	period,
	onPeriodChange,
	costRow,
	isCostLoading,
	invoices,
	isInvoicesLoading,
	hasCustomer,
	hasPinnedAddress,
	hasQuote,
	mode,
	onModeChange,
	percentageInput,
	onPercentageInputChange,
	percentagePreviewCents,
	freeAmountInput,
	onFreeAmountInputChange,
	freeLabel,
	onFreeLabelChange,
	freeCustomerContexts,
	isFreeCustomerContextsLoading,
	freeCustomerContextId,
	onFreeCustomerContextIdChange,
	dueAt,
	onDueAtChange,
	notes,
	onNotesChange,
	isIssuing,
	error,
	onConfirm,
}: ProjectDetailUIProps) {
	const plannedCost = costRow ? plannedCostCents(costRow) : 0
	const dueAtLabel = dueAt
		? formatDate(new Date(dueAt).toISOString())
		: 'Sans échéance'

	return (
		<PageShell>
			<PageHeader
				eyebrow={customerName ?? 'Projet interne'}
				title={project.name}
				description="Ce qui a été devisé, ce qui a été facturé, ce qu'il reste à facturer, et ce que le projet coûte d'après le planning."
				actions={
					<Button asChild variant="outline">
						<Link to={buildOrgPath(organizationSlug, '/planning/projects')}>
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

			<section className="grid grid-cols-2 gap-4 lg:grid-cols-4">
				<MetricCard
					label="Devisé"
					value={
						isBillingSummaryLoading
							? '…'
							: billingSummary?.quoted_cents == null
								? '—'
								: formatMoney(billingSummary.quoted_cents)
					}
					hint={
						billingSummary?.quoted_cents == null ? 'Aucun devis' : undefined
					}
				/>
				<MetricCard
					label="Facturé"
					value={
						isBillingSummaryLoading
							? '…'
							: formatMoney(billingSummary?.billed_cents ?? 0)
					}
					hint="Factures émises, avoirs déduits"
				/>
				<MetricCard
					label="Reste à facturer"
					value={
						isBillingSummaryLoading
							? '…'
							: billingSummary?.remaining_cents == null
								? '—'
								: formatMoney(billingSummary.remaining_cents)
					}
					hint={
						billingSummary?.remaining_cents != null &&
						billingSummary.remaining_cents < 0
							? 'Déjà facturé au-delà du devis'
							: undefined
					}
				/>
				{/* No `VIEW_COST` gate yet: epic #283 has not landed on this branch.
				 * Shown to everyone for now — see the commit message. */}
				<MetricCard
					label="Coût planifié"
					value={isCostLoading ? '…' : formatMoney(plannedCost)}
					hint={
						costRow
							? (incompleteReason(costRow) ?? undefined)
							: 'Sur la période choisie'
					}
					icon={<Clock className="size-4" />}
				/>
			</section>

			<SectionCard>
				<SectionHeader
					title="Période du coût"
					description="Le coût planifié ci-dessus porte sur cette période — comme sur l'écran de rentabilité."
					actions={
						<div className="flex flex-wrap items-end gap-2">
							<div className="space-y-1">
								<Label
									htmlFor="cost-from"
									className="text-xs text-muted-foreground"
								>
									Du
								</Label>
								<Input
									id="cost-from"
									type="date"
									className="w-40"
									value={period.from}
									onChange={(event) =>
										onPeriodChange({ ...period, from: event.target.value })
									}
								/>
							</div>
							<div className="space-y-1">
								<Label
									htmlFor="cost-to"
									className="text-xs text-muted-foreground"
								>
									Au
								</Label>
								<Input
									id="cost-to"
									type="date"
									className="w-40"
									value={period.to}
									onChange={(event) =>
										onPeriodChange({ ...period, to: event.target.value })
									}
								/>
							</div>
						</div>
					}
				/>
			</SectionCard>

			<SectionCard>
				<SectionHeader
					title="Facturer"
					description="Ce que contiendra la facture est prévisualisé avant toute création."
				/>
				<div className="p-5">
					{!hasCustomer ? (
						<p className="flex items-center gap-2 text-sm text-muted-foreground">
							<Building2 className="size-4" />
							Projet interne : sans client, rien à facturer.
						</p>
					) : (
						<Invoicer
							mode={mode}
							onModeChange={onModeChange}
							hasQuote={hasQuote}
							hasPinnedAddress={hasPinnedAddress}
							billingSummary={billingSummary}
							percentageInput={percentageInput}
							onPercentageInputChange={onPercentageInputChange}
							percentagePreviewCents={percentagePreviewCents}
							freeAmountInput={freeAmountInput}
							onFreeAmountInputChange={onFreeAmountInputChange}
							freeLabel={freeLabel}
							onFreeLabelChange={onFreeLabelChange}
							freeCustomerContexts={freeCustomerContexts}
							isFreeCustomerContextsLoading={isFreeCustomerContextsLoading}
							freeCustomerContextId={freeCustomerContextId}
							onFreeCustomerContextIdChange={onFreeCustomerContextIdChange}
							dueAt={dueAt}
							onDueAtChange={onDueAtChange}
							dueAtLabel={dueAtLabel}
							notes={notes}
							onNotesChange={onNotesChange}
							isIssuing={isIssuing}
							onConfirm={onConfirm}
						/>
					)}
				</div>
			</SectionCard>

			<SectionCard>
				<SectionHeader
					title={`Factures (${invoices.length})`}
					description="Toutes les factures émises ou en brouillon contre ce projet."
				/>
				{isInvoicesLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : invoices.length === 0 ? (
					<p className="p-5 text-sm text-muted-foreground">
						Aucune facture pour l'instant.
					</p>
				) : (
					<ul className="divide-y">
						{invoices.map((invoice) => (
							<li key={invoice.id}>
								<Link
									to={buildOrgPath(
										organizationSlug,
										'/crm/invoices/$invoiceId',
									)}
									params={{ invoiceId: invoice.id }}
									className="flex items-center justify-between gap-4 px-5 py-3 text-sm transition hover:bg-muted/40"
								>
									<div className="min-w-0">
										<p className="truncate font-medium">
											{invoice.number ?? 'Numéro attribué à l’émission'}
										</p>
										<p className="text-xs text-muted-foreground">
											{invoiceKindLabel(invoice.kind)} ·{' '}
											{formatDate(invoice.created_at)}
										</p>
									</div>
									<div className="flex shrink-0 items-center gap-3">
										<StatusBadge tone={statusTone(invoice.status)}>
											{invoiceStatusLabel(invoice.status)}
										</StatusBadge>
										<p className="font-semibold tabular-nums">
											{formatMoney(invoice.gross_cents)}
										</p>
									</div>
								</Link>
							</li>
						))}
					</ul>
				)}
			</SectionCard>
		</PageShell>
	)
}

interface InvoicerProps {
	mode: InvoicerMode
	onModeChange: (mode: InvoicerMode) => void
	hasQuote: boolean
	hasPinnedAddress: boolean
	billingSummary: ProjectBillingSummary | null
	percentageInput: string
	onPercentageInputChange: (value: string) => void
	percentagePreviewCents: number | null
	freeAmountInput: string
	onFreeAmountInputChange: (value: string) => void
	freeLabel: string
	onFreeLabelChange: (value: string) => void
	freeCustomerContexts: CustomerContext[]
	isFreeCustomerContextsLoading: boolean
	freeCustomerContextId: string
	onFreeCustomerContextIdChange: (value: string) => void
	dueAt: string
	onDueAtChange: (value: string) => void
	dueAtLabel: string
	notes: string
	onNotesChange: (value: string) => void
	isIssuing: boolean
	onConfirm: () => void
}

/**
 * Three ways to bill a project, each previewed before anything is created.
 * `REMAINING`/`PERCENTAGE` need a quote (and a pinned billing address on the
 * project — the same precondition `issue_deposit`/`issue_final_invoice`
 * enforce server-side); `FREE_AMOUNT` needs neither, and picks its own
 * address the way any manually composed invoice does.
 */
function Invoicer({
	mode,
	onModeChange,
	hasQuote,
	hasPinnedAddress,
	billingSummary,
	percentageInput,
	onPercentageInputChange,
	percentagePreviewCents,
	freeAmountInput,
	onFreeAmountInputChange,
	freeLabel,
	onFreeLabelChange,
	freeCustomerContexts,
	isFreeCustomerContextsLoading,
	freeCustomerContextId,
	onFreeCustomerContextIdChange,
	dueAt,
	onDueAtChange,
	dueAtLabel,
	notes,
	onNotesChange,
	isIssuing,
	onConfirm,
}: InvoicerProps) {
	const canInvoiceQuoted = hasQuote && hasPinnedAddress
	const remainingCents = billingSummary?.remaining_cents ?? null

	const canConfirm =
		mode === 'REMAINING'
			? canInvoiceQuoted && remainingCents !== null
			: mode === 'PERCENTAGE'
				? canInvoiceQuoted && percentagePreviewCents !== null
				: Boolean(
						freeCustomerContextId && freeLabel.trim() && freeAmountInput.trim(),
					)

	return (
		<div className="space-y-4">
			<Tabs
				value={mode}
				onValueChange={(value) => onModeChange(value as InvoicerMode)}
			>
				<TabsList>
					<TabsTrigger value="REMAINING">Tout le restant</TabsTrigger>
					<TabsTrigger value="PERCENTAGE">Un pourcentage</TabsTrigger>
					<TabsTrigger value="FREE_AMOUNT">Montant libre</TabsTrigger>
				</TabsList>

				<TabsContent value="REMAINING" className="space-y-3">
					<UnavailableNotice
						hasQuote={hasQuote}
						hasPinnedAddress={hasPinnedAddress}
					/>
					{canInvoiceQuoted ? (
						<PreviewRow
							label="Solde (tout ce qui reste à facturer)"
							amountCents={remainingCents}
							dueAtLabel={dueAtLabel}
						/>
					) : null}
				</TabsContent>

				<TabsContent value="PERCENTAGE" className="space-y-3">
					<UnavailableNotice
						hasQuote={hasQuote}
						hasPinnedAddress={hasPinnedAddress}
					/>
					{canInvoiceQuoted ? (
						<>
							<div className="max-w-40 space-y-1">
								<Label htmlFor="invoicer-percentage">
									Pourcentage du devis
								</Label>
								<div className="relative">
									<Input
										id="invoicer-percentage"
										inputMode="decimal"
										placeholder="30"
										value={percentageInput}
										onChange={(event) =>
											onPercentageInputChange(event.target.value)
										}
									/>
									<span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-sm text-muted-foreground">
										%
									</span>
								</div>
							</div>
							<PreviewRow
								label={`Acompte (${percentageInput || '0'} %)`}
								amountCents={percentagePreviewCents}
								dueAtLabel={dueAtLabel}
							/>
						</>
					) : null}
				</TabsContent>

				<TabsContent value="FREE_AMOUNT" className="space-y-3">
					<div className="grid gap-3 sm:grid-cols-2">
						<div className="space-y-1">
							<Label htmlFor="invoicer-free-label">Libellé de la ligne</Label>
							<Input
								id="invoicer-free-label"
								value={freeLabel}
								onChange={(event) => onFreeLabelChange(event.target.value)}
								placeholder="Travaux supplémentaires…"
							/>
						</div>
						<div className="space-y-1">
							<Label htmlFor="invoicer-free-amount">Montant (€)</Label>
							<Input
								id="invoicer-free-amount"
								inputMode="decimal"
								value={freeAmountInput}
								onChange={(event) =>
									onFreeAmountInputChange(event.target.value)
								}
								placeholder="0,00"
							/>
						</div>
						<div className="space-y-1 sm:col-span-2">
							<Label htmlFor="invoicer-free-address">
								Adresse de facturation
							</Label>
							<BillingAddressField
								value={freeCustomerContextId}
								addresses={freeCustomerContexts}
								hasCustomer
								isLoading={isFreeCustomerContextsLoading}
								onChange={onFreeCustomerContextIdChange}
							/>
						</div>
					</div>
					<PreviewRow
						label={freeLabel.trim() || 'Facture'}
						amountCents={
							freeAmountInput.trim() ? eurosToCents(freeAmountInput) : null
						}
						dueAtLabel={dueAtLabel}
					/>
				</TabsContent>
			</Tabs>

			<div className="grid gap-3 border-t pt-4 sm:grid-cols-2">
				<div className="space-y-1">
					<Label htmlFor="invoicer-due-at">Échéance (optionnel)</Label>
					<Input
						id="invoicer-due-at"
						type="date"
						value={dueAt}
						onChange={(event) => onDueAtChange(event.target.value)}
					/>
				</div>
				<div className="space-y-1">
					<Label htmlFor="invoicer-notes">Note (optionnel)</Label>
					<Textarea
						id="invoicer-notes"
						rows={1}
						value={notes}
						onChange={(event) => onNotesChange(event.target.value)}
					/>
				</div>
			</div>

			<Button
				type="button"
				disabled={!canConfirm || isIssuing}
				onClick={onConfirm}
			>
				{isIssuing ? <Loader2 className="animate-spin" /> : <Receipt />}
				Émettre la facture
			</Button>
		</div>
	)
}

function UnavailableNotice({
	hasQuote,
	hasPinnedAddress,
}: {
	hasQuote: boolean
	hasPinnedAddress: boolean
}) {
	if (!hasQuote) {
		return (
			<p className="flex items-center gap-2 text-sm text-amber-600 dark:text-amber-500">
				<AlertCircle className="size-4 shrink-0" />
				Ce projet n'a pas de devis : impossible de calculer un acompte ou un
				solde.
			</p>
		)
	}

	if (!hasPinnedAddress) {
		return (
			<p className="flex items-center gap-2 text-sm text-amber-600 dark:text-amber-500">
				<AlertCircle className="size-4 shrink-0" />
				Ce projet n'a pas d'adresse de facturation épinglée. Ouvrez-le en
				édition depuis la liste des projets pour en choisir une.
			</p>
		)
	}

	return null
}

/**
 * Display-only, never trusted for the write itself: confirming re-reads
 * whatever the corresponding mutation actually returns from the server.
 */
function PreviewRow({
	label,
	amountCents,
	dueAtLabel,
}: {
	label: string
	amountCents: number | null
	dueAtLabel: string
}) {
	return (
		<div className="rounded-lg bg-muted p-4 text-sm">
			<div className="flex items-center justify-between gap-4">
				<span className="min-w-0 truncate text-muted-foreground">{label}</span>
				<span className="shrink-0 font-semibold tabular-nums">
					{amountCents === null ? '—' : formatMoney(amountCents)}
				</span>
			</div>
			<p className="mt-1 text-xs text-muted-foreground">
				Échéance : {dueAtLabel}
			</p>
		</div>
	)
}

function statusTone(status: Invoice['status']) {
	if (status === 'PAID') return 'success'
	if (status === 'PARTIALLY_PAID') return 'warning'
	if (status === 'ISSUED') return 'brand'
	if (status === 'CANCELLED') return 'error'
	return 'neutral'
}
