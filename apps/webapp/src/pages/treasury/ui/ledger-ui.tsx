import { HandCoins, Plus, Wallet } from 'lucide-react'
import { useState } from 'react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Customer } from '#/hooks/use-customers'
import { formatMoney, formatMonth } from '#/pages/customers/lib/format'
import { Money } from '#/pages/customers/ui/money'
import { Notice } from '#/pages/customers/ui/notice'
import { StatTile } from '#/pages/customers/ui/stat-tile'
import {
	type ExpenseFormValues,
	emptyExpenseValues,
	emptyPaymentValues,
	expenseValuesOf,
	hasLedgerFilters,
	type LedgerSearch,
	type LedgerSearchPatch,
	type LedgerSummary,
	ledgerDescription,
	ledgerTitle,
	type MonthTrend,
	type PaymentFormValues,
	type VatSummary,
} from '#/pages/treasury/lib/ledger-model'
import type {
	Expense,
	ExpenseCategory,
	LedgerEntry,
	OpenInvoice,
	Receipt,
} from '#/pages/treasury/types'
import {
	ExpenseFormDialog,
	type SupplierOption,
} from '#/pages/treasury/ui/expense-form-dialog'
import {
	type LedgerDetailActions,
	LedgerDetailDialog,
} from '#/pages/treasury/ui/ledger-detail-dialog'
import { LedgerTable } from '#/pages/treasury/ui/ledger-table'
import { LedgerToolbar } from '#/pages/treasury/ui/ledger-toolbar'
import { LedgerTrendChart } from '#/pages/treasury/ui/ledger-trend-chart'
import { PaymentFormDialog } from '#/pages/treasury/ui/payment-form-dialog'

interface LedgerUIProps {
	organizationSlug: string
	today: string
	search: LedgerSearch
	onSearchChange: (patch: LedgerSearchPatch) => void
	notice: string | null
	onDismissNotice: () => void
	entries: readonly LedgerEntry[]
	visible: readonly LedgerEntry[]
	period: LedgerSummary
	periodMonth: string | null
	toCollect: number
	toCollectCount: number
	toPay: number
	overdueToPay: number
	vat: VatSummary
	trend: readonly MonthTrend[]
	months: readonly string[]
	categories: readonly ExpenseCategory[]
	expenses: readonly Expense[]
	receipts: readonly Receipt[]
	openInvoices: readonly OpenInvoice[]
	suppliers: readonly SupplierOption[]
	customers: readonly Customer[]
	detailActions: Omit<LedgerDetailActions, 'onEditExpense' | 'today'>
	onSaveExpense: (
		expenseId: string | null,
		values: ExpenseFormValues,
	) => Promise<void>
	onRecordReceipt: (values: PaymentFormValues) => Promise<void>
}

export function LedgerUI({
	organizationSlug,
	today,
	search,
	onSearchChange,
	notice,
	onDismissNotice,
	entries,
	visible,
	period,
	periodMonth,
	toCollect,
	toCollectCount,
	toPay,
	overdueToPay,
	vat,
	trend,
	months,
	categories,
	expenses,
	receipts,
	openInvoices,
	suppliers,
	customers,
	detailActions,
	onSaveExpense,
	onRecordReceipt,
}: LedgerUIProps) {
	const [selectedKey, setSelectedKey] = useState<string | null>(null)
	const [expenseOpen, setExpenseOpen] = useState(false)
	const [editingExpense, setEditingExpense] = useState<Expense | undefined>()
	const [paymentOpen, setPaymentOpen] = useState(false)
	const selected = entries.find((entry) => entry.key === selectedKey) ?? null
	const periodLabel = periodMonth
		? formatMonth(periodMonth)
		: 'toute la période'
	const payTilePressed = search.statut === 'pending'

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title={ledgerTitle(search.type)}
				description={ledgerDescription(search.type)}
				actions={
					<div className="flex items-center gap-2">
						<Button
							variant={search.type === 'depenses' ? 'outline' : 'default'}
							onClick={() => setPaymentOpen(true)}
						>
							<HandCoins aria-hidden />
							Enregistrer une recette
						</Button>
						<Button
							variant={search.type === 'depenses' ? 'default' : 'outline'}
							onClick={() => {
								setEditingExpense(undefined)
								setExpenseOpen(true)
							}}
						>
							<Wallet aria-hidden />
							Saisir une dépense
						</Button>
					</div>
				}
			/>
			<Notice message={notice} onDismiss={onDismissNotice} />
			<section
				aria-label={`Chiffres clés, ${periodLabel}`}
				className="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-3"
			>
				<StatTile
					label="Recettes"
					value={<Money cents={period.receipts} />}
					hint={
						period.unallocated > 0 ? (
							<>
								dont <Money cents={period.unallocated} /> à affecter
							</>
						) : (
							periodLabel
						)
					}
				/>
				<StatTile
					label="Dépenses"
					value={<Money cents={period.expenses} />}
					hint={periodLabel}
				/>
				<StatTile
					label="Solde net"
					value={<Money cents={period.net} signed />}
					tone={period.net < 0 ? 'alert' : 'ok'}
					hint={periodLabel}
				/>
				<StatTile
					label="À encaisser"
					value={<Money cents={toCollect} />}
					hint={`${toCollectCount} client${toCollectCount > 1 ? 's' : ''} avec un encours`}
				/>
				<StatTile
					label="À payer"
					value={<Money cents={toPay} />}
					tone={overdueToPay > 0 ? 'alert' : 'default'}
					hint={
						overdueToPay > 0 ? (
							<>
								dont <Money cents={overdueToPay} /> en retard · filtrer les
								dépenses à payer
							</>
						) : (
							'Filtrer les dépenses à payer'
						)
					}
					pressed={payTilePressed}
					onClick={() =>
						onSearchChange({ statut: payTilePressed ? null : 'pending' })
					}
				/>
				<StatTile
					label={periodMonth ? 'TVA du mois' : 'TVA, toute la période'}
					value={<Money cents={vat.due} />}
					hint={`${vat.due >= 0 ? 'À reverser' : 'Crédit de TVA'} · collectée ${formatMoney(vat.collected)}, déductible ${formatMoney(vat.deductible)}`}
				/>
			</section>
			<LedgerTrendChart trend={trend} />
			<LedgerToolbar
				search={search}
				months={months}
				categories={categories}
				onChange={onSearchChange}
			/>
			{visible.length === 0 ? (
				<EmptyState
					title={
						hasLedgerFilters(search)
							? 'Aucune écriture ne correspond à ces filtres'
							: 'Aucune écriture pour le moment'
					}
					hint={
						hasLedgerFilters(search)
							? 'Élargis la période ou retire un filtre.'
							: 'Enregistre une recette ou saisis une dépense pour démarrer le journal.'
					}
				>
					{hasLedgerFilters(search) ? (
						<Button
							variant="outline"
							onClick={() =>
								onSearchChange({
									mois: null,
									q: null,
									categorie: null,
									statut: null,
								})
							}
						>
							Réinitialiser les filtres
						</Button>
					) : (
						<Button
							onClick={() => {
								setEditingExpense(undefined)
								setExpenseOpen(true)
							}}
						>
							<Plus aria-hidden />
							Saisir une dépense
						</Button>
					)}
				</EmptyState>
			) : (
				<LedgerTable
					entries={visible}
					categories={categories}
					organizationSlug={organizationSlug}
					onOpen={setSelectedKey}
				/>
			)}
			<LedgerDetailDialog
				entry={selected}
				expense={
					selected?.kind === 'expense'
						? expenses.find((expense) => expense.id === selected.sourceId)
						: undefined
				}
				receipt={
					selected?.kind === 'receipt'
						? receipts.find((receipt) => receipt.id === selected.sourceId)
						: undefined
				}
				categories={categories}
				openInvoices={openInvoices}
				organizationSlug={organizationSlug}
				actions={{
					...detailActions,
					today,
					onEditExpense: (expense) => {
						setEditingExpense(expense)
						setExpenseOpen(true)
					},
				}}
				onClose={() => setSelectedKey(null)}
			/>
			<ExpenseFormDialog
				key={editingExpense?.id ?? 'new'}
				open={expenseOpen}
				onOpenChange={setExpenseOpen}
				expense={editingExpense}
				initial={
					editingExpense
						? expenseValuesOf(editingExpense)
						: emptyExpenseValues(today)
				}
				categories={categories}
				suppliers={suppliers}
				customers={customers}
				onSubmit={(values) => onSaveExpense(editingExpense?.id ?? null, values)}
			/>
			<PaymentFormDialog
				open={paymentOpen}
				onOpenChange={setPaymentOpen}
				initial={emptyPaymentValues(today)}
				today={today}
				customers={customers}
				openInvoices={openInvoices}
				onSubmit={onRecordReceipt}
			/>
		</PageShell>
	)
}
