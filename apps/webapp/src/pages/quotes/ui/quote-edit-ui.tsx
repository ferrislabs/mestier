import { Link } from '@tanstack/react-router'
import { Eye, Pencil } from 'lucide-react'
import { useState } from 'react'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import type { Quote } from '#/hooks/use-quotes'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { invoiceDisplayState } from '#/pages/invoices/types'
import type { BillingPipelineStep } from '#/pages/quotes/lib/billing-pipeline'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import { vatNoticeOf } from '#/pages/quotes/lib/paper-parties'
import type { QuoteSideInfo } from '#/pages/quotes/lib/quote-side-info'
import {
	customerDisplayName,
	type QuoteFormValues,
	type QuoteLineFormValues,
} from '#/pages/quotes/types'
import { BillingPipelineStepperUi } from '#/pages/quotes/ui/billing-pipeline-stepper-ui'
import {
	InvoiceStateBadge,
	QuoteStatusBadge,
} from '#/pages/quotes/ui/doc/badges'
import {
	BackLink,
	DetailShell,
	SideSection,
} from '#/pages/quotes/ui/doc/detail-shell'
import {
	AccountLink,
	InvoiceLink,
	QuoteLink,
} from '#/pages/quotes/ui/doc/doc-links'
import {
	DocumentPaper,
	type PaperDocument,
} from '#/pages/quotes/ui/doc/document-paper'
import {
	type HistoryEntry,
	HistoryList,
} from '#/pages/quotes/ui/doc/history-list'
import { LockBanner, Money } from '#/pages/quotes/ui/doc/primitives'
import { type DocTotals, TotalsPanel } from '#/pages/quotes/ui/doc/totals-panel'
import { QuoteActionsPanel } from '#/pages/quotes/ui/quote-actions-panel'
import { QuoteEditForm } from '#/pages/quotes/ui/quote-edit-form'
import type { QuoteOwner } from '#/pages/quotes/ui/quote-terms-fields'
import { LEGAL_IDENTITY_FIELD_LABELS } from '#/pages/settings/types'

type Mode = 'edit' | 'preview'

export interface LinkedInvoice {
	id: string
	label: string
	kindLabel: string
	displayState: Invoice['display_state']
	grossCents: number
}

interface QuoteEditUIProps {
	quote: Quote
	info: QuoteSideInfo
	owners: readonly QuoteOwner[]
	defaultSigner: string
	values: QuoteFormValues
	customer: Customer | undefined
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	photoUrls: Record<string, string | undefined>
	vatEnabled: boolean
	customUnits: string[]
	error: string | null
	isLoading: boolean
	isSaving: boolean
	isDeleting: boolean
	isDuplicating: boolean
	isRevising: boolean
	isUploading: boolean
	isDownloading: boolean
	billingSteps: BillingPipelineStep[]
	linkedInvoices: LinkedInvoice[]
	paper: PaperDocument
	history: HistoryEntry[]
	totals: DocTotals
	unsavedNotice: string | null
	canSave: boolean
	canSend: boolean
	identityIncompleteForSending: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onAddLine: () => string
	onAddCatalogItem: (item: CatalogItem) => string
	onMoveLine: (index: number, delta: -1 | 1) => void
	onDuplicateLine: (index: number) => string
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSave: () => void
	onTransition: (status: Quote['status']) => void
	onAccept: (signer: string) => void
	onDecline: (reason: string) => void
	onRevise: () => void
	onDelete: () => void
	onDuplicate: () => void
	onDownloadPdf: () => void
}

function ModeToggle({
	mode,
	onChange,
}: {
	mode: Mode
	onChange: (mode: Mode) => void
}) {
	return (
		<ToggleGroup
			type="single"
			variant="outline"
			size="sm"
			value={mode}
			onValueChange={(value) => {
				if (value === 'edit' || value === 'preview') onChange(value)
			}}
			aria-label="Affichage du devis"
		>
			<ToggleGroupItem value="edit">
				<Pencil aria-hidden /> Édition
			</ToggleGroupItem>
			<ToggleGroupItem value="preview">
				<Eye aria-hidden /> Aperçu
			</ToggleGroupItem>
		</ToggleGroup>
	)
}

export function QuoteEditUI({
	quote,
	info,
	owners,
	defaultSigner,
	values,
	customer,
	customers,
	customerContexts,
	catalogItems,
	photoUrls,
	vatEnabled,
	customUnits,
	error,
	isLoading,
	isSaving,
	isDeleting,
	isDuplicating,
	isRevising,
	isUploading,
	isDownloading,
	billingSteps,
	linkedInvoices,
	paper,
	history,
	totals,
	unsavedNotice,
	canSave,
	canSend,
	identityIncompleteForSending,
	onChange,
	onLineChange,
	onAddLine,
	onAddCatalogItem,
	onMoveLine,
	onDuplicateLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSave,
	onTransition,
	onAccept,
	onDecline,
	onRevise,
	onDelete,
	onDuplicate,
	onDownloadPdf,
}: QuoteEditUIProps) {
	const { activeOrganization } = useActiveOrganization()
	const [mode, setMode] = useState<Mode>(
		quote.status === 'DRAFT' ? 'edit' : 'preview',
	)
	const draft = quote.status === 'DRAFT'
	const label = quote.reference ?? 'sans numéro'
	const missingLegalIdentityFields =
		activeOrganization.missing_legal_identity_fields

	const main = (
		<>
			<div className="grid gap-3">
				<BackLink to="/crm/devis" label="Devis" />
				<div className="flex flex-wrap items-start justify-between gap-3">
					<div className="min-w-0">
						<div className="flex flex-wrap items-center gap-2.5">
							<h1 className="font-semibold text-2xl tracking-tight">
								{quote.reference
									? `Devis ${quote.reference}`
									: 'Brouillon de devis'}
							</h1>
							<QuoteStatusBadge status={info.displayStatus} />
						</div>
						<p className="mt-1 text-muted-foreground text-sm">
							{customer && (
								<>
									<AccountLink
										customerId={customer.id}
										className="font-medium text-foreground"
									>
										{customerDisplayName(customer)}
									</AccountLink>
									{quote.title && ' · '}
								</>
							)}
							{quote.title}
						</p>
					</div>
					<ModeToggle mode={mode} onChange={setMode} />
				</div>
			</div>

			{error ? (
				<div
					role="alert"
					className="rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm text-state-alert"
				>
					{error}
				</div>
			) : null}

			{identityIncompleteForSending ? (
				<LockBanner tone="warn" title="Ce devis ne peut pas être envoyé">
					<p>
						L’identité légale de l’organisation est incomplète. Il manque{' '}
						{missingLegalIdentityFields
							.map((field) => LEGAL_IDENTITY_FIELD_LABELS[field] ?? field)
							.join(', ')}
						.{' '}
						<a
							href={`${buildOrgPath(activeOrganization.slug, '/settings')}#organisation`}
							className="rounded-sm font-medium text-foreground underline underline-offset-2 outline-none focus-visible:ring-2 focus-visible:ring-ring"
						>
							Compléter dans les paramètres
						</a>
					</p>
				</LockBanner>
			) : null}

			{mode === 'edit' ? (
				<QuoteEditForm
					savedLineCount={quote.lines.length}
					organizationSlug={activeOrganization.slug}
					idPrefix={`quote-${quote.id}`}
					values={values}
					customers={customers}
					customerContexts={customerContexts}
					catalogItems={catalogItems}
					photoUrls={photoUrls}
					vatEnabled={vatEnabled}
					customUnits={customUnits}
					owners={owners}
					isLoading={isLoading}
					isUploading={isUploading}
					onChange={onChange}
					onLineChange={onLineChange}
					onAddLine={onAddLine}
					onAddCatalogItem={onAddCatalogItem}
					onMoveLine={onMoveLine}
					onDuplicateLine={onDuplicateLine}
					onRemoveLine={onRemoveLine}
					onUploadLinePhoto={onUploadLinePhoto}
				/>
			) : (
				<div className="overflow-x-auto rounded-lg bg-muted/50 p-3 sm:p-6">
					<DocumentPaper document={paper} />
				</div>
			)}
		</>
	)

	const side = (
		<>
			<SideSection title="État">
				<div className="grid gap-1.5">
					<QuoteStatusBadge status={info.displayStatus} />
					<p className="text-muted-foreground text-sm">
						{draft ? 'Le numéro est attribué à l’envoi.' : `Numéro ${label}`}
					</p>
					<p
						className={cn(
							'text-sm',
							info.validity.expired
								? 'font-medium text-state-alert'
								: 'text-muted-foreground',
						)}
					>
						{info.validity.text}
						{info.validity.expired ? ' (expiré)' : ''}
					</p>
					{info.answerNote && (
						<p className="text-muted-foreground text-sm">{info.answerNote}</p>
					)}
					{info.declineReason && (
						<p className="text-muted-foreground text-sm">
							Motif : {info.declineReason}
						</p>
					)}
					{info.ownerName && (
						<p className="text-muted-foreground text-sm">
							Responsable : {info.ownerName}
						</p>
					)}
					<p className="text-muted-foreground text-sm">
						Créé le {formatDocumentDate(quote.created_at)}
					</p>
				</div>
			</SideSection>
			<SideSection title="Totaux">
				<TotalsPanel
					totals={totals}
					vatNotice={
						totals.vat.length === 0 ? vatNoticeOf(activeOrganization) : null
					}
					notice={unsavedNotice}
				/>
			</SideSection>
			<SideSection title="Actions">
				<QuoteActionsPanel
					organizationSlug={activeOrganization.slug}
					quoteId={quote.id}
					status={quote.status}
					displayStatus={info.displayStatus}
					label={label}
					validityNote={info.validity.text}
					defaultSigner={defaultSigner}
					canSave={canSave}
					canSend={canSend}
					identityIncompleteForSending={identityIncompleteForSending}
					isSaving={isSaving}
					isDeleting={isDeleting}
					isDuplicating={isDuplicating}
					isRevising={isRevising}
					isDownloading={isDownloading}
					onSave={onSave}
					onTransition={onTransition}
					onAccept={onAccept}
					onDecline={onDecline}
					onRevise={onRevise}
					onDelete={onDelete}
					onDuplicate={onDuplicate}
					onDownloadPdf={onDownloadPdf}
				/>
			</SideSection>
			<SideSection title="Facturation">
				<BillingPipelineStepperUi steps={billingSteps} orientation="vertical" />
				{linkedInvoices.length > 0 && (
					<div className="grid gap-1.5">
						<p className="text-muted-foreground text-xs">Factures liées</p>
						<ul className="grid divide-y rounded-md border">
							{linkedInvoices.map((invoice) => (
								<li
									key={invoice.id}
									className="grid grid-cols-[1fr_auto] items-center gap-x-3 gap-y-1 px-3 py-2 text-sm"
								>
									<span className="min-w-0 truncate">
										<InvoiceLink invoiceId={invoice.id} className="font-medium">
											{invoice.label}
										</InvoiceLink>
										<span className="ml-1.5 text-muted-foreground text-xs">
											{invoice.kindLabel}
										</span>
									</span>
									<Money cents={invoice.grossCents} className="text-right" />
									<InvoiceStateBadge
										state={invoiceDisplayState({
											display_state: invoice.displayState,
										})}
									/>
								</li>
							))}
						</ul>
					</div>
				)}
			</SideSection>
			<SideSection title="Liaisons">
				<dl className="grid gap-2.5">
					<div className="grid gap-0.5">
						<dt className="text-muted-foreground text-xs">Client</dt>
						<dd className="min-w-0 truncate text-sm">
							{customer ? (
								<AccountLink customerId={customer.id} className="font-medium">
									{customerDisplayName(customer)}
								</AccountLink>
							) : (
								<span className="text-muted-foreground">Client inconnu</span>
							)}
						</dd>
					</div>
					<div className="grid gap-0.5">
						<dt className="text-muted-foreground text-xs">Opportunité</dt>
						<dd className="min-w-0 truncate text-sm">
							{info.dealId ? (
								<Link
									to={buildOrgPath(activeOrganization.slug, '/crm/pipeline')}
									className="rounded-sm font-medium outline-none hover:text-primary hover:underline focus-visible:ring-2 focus-visible:ring-ring"
								>
									Voir dans le pipeline
								</Link>
							) : (
								<span className="text-muted-foreground">
									Aucune opportunité liée
								</span>
							)}
						</dd>
					</div>
					{info.revisionOf && (
						<div className="grid gap-0.5">
							<dt className="text-muted-foreground text-xs">Révision de</dt>
							<dd className="min-w-0 truncate text-sm">
								<QuoteLink quoteId={info.revisionOf.id} className="font-medium">
									{info.revisionOf.label}
								</QuoteLink>
							</dd>
						</div>
					)}
					{info.supersededBy && (
						<div className="grid gap-0.5">
							<dt className="text-muted-foreground text-xs">Remplacé par</dt>
							<dd className="min-w-0 truncate text-sm">
								<QuoteLink
									quoteId={info.supersededBy.id}
									className="font-medium"
								>
									{info.supersededBy.label}
								</QuoteLink>
							</dd>
						</div>
					)}
				</dl>
			</SideSection>
			<SideSection title="Historique">
				<HistoryList entries={history} />
			</SideSection>
		</>
	)

	return (
		<DetailShell
			roomy
			main={main}
			side={side}
			mainLabel="Contenu du devis"
			sideLabel="État et actions du devis"
		/>
	)
}
