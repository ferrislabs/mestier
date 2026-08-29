import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	Calculator,
	FileText,
	Loader2,
	MapPin,
	Plus,
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
import { buildOrgPath } from '#/modules/org-path'
import {
	billingAddressLines,
	customerDisplayName,
	formatCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { QuoteLineEditor } from '#/pages/quotes/ui/quote-line-editor'

interface QuoteNewUIProps {
	organizationSlug: string
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
 * The quote composer, on a page of its own.
 *
 * It used to be a modal over the quote list. Writing a quote is not a dialog:
 * it takes minutes, it has its own scroll, and it deserves a url you can send
 * to someone or reload without losing where you were.
 */
export function QuoteNewUI({
	organizationSlug,
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

	const selectedCustomer = customers.find((customer) => {
		return customer.id === values.customerId
	})
	const selectedCustomerContext = customerContexts.find((customerContext) => {
		return customerContext.id === values.customerContextId
	})
	const draftTotalCents = values.lines.reduce((sum, line) => {
		return sum + quoteLineTotalCents(line)
	}, 0)
	const completedLineCount = values.lines.filter((line) => {
		return line.label.trim() && quoteLineTotalCents(line) > 0
	}).length
	const serviceCount = catalogItems.filter(
		(item) => item.type === 'SERVICE',
	).length
	const productCount = catalogItems.filter(
		(item) => item.type === 'PRODUCT',
	).length

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

				<div className="grid gap-5 p-5 xl:grid-cols-[minmax(0,1fr)_300px]">
					<div className="space-y-5">
						<FormSection
							icon={<UserRound className="size-4" />}
							title="Objet et client"
						>
							<div className="grid gap-4 md:grid-cols-2">
								<div className="md:col-span-2">
									<Field label="Objet du devis">
										<Input
											value={values.title}
											onChange={(event) =>
												onChange({ title: event.target.value })
											}
											placeholder="Ex. Rénovation salle de bain"
										/>
									</Field>
								</div>
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
						</FormSection>

						<FormSection
							icon={<FileText className="size-4" />}
							title="Lignes du devis"
							description={`${serviceCount} service${
								serviceCount > 1 ? 's' : ''
							} · ${productCount} produit${productCount > 1 ? 's' : ''}`}
							actions={
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={onAddLine}
								>
									<Plus />
									Ajouter
								</Button>
							}
						>
							<div className="-m-4 divide-y">
								{values.lines.map((line, index) => (
									<QuoteLineEditor
										key={line.clientId}
										index={index}
										line={line}
										catalogItems={catalogItems}
										photos={line.photoKeys.map((key) => ({
											key,
											url: photoUrls[key],
										}))}
										isOpen={openLineId === line.clientId}
										canRemove={values.lines.length > 1}
										isUploading={isUploading}
										vatEnabled={vatEnabled}
										onOpenChange={(open) =>
											setOpenLineId(open ? line.clientId : null)
										}
										onChange={(patch) => onLineChange(index, patch)}
										onSelectCatalogItem={(catalogItemId) =>
											onSelectCatalogItem(index, catalogItemId)
										}
										onRemove={() => onRemoveLine(index)}
										onUploadPhoto={(file) =>
											void onUploadLinePhoto(index, file)
										}
										onRemovePhoto={(key) =>
											onLineChange(index, {
												photoKeys: line.photoKeys.filter(
													(photoKey) => photoKey !== key,
												),
											})
										}
									/>
								))}
							</div>
						</FormSection>
					</div>

					<QuoteDraftSummary
						title={values.title.trim() || 'Non renseigné'}
						customerName={
							selectedCustomer
								? customerDisplayName(selectedCustomer)
								: 'Non sélectionné'
						}
						billingAddress={
							selectedCustomerContext
								? [
										selectedCustomerContext.label,
										...billingAddressLines(selectedCustomerContext),
									].join(' · ')
								: 'Non sélectionnée'
						}
						lineCount={values.lines.length}
						completedLineCount={completedLineCount}
						totalCents={draftTotalCents}
						canSubmit={canSubmit}
					/>
				</div>
			</PageShell>
		</form>
	)
}

interface FormSectionProps {
	icon: React.ReactNode
	title: string
	description?: string
	actions?: React.ReactNode
	children: React.ReactNode
}

function FormSection({
	icon,
	title,
	description,
	actions,
	children,
}: FormSectionProps) {
	return (
		<section className="rounded-lg border bg-card shadow-sm">
			<div className="flex items-center justify-between gap-3 border-b px-4 py-3">
				<div className="flex min-w-0 items-center gap-3">
					<div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
						{icon}
					</div>
					<div className="min-w-0">
						<h2 className="truncate font-semibold">{title}</h2>
						{description ? (
							<p className="truncate text-xs text-muted-foreground">
								{description}
							</p>
						) : null}
					</div>
				</div>
				{actions}
			</div>
			<div className="p-4">{children}</div>
		</section>
	)
}

interface QuoteDraftSummaryProps {
	title: string
	customerName: string
	billingAddress: string
	lineCount: number
	completedLineCount: number
	totalCents: number
	canSubmit: boolean
}

function QuoteDraftSummary({
	title,
	customerName,
	billingAddress,
	lineCount,
	completedLineCount,
	totalCents,
	canSubmit,
}: QuoteDraftSummaryProps) {
	return (
		<aside className="h-fit rounded-lg border bg-card p-4 shadow-sm xl:sticky xl:top-5">
			<div className="flex items-center gap-3">
				<div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
					<Calculator className="size-5" />
				</div>
				<div>
					<p className="text-sm font-semibold">Aperçu du devis</p>
					<p className="text-xs text-muted-foreground">
						{canSubmit ? 'Prêt à créer' : 'Brouillon incomplet'}
					</p>
				</div>
			</div>

			<div className="mt-5 space-y-4">
				<SummaryRow icon={<FileText />} label="Objet" value={title} />
				<SummaryRow icon={<UserRound />} label="Client" value={customerName} />
				<SummaryRow
					icon={<MapPin />}
					label="Adresse de facturation"
					value={billingAddress}
				/>
				<SummaryRow
					icon={<FileText />}
					label="Lignes"
					value={`${completedLineCount}/${lineCount} chiffrée${
						completedLineCount > 1 ? 's' : ''
					}`}
				/>
			</div>

			<div className="mt-5 rounded-lg bg-muted p-4">
				<p className="text-xs font-medium text-muted-foreground">
					Total HT estimé
				</p>
				<p className="mt-1 text-2xl font-bold">{formatCents(totalCents)}</p>
			</div>
		</aside>
	)
}

interface SummaryRowProps {
	icon: React.ReactNode
	label: string
	value: string
}

function SummaryRow({ icon, label, value }: SummaryRowProps) {
	return (
		<div className="flex min-w-0 items-start gap-3">
			<div className="mt-0.5 text-muted-foreground [&>svg]:size-4">{icon}</div>
			<div className="min-w-0">
				<p className="text-xs font-medium text-muted-foreground">{label}</p>
				<p className="truncate text-sm font-medium">{value}</p>
			</div>
		</div>
	)
}
