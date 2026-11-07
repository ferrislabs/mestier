import { Link } from '@tanstack/react-router'
import { ChevronRight } from 'lucide-react'
import { Fragment, useState } from 'react'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { formatDateTime, formatMoney } from '#/pages/customers/lib/format'
import { Money } from '#/pages/customers/ui/money'
import {
	STRIPE_REASON_LABELS,
	STRIPE_TYPE_LABELS,
	type StripeEvent,
} from '#/pages/stripe/types'
import { StripeStatusBadge } from '#/pages/stripe/ui/stripe-status-badge'

const DASH = <span className="text-muted-foreground">—</span>

const QUANTITY = new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 3 })

function formatVat(rate: number): string {
	return `${String(rate).replace('.', ',')} %`
}

function EventDetails({ event }: { event: StripeEvent }) {
	return (
		<div className="grid gap-4 py-1">
			<dl className="grid gap-x-6 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
				<div className="min-w-0">
					<dt className="text-[12px] text-muted-foreground">
						Identifiant client Stripe
					</dt>
					<dd className="mt-0.5 break-all font-mono text-[13px]">
						{event.customerId ?? '—'}
					</dd>
				</div>
				<div className="min-w-0">
					<dt className="text-[12px] text-muted-foreground">
						Intention de paiement
					</dt>
					<dd className="mt-0.5 break-all font-mono text-[13px]">
						{event.paymentIntentId ?? '—'}
					</dd>
				</div>
				<div className="min-w-0">
					<dt className="text-[12px] text-muted-foreground">Facture Stripe</dt>
					<dd className="mt-0.5 break-all text-sm">
						{event.stripeInvoiceNumber ?? '—'}
					</dd>
				</div>
				<div className="min-w-0">
					<dt className="text-[12px] text-muted-foreground">
						Facture Mestier citée
					</dt>
					<dd className="mt-0.5 break-all text-sm">
						{event.mestierInvoiceNumber ?? '—'}
					</dd>
				</div>
			</dl>
			{event.lines.length === 0 ? (
				<p className="text-[13px] text-muted-foreground">
					Cet événement ne détaille aucune ligne.
				</p>
			) : (
				<table className="w-full max-w-2xl text-[13px]">
					<caption className="sr-only">
						Lignes de l’événement {event.id}
					</caption>
					<thead>
						<tr className="text-left text-[12px] text-muted-foreground">
							<th scope="col" className="py-1 pr-3 font-normal">
								Description
							</th>
							<th scope="col" className="px-3 py-1 text-right font-normal">
								Quantité
							</th>
							<th scope="col" className="px-3 py-1 text-right font-normal">
								Prix unitaire HT
							</th>
							<th scope="col" className="py-1 pl-3 text-right font-normal">
								TVA
							</th>
						</tr>
					</thead>
					<tbody>
						{event.lines.map((line) => (
							<tr key={`${line.description}-${line.unitPrice}`}>
								<td className="py-1 pr-3">{line.description}</td>
								<td className="tabular px-3 py-1 text-right">
									{QUANTITY.format(line.quantity)}
								</td>
								<td className="tabular px-3 py-1 text-right">
									{formatMoney(line.unitPrice)}
								</td>
								<td className="tabular py-1 pl-3 text-right">
									{formatVat(line.vatRate)}
								</td>
							</tr>
						))}
					</tbody>
				</table>
			)}
		</div>
	)
}

export function HistoryTable({
	events,
	organizationSlug,
}: {
	events: readonly StripeEvent[]
	organizationSlug: string
}) {
	const [openId, setOpenId] = useState<string | null>(null)
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Date</TableHead>
						<TableHead>Événement</TableHead>
						<TableHead className="max-md:hidden">Client</TableHead>
						<TableHead className="text-right">Montant</TableHead>
						<TableHead className="max-lg:hidden">Résultat</TableHead>
						<TableHead className="pr-4">Statut</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{events.map((event) => {
						const expanded = openId === event.id
						const detailId = `stripe-detail-${event.id}`
						return (
							<Fragment key={event.id}>
								<TableRow
									className={cn(
										'relative cursor-pointer focus-within:bg-muted/50',
										expanded && 'border-b-0 bg-muted/30',
									)}
								>
									<TableCell className="py-2.5 pl-4 whitespace-nowrap">
										<button
											type="button"
											aria-expanded={expanded}
											aria-controls={detailId}
											onClick={() => setOpenId(expanded ? null : event.id)}
											className="flex items-center gap-1.5 rounded-sm text-left outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
										>
											<ChevronRight
												className={cn(
													'size-4 text-muted-foreground transition-transform',
													expanded && 'rotate-90',
												)}
												aria-hidden
											/>
											{formatDateTime(event.receivedAt)}
											<span className="sr-only">
												, {expanded ? 'masquer' : 'afficher'} les détails de{' '}
												{event.id}
											</span>
										</button>
									</TableCell>
									<TableCell>
										<span className="block font-medium">
											{STRIPE_TYPE_LABELS[event.type]}
										</span>
										<span className="block font-mono text-[12px] text-muted-foreground">
											{event.id}
										</span>
									</TableCell>
									<TableCell className="max-md:hidden">
										{event.accountId !== null ? (
											<Link
												to={buildOrgPath(
													organizationSlug,
													'/crm/clients/$customerId',
												)}
												params={{ customerId: event.accountId }}
												className="relative rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
											>
												{event.accountName}
											</Link>
										) : event.accountName !== '' ? (
											<span>{event.accountName}</span>
										) : (
											<>
												<span>{event.customerName || '—'}</span>
												<span className="block text-[12px] text-muted-foreground">
													Non rattaché
												</span>
											</>
										)}
									</TableCell>
									<TableCell className="text-right font-medium">
										<Money
											cents={
												event.type === 'charge.refunded'
													? -event.amount
													: event.amount
											}
											signed={event.type === 'charge.refunded'}
										/>
									</TableCell>
									<TableCell className="max-w-72 whitespace-normal max-lg:hidden">
										{event.outcome !== '' ? (
											<span>{event.outcome}</span>
										) : event.reviewReason !== null ? (
											<span className="text-muted-foreground">
												{STRIPE_REASON_LABELS[event.reviewReason]}
											</span>
										) : (
											DASH
										)}
									</TableCell>
									<TableCell className="pr-4">
										<StripeStatusBadge status={event.status} />
									</TableCell>
								</TableRow>
								{expanded && (
									<TableRow
										id={detailId}
										className="bg-muted/30 hover:bg-muted/30"
									>
										<TableCell
											colSpan={6}
											className="px-4 pb-4 whitespace-normal"
										>
											<EventDetails event={event} />
										</TableCell>
									</TableRow>
								)}
							</Fragment>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
