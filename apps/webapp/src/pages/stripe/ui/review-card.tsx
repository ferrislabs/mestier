import { Link } from '@tanstack/react-router'
import { type ReactNode, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import type { Customer } from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import { formatDateTime } from '#/pages/customers/lib/format'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { CustomerPicker } from '#/pages/customers/ui/customer-picker'
import { Money } from '#/pages/customers/ui/money'
import { reviewActionOf } from '#/pages/stripe/lib/stripe-model'
import {
	STRIPE_REASON_LABELS,
	STRIPE_TYPE_LABELS,
	type StripeEvent,
} from '#/pages/stripe/types'
import type { OpenInvoice } from '#/pages/treasury/types'
import { OpenInvoiceSelect } from '#/pages/treasury/ui/open-invoice-select'

function Fact({ label, children }: { label: string; children: ReactNode }) {
	return (
		<div className="min-w-0">
			<dt className="text-[12px] text-muted-foreground">{label}</dt>
			<dd className="mt-0.5 min-w-0 break-words text-sm">{children}</dd>
		</div>
	)
}

export function ReviewCard({
	event,
	organizationSlug,
	customers,
	openInvoices,
	onLink,
	onAllocate,
	onIgnore,
}: {
	event: StripeEvent
	organizationSlug: string
	customers: readonly Customer[]
	openInvoices: readonly OpenInvoice[]
	onLink: (event: StripeEvent, customer: Customer) => void
	onAllocate: (event: StripeEvent, invoice: OpenInvoice) => void
	onIgnore: (event: StripeEvent) => void
}) {
	const action = reviewActionOf(event)
	const fitting = openInvoices.filter(
		(option) => option.balance >= event.amount,
	)
	const [invoiceId, setInvoiceId] = useState<string | null>(
		() =>
			fitting.find((option) => option.label === event.mestierInvoiceNumber)
				?.id ??
			fitting[0]?.id ??
			null,
	)
	const [customerId, setCustomerId] = useState<string | null>(null)
	const [ignoring, setIgnoring] = useState(false)
	const titleId = `review-${event.id}`
	const chosenCustomer = customers.find(
		(customer) => customer.id === customerId,
	)
	const hint =
		openInvoices.length === 0
			? 'Aucune facture émise avec un solde à payer. Émets la facture, puis reviens affecter ce paiement.'
			: fitting.length === 0
				? 'Aucune facture ouverte ne peut recevoir ce montant.'
				: 'Factures émises avec un solde à payer.'

	return (
		<li>
			<article
				aria-labelledby={titleId}
				className="grid gap-4 rounded-lg border border-state-today/40 bg-state-today/5 p-4"
			>
				<header className="flex flex-wrap items-start justify-between gap-2">
					<div className="min-w-0">
						<h3 id={titleId} className="font-semibold">
							{STRIPE_TYPE_LABELS[event.type]}
						</h3>
						<p className="mt-1 flex flex-wrap items-center gap-2">
							{event.reviewReason !== null && (
								<StatusBadge tone="warn">
									{STRIPE_REASON_LABELS[event.reviewReason]}
								</StatusBadge>
							)}
							<span className="font-mono text-[12px] text-muted-foreground">
								{event.id}
							</span>
						</p>
					</div>
					<Money
						cents={
							event.type === 'charge.refunded' ? -event.amount : event.amount
						}
						signed={event.type === 'charge.refunded'}
						className="font-semibold text-xl"
					/>
				</header>
				<dl className="grid gap-x-6 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
					<Fact label="Client Stripe">{event.customerName || '—'}</Fact>
					<Fact label="E-mail">{event.customerEmail || '—'}</Fact>
					<Fact label="Identifiant">
						<code className="font-mono text-[13px]">
							{event.customerId ?? '—'}
						</code>
					</Fact>
					<Fact label="Reçu le">{formatDateTime(event.receivedAt)}</Fact>
					{event.description && (
						<div className="sm:col-span-2 lg:col-span-4">
							<Fact label="Description">{event.description}</Fact>
						</div>
					)}
					{event.accountId !== null && (
						<Fact label="Client rattaché">
							<Link
								to={buildOrgPath(organizationSlug, '/crm/clients/$customerId')}
								params={{ customerId: event.accountId }}
								className="rounded-sm outline-none transition-colors hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
							>
								{event.accountName}
							</Link>
						</Fact>
					)}
				</dl>
				{action === 'link' && (
					<div className="grid gap-2 rounded-md border bg-card p-3">
						<FormField
							id={`${titleId}-account`}
							label="Client à rattacher"
							hint="L’identifiant Stripe est mémorisé sur la fiche du client : ses prochains paiements seront rapprochés automatiquement."
						>
							<CustomerPicker
								id={`${titleId}-account`}
								customers={customers}
								value={customerId}
								onChange={setCustomerId}
							/>
						</FormField>
						<div className="flex flex-wrap justify-end gap-2">
							<Button
								type="button"
								variant="outline"
								onClick={() => setIgnoring(true)}
							>
								Ignorer
							</Button>
							<Button
								type="button"
								disabled={!chosenCustomer}
								onClick={() => chosenCustomer && onLink(event, chosenCustomer)}
							>
								Rattacher à un client
							</Button>
						</div>
					</div>
				)}
				{action === 'allocate' && (
					<div className="grid gap-2 rounded-md border bg-card p-3">
						<FormField
							id={`${titleId}-invoice`}
							label="Facture à affecter"
							hint={hint}
						>
							<OpenInvoiceSelect
								id={`${titleId}-invoice`}
								options={openInvoices}
								value={invoiceId}
								onChange={setInvoiceId}
								amount={event.amount}
								disabled={fitting.length === 0}
							/>
						</FormField>
						<div className="flex flex-wrap items-center justify-end gap-2">
							{openInvoices.length === 0 && event.accountId !== null && (
								<Link
									to={buildOrgPath(organizationSlug, '/crm/factures/nouvelle')}
									className="rounded-sm text-primary text-sm underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
								>
									Créer une facture pour ce client
								</Link>
							)}
							<Button
								type="button"
								variant="outline"
								onClick={() => setIgnoring(true)}
							>
								Ignorer
							</Button>
							<Button
								type="button"
								disabled={invoiceId === null}
								onClick={() => {
									const chosen = openInvoices.find(
										(option) => option.id === invoiceId,
									)
									if (chosen) onAllocate(event, chosen)
								}}
							>
								Affecter
							</Button>
						</div>
					</div>
				)}
				{action === 'none' && (
					<div className="flex flex-wrap items-center justify-between gap-3 rounded-md border bg-card p-3">
						<p className="text-sm text-muted-foreground">
							Ce remboursement ne correspond à aucun paiement Stripe connu : il
							ne peut qu’être ignoré.
						</p>
						<Button
							type="button"
							variant="outline"
							onClick={() => setIgnoring(true)}
						>
							Ignorer
						</Button>
					</div>
				)}
			</article>
			<ConfirmDialog
				open={ignoring}
				onOpenChange={setIgnoring}
				title="Ignorer cet événement ?"
				description={
					event.paymentId !== null
						? 'L’événement ne sera plus proposé au rapprochement. Le paiement déjà enregistré reste dans le journal, à affecter à une facture.'
						: 'L’événement ne sera plus proposé au rapprochement.'
				}
				confirmLabel="Ignorer l’événement"
				onConfirm={() => {
					onIgnore(event)
					setIgnoring(false)
				}}
			/>
		</li>
	)
}
