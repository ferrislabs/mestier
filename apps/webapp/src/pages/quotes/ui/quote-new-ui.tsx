import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	FileText,
	Loader2,
	UserRound,
} from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { buildOrgPath } from '#/modules/org-path'
import {
	customerDisplayName,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLinesGrossCents,
	quoteLinesVatBreakdown,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { QuoteIssuerBlock } from '#/pages/quotes/ui/quote-issuer-block'
import { QuoteLinesTable } from '#/pages/quotes/ui/quote-lines-table'
import { QuoteTotalsFooter } from '#/pages/quotes/ui/quote-totals-footer'

interface QuoteNewUIProps {
	organizationSlug: string
	organization: Organization
	values: QuoteFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	/** Presigned preview urls by storage key, resolved by the feature layer. */
	photoUrls: Record<string, string | undefined>
	/** Whether the active organization charges VAT — see `QuoteLineEditor`. */
	vatEnabled: boolean
	error?: string | null
	isCreating?: boolean
	isUploading?: boolean
	isCustomerContextsLoading?: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSubmit: () => void
}

/**
 * The quote composer, on a page of its own, laid out to read as the document
 * it becomes rather than as a settings form. It used to be a modal over the
 * quote list; writing a quote takes minutes and deserves a url you can send
 * to someone or reload without losing where you were.
 */
export function QuoteNewUI({
	organizationSlug,
	organization,
	values,
	customers,
	customerContexts,
	catalogItems,
	photoUrls,
	vatEnabled,
	error,
	isCreating,
	isUploading,
	isCustomerContextsLoading,
	onChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSubmit,
}: QuoteNewUIProps) {
	// Only one line is expanded at a time: that is what keeps a six-line quote
	// readable. A blank draft opens on its first line so the form is not a wall
	// of folded rows with nothing to do.
	const [openLineId, setOpenLineId] = useState<string | null>(
		values.lines[0]?.clientId ?? null,
	)

	const canSubmit =
		Boolean(values.title.trim()) &&
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.length > 0 &&
		values.lines.every((line) => {
			return (
				line.label.trim() &&
				line.quantity.trim() &&
				Number(line.quantity.replace(',', '.')) > 0 &&
				line.unitPrice.trim() &&
				Number(line.unitPrice.replace(',', '.')) >= 0
			)
		})

	const netCents = values.lines.reduce((sum, line) => {
		return sum + quoteLineTotalCents(line)
	}, 0)
	const vatBreakdown = quoteLinesVatBreakdown(values.lines, vatEnabled)
	const grossCents = quoteLinesGrossCents(netCents, vatBreakdown)

	return (
		<form
			onSubmit={(event) => {
				event.preventDefault()
				onSubmit()
			}}
		>
			<PageShell>
				<PageHeader
					eyebrow="Devis"
					title="Nouveau devis"
					description="Sélectionnez le client, puis ajoutez des services, produits ou lignes libres."
					actions={
						<div className="flex flex-col gap-2 sm:flex-row">
							<Button asChild type="button" variant="outline">
								<Link to={buildOrgPath(organizationSlug, '/crm/quotes')}>
									<ArrowLeft />
									Retour
								</Link>
							</Button>
							<Button type="submit" disabled={!canSubmit || isCreating}>
								{isCreating ? (
									<Loader2 className="animate-spin" />
								) : (
									<FileText />
								)}
								Créer le devis
							</Button>
						</div>
					}
				/>

				{error ? (
					<SectionCard className="flex items-center gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</SectionCard>
				) : null}

				<SectionCard className="mx-auto w-full max-w-4xl">
					<div className="flex flex-wrap items-start justify-between gap-4 border-b p-5">
						<QuoteIssuerBlock organization={organization} />
						<div className="text-right">
							<p className="text-2xl font-bold tracking-tight">DEVIS</p>
							<p className="text-sm text-muted-foreground">
								Numéro attribué à l’envoi
							</p>
						</div>
					</div>

					<div className="grid gap-4 border-b p-5 md:grid-cols-2">
						<Field label="Client">
							<Select
								value={values.customerId}
								onValueChange={(customerId) => onChange({ customerId })}
							>
								<SelectTrigger className="w-full">
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
						</Field>

						<Field label="Adresse de facturation">
							<BillingAddressField
								value={values.customerContextId}
								addresses={customerContexts}
								hasCustomer={Boolean(values.customerId)}
								isLoading={isCustomerContextsLoading}
								onChange={(customerContextId) =>
									onChange({ customerContextId })
								}
							/>
						</Field>
					</div>

					<div className="border-b p-5">
						<Field label="Objet du devis">
							<Input
								value={values.title}
								onChange={(event) => onChange({ title: event.target.value })}
								placeholder="Ex. Rénovation salle de bain"
							/>
						</Field>
					</div>

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

					<QuoteTotalsFooter
						netCents={netCents}
						vatBreakdown={vatBreakdown}
						grossCents={grossCents}
						vatExemptionNotice={
							organization.vat_status?.type === 'not_subject'
								? `TVA non applicable, ${organization.vat_status.basis}`
								: null
						}
						notice="Estimation, non enregistrée"
					/>
				</SectionCard>

				{customers.length === 0 ? (
					<SectionCard className="mx-auto flex w-full max-w-4xl items-center gap-3 p-5 text-sm text-muted-foreground">
						<UserRound className="size-4 shrink-0" />
						Aucun client n’existe encore dans cette organisation.
					</SectionCard>
				) : null}
			</PageShell>
		</form>
	)
}
