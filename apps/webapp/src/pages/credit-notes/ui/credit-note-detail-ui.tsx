import { Download, Loader2, Lock } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import {
	customerDisplayName,
	type InvoiceDisplayState,
} from '#/pages/invoices/types'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { formatVatRateBp } from '#/pages/quotes/types'
import { DocBadge, InvoiceStateBadge } from '#/pages/quotes/ui/doc/badges'
import {
	BackLink,
	DetailShell,
	SideSection,
} from '#/pages/quotes/ui/doc/detail-shell'
import { AccountLink, InvoiceLink } from '#/pages/quotes/ui/doc/doc-links'
import {
	DocumentPaper,
	type PaperDocument,
} from '#/pages/quotes/ui/doc/document-paper'
import { LockBanner, Money } from '#/pages/quotes/ui/doc/primitives'
import type { DocTotals } from '#/pages/quotes/ui/doc/totals-panel'

function Row({ label, children }: { label: string; children: ReactNode }) {
	return (
		<div className="flex items-baseline justify-between gap-4">
			<dt className="text-muted-foreground">{label}</dt>
			<dd className="text-right">{children}</dd>
		</div>
	)
}

interface CreditNoteDetailUIProps {
	note: Invoice
	source: Invoice | undefined
	sourceState: InvoiceDisplayState | null
	sourceRemainingCents: number | null
	customer: Customer | undefined
	kindLabel: string
	paper: PaperDocument
	totals: DocTotals
	vatNotice: string | null
	error: string | null
	isDownloading: boolean
	onDownloadPdf: () => void
}

export function CreditNoteDetailUI({
	note,
	source,
	sourceState,
	sourceRemainingCents,
	customer,
	kindLabel,
	paper,
	totals,
	vatNotice,
	error,
	isDownloading,
	onDownloadPdf,
}: CreditNoteDetailUIProps) {
	const cancellation = kindLabel === 'Annulation totale'
	const main = (
		<>
			<header className="grid gap-3">
				<BackLink to="/crm/avoirs" label="Avoirs" />
				<div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
					<Lock
						className="size-5 text-muted-foreground"
						role="img"
						aria-label="Avoir verrouillé"
					/>
					<h1 className="font-semibold text-2xl tracking-tight">
						{note.number ?? 'Brouillon'}
					</h1>
					<DocBadge tone={cancellation ? 'neutral' : 'warn'}>
						{kindLabel}
					</DocBadge>
				</div>
				<p className="text-muted-foreground text-sm">
					{source
						? `Sur la facture ${source.number ?? 'en brouillon'}`
						: 'Avoir'}
					{customer && ` · ${customerDisplayName(customer)}`}
				</p>
				<div className="flex flex-wrap items-center gap-2">
					<Button
						type="button"
						size="sm"
						variant="outline"
						disabled={isDownloading}
						onClick={onDownloadPdf}
					>
						{isDownloading ? (
							<Loader2 className="animate-spin" />
						) : (
							<Download aria-hidden />
						)}
						Télécharger le PDF
					</Button>
				</div>
			</header>
			{error ? (
				<div
					role="alert"
					className="rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm text-state-alert"
				>
					{error}
				</div>
			) : null}
			<LockBanner title="Avoir émis : il ne peut plus être modifié">
				Il est archivé tel qu’il a été émis, dans la série numérotée des avoirs.
			</LockBanner>
			<DocumentPaper document={paper} />
		</>
	)

	const side = (
		<>
			<SideSection title="Facture d’origine">
				{source ? (
					<div className="grid gap-2">
						<div className="flex flex-wrap items-center gap-2">
							<InvoiceLink invoiceId={source.id} className="font-medium">
								{source.number ?? 'Brouillon'}
							</InvoiceLink>
							{sourceState && <InvoiceStateBadge state={sourceState} />}
						</div>
						<dl className="grid gap-1.5 rounded-lg border bg-background p-3 text-sm">
							<Row label="Total TTC">
								<Money cents={source.gross_cents} />
							</Row>
							<Row label="Reste dû">
								{sourceRemainingCents === null ? (
									<span className="text-muted-foreground">—</span>
								) : (
									<Money cents={Math.max(0, sourceRemainingCents)} />
								)}
							</Row>
						</dl>
					</div>
				) : (
					<p className="text-muted-foreground text-sm">
						La facture d’origine est introuvable.
					</p>
				)}
			</SideSection>
			<SideSection title="Montant crédité">
				<dl className="grid gap-1.5 rounded-lg border bg-background p-3 text-sm">
					<Row label="Total HT">
						<Money cents={totals.netCents} />
					</Row>
					{totals.vat.map((line) => (
						<Row
							key={line.rateBp}
							label={`TVA ${formatVatRateBp(line.rateBp)}`}
						>
							<Money cents={line.vatCents} />
						</Row>
					))}
					{vatNotice && (
						<p className="text-muted-foreground text-xs">{vatNotice}</p>
					)}
					<div className="flex items-baseline justify-between gap-4 border-t pt-2 font-semibold">
						<dt>Total TTC crédité</dt>
						<dd>
							<Money cents={totals.grossCents} />
						</dd>
					</div>
				</dl>
			</SideSection>
			<SideSection title="Motif">
				<p className="whitespace-pre-line text-sm">
					{note.reason ?? 'Aucun motif renseigné.'}
				</p>
				<p className="text-muted-foreground text-xs">
					{kindLabel}
					{cancellation
						? ' : toute la facture d’origine est créditée.'
						: ' : seules les quantités indiquées sont créditées.'}
				</p>
			</SideSection>
			<SideSection title="Client">
				{customer ? (
					<AccountLink customerId={customer.id} className="font-medium text-sm">
						{customerDisplayName(customer)}
					</AccountLink>
				) : (
					<p className="text-muted-foreground text-sm">Client introuvable.</p>
				)}
			</SideSection>
			<SideSection title="Émission">
				<p className="text-sm">
					Émis le {formatDocumentDate(note.issued_at ?? note.created_at)}.
				</p>
			</SideSection>
		</>
	)

	return <DetailShell main={main} side={side} sideLabel="Détails de l’avoir" />
}
