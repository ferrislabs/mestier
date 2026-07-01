import type { BillingSettings } from '#/hooks/use-billing-settings'
import type { Invoice } from '#/hooks/use-invoices'
import type { LegalMentionTemplate } from '#/hooks/use-legal-mentions'
import type { OrganizationContext } from '#/hooks/use-organization-contexts'
import { formatCents, formatUnit } from '#/pages/invoices/types'

interface DocumentPreviewProps {
	invoice: Invoice
	billingSettings: BillingSettings | null
	templates: LegalMentionTemplate[]
	customerName: string
	emitter: OrganizationContext | null
}

export function DocumentPreview({
	invoice,
	billingSettings,
	templates,
	customerName,
	emitter,
}: DocumentPreviewProps) {
	const resolvedTemplates = invoice.legal_mention_template_ids
		.map((id) => templates.find((t) => t.id === id))
		.filter((t): t is LegalMentionTemplate => t !== undefined)

	const resteAPayer =
		invoice.invoice_type !== 'DEPOSIT'
			? invoice.total_ttc_cents - invoice.amount_paid_cents
			: null

	return (
		<div
			// A4 page at 96dpi (794×1123px). min-height fills a full page; taller
			// content flows onto further pages (the container scrolls vertically),
			// with a faint separator drawn at each A4 page boundary.
			className="mx-auto min-h-[1123px] w-[794px] max-w-full bg-white p-[48px] text-sm text-zinc-800 shadow-lg print:shadow-none"
			style={{
				backgroundImage:
					'repeating-linear-gradient(to bottom, transparent 0, transparent 1122px, rgba(0,0,0,0.08) 1122px, rgba(0,0,0,0.08) 1123px)',
			}}
		>
			{/* Header — seller identity + document title */}
			<div className="flex flex-col gap-6 border-b pb-6 sm:flex-row sm:justify-between">
				{/* Seller block */}
				<div className="min-w-0 space-y-0.5">
					<p className="text-base font-bold">{emitter?.label ?? '—'}</p>
					{emitter?.address_line ? (
						<p className="text-xs text-zinc-500">{emitter.address_line}</p>
					) : null}
					{emitter?.postal_code || emitter?.city ? (
						<p className="text-xs text-zinc-500">
							{[emitter.postal_code, emitter.city].filter(Boolean).join(' ')}
						</p>
					) : null}
					{emitter?.country ? (
						<p className="text-xs text-zinc-500">{emitter.country}</p>
					) : null}
					{emitter?.siret ? (
						<p className="text-xs text-zinc-500">SIRET : {emitter.siret}</p>
					) : null}
					{emitter?.rcs ? (
						<p className="text-xs text-zinc-500">RCS : {emitter.rcs}</p>
					) : null}
					{emitter?.ape ? (
						<p className="text-xs text-zinc-500">APE : {emitter.ape}</p>
					) : null}
					{emitter?.vat_intracom ? (
						<p className="text-xs text-zinc-500">
							TVA intracom. : {emitter.vat_intracom}
						</p>
					) : null}
					{emitter?.iban ? (
						<p className="text-xs text-zinc-500">IBAN : {emitter.iban}</p>
					) : null}
					{emitter?.bic ? (
						<p className="text-xs text-zinc-500">BIC : {emitter.bic}</p>
					) : null}
				</div>

				{/* Document title + metadata */}
				<div className="shrink-0 space-y-1 text-right">
					<p className="text-lg font-bold uppercase tracking-wide">Facture</p>
					<p className="text-xs text-zinc-500">
						Référence :{' '}
						<span className="font-medium text-zinc-800">
							{invoice.reference ?? 'Brouillon'}
						</span>
					</p>
					{invoice.issued_at ? (
						<p className="text-xs text-zinc-500">
							Date d'émission :{' '}
							<span className="font-medium text-zinc-800">
								{formatShortDate(invoice.issued_at)}
							</span>
						</p>
					) : null}
					{invoice.due_at ? (
						<p className="text-xs text-zinc-500">
							Échéance :{' '}
							<span className="font-medium text-zinc-800">
								{formatShortDate(invoice.due_at)}
							</span>
						</p>
					) : billingSettings?.payment_terms_days ? (
						<p className="text-xs text-zinc-500">
							Délai de paiement :{' '}
							<span className="font-medium text-zinc-800">
								{billingSettings.payment_terms_days} jour
								{billingSettings.payment_terms_days > 1 ? 's' : ''}
							</span>
						</p>
					) : null}
				</div>
			</div>

			{/* Customer block */}
			<div className="mt-6 flex justify-end">
				<div className="rounded-md border border-zinc-200 bg-zinc-50 px-4 py-3 text-right">
					<p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">
						Facturé à
					</p>
					<p className="mt-0.5 font-semibold">{customerName || '—'}</p>
				</div>
			</div>

			{/* Object */}
			<div className="mt-6">
				<p className="font-semibold">{invoice.title}</p>
			</div>

			{/* Line items table */}
			<div className="mt-5 overflow-x-auto">
				<table className="w-full text-xs">
					<thead>
						<tr className="border-b text-left text-zinc-500">
							<th className="pb-2 font-medium">Désignation</th>
							<th className="pb-2 text-right font-medium">Qté</th>
							<th className="pb-2 text-right font-medium">Unité</th>
							<th className="pb-2 text-right font-medium">PU HT</th>
							<th className="pb-2 text-right font-medium">TVA %</th>
							<th className="pb-2 text-right font-medium">Total HT</th>
						</tr>
					</thead>
					<tbody className="divide-y divide-zinc-100">
						{invoice.lines.map((line) => {
							const qty = Number(line.quantity)
							const lineHt = Number.isFinite(qty)
								? Math.round(qty * line.unit_price_cents)
								: 0
							const unit = line.unit === 'HOUR' ? 'h' : formatUnit(line.unit)

							return (
								<tr key={line.id} className="align-top">
									<td className="py-2 pr-3">
										<p className="font-medium">{line.label}</p>
										{line.notes ? (
											<p className="mt-0.5 text-zinc-400">{line.notes}</p>
										) : null}
									</td>
									<td className="py-2 text-right">{line.quantity}</td>
									<td className="py-2 text-right">{unit}</td>
									<td className="py-2 text-right">
										{formatCents(line.unit_price_cents)}
									</td>
									<td className="py-2 text-right">{line.vat_rate} %</td>
									<td className="py-2 text-right font-medium">
										{formatCents(lineHt)}
									</td>
								</tr>
							)
						})}
					</tbody>
				</table>
			</div>

			{/* Totals */}
			<div className="mt-6 flex justify-end">
				<dl className="w-full max-w-xs space-y-1.5 text-sm">
					<TotalRow
						label="Total HT"
						value={formatCents(invoice.total_ht_cents)}
					/>
					<TotalRow label="TVA" value={formatCents(invoice.total_vat_cents)} />
					<div className="border-t pt-1.5">
						<TotalRow
							label="Total TTC"
							value={formatCents(invoice.total_ttc_cents)}
							bold
						/>
					</div>
					{invoice.invoice_type === 'BALANCE' &&
					invoice.deposit_amount_cents != null ? (
						<TotalRow
							label="Acompte déduit"
							value={`− ${formatCents(invoice.deposit_amount_cents)}`}
						/>
					) : null}
					{resteAPayer !== null ? (
						<TotalRow
							label="Reste à payer"
							value={formatCents(resteAPayer)}
							bold
						/>
					) : null}
				</dl>
			</div>

			{/* Legal mentions */}
			{resolvedTemplates.length > 0 ? (
				<div className="mt-8 space-y-3 border-t pt-6">
					<p className="text-xs font-semibold uppercase tracking-wide text-zinc-400">
						Mentions légales
					</p>
					{resolvedTemplates.map((t) => (
						<div key={t.id}>
							<p className="font-medium">{t.name}</p>
							<p className="whitespace-pre-wrap text-xs text-zinc-500">
								{t.body}
							</p>
						</div>
					))}
				</div>
			) : null}

			{/* Footer */}
			{billingSettings?.footer ? (
				<div className="mt-8 border-t pt-4 text-xs text-zinc-400">
					<p className="whitespace-pre-wrap">{billingSettings.footer}</p>
				</div>
			) : null}

			{/* Payment terms */}
			{billingSettings ? (
				<div className="mt-4 text-xs text-zinc-400">
					{billingSettings.late_penalty_rate ? (
						<p>
							Pénalités de retard : {billingSettings.late_penalty_rate} % par
							an.
						</p>
					) : null}
					{billingSettings.recovery_indemnity_cents > 0 ? (
						<p>
							Indemnité forfaitaire de recouvrement :{' '}
							{formatCents(billingSettings.recovery_indemnity_cents)}.
						</p>
					) : null}
				</div>
			) : null}
		</div>
	)
}

function TotalRow({
	label,
	value,
	bold,
}: {
	label: string
	value: string
	bold?: boolean
}) {
	return (
		<div className="flex items-center justify-between gap-4">
			<dt className={bold ? 'font-semibold' : 'text-zinc-500'}>{label}</dt>
			<dd className={bold ? 'font-bold' : 'font-medium'}>{value}</dd>
		</div>
	)
}

function formatShortDate(iso: string): string {
	try {
		return new Intl.DateTimeFormat('fr-FR', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric',
		}).format(new Date(iso))
	} catch {
		return iso
	}
}
