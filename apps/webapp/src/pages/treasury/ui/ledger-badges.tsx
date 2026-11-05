import { StatusBadge, type Tone } from '#/pages/customers/ui/badges'
import {
	EXPENSE_STATE_LABELS,
	type ExpenseDisplayState,
	LEDGER_STATUS_LABELS,
	type LedgerStatus,
} from '#/pages/treasury/types'

const EXPENSE_TONES: Record<ExpenseDisplayState, Tone> = {
	to_pay: 'info',
	overdue: 'alert',
	paid: 'ok',
	to_reimburse: 'warn',
	reimbursed: 'ok',
}

export function ExpenseStateBadge({ state }: { state: ExpenseDisplayState }) {
	return (
		<StatusBadge tone={EXPENSE_TONES[state]}>
			{EXPENSE_STATE_LABELS[state]}
		</StatusBadge>
	)
}

const LEDGER_TONES: Record<LedgerStatus, Tone> = {
	done: 'ok',
	pending: 'info',
	late: 'alert',
	unallocated: 'warn',
	to_reimburse: 'warn',
}

export function LedgerStatusBadge({ status }: { status: LedgerStatus }) {
	return (
		<StatusBadge tone={LEDGER_TONES[status]}>
			{LEDGER_STATUS_LABELS[status]}
		</StatusBadge>
	)
}
