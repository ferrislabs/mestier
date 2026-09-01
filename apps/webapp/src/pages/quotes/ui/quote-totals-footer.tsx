import { formatCents, formatVatRateBp } from '../types'

interface QuoteTotalsFooterProps {
	netCents: number
	vatBreakdown: { rateBp: number; vatCents: number }[]
	grossCents: number
	/** Shown once, above the box, when the organization is not subject to VAT
	 * — "exempt" and "nothing to report" are different facts (see the backend
	 * comment on `calculate_totals`), so this reads out the reason rather than
	 * leaving the absence of a VAT line unexplained. */
	vatExemptionNotice?: string | null
	/** e.g. "Estimation, non enregistrée" on the create screen, or
	 * "Des modifications non enregistrées ne sont pas reflétées" on edit. */
	notice?: string | null
}

/**
 * The one place a quote states what it costs. Right-aligned like the total on
 * a paper invoice: HT, each VAT rate charged, then TTC — the figure a
 * customer actually reads.
 */
export function QuoteTotalsFooter({
	netCents,
	vatBreakdown,
	grossCents,
	vatExemptionNotice,
	notice,
}: QuoteTotalsFooterProps) {
	return (
		<div className="flex justify-end border-t p-5">
			<div className="w-full max-w-xs space-y-1.5 text-sm">
				<TotalsRow label="Total HT" value={formatCents(netCents)} />
				{vatBreakdown.map((line) => (
					<TotalsRow
						key={line.rateBp}
						label={`TVA ${formatVatRateBp(line.rateBp)}`}
						value={formatCents(line.vatCents)}
					/>
				))}
				{vatExemptionNotice ? (
					<p className="text-xs text-muted-foreground">{vatExemptionNotice}</p>
				) : null}
				<div className="flex items-baseline justify-between border-t pt-1.5 text-base font-bold">
					<span>Total TTC</span>
					<span className="tabular-nums">{formatCents(grossCents)}</span>
				</div>
				{notice ? (
					<p className="pt-1 text-right text-xs text-muted-foreground italic">
						{notice}
					</p>
				) : null}
			</div>
		</div>
	)
}

function TotalsRow({ label, value }: { label: string; value: string }) {
	return (
		<div className="flex items-baseline justify-between text-muted-foreground">
			<span>{label}</span>
			<span className="tabular-nums text-foreground">{value}</span>
		</div>
	)
}
