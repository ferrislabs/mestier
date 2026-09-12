import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	ArrowRightLeft,
	FileText,
	Loader2,
	Send,
	Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import type { Quote, QuoteStatus } from '#/hooks/use-quotes'
import { buildOrgPath } from '#/modules/org-path'
import type { BillingPipelineStep } from '#/pages/quotes/lib/billing-pipeline'
import {
	customerDisplayName,
	formatCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteReferenceLabel,
	quoteStatusLabel,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { BillingPipelineStepperUi } from '#/pages/quotes/ui/billing-pipeline-stepper-ui'
import { QuoteLinesTable } from '#/pages/quotes/ui/quote-lines-table'
import { QuoteTotalsFooter } from '#/pages/quotes/ui/quote-totals-footer'
import { LEGAL_IDENTITY_FIELD_LABELS } from '#/pages/settings/types'

export const QUOTE_STATUSES: QuoteStatus[] = [
	'DRAFT',
	'SENT',
	'ACCEPTED',
	'DECLINED',
	'CANCELLED',
]

interface QuoteEditUIProps {
	quote: Quote
	values: QuoteFormValues
	status: QuoteStatus
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	customUnits: string[]
	error: string | null
	isLoading: boolean
	isSaving: boolean
	isDeleting: boolean
	isUploading: boolean
	billingSteps: BillingPipelineStep[]
	totalCents: number
	canSave: boolean
	canSend: boolean
	identityIncompleteForSending: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onStatusChange: (status: QuoteStatus) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSave: () => void
	onSend: () => void
	onDelete: () => void
}

/**
 * The quote composer for an existing quote, on the same classic
 * settings-form layout as `InvoiceNewUI`: an "Informations" card, a "Lignes"
 * card, and a sticky summary aside — rather than the click-to-edit paper
 * document the create screen still uses. This screen carries state a fresh
 * draft doesn't: a billing pipeline stepper, an inline status selector, a
 * legal-identity warning, and delete/send/"transform to project" actions.
 */
export function QuoteEditUI({
	quote,
	values,
	status,
	customers,
	customerContexts,
	catalogItems,
	customUnits,
	error,
	isLoading,
	isSaving,
	isDeleting,
	isUploading,
	billingSteps,
	totalCents,
	canSave,
	canSend,
	identityIncompleteForSending,
	onChange,
	onStatusChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSave,
	onSend,
	onDelete,
}: QuoteEditUIProps) {
	const { activeOrganization } = useActiveOrganization()
	const vatEnabled = activeOrganization.vat_status?.type === 'subject'
	const missingLegalIdentityFields =
		activeOrganization.missing_legal_identity_fields
	// Only one line open at a time, so a long quote stays readable. Opening on
	// the first line matches the create form.
	const [openLineId, setOpenLineId] = useState<string | null>(
		values.lines[0]?.clientId ?? null,
	)
	const photoPreviews = useFileUrls(
		values.lines.flatMap((line) => line.photoKeys),
	)
	const photoUrls = Object.fromEntries(
		photoPreviews.map((preview) => [preview.key, preview.url]),
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={quoteReferenceLabel(quote.reference)}
				title={quote.title}
				description="Visualisez et modifiez le contenu du devis."
				actions={
					<div className="flex flex-col gap-2 sm:flex-row">
						<Button asChild variant="outline">
							<Link to={buildOrgPath(activeOrganization.slug, '/crm/quotes')}>
								<ArrowLeft />
								Retour
							</Link>
						</Button>
						{quote.status === 'ACCEPTED' ? (
							<RequirePermission permission="MANAGE_QUOTES">
								<Button asChild variant="outline">
									<Link
										to={buildOrgPath(
											activeOrganization.slug,
											`/crm/quotes/${quote.id}/handover`,
										)}
									>
										<ArrowRightLeft />
										Transformer en projet
									</Link>
								</Button>
							</RequirePermission>
						) : null}
						<RequirePermission permission="MANAGE_QUOTES">
							<AlertDialog>
								<AlertDialogTrigger asChild>
									<Button
										type="button"
										variant="destructive"
										disabled={isDeleting || isSaving}
									>
										{isDeleting ? (
											<Loader2 className="animate-spin" />
										) : (
											<Trash2 />
										)}
										Supprimer
									</Button>
								</AlertDialogTrigger>
								<AlertDialogContent>
									<AlertDialogHeader>
										<AlertDialogTitle>Supprimer ce devis ?</AlertDialogTitle>
										<AlertDialogDescription>
											Le devis {quoteReferenceLabel(quote.reference)} sera
											supprimé de la liste. Cette action est irréversible.
										</AlertDialogDescription>
									</AlertDialogHeader>
									<AlertDialogFooter>
										<AlertDialogCancel disabled={isDeleting}>
											Annuler
										</AlertDialogCancel>
										<AlertDialogAction
											disabled={isDeleting}
											className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
											onClick={(event) => {
												event.preventDefault()
												void onDelete()
											}}
										>
											{isDeleting ? (
												<Loader2 className="animate-spin" />
											) : (
												<Trash2 />
											)}
											Supprimer
										</AlertDialogAction>
									</AlertDialogFooter>
								</AlertDialogContent>
							</AlertDialog>
						</RequirePermission>
					</div>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{identityIncompleteForSending ? (
				<div className="flex items-start gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-800 dark:text-amber-300">
					<AlertCircle className="mt-0.5 size-4 shrink-0" />
					<p>
						Ce devis ne peut pas être envoyé : l'identité légale de
						l'organisation est incomplète. Il manque{' '}
						{missingLegalIdentityFields
							.map((field) => LEGAL_IDENTITY_FIELD_LABELS[field] ?? field)
							.join(', ')}
						.{' '}
						<a
							href={`${buildOrgPath(activeOrganization.slug, '/settings')}#organisation`}
							className="font-medium underline underline-offset-2"
						>
							Compléter dans les paramètres
						</a>
					</p>
				</div>
			) : null}

			<SectionCard className="border shadow-sm">
				<div className="p-5 sm:p-6">
					<BillingPipelineStepperUi steps={billingSteps} />
				</div>
			</SectionCard>

			<div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
				<div className="space-y-5">
					<SectionCard>
						<SectionHeader
							title="Informations"
							description="Statut, client et adresse de facturation."
						/>
						<div className="grid gap-4 p-5 md:grid-cols-2">
							<FieldBlock label="Statut" htmlFor="quote-edit-status">
								<Select
									value={status}
									onValueChange={(value) =>
										onStatusChange(value as QuoteStatus)
									}
								>
									<SelectTrigger id="quote-edit-status" className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{QUOTE_STATUSES.map((value) => (
											<SelectItem key={value} value={value}>
												{quoteStatusLabel(value)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Client" htmlFor="quote-edit-customer">
								<Select
									value={values.customerId}
									onValueChange={(customerId) => onChange({ customerId })}
								>
									<SelectTrigger id="quote-edit-customer" className="w-full">
										<SelectValue placeholder="Sélectionner un client" />
									</SelectTrigger>
									<SelectContent>
										{customers.map((customer) => (
											<SelectItem key={customer.id} value={customer.id}>
												{customerDisplayName(customer)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock
								label="Adresse de facturation"
								htmlFor="quote-edit-billing-address"
							>
								<BillingAddressField
									id="quote-edit-billing-address"
									value={values.customerContextId}
									addresses={customerContexts}
									hasCustomer={Boolean(values.customerId)}
									isLoading={isLoading}
									onChange={(customerContextId) =>
										onChange({ customerContextId })
									}
								/>
							</FieldBlock>
							<FieldBlock label="Objet du devis" htmlFor="quote-edit-title">
								<Input
									id="quote-edit-title"
									value={values.title}
									onChange={(event) => onChange({ title: event.target.value })}
									placeholder="Ex. Rénovation salle de bain"
								/>
							</FieldBlock>
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader title={`Lignes (${values.lines.length})`} />
						<QuoteLinesTable
							lines={values.lines}
							catalogItems={catalogItems}
							photosByLine={Object.fromEntries(
								values.lines.map((line) => [
									line.clientId,
									line.photoKeys.map((key) => ({ key, url: photoUrls[key] })),
								]),
							)}
							isUploading={isUploading}
							openLineId={openLineId}
							vatEnabled={vatEnabled}
							customUnits={customUnits}
							onOpenLineChange={(clientId, open) =>
								setOpenLineId(open ? clientId : null)
							}
							onLineChange={(clientId, patch) => {
								const index = values.lines.findIndex(
									(line) => line.clientId === clientId,
								)
								if (index !== -1) onLineChange(index, patch)
							}}
							onSelectCatalogItem={(clientId, catalogItemId) => {
								const index = values.lines.findIndex(
									(line) => line.clientId === clientId,
								)
								if (index !== -1) onSelectCatalogItem(index, catalogItemId)
							}}
							onRemoveLine={(clientId) => {
								const index = values.lines.findIndex(
									(line) => line.clientId === clientId,
								)
								if (index !== -1) onRemoveLine(index)
							}}
							onAddLine={onAddLine}
							onUploadLinePhoto={(clientId, file) => {
								const index = values.lines.findIndex(
									(line) => line.clientId === clientId,
								)
								if (index !== -1) void onUploadLinePhoto(index, file)
							}}
							onRemoveLinePhoto={(clientId, key) => {
								const index = values.lines.findIndex(
									(line) => line.clientId === clientId,
								)
								const line = values.lines[index]
								if (index === -1 || !line) return
								onLineChange(index, {
									photoKeys: line.photoKeys.filter(
										(photoKey) => photoKey !== key,
									),
								})
							}}
						/>
					</SectionCard>
				</div>

				<aside className="h-fit rounded-lg border bg-card p-5 shadow-sm xl:sticky xl:top-5">
					<p className="text-sm font-semibold">Résumé</p>
					{/* The quote's actual totals — read from what was last saved, never
					    recomputed here (CLAUDE.md): the screen and the PDF must never be
					    able to disagree. `totalCents` (from the in-memory draft) only
					    surfaces as the small notice below when it disagrees with what
					    is saved. */}
					<QuoteTotalsFooter
						netCents={quote.net_cents}
						vatBreakdown={quote.vat_breakdown.map((line) => ({
							rateBp: line.rate_bp,
							vatCents: line.vat_cents,
						}))}
						grossCents={quote.gross_cents}
						vatExemptionNotice={
							quote.vat_breakdown.length === 0 &&
							activeOrganization.vat_status?.type === 'not_subject'
								? `TVA non applicable, ${activeOrganization.vat_status.basis}`
								: null
						}
						notice={
							totalCents !== quote.net_cents
								? `Aperçu non enregistré : ${formatCents(totalCents)}`
								: null
						}
					/>
					<div className="mt-5 flex flex-col gap-2">
						<RequirePermission permission="MANAGE_QUOTES">
							<Button
								type="button"
								variant="outline"
								className="w-full"
								disabled={!canSave || isSaving || isDeleting}
								onClick={onSave}
							>
								{isSaving ? <Loader2 className="animate-spin" /> : <FileText />}
								Enregistrer
							</Button>
						</RequirePermission>
						{quote.status === 'DRAFT' ? (
							<RequirePermission permission="MANAGE_QUOTES">
								<Button
									type="button"
									className="w-full"
									disabled={!canSend || isSaving || isDeleting}
									onClick={onSend}
								>
									{isSaving ? <Loader2 className="animate-spin" /> : <Send />}
									Envoyer le devis
								</Button>
							</RequirePermission>
						) : null}
					</div>
				</aside>
			</div>
		</PageShell>
	)
}

function FieldBlock({
	label,
	htmlFor,
	children,
}: {
	label: string
	htmlFor: string
	children: React.ReactNode
}) {
	return (
		<div className="flex min-w-0 flex-col gap-2">
			<Label htmlFor={htmlFor}>{label}</Label>
			{children}
		</div>
	)
}
