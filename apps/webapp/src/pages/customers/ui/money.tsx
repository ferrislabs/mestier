import { cn } from '#/lib/utils'
import { formatMoney } from '#/pages/customers/lib/format'

export function Money({
	cents,
	signed,
	className,
}: {
	cents: number
	signed?: boolean
	className?: string
}) {
	const sign = signed ? (cents > 0 ? '+' : cents < 0 ? '−' : '') : ''
	const value = signed ? Math.abs(cents) : cents
	return (
		<span className={cn('tabular whitespace-nowrap', className)}>
			{sign}
			{sign ? ' ' : ''}
			{formatMoney(value)}
		</span>
	)
}
