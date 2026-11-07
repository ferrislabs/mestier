import { Link } from '@tanstack/react-router'
import { type ReactNode, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { FormField } from '#/components/form-field'
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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { buildOrgPath } from '#/modules/org-path'
import { findPerson } from '#/pages/customers/feature/mocks/people'
import { formatDate } from '#/pages/customers/lib/format'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { Money } from '#/pages/customers/ui/money'
import {
	canMarkPaid,
	canReimburse,
	expenseState,
	expenseTotal,
	validatePaidDate,
} from '#/pages/treasury/lib/ledger-model'
import {
	type Expense,
	type ExpenseCategory,
	type LedgerEntry,
	type OpenInvoice,
	PAYMENT_METHOD_LABELS,
	PAYMENT_METHODS,
	type PaymentMethod,
	type Receipt,
} from '#/pages/treasury/types'
import { CategoryChip } from '#/pages/treasury/ui/category-chip'
import {
	ExpenseStateBadge,
	LedgerStatusBadge,
} from '#/pages/treasury/ui/ledger-badges'
import { OpenInvoiceSelect } from '#/pages/treasury/ui/open-invoice-select'

function Row({ label, children }: { label: string; children: ReactNode }) {
	return (
		<div className="min-w-0">
			<dt className="text-[12px] text-muted-foreground">{label}</dt>
			<dd className="mt-0.5 min-w-0 break-words text-sm">{children}</dd>
		</div>
	)
}

const EMPTY = <span className="text-muted-foreground">—</span>

export interface LedgerDetailActions {
	today: string
	onEditExpense: (expense: Expense) => void
	onDeleteExpense: (expense: Expense) => void
	onMarkPaid: (expense: Expense, paidAt: string, method: PaymentMethod) => void
	onReimburse: (expense: Expense) => void
	onAllocate: (receipt: Receipt, invoice: OpenInvoice) => void
}

export function LedgerDetailDialog({
	entry,
	expense,
	receipt,
	categories,
	openInvoices,
	organizationSlug,
	actions,
	onClose,
}: {
	entry: LedgerEntry | null
	expense: Expense | undefined
	receipt: Receipt | undefined
	categories: readonly ExpenseCategory[]
	openInvoices: readonly OpenInvoice[]
	organizationSlug: string
	actions: LedgerDetailActions
	onClose: () => void
}) {
	const [deleting, setDeleting] = useState(false)

	function close() {
		setDeleting(false)
		onClose()
	}

	return (
		<>
			<Dialog
				open={entry !== null && !deleting}
				onOpenChange={(open) => {
					if (!open) close()
				}}
			>
				<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-xl">
					{entry && (
						<DialogHeader>
							<DialogTitle>{entry.label}</DialogTitle>
							<DialogDescription>
								{entry.kind === 'expense' ? 'Dépense' : 'Recette'} du{' '}
								{formatDate(entry.date)}
							</DialogDescription>
						</DialogHeader>
					)}
					{expense && (
						<ExpenseDetail
							expense={expense}
							categories={categories}
							organizationSlug={organizationSlug}
							actions={actions}
							onDelete={() => setDeleting(true)}
							onEdit={() => {
								onClose()
								actions.onEditExpense(expense)
							}}
						/>
					)}
					{receipt && entry && (
						<ReceiptDetail
							entry={entry}
							receipt={receipt}
							openInvoices={openInvoices}
							organizationSlug={organizationSlug}
							actions={actions}
						/>
					)}
				</DialogContent>
			</Dialog>
			<ConfirmDialog
				open={deleting && expense !== undefined}
				onOpenChange={setDeleting}
				title="Supprimer cette dépense ?"
				description={
					expense
						? `« ${expense.label} » sera retirée du journal. Cette action est définitive.`
						: ''
				}
				confirmLabel="Supprimer la dépense"
				destructive
				onConfirm={() => {
					if (!expense) return
					actions.onDeleteExpense(expense)
					close()
				}}
			/>
		</>
	)
}

function CustomerName({
	expenseOrReceipt,
	organizationSlug,
}: {
	expenseOrReceipt: { customerId: string | null; customerName: string }
	organizationSlug: string
}) {
	if (expenseOrReceipt.customerName === '') return EMPTY
	if (expenseOrReceipt.customerId === null)
		return <>{expenseOrReceipt.customerName}</>
	return (
		<Link
			to={buildOrgPath(organizationSlug, '/crm/clients/$customerId')}
			params={{ customerId: expenseOrReceipt.customerId }}
			className="rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
		>
			{expenseOrReceipt.customerName}
		</Link>
	)
}

function ExpenseDetail({
	expense,
	categories,
	organizationSlug,
	actions,
	onEdit,
	onDelete,
}: {
	expense: Expense
	categories: readonly ExpenseCategory[]
	organizationSlug: string
	actions: LedgerDetailActions
	onEdit: () => void
	onDelete: () => void
}) {
	const { today } = actions
	const [paying, setPaying] = useState(false)
	const [paidAt, setPaidAt] = useState(today)
	const [method, setMethod] = useState<PaymentMethod>('transfer')
	const category = categories.find((item) => item.id === expense.categoryId)
	const advancer = findPerson(expense.advancedById)
	const display = expenseState(expense, today)
	const paidErrors = validatePaidDate(expense, paidAt, today)

	function pay() {
		if (paidErrors.length > 0) return
		actions.onMarkPaid(expense, paidAt, method)
		setPaying(false)
	}

	return (
		<>
			<dl className="grid gap-x-6 gap-y-3 sm:grid-cols-2">
				<Row label="Statut">
					<ExpenseStateBadge state={display} />
				</Row>
				<Row label="Montant TTC">
					<Money cents={expenseTotal(expense)} className="font-medium" />
				</Row>
				<Row label="Montant HT">
					<Money cents={expense.amountHt} />
				</Row>
				<Row label="TVA">
					<Money cents={expense.vatAmount} />
				</Row>
				<Row label="Fournisseur">
					{expense.supplierName || 'Sans fournisseur'}
				</Row>
				<Row label="Catégorie">
					<CategoryChip category={category} />
				</Row>
				<Row label="N° de facture fournisseur">
					{expense.reference || EMPTY}
				</Row>
				<Row label="Échéance">
					{expense.dueDate ? formatDate(expense.dueDate) : EMPTY}
				</Row>
				{expense.paidAt && (
					<Row label="Payée le">
						{formatDate(expense.paidAt)}
						{expense.method ? `, ${PAYMENT_METHOD_LABELS[expense.method]}` : ''}
					</Row>
				)}
				{expense.advancedById !== null && (
					<Row label="Avancée par">
						{advancer?.name ?? 'Collaborateur inconnu'}
						{expense.reimbursedAt
							? `, remboursée le ${formatDate(expense.reimbursedAt)}`
							: ', à rembourser'}
					</Row>
				)}
				<Row label="Client concerné">
					<CustomerName
						expenseOrReceipt={expense}
						organizationSlug={organizationSlug}
					/>
				</Row>
				<Row label="Justificatif">{expense.proofName ?? EMPTY}</Row>
				{expense.note && (
					<div className="sm:col-span-2">
						<Row label="Note">{expense.note}</Row>
					</div>
				)}
			</dl>
			{paying && (
				<form
					onSubmit={(event) => {
						event.preventDefault()
						pay()
					}}
					noValidate
					className="grid gap-3 rounded-md border bg-muted/30 p-3 sm:grid-cols-2"
				>
					<FormField
						id={`paid-at-${expense.id}`}
						label="Payée le"
						required
						error={paidErrors[0]}
					>
						<Input
							id={`paid-at-${expense.id}`}
							type="date"
							max={today}
							min={expense.date}
							value={paidAt}
							onChange={(event) => setPaidAt(event.target.value)}
							aria-invalid={paidErrors.length > 0 || undefined}
							aria-describedby={
								paidErrors.length > 0
									? `paid-at-${expense.id}-error`
									: undefined
							}
						/>
					</FormField>
					<FormField id={`paid-method-${expense.id}`} label="Moyen de paiement">
						<Select
							value={method}
							onValueChange={(value) => setMethod(value as PaymentMethod)}
						>
							<SelectTrigger
								id={`paid-method-${expense.id}`}
								className="w-full"
							>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{PAYMENT_METHODS.map((item) => (
									<SelectItem key={item} value={item}>
										{PAYMENT_METHOD_LABELS[item]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<div className="flex justify-end gap-2 sm:col-span-2">
						<Button
							type="button"
							variant="outline"
							onClick={() => setPaying(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={paidErrors.length > 0}>
							Confirmer le paiement
						</Button>
					</div>
				</form>
			)}
			<DialogFooter className="sm:justify-between">
				<Button type="button" variant="outline" onClick={onDelete}>
					Supprimer
				</Button>
				<div className="flex flex-col-reverse gap-2 sm:flex-row">
					<Button type="button" variant="outline" onClick={onEdit}>
						Modifier
					</Button>
					{canMarkPaid(expense) && !paying && (
						<Button type="button" onClick={() => setPaying(true)}>
							Marquer payée
						</Button>
					)}
					{canReimburse(expense) && (
						<Button type="button" onClick={() => actions.onReimburse(expense)}>
							Rembourser {advancer?.name ?? 'la personne'}
						</Button>
					)}
				</div>
			</DialogFooter>
		</>
	)
}

function ReceiptDetail({
	entry,
	receipt,
	openInvoices,
	organizationSlug,
	actions,
}: {
	entry: LedgerEntry
	receipt: Receipt
	openInvoices: readonly OpenInvoice[]
	organizationSlug: string
	actions: LedgerDetailActions
}) {
	const [target, setTarget] = useState<string | null>(null)
	const options =
		entry.status === 'unallocated'
			? openInvoices.filter((option) => option.balance >= receipt.amount)
			: []

	return (
		<>
			<dl className="grid gap-x-6 gap-y-3 sm:grid-cols-2">
				<Row label="Statut">
					<LedgerStatusBadge status={entry.status} />
				</Row>
				<Row label="Montant">
					<Money cents={receipt.amount} signed className="font-medium" />
				</Row>
				<Row label="Moyen de paiement">
					{PAYMENT_METHOD_LABELS[receipt.method]}
				</Row>
				<Row label="Origine">
					{receipt.source === 'stripe' ? (
						<StatusBadge tone="info">Stripe</StatusBadge>
					) : (
						'Saisie manuelle'
					)}
				</Row>
				<Row label="Client">
					<CustomerName
						expenseOrReceipt={receipt}
						organizationSlug={organizationSlug}
					/>
				</Row>
				<Row label="Facture">{receipt.invoiceLabel ?? 'Non affectée'}</Row>
				<Row label="Référence">{receipt.reference || EMPTY}</Row>
				<Row label="Note">{receipt.note || EMPTY}</Row>
			</dl>
			{entry.status === 'unallocated' && (
				<div className="grid gap-3 rounded-md border bg-muted/30 p-3">
					<FormField
						id={`allocate-${receipt.id}`}
						label="Affecter à une facture"
						hint={
							options.length === 0
								? 'Aucune facture émise ne peut recevoir ce montant.'
								: 'Factures émises avec un solde à payer.'
						}
					>
						<OpenInvoiceSelect
							id={`allocate-${receipt.id}`}
							options={options}
							value={target}
							onChange={setTarget}
							amount={receipt.amount}
							disabled={options.length === 0}
						/>
					</FormField>
					<div className="flex justify-end">
						<Button
							type="button"
							disabled={target === null}
							onClick={() => {
								const chosen = options.find((option) => option.id === target)
								if (chosen) actions.onAllocate(receipt, chosen)
								setTarget(null)
							}}
						>
							Affecter
						</Button>
					</div>
				</div>
			)}
			<p className="text-[13px] text-muted-foreground">
				Pour corriger une recette, enregistre un remboursement.
			</p>
		</>
	)
}
