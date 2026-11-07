import { Lock } from 'lucide-react'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type {
	Invoice,
	InvoiceBalance,
	InvoicePayment,
} from '#/hooks/use-invoices'
import type { InvoiceActivityRecord } from '#/pages/invoices/feature/mocks/invoice-activity'
import type { RefundRecord } from '#/pages/invoices/feature/mocks/invoice-extras'
import type { creditNoteLines } from '#/pages/invoices/lib/credit-note-request'
import type { InvoiceActions } from '#/pages/invoices/lib/invoice-actions'
import type { PaymentValues } from '#/pages/invoices/lib/payment-form'
import {
	customerDisplayName,
	type InvoiceDisplayState,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceKindLabel,
} from '#/pages/invoices/types'
import { InvoiceActionsBar } from '#/pages/invoices/ui/invoice-actions-bar'
import { InvoiceDraftForm } from '#/pages/invoices/ui/invoice-draft-form'
import {
	DraftSidebar,
	IssuedSidebar,
} from '#/pages/invoices/ui/invoice-sidebar'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { DocBadge, InvoiceStateBadge } from '#/pages/quotes/ui/doc/badges'
import { BackLink, DetailShell } from '#/pages/quotes/ui/doc/detail-shell'
import { AccountLink } from '#/pages/quotes/ui/doc/doc-links'
import {
	DocumentPaper,
	type PaperDocument,
} from '#/pages/quotes/ui/doc/document-paper'
import type { HistoryEntry } from '#/pages/quotes/ui/doc/history-list'
import { LockBanner } from '#/pages/quotes/ui/doc/primitives'
import type { DocTotals } from '#/pages/quotes/ui/doc/totals-panel'

export type InvoiceView = 'edition' | 'apercu'

function isView(value: string): value is InvoiceView {
	return value === 'edition' || value === 'apercu'
}

interface InvoiceDetailUIProps {
	invoice: Invoice
	view: InvoiceView
	values: InvoiceFormValues
	isDirty: boolean
	customer: Customer | undefined
	customers: Customer[]
	customerContexts: CustomerContext[]
	isContextsLoading: boolean
	project: { id: string; name: string } | null
	vatEnabled: boolean
	displayState: InvoiceDisplayState
	actions: InvoiceActions
	balance: InvoiceBalance | undefined
	remainingCents: number | null
	refunds: RefundRecord[]
	quote: { id: string; reference: string | null } | null
	issueBounds: { min: string | null; max: string }
	payments: InvoicePayment[]
	paymentsLoading: boolean
	creditNotes: Invoice[]
	creditNotesLoading: boolean
	activity: InvoiceActivityRecord
	paper: PaperDocument
	history: HistoryEntry[]
	totals: DocTotals
	vatNotice: string | null
	error: string | null
	canSave: boolean
	isSaving: boolean
	isIssuing: boolean
	isCancelling: boolean
	isRecordingPayment: boolean
	isIssuingCreditNote: boolean
	isDeletingPayment: boolean
	isSending: boolean
	isReminding: boolean
	isDownloading: boolean
	issueError: string | null
	paymentError: string | null
	creditNoteError: string | null
	onViewChange: (view: InvoiceView) => void
	onChange: (patch: Partial<InvoiceFormValues>) => void
	onLineChange: (index: number, patch: Partial<InvoiceLineFormValues>) => void
	onAddLine: () => string
	onMoveLine: (index: number, delta: -1 | 1) => void
	onDuplicateLine: (index: number) => string
	onRemoveLine: (index: number) => void
	onSave: () => void
	onIssue: (issueDate: string) => Promise<boolean>
	onCancel: () => Promise<boolean>
	onDeleteDraft: () => void
	onRefund: (values: PaymentValues) => Promise<boolean>
	onRecordPayment: (values: PaymentValues) => Promise<boolean>
	onDeletePayment: (paymentId: string) => void
	onIssueCreditNote: (request: {
		reason: string
		lines: ReturnType<typeof creditNoteLines>
	}) => Promise<boolean>
	onSend: () => Promise<string>
	onRemind: () => Promise<string>
	onSaveNote: (text: string) => void
	onDownloadPdf: () => void
}

export function InvoiceDetailUI(props: InvoiceDetailUIProps) {
	const {
		invoice,
		view,
		values,
		isDirty,
		customer,
		customers,
		customerContexts,
		isContextsLoading,
		project,
		vatEnabled,
		displayState,
		paper,
		history,
		totals,
		vatNotice,
		error,
	} = props
	const draft = invoice.status === 'DRAFT'
	const locked = !draft
	const remainingCents = props.remainingCents

	const main = (
		<>
			<header className="grid gap-3">
				<BackLink to="/crm/factures" label="Factures" />
				<div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
					{locked && (
						<Lock
							className="size-5 text-muted-foreground"
							role="img"
							aria-label="Facture verrouillée"
						/>
					)}
					<h1 className="font-semibold text-2xl tracking-tight">
						{invoice.number ?? 'Brouillon'}
					</h1>
					<InvoiceStateBadge state={displayState} />
					{(invoice.kind === 'DEPOSIT' || invoice.kind === 'FINAL') && (
						<DocBadge tone="neutral">{invoiceKindLabel(invoice.kind)}</DocBadge>
					)}
				</div>
				<p className="text-muted-foreground text-sm">
					{customer ? (
						<AccountLink customerId={customer.id} className="text-foreground">
							{customerDisplayName(customer)}
						</AccountLink>
					) : (
						'Client inconnu'
					)}
				</p>
				<InvoiceActionsBar
					invoice={invoice}
					actions={props.actions}
					remainingCents={remainingCents}
					issueBounds={props.issueBounds}
					isDirty={isDirty}
					canSave={props.canSave}
					isSaving={props.isSaving}
					isIssuing={props.isIssuing}
					isCancelling={props.isCancelling}
					isRecordingPayment={props.isRecordingPayment}
					isIssuingCreditNote={props.isIssuingCreditNote}
					isSending={props.isSending}
					isReminding={props.isReminding}
					isDownloading={props.isDownloading}
					issueError={props.issueError}
					paymentError={props.paymentError}
					creditNoteError={props.creditNoteError}
					onSave={props.onSave}
					onIssue={props.onIssue}
					onCancel={props.onCancel}
					onDeleteDraft={props.onDeleteDraft}
					onRefund={props.onRefund}
					onRecordPayment={props.onRecordPayment}
					onIssueCreditNote={props.onIssueCreditNote}
					onSend={props.onSend}
					onRemind={props.onRemind}
					onDownloadPdf={props.onDownloadPdf}
				/>
			</header>

			{error ? (
				<div
					role="alert"
					className="rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm text-state-alert"
				>
					{error}
				</div>
			) : null}

			{draft ? (
				<>
					<ToggleGroup
						type="single"
						variant="outline"
						size="sm"
						value={view}
						onValueChange={(value) => {
							if (isView(value)) props.onViewChange(value)
						}}
						aria-label="Mode d’affichage"
					>
						<ToggleGroupItem value="edition">Édition</ToggleGroupItem>
						<ToggleGroupItem value="apercu">Aperçu</ToggleGroupItem>
					</ToggleGroup>
					{view === 'apercu' ? (
						<>
							{isDirty && (
								<LockBanner tone="info" title="Aperçu des données enregistrées">
									Enregistre la facture pour y voir tes dernières modifications.
								</LockBanner>
							)}
							<DocumentPaper document={paper} />
						</>
					) : customer ? (
						<InvoiceDraftForm
							idPrefix={`invoice-${invoice.id}`}
							values={values}
							customers={customers}
							customerContexts={customerContexts}
							isContextsLoading={isContextsLoading}
							vatEnabled={vatEnabled}
							totals={totals}
							vatNotice={vatNotice}
							unsavedNotice={
								isDirty
									? 'Totaux enregistrés : enregistre pour les mettre à jour'
									: null
							}
							onChange={props.onChange}
							onLineChange={props.onLineChange}
							onAddLine={props.onAddLine}
							onMoveLine={props.onMoveLine}
							onDuplicateLine={props.onDuplicateLine}
							onRemoveLine={props.onRemoveLine}
						/>
					) : (
						<LockBanner
							tone="warn"
							title="Le client de cette facture est introuvable"
						>
							Sans client, le brouillon ne peut pas être complété.
						</LockBanner>
					)}
				</>
			) : (
				<>
					<LockBanner
						title={
							invoice.status === 'CANCELLED'
								? 'Facture annulée : elle reste archivée, intacte.'
								: 'Facture émise : elle ne peut plus être modifiée. Pour la corriger, émets un avoir.'
						}
					>
						Numéro, destinataire, lignes et dates sont archivés tels qu’à
						l’émission du {formatDocumentDate(invoice.issued_at)}.
					</LockBanner>
					<DocumentPaper document={paper} />
				</>
			)}
		</>
	)

	const side = draft ? (
		<DraftSidebar
			invoice={invoice}
			customer={customer}
			project={project}
			displayState={displayState}
			activity={props.activity}
			history={history}
			quote={props.quote}
			onSaveNote={props.onSaveNote}
		/>
	) : (
		<IssuedSidebar
			invoice={invoice}
			customer={customer}
			project={project}
			displayState={displayState}
			activity={props.activity}
			history={history}
			quote={props.quote}
			balance={props.balance}
			remainingCents={props.remainingCents}
			refunds={props.refunds}
			payments={props.payments}
			paymentsLoading={props.paymentsLoading}
			creditNotes={props.creditNotes}
			creditNotesLoading={props.creditNotesLoading}
			isDeletingPayment={props.isDeletingPayment}
			onDeletePayment={props.onDeletePayment}
			onSaveNote={props.onSaveNote}
		/>
	)

	return (
		<DetailShell main={main} side={side} sideLabel="Détails de la facture" />
	)
}
