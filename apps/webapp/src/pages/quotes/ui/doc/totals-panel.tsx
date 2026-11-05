import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'
import { formatVatRateBp } from '#/pages/quotes/types'
import { Money } from '#/pages/quotes/ui/doc/primitives'

export interface DocTotals {
	netCents: number
	vat: { rateBp: number; vatCents: number }[]
	grossCents: number
}

function Row({ label, children }: { label: string; children: ReactNode }) {
	return (
		<div className="flex items-baseline justify-between gap-4">
			<dt className="text-muted-foreground">{label}</dt>
			<dd>{children}</dd>
		</div>
	)
}

export function TotalsPanel({
	totals,
	grossLabel = 'Total TTC',
	vatNotice,
	notice,
	className,
}: {
	totals: DocTotals
	grossLabel?: string
	vatNotice?: string | null
	notice?: string | null
	className?: string
}) {
	return (
		<dl
			className={cn(
				'grid gap-2 rounded-lg border bg-card p-4 text-sm',
				className,
			)}
		>
			<Row label="Total HT">
				<Money cents={totals.netCents} />
			</Row>
			{totals.vat.map((row) => (
				<Row key={row.rateBp} label={`TVA ${formatVatRateBp(row.rateBp)}`}>
					<Money cents={row.vatCents} />
				</Row>
			))}
			{vatNotice && (
				<p className="text-muted-foreground text-xs">{vatNotice}</p>
			)}
			<div className="mt-1 flex items-baseline justify-between gap-4 border-t pt-3">
				<dt className="font-semibold">{grossLabel}</dt>
				<dd className="font-semibold text-xl">
					<Money cents={totals.grossCents} />
				</dd>
			</div>
			{notice && (
				<p className="text-right text-muted-foreground text-xs italic">
					{notice}
				</p>
			)}
		</dl>
	)
}
