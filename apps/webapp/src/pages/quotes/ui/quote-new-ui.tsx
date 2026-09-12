import { Link } from '@tanstack/react-router'
import { ArrowLeft, FileText, Loader2, UserRound } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
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
	/** The organization's own units (#449), offered alongside the built-in
	 * ones in each line's unit picker — see `QuoteLineEditor`. */
	customUnits?: string[]
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
 * The quote composer, laid out as the same classic settings form as the
 * invoice builder: client and object up top, lines below, totals and the
 * submit action in a sticky summary aside.
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
	customUnits,
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
					title="Nouveau devis"
					description="Renseignez le client, l'objet et les lignes du devis."
					actions={
						<Button asChild type="button" variant="outline">
							<Link to={buildOrgPath(organizationSlug, '/crm/quotes')}>
								<ArrowLeft />
								Retour
							</Link>
						</Button>
					}
				/>

				{error ? (
					<SectionCard className="p-5 text-sm text-destructive">
						{error}
					</SectionCard>
				) : null}

				<div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
					<div className="space-y-5">
						<SectionCard>
							<SectionHeader
								title="Informations"
								description="Client, adresse de facturation et objet du devis."
							/>
							<div className="grid gap-4 p-5 md:grid-cols-2">
								<FieldBlock label="Client">
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
									{customers.length === 0 ? (
										<p className="flex items-center gap-1.5 text-xs text-muted-foreground">
											<UserRound className="size-3.5 shrink-0" />
											Aucun client n’existe encore dans cette organisation.
										</p>
									) : null}
								</FieldBlock>
								<FieldBlock label="Adresse de facturation">
									<BillingAddressField
										value={values.customerContextId}
										addresses={customerContexts}
										hasCustomer={Boolean(values.customerId)}
										isLoading={isCustomerContextsLoading}
										onChange={(customerContextId) =>
											onChange({ customerContextId })
										}
									/>
								</FieldBlock>
								<FieldBlock label="Objet du devis" className="md:col-span-2">
									<Input
										value={values.title}
										onChange={(event) =>
											onChange({ title: event.target.value })
										}
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
										line.photoKeys.map((key) => ({
											key,
											url: photoUrls[key],
										})),
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
						<div className="mt-4 space-y-3 text-sm">
							<SummaryRow label="Lignes" value={String(values.lines.length)} />
						</div>
						<div className="-mx-5 mt-2">
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
						</div>
						<RequirePermission permission="MANAGE_QUOTES">
							<Button
								type="submit"
								className="mt-5 w-full"
								disabled={!canSubmit || isCreating}
							>
								{isCreating ? (
									<Loader2 className="animate-spin" />
								) : (
									<FileText />
								)}
								Créer le devis
							</Button>
						</RequirePermission>
					</aside>
				</div>
			</PageShell>
		</form>
	)
}

function FieldBlock({
	label,
	children,
	className,
}: {
	label: string
	children: React.ReactNode
	className?: string
}) {
	return (
		<div className={`flex min-w-0 flex-col gap-2 ${className ?? ''}`.trim()}>
			<Label>{label}</Label>
			{children}
		</div>
	)
}

function SummaryRow({ label, value }: { label: string; value: string }) {
	return (
		<div className="flex items-center justify-between gap-4">
			<span className="text-muted-foreground">{label}</span>
			<span className="truncate font-medium">{value}</span>
		</div>
	)
}
