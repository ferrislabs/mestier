import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'
import {
	type BillingFrequency,
	FREQUENCY_LABELS,
	FREQUENCY_SUFFIX,
} from '#/pages/quotes/lib/billing-frequency'
import { formatDocumentDate, formatSiret } from '#/pages/quotes/lib/doc-format'
import { formatUnit, formatVatRateBp } from '#/pages/quotes/types'
import { Money } from '#/pages/quotes/ui/doc/primitives'
import type { DocTotals } from '#/pages/quotes/ui/doc/totals-panel'

export type PaperKind = 'quote' | 'invoice' | 'creditNote'

export interface PaperIssuer {
	name: string
	legalForm: string
	capital: string
	addressLines: string[]
	phone: string
	email: string
	legalId: string
	vatNumber: string
	insurance: string
}

export interface PaperRecipient {
	name: string
	contactName: string
	addressLines: string[]
	legalId: string
}

export interface PaperLine {
	id: string
	title: string
	details: string
	quantity: string
	unit: string | null
	unitPriceCents: number
	vatRateBp: number | null
	totalCents: number
	discountPercent?: number
	frequency?: BillingFrequency
}

export interface PaperDocument {
	kind: PaperKind
	draft: boolean
	heading: string
	number: string | null
	title: string
	issueDate: string | null
	dueDate: string | null
	validUntil?: string | null
	siteAddressLines?: readonly string[]
	issuer: PaperIssuer
	recipient: PaperRecipient
	lines: readonly PaperLine[]
	totals: DocTotals
	showVat: boolean
	vatNotice: string | null
	notes: string
	reference: string | null
	reason: string
}

function Lines({ lines }: { lines: readonly string[] }) {
	return (
		<>
			{lines.map((line) => (
				<p key={line}>{line}</p>
			))}
		</>
	)
}

function Caption({ children }: { children: ReactNode }) {
	return (
		<p className="mb-1 font-medium text-[10px] text-zinc-500 uppercase tracking-wider">
			{children}
		</p>
	)
}

function IssuerBlock({ issuer }: { issuer: PaperIssuer }) {
	const status = [
		issuer.legalForm,
		issuer.capital && `au capital de ${issuer.capital}`,
	]
		.filter(Boolean)
		.join(' ')
	return (
		<div className="min-w-0 space-y-0.5">
			<p className="font-semibold text-base">{issuer.name}</p>
			{status && <p className="text-zinc-600">{status}</p>}
			<div className="text-zinc-600">
				<Lines lines={issuer.addressLines} />
			</div>
			{issuer.phone && <p className="text-zinc-600">Tél. {issuer.phone}</p>}
			{issuer.email && <p className="text-zinc-600">{issuer.email}</p>}
			<div className="pt-1 text-[11px] text-zinc-500">
				{issuer.legalId && <p>SIRET {formatSiret(issuer.legalId)}</p>}
				{issuer.vatNumber && <p>TVA {issuer.vatNumber}</p>}
			</div>
		</div>
	)
}

function DateRow({ label, date }: { label: string; date: string | null }) {
	return (
		<div className="flex justify-end gap-2">
			<dt className="text-zinc-500">{label}</dt>
			<dd className="tabular font-medium">{formatDocumentDate(date)}</dd>
		</div>
	)
}

function TitleBlock({ document }: { document: PaperDocument }) {
	return (
		<div className="space-y-1 text-right">
			<p className="font-semibold text-2xl uppercase tracking-wide">
				{document.heading}
			</p>
			<p className="font-medium">
				{document.number
					? `N° ${document.number}`
					: 'Numéro attribué à l’émission'}
			</p>
			<dl className="space-y-0.5 pt-1">
				<DateRow label="Date d’émission" date={document.issueDate} />
				{document.kind === 'invoice' && (
					<DateRow label="Échéance" date={document.dueDate} />
				)}
				{document.kind === 'quote' && document.validUntil !== undefined && (
					<DateRow label="Valable jusqu’au" date={document.validUntil} />
				)}
			</dl>
		</div>
	)
}

function RecipientBlock({ recipient }: { recipient: PaperRecipient }) {
	const empty = recipient.name === ''
	return (
		<div className="min-w-56 rounded-sm border border-zinc-300 p-4">
			<Caption>Destinataire</Caption>
			{empty ? (
				<p className="text-zinc-400 italic">Client à renseigner</p>
			) : (
				<div className="space-y-0.5">
					<p className="font-semibold">{recipient.name}</p>
					{recipient.contactName && (
						<p className="text-zinc-600">
							À l’attention de {recipient.contactName}
						</p>
					)}
					<Lines lines={recipient.addressLines} />
					{recipient.legalId && (
						<p className="pt-1 text-[11px] text-zinc-500">
							SIRET {formatSiret(recipient.legalId)}
						</p>
					)}
				</div>
			)}
		</div>
	)
}

function SiteBlock({ lines }: { lines: readonly string[] }) {
	return (
		<div className="min-w-48">
			<Caption>Adresse du chantier</Caption>
			<div className="space-y-0.5">
				<Lines lines={lines} />
			</div>
		</div>
	)
}

function formatPercent(value: number): string {
	return `${new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 2 }).format(value)} %`
}

function Amount({ cents, className }: { cents: number; className?: string }) {
	return (
		<Money
			cents={cents}
			className={cn(cents < 0 && 'text-red-700', className)}
		/>
	)
}

const HEAD =
	'border-zinc-300 border-y bg-zinc-50 px-2 py-1.5 font-medium text-[10px] text-zinc-500 uppercase tracking-wider'

function LinesTable({ document }: { document: PaperDocument }) {
	const showUnit = document.lines.some((line) => line.unit !== null)
	const showDiscount = document.lines.some(
		(line) => line.discountPercent !== undefined,
	)
	const columns =
		4 + (showUnit ? 1 : 0) + (showDiscount ? 1 : 0) + (document.showVat ? 1 : 0)
	return (
		<div className="overflow-x-auto">
			<table className="w-full min-w-[34rem] border-collapse text-left">
				<caption className="sr-only">Détail des lignes</caption>
				<thead>
					<tr>
						<th scope="col" className={HEAD}>
							Désignation
						</th>
						<th scope="col" className={cn(HEAD, 'text-right')}>
							Qté
						</th>
						{showUnit && (
							<th scope="col" className={HEAD}>
								Unité
							</th>
						)}
						<th scope="col" className={cn(HEAD, 'text-right')}>
							PU HT
						</th>
						{showDiscount && (
							<th scope="col" className={cn(HEAD, 'text-right')}>
								Remise
							</th>
						)}
						{document.showVat && (
							<th scope="col" className={cn(HEAD, 'text-right')}>
								TVA
							</th>
						)}
						<th scope="col" className={cn(HEAD, 'text-right')}>
							Total HT
						</th>
					</tr>
				</thead>
				<tbody>
					{document.lines.length === 0 && (
						<tr>
							<td
								colSpan={columns}
								className="border-zinc-200 border-b px-2 py-6 text-center text-zinc-400 italic"
							>
								Aucune ligne
							</td>
						</tr>
					)}
					{document.lines.map((line) => (
						<tr key={line.id} className="border-zinc-200 border-b align-top">
							<td className="px-2 py-2">
								<p className="font-medium">{line.title}</p>
								{line.details && (
									<p className="whitespace-pre-line text-[11px] text-zinc-600">
										{line.details}
									</p>
								)}
							</td>
							<td className="tabular px-2 py-2 text-right">{line.quantity}</td>
							{showUnit && (
								<td className="px-2 py-2 text-zinc-600">
									{line.unit ? formatUnit(line.unit) : ''}
								</td>
							)}
							<td className="px-2 py-2 text-right">
								<Amount cents={line.unitPriceCents} />
								{line.frequency && line.frequency !== 'one_shot' && (
									<span className="block text-[11px] text-zinc-500">
										{FREQUENCY_SUFFIX[line.frequency]}
									</span>
								)}
							</td>
							{showDiscount && (
								<td className="tabular px-2 py-2 text-right text-zinc-600">
									{line.discountPercent
										? formatPercent(line.discountPercent)
										: '—'}
								</td>
							)}
							{document.showVat && (
								<td className="tabular whitespace-nowrap px-2 py-2 text-right text-zinc-600">
									{line.vatRateBp === null
										? '—'
										: formatVatRateBp(line.vatRateBp)}
								</td>
							)}
							<td className="px-2 py-2 text-right font-medium">
								<Amount cents={line.totalCents} />
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	)
}

function TotalsBlock({ document }: { document: PaperDocument }) {
	const { totals } = document
	return (
		<div className="ml-auto grid w-full max-w-72 gap-3">
			<dl className="ml-auto w-full max-w-72 space-y-1">
				<div className="flex justify-between gap-4">
					<dt className="text-zinc-600">Total HT</dt>
					<dd>
						<Amount cents={totals.netCents} />
					</dd>
				</div>
				{totals.vat.map((row) => (
					<div key={row.rateBp} className="flex justify-between gap-4">
						<dt className="text-zinc-600">TVA {formatVatRateBp(row.rateBp)}</dt>
						<dd>
							<Amount cents={row.vatCents} />
						</dd>
					</div>
				))}
				{document.vatNotice && (
					<p className="text-[11px] text-zinc-500">{document.vatNotice}</p>
				)}
				<div className="flex items-baseline justify-between gap-4 border-zinc-900 border-t-2 pt-2 font-semibold text-sm">
					<dt>
						{document.kind === 'creditNote' ? 'Total TTC crédité' : 'Total TTC'}
					</dt>
					<dd className="text-base">
						<Amount cents={totals.grossCents} />
					</dd>
				</div>
				{totals.recurring && totals.recurring.length > 0 && (
					<div className="space-y-1 border-zinc-200 border-t pt-2">
						<dt className="text-[11px] text-zinc-500">
							Dont facturé de façon récurrente
						</dt>
						{totals.recurring.map((row) => (
							<div key={row.frequency} className="flex justify-between gap-4">
								<dt className="text-zinc-600">
									{FREQUENCY_LABELS[row.frequency]} (HT)
								</dt>
								<dd>
									<Amount cents={row.netCents} />{' '}
									{FREQUENCY_SUFFIX[row.frequency]}
								</dd>
							</div>
						))}
					</div>
				)}
			</dl>
		</div>
	)
}

export function DocumentPaper({
	document,
	className,
}: {
	document: PaperDocument
	className?: string
}) {
	const { issuer } = document
	return (
		<article
			aria-label={`${document.heading} ${document.number ?? 'en brouillon'}`}
			className={cn(
				'relative isolate mx-auto flex w-full max-w-[210mm] flex-col gap-6 overflow-hidden rounded-sm border border-zinc-200 bg-white p-6 text-[12px] text-zinc-900 leading-relaxed shadow-lg [color-scheme:light] sm:p-10 sm:text-[13px] md:min-h-[297mm]',
				className,
			)}
		>
			<header className="flex flex-wrap items-start justify-between gap-6">
				<IssuerBlock issuer={issuer} />
				<TitleBlock document={document} />
			</header>

			<div className="flex flex-wrap items-start justify-between gap-4">
				{document.siteAddressLines && document.siteAddressLines.length > 0 ? (
					<SiteBlock lines={document.siteAddressLines} />
				) : (
					<span />
				)}
				<RecipientBlock recipient={document.recipient} />
			</div>

			<div className="space-y-0.5">
				<p>
					<span className="font-semibold">Objet : </span>
					{document.title || (
						<span className="text-zinc-400 italic">Sans objet</span>
					)}
				</p>
				{document.reference && (
					<p className="text-zinc-600">Référence : {document.reference}</p>
				)}
			</div>

			<LinesTable document={document} />
			<TotalsBlock document={document} />

			{document.reason && (
				<div className="rounded-sm border border-zinc-300 bg-zinc-50 px-4 py-3">
					<Caption>Motif de l’avoir</Caption>
					<p className="whitespace-pre-line">{document.reason}</p>
				</div>
			)}

			{document.notes && (
				<div>
					<Caption>Notes</Caption>
					<p className="whitespace-pre-line">{document.notes}</p>
				</div>
			)}

			{document.kind !== 'creditNote' && issuer.insurance && (
				<div className="space-y-1.5 border-zinc-200 border-t pt-4 text-[11px] text-zinc-600">
					<p className="whitespace-pre-line">{issuer.insurance}</p>
				</div>
			)}

			{document.kind === 'quote' && (
				<div className="space-y-2">
					<p className="text-[11px] text-zinc-600">
						Bon pour accord : date et signature du client, précédées de la
						mention « Lu et approuvé ».
					</p>
					<div className="h-20 rounded-sm border border-zinc-300 border-dashed" />
				</div>
			)}

			{document.draft && (
				<div
					aria-hidden
					className="pointer-events-none absolute inset-0 flex select-none items-center justify-center"
				>
					<span className="-rotate-30 font-bold text-[4.5rem] text-zinc-900/[0.06] uppercase tracking-widest sm:text-[8rem]">
						Brouillon
					</span>
				</div>
			)}
		</article>
	)
}
