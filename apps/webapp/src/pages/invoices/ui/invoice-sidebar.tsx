import { Link } from '@tanstack/react-router'
import { Trash2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Customer } from '#/hooks/use-customers'
import type {
	Invoice,
	InvoiceBalance,
	InvoicePayment,
} from '#/hooks/use-invoices'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	creditNoteKindLabel,
	type InvoiceActivity,
} from '#/pages/invoices/lib/invoice-document'
import {
	PAYMENT_METHOD_LABELS,
	type RefundRecord,
} from '#/pages/invoices/lib/payment-form'
import {
	customerDisplayName,
	type InvoiceDisplayState,
	invoiceKindLabel,
} from '#/pages/invoices/types'
import {
	dueLabel,
	formatDocumentDate,
	formatDocumentDateTime,
} from '#/pages/quotes/lib/doc-format'
import { DocBadge, InvoiceStateBadge } from '#/pages/quotes/ui/doc/badges'
import { SideSection } from '#/pages/quotes/ui/doc/detail-shell'
import {
	AccountLink,
	CreditNoteLink,
	QuoteLink,
} from '#/pages/quotes/ui/doc/doc-links'
import {
	type HistoryEntry,
	HistoryList,
} from '#/pages/quotes/ui/doc/history-list'
import { AutoTextarea, Money } from '#/pages/quotes/ui/doc/primitives'

function Row({
	label,
	hint,
	strong,
	children,
}: {
	label: string
	hint?: ReactNode
	strong?: boolean
	children: ReactNode
}) {
	return (
		<div
			className={cn(
				'flex items-baseline justify-between gap-4',
				strong && 'border-t pt-2 font-semibold',
			)}
		>
			<dt className={strong ? undefined : 'text-muted-foreground'}>
				{label}
				{hint && (
					<span className="block font-normal text-muted-foreground text-xs">
						{hint}
					</span>
				)}
			</dt>
			<dd className="text-right">{children}</dd>
		</div>
	)
}

function KindChips({ invoice }: { invoice: Invoice }) {
	if (invoice.kind !== 'DEPOSIT' && invoice.kind !== 'FINAL') return null
	return <DocBadge tone="neutral">{invoiceKindLabel(invoice.kind)}</DocBadge>
}

function PaymentItem({
	payment,
	deleting,
	onDelete,
}: {
	payment: InvoicePayment
	deleting: boolean
	onDelete: (paymentId: string) => void
}) {
	return (
		<li className="grid gap-0.5 rounded-md border bg-background px-3 py-2 text-sm">
			<div className="flex items-baseline justify-between gap-3">
				<time dateTime={payment.paid_on} className="font-medium">
					{formatDocumentDate(payment.paid_on)}
				</time>
				<span className="flex items-center gap-1">
					<Money cents={payment.amount_cents} className="font-medium" />
					<RequirePermission permission="MANAGE_INVOICES">
						<Button
							type="button"
							variant="ghost"
							size="icon-xs"
							disabled={deleting}
							onClick={() => onDelete(payment.id)}
						>
							<Trash2 aria-hidden />
							<span className="sr-only">Supprimer le paiement</span>
						</Button>
					</RequirePermission>
				</span>
			</div>
			<p className="text-muted-foreground text-xs">
				{PAYMENT_METHOD_LABELS[payment.method]}
				{payment.reference && ` · ${payment.reference}`}
			</p>
			{payment.note && <p className="text-xs">{payment.note}</p>}
		</li>
	)
}

function RefundItem({ refund }: { refund: RefundRecord }) {
	return (
		<li className="grid gap-0.5 rounded-md border bg-background px-3 py-2 text-sm">
			<div className="flex items-baseline justify-between gap-3">
				<time dateTime={refund.date} className="font-medium">
					{formatDocumentDate(refund.date)}
				</time>
				<Money
					cents={-refund.amountCents}
					className="font-medium text-state-today"
				/>
			</div>
			<p className="text-muted-foreground text-xs">
				Remboursement · {PAYMENT_METHOD_LABELS[refund.method]}
				{refund.reference && ` · ${refund.reference}`}
			</p>
			{refund.note && <p className="text-xs">{refund.note}</p>}
		</li>
	)
}

function InternalNote({
	invoiceId,
	activity,
	onSave,
}: {
	invoiceId: string
	activity: InvoiceActivity
	onSave: (text: string) => void
}) {
	const id = `internal-note-${invoiceId}`
	const [text, setText] = useState(activity.internalNote)
	return (
		<SideSection title="Note interne">
			<label htmlFor={id} className="sr-only">
				Note interne
			</label>
			<AutoTextarea
				id={id}
				value={text}
				placeholder="Une information utile pour l’équipe…"
				onChange={(event) => setText(event.target.value)}
				onBlur={() => {
					if (text !== activity.internalNote) onSave(text)
				}}
				aria-describedby={`${id}-hint`}
				className="min-h-20"
			/>
			<p id={`${id}-hint`} className="text-muted-foreground text-xs">
				Visible uniquement en interne : elle ne figure jamais sur la facture.
			</p>
		</SideSection>
	)
}

function Links({
	invoice,
	customer,
	project,
	quote,
}: {
	invoice: Invoice
	customer: Customer | undefined
	project: { id: string; name: string } | null
	quote: { id: string; reference: string | null } | null
}) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<SideSection title="Liens">
			<dl className="grid gap-1.5 text-sm">
				<Row label="Client">
					{customer ? (
						<AccountLink customerId={customer.id} className="font-medium">
							{customerDisplayName(customer)}
						</AccountLink>
					) : (
						<span className="text-muted-foreground">Client inconnu</span>
					)}
				</Row>
				{quote && (
					<Row label="Devis d’origine">
						<QuoteLink quoteId={quote.id} className="font-medium">
							{quote.reference ?? 'Devis'}
						</QuoteLink>
					</Row>
				)}
				{invoice.project_id && project && (
					<Row label="Projet">
						<Link
							to={buildOrgPath(activeOrganization.slug, '/projet/$projectId')}
							params={{ projectId: project.id }}
							className="rounded-sm font-medium outline-none hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
						>
							{project.name}
						</Link>
					</Row>
				)}
			</dl>
		</SideSection>
	)
}

interface SidebarProps {
	invoice: Invoice
	customer: Customer | undefined
	project: { id: string; name: string } | null
	displayState: InvoiceDisplayState
	activity: InvoiceActivity
	history: HistoryEntry[]
	quote: { id: string; reference: string | null } | null
	onSaveNote: (text: string) => void
}

interface IssuedSidebarProps extends SidebarProps {
	balance: InvoiceBalance | undefined
	remainingCents: number | null
	refunds: RefundRecord[]
	payments: InvoicePayment[]
	paymentsLoading: boolean
	creditNotes: Invoice[]
	creditNotesLoading: boolean
	isDeletingPayment: boolean
	onDeletePayment: (paymentId: string) => void
}

export function IssuedSidebar({
	invoice,
	customer,
	project,
	displayState,
	activity,
	history,
	quote,
	balance,
	remainingCents,
	refunds,
	payments,
	paymentsLoading,
	creditNotes,
	creditNotesLoading,
	isDeletingPayment,
	onDeletePayment,
	onSaveNote,
}: IssuedSidebarProps) {
	const remaining = remainingCents
	const refunded = refunds.reduce((sum, refund) => sum + refund.amountCents, 0)
	const credited = balance?.credited_cents ?? 0
	const paid = balance?.paid_cents ?? 0
	const orderedPayments = [...payments].sort(
		(a, b) =>
			b.paid_on.localeCompare(a.paid_on) ||
			b.created_at.localeCompare(a.created_at),
	)
	return (
		<>
			<SideSection title="État">
				<div className="flex flex-wrap items-center gap-1.5">
					<InvoiceStateBadge state={displayState} />
					<KindChips invoice={invoice} />
				</div>
				<p className="text-muted-foreground text-sm">
					Émise le {formatDocumentDate(invoice.issued_at)}
					{activity.sentAt
						? `, envoyée le ${formatDocumentDateTime(activity.sentAt)}`
						: ', pas encore envoyée'}
					.
				</p>
			</SideSection>
			<SideSection title="Règlement">
				{balance ? (
					<dl className="grid gap-1.5 rounded-lg border bg-background p-3 text-sm">
						<Row label="Total TTC">
							<Money cents={balance.gross_cents} />
						</Row>
						<Row label="Crédité par avoirs">
							{credited > 0 ? (
								<Money cents={-credited} signed />
							) : (
								<span className="text-muted-foreground">—</span>
							)}
						</Row>
						<Row label="Payé">
							<Money cents={paid} />
						</Row>
						{refunded > 0 && (
							<Row label="Remboursé">
								<Money cents={-refunded} signed />
							</Row>
						)}
						<Row
							label={
								remaining !== null && remaining < 0
									? 'À rembourser'
									: 'Reste dû'
							}
							strong
						>
							<Money
								cents={Math.abs(remaining ?? 0)}
								className={cn(
									displayState === 'overdue' && 'text-state-alert',
									remaining !== null && remaining < 0 && 'text-state-today',
								)}
							/>
						</Row>
						<Row
							label="Échéance"
							hint={
								remaining !== null && remaining > 0
									? dueLabel(invoice.due_at)
									: undefined
							}
						>
							<span className="tabular">
								{formatDocumentDate(invoice.due_at)}
							</span>
						</Row>
					</dl>
				) : (
					<p className="text-muted-foreground text-sm">Chargement…</p>
				)}
			</SideSection>
			<SideSection title={`Paiements (${payments.length + refunds.length})`}>
				{paymentsLoading ? (
					<p className="text-muted-foreground text-sm">Chargement…</p>
				) : orderedPayments.length === 0 && refunds.length === 0 ? (
					<p className="text-muted-foreground text-sm">
						Aucun paiement enregistré.
					</p>
				) : (
					<ul className="grid gap-2">
						{orderedPayments.map((payment) => (
							<PaymentItem
								key={payment.id}
								payment={payment}
								deleting={isDeletingPayment}
								onDelete={onDeletePayment}
							/>
						))}
						{refunds.map((refund) => (
							<RefundItem key={refund.id} refund={refund} />
						))}
					</ul>
				)}
			</SideSection>
			<SideSection title={`Avoirs (${creditNotes.length})`}>
				{creditNotesLoading ? (
					<p className="text-muted-foreground text-sm">Chargement…</p>
				) : creditNotes.length === 0 ? (
					<p className="text-muted-foreground text-sm">
						Aucun avoir émis sur cette facture.
					</p>
				) : (
					<ul className="grid gap-2">
						{creditNotes.map((note) => (
							<li
								key={note.id}
								className="grid gap-0.5 rounded-md border bg-background px-3 py-2 text-sm"
							>
								<div className="flex items-baseline justify-between gap-3">
									<CreditNoteLink
										creditNoteId={note.id}
										className="font-medium"
									>
										{note.number ?? 'Brouillon'}
									</CreditNoteLink>
									<Money
										cents={-note.gross_cents}
										signed
										className="font-medium"
									/>
								</div>
								<p className="text-muted-foreground text-xs">
									{formatDocumentDate(note.issued_at ?? note.created_at)}
									{' · '}
									{creditNoteKindLabel(note)}
								</p>
								{note.notes && (
									<p className="line-clamp-2 text-xs">{note.notes}</p>
								)}
							</li>
						))}
					</ul>
				)}
			</SideSection>
			<Links
				invoice={invoice}
				customer={customer}
				project={project}
				quote={quote}
			/>
			<InternalNote
				key={activity.internalNote}
				invoiceId={invoice.id}
				activity={activity}
				onSave={onSaveNote}
			/>
			<SideSection title="Historique">
				<HistoryList entries={history} />
			</SideSection>
		</>
	)
}

export function DraftSidebar({
	invoice,
	customer,
	project,
	displayState,
	activity,
	history,
	quote,
	onSaveNote,
}: SidebarProps) {
	return (
		<>
			<SideSection title="État">
				<div className="flex flex-wrap items-center gap-1.5">
					<InvoiceStateBadge state={displayState} />
					<KindChips invoice={invoice} />
				</div>
				<p className="text-muted-foreground text-sm">
					Le numéro définitif est attribué à l’émission, dans la série des
					factures.
				</p>
			</SideSection>
			<Links
				invoice={invoice}
				customer={customer}
				project={project}
				quote={quote}
			/>
			<InternalNote
				key={activity.internalNote}
				invoiceId={invoice.id}
				activity={activity}
				onSave={onSaveNote}
			/>
			<SideSection title="Historique">
				<HistoryList entries={history} />
			</SideSection>
		</>
	)
}
