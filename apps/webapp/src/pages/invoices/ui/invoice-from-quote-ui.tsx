import { Link } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Quote } from '#/hooks/use-quotes'
import { buildOrgPath } from '#/modules/org-path'
import {
	defaultDepositPercent,
	depositPercentError,
	depositPreviewCents,
	formatPercent,
	initialQuoteMode,
	parsePercent,
	type QuoteInvoiceMode,
	type QuoteInvoicing,
} from '#/pages/invoices/lib/invoice-new'
import { quoteStatusText } from '#/pages/invoices/types'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { DocBadge } from '#/pages/quotes/ui/doc/badges'
import { BackLink } from '#/pages/quotes/ui/doc/detail-shell'
import {
	AccountLink,
	InvoiceLink,
	QuoteLink,
} from '#/pages/quotes/ui/doc/doc-links'
import { LockBanner, Money, OptionCard } from '#/pages/quotes/ui/doc/primitives'

export interface DepositRow {
	invoiceId: string
	label: string
	draft: boolean
	issuedAt: string | null
	grossCents: number | null
}

export type QuoteInvoiceRequest =
	| { kind: 'full' }
	| { kind: 'deposit'; percent: number }

interface InvoiceFromQuoteUIProps {
	organizationSlug: string
	quote: Quote
	customerName: string | null
	info: QuoteInvoicing
	deposits: DepositRow[]
	existingInvoiceLabel: string | null
	preferred: 'acompte' | 'complete' | undefined
	isCreating: boolean
	error: string | null
	onSubmit: (request: QuoteInvoiceRequest) => void
}

function Recap({
	quote,
	customerName,
	info,
	deposits,
}: Pick<
	InvoiceFromQuoteUIProps,
	'quote' | 'customerName' | 'info' | 'deposits'
>) {
	return (
		<section
			aria-label="Récapitulatif du devis"
			className="rounded-lg border bg-card p-4"
		>
			<dl className="grid gap-x-6 gap-y-3 text-sm sm:grid-cols-2">
				<div>
					<dt className="text-muted-foreground">Devis</dt>
					<dd className="mt-0.5 flex items-center gap-2 font-medium">
						<QuoteLink quoteId={quote.id}>
							{quote.reference ?? 'Devis'}
						</QuoteLink>
						<DocBadge tone={quote.status === 'ACCEPTED' ? 'ok' : 'neutral'}>
							{quoteStatusText(quote.status)}
						</DocBadge>
					</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Client</dt>
					<dd className="mt-0.5 font-medium">
						<AccountLink customerId={quote.customer_id}>
							{customerName ?? 'Client inconnu'}
						</AccountLink>
					</dd>
				</div>
				<div className="sm:col-span-2">
					<dt className="text-muted-foreground">Objet</dt>
					<dd className="mt-0.5">{quote.title || '—'}</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Total TTC du devis</dt>
					<dd className="mt-0.5 font-semibold text-lg">
						<Money cents={quote.gross_cents} />
					</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Acompte déjà facturé</dt>
					<dd className="mt-0.5">
						<span className="font-semibold text-lg">
							{formatPercent(info.invoiced)}
						</span>
						<span className="ml-2 text-muted-foreground">
							du devis, reste {formatPercent(info.remaining)} à facturer
						</span>
					</dd>
				</div>
			</dl>
			{deposits.length > 0 && (
				<ul className="mt-4 grid gap-1 border-t pt-3 text-sm">
					{deposits.map((deposit) => (
						<li
							key={deposit.invoiceId}
							className="flex flex-wrap items-baseline justify-between gap-2"
						>
							<span>
								<InvoiceLink
									invoiceId={deposit.invoiceId}
									className="font-medium"
								>
									{deposit.label}
								</InvoiceLink>
								<span className="ml-2 text-muted-foreground">
									{deposit.draft
										? 'pas encore émis'
										: `émise le ${formatDocumentDate(deposit.issuedAt)}`}
								</span>
							</span>
							{deposit.grossCents !== null && (
								<Money
									cents={deposit.grossCents}
									className="text-muted-foreground"
								/>
							)}
						</li>
					))}
				</ul>
			)}
		</section>
	)
}

export function InvoiceFromQuoteUI({
	organizationSlug,
	quote,
	customerName,
	info,
	deposits,
	existingInvoiceLabel,
	preferred,
	isCreating,
	error,
	onSubmit,
}: InvoiceFromQuoteUIProps) {
	const baseId = useId()
	const [mode, setMode] = useState<QuoteInvoiceMode>(() =>
		initialQuoteMode(info, preferred),
	)
	const [percent, setPercent] = useState(() =>
		String(defaultDepositPercent(info.remaining)).replace('.', ','),
	)
	const percentError =
		mode === 'deposit' ? depositPercentError(info.invoiced, percent) : null
	const parsed = parsePercent(percent)
	const available = info.canFull || info.canDeposit
	const blocked =
		!available ||
		(mode === 'deposit' && (!info.canDeposit || percentError !== null)) ||
		(mode === 'full' && !info.canFull)
	const listPath = buildOrgPath(organizationSlug, '/crm/factures')

	const submit = (event: FormEvent) => {
		event.preventDefault()
		if (blocked) return
		onSubmit(
			mode === 'full'
				? { kind: 'full' }
				: { kind: 'deposit', percent: parsed ?? 0 },
		)
	}

	return (
		<PageShell className="max-w-3xl gap-0">
			<div className="mb-3">
				<BackLink to="/crm/factures" label="Factures" />
			</div>
			<PageHeader
				className="mb-6"
				title="Nouvelle facture"
				description="Facture un devis accepté, en acompte ou en totalité."
			/>
			<div className="grid gap-5">
				<Recap
					quote={quote}
					customerName={customerName}
					info={info}
					deposits={deposits}
				/>
				{info.reasons.length > 0 && (
					<LockBanner
						tone={available ? 'info' : 'warn'}
						title={
							available
								? 'Une option n’est pas disponible'
								: 'Ce devis ne peut pas être facturé'
						}
					>
						<ul className="grid gap-0.5">
							{info.reasons.map((reason) => (
								<li key={reason}>{reason}</li>
							))}
						</ul>
						{info.existingInvoiceId && (
							<p className="mt-1.5">
								<InvoiceLink
									invoiceId={info.existingInvoiceId}
									className="font-medium underline underline-offset-2"
								>
									Voir la facture {existingInvoiceLabel ?? ''}
								</InvoiceLink>
							</p>
						)}
					</LockBanner>
				)}
				{available ? (
					<form className="grid gap-5" noValidate onSubmit={submit}>
						<fieldset className="grid gap-3">
							<legend className="mb-1 font-medium text-sm">
								Type de facture
							</legend>
							<OptionCard
								name={`${baseId}-mode`}
								value="deposit"
								checked={mode === 'deposit'}
								disabled={!info.canDeposit}
								onSelect={() => setMode('deposit')}
								title="Facture d’acompte"
								description={`Facture un pourcentage du devis, dans la limite du reste à facturer (${formatPercent(info.remaining)}).`}
							>
								{mode === 'deposit' && (
									<div className="grid max-w-xs gap-2">
										<FormField
											id={`${baseId}-percent`}
											label="Pourcentage de l’acompte"
											error={percentError ?? undefined}
										>
											<div className="flex items-center gap-2">
												<Input
													id={`${baseId}-percent`}
													inputMode="decimal"
													autoComplete="off"
													value={percent}
													aria-invalid={percentError !== null || undefined}
													onChange={(event) => setPercent(event.target.value)}
													className="w-24 text-right tabular-nums"
												/>
												<span aria-hidden>%</span>
											</div>
										</FormField>
										{percentError === null && parsed !== null && (
											<output className="text-sm">
												Soit{' '}
												<Money
													cents={depositPreviewCents(quote, parsed)}
													className="font-medium"
												/>{' '}
												TTC (aperçu).
											</output>
										)}
									</div>
								)}
							</OptionCard>
							<OptionCard
								name={`${baseId}-mode`}
								value="full"
								checked={mode === 'full'}
								disabled={!info.canFull}
								onSelect={() => setMode('full')}
								title="Facture complète"
								description="Elle reprend toutes les lignes du devis."
							>
								{mode === 'full' && (
									<div className="grid gap-1 text-sm">
										{info.deposits.issued.length > 0 && (
											<p>
												Les acomptes déjà émis ne sont pas déduits
												automatiquement : ajoute une ligne de déduction dans le
												brouillon.
											</p>
										)}
										{info.deposits.drafts.length > 0 && (
											<p className="text-state-today">
												{info.deposits.drafts.length === 1
													? 'Un acompte est encore en brouillon'
													: `${info.deposits.drafts.length} acomptes sont encore en brouillon`}{' '}
												: émets-le avant de créer la facture complète.
											</p>
										)}
									</div>
								)}
							</OptionCard>
						</fieldset>
						{error && (
							<p role="alert" className="text-sm text-state-alert">
								{error}
							</p>
						)}
						<div className="flex flex-wrap justify-end gap-2">
							<Button type="button" variant="outline" asChild>
								<Link to={listPath}>Annuler</Link>
							</Button>
							<RequirePermission permission="MANAGE_INVOICES">
								<Button type="submit" disabled={blocked || isCreating}>
									{isCreating ? <Loader2 className="animate-spin" /> : null}
									{mode === 'deposit'
										? 'Créer la facture d’acompte'
										: 'Créer la facture complète'}
								</Button>
							</RequirePermission>
						</div>
					</form>
				) : (
					<div className="flex flex-wrap justify-end gap-2">
						<Button variant="outline" asChild>
							<Link to={listPath}>Retour aux factures</Link>
						</Button>
					</div>
				)}
			</div>
		</PageShell>
	)
}
