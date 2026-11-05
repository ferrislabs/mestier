import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'
import {
	INVOICE_STATE_LABELS,
	type InvoiceDisplayState,
} from '#/pages/invoices/types'
import { quoteStatusLabel } from '#/pages/quotes/types'

export type DocTone = 'neutral' | 'ok' | 'warn' | 'alert' | 'info'

const TONES: Record<DocTone, string> = {
	neutral: 'bg-muted text-muted-foreground',
	ok: 'bg-state-progress/12 text-state-progress',
	warn: 'bg-state-today/14 text-state-today',
	alert: 'bg-state-alert/12 text-state-alert',
	info: 'bg-primary/10 text-primary',
}

export function DocBadge({
	tone,
	children,
	className,
}: {
	tone: DocTone
	children: ReactNode
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex w-fit shrink-0 items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 font-medium text-xs',
				TONES[tone],
				className,
			)}
		>
			{children}
		</span>
	)
}

const QUOTE_TONES: Record<string, DocTone> = {
	DRAFT: 'neutral',
	SENT: 'info',
	ACCEPTED: 'ok',
	DECLINED: 'alert',
	CANCELLED: 'neutral',
}

export function QuoteStatusBadge({ status }: { status: string }) {
	return (
		<DocBadge tone={QUOTE_TONES[status] ?? 'neutral'}>
			{quoteStatusLabel(status)}
		</DocBadge>
	)
}

const INVOICE_TONES: Record<InvoiceDisplayState, DocTone> = {
	draft: 'neutral',
	unpaid: 'info',
	partial: 'warn',
	overdue: 'alert',
	paid: 'ok',
	cancelled: 'neutral',
	refund_due: 'warn',
}

export function InvoiceStateBadge({ state }: { state: InvoiceDisplayState }) {
	return (
		<DocBadge tone={INVOICE_TONES[state]}>
			{INVOICE_STATE_LABELS[state]}
		</DocBadge>
	)
}
