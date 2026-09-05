import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	FileText,
	Loader2,
	UserRound,
} from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { buildOrgPath } from '#/modules/org-path'
import {
	billingAddressLines,
	customerDisplayName,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLinesGrossCents,
	quoteLinesVatBreakdown,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { EditablePaperField } from '#/pages/quotes/ui/editable-paper-field'
import { PaperOptionList } from '#/pages/quotes/ui/paper-option-list'
import {
	QuoteIssuerDetails,
	QuoteIssuerMark,
} from '#/pages/quotes/ui/quote-issuer-block'
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
 * it becomes rather than as a settings form. Every value prints as plain
 * text; clicking a section — the client, the object — opens the actual form
 * control in a popover anchored to that spot, instead of surrounding the
 * document with Selects and Inputs the way a settings page would.
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

	const selectedCustomer = customers.find(
		(customer) => customer.id === values.customerId,
	)
	const selectedCustomerContext = customerContexts.find(
		(customerContext) => customerContext.id === values.customerContextId,
	)

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
					description="Cliquez sur le client ou l’objet du devis pour les renseigner, puis ajoutez des lignes."
					actions={
						<div className="flex flex-col gap-2 sm:flex-row">
							<Button asChild type="button" variant="outline">
								<Link to={buildOrgPath(organizationSlug, '/crm/quotes')}>
									<ArrowLeft />
									Retour
								</Link>
							</Button>
							<RequirePermission permission="MANAGE_QUOTES">
								<Button type="submit" disabled={!canSubmit || isCreating}>
									{isCreating ? (
										<Loader2 className="animate-spin" />
									) : (
										<FileText />
									)}
									Créer le devis
								</Button>
							</RequirePermission>
						</div>
					}
				/>

				{error ? (
					<SectionCard className="flex items-center gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</SectionCard>
				) : null}

				<div className="mx-auto w-full max-w-4xl bg-muted/40 p-4 sm:p-10">
					<SectionCard className="border shadow-sm">
						<div className="border-b p-6 sm:p-8">
							<div className="flex items-start justify-between gap-4">
								<QuoteIssuerMark organization={organization} />
								<div className="text-right">
									<p className="text-sm font-semibold">Nouveau devis</p>
									<p className="text-xs text-muted-foreground">
										Numéro attribué à l’envoi
									</p>
								</div>
							</div>

							<div className="mt-6 grid gap-8 md:grid-cols-2">
								<QuoteIssuerDetails organization={organization} />

								<EditablePaperField
									label="Facturé à"
									renderEditor={(close) => (
										<div className="space-y-4">
											<Field label="Client" htmlFor={null}>
												<PaperOptionList
													ariaLabel="Client"
													value={values.customerId}
													options={customers.map((customer) => ({
														value: customer.id,
														label: customerDisplayName(customer),
													}))}
													onChange={(customerId) => onChange({ customerId })}
													emptyLabel="Aucun client n’existe encore dans cette organisation."
												/>
											</Field>
											<Field label="Adresse de facturation" htmlFor={null}>
												{!values.customerId ? (
													<p className="border px-3 py-2 text-sm text-muted-foreground">
														Choisir un client d’abord
													</p>
												) : isCustomerContextsLoading ? (
													<p className="border px-3 py-2 text-sm text-muted-foreground">
														Chargement…
													</p>
												) : (
													<PaperOptionList
														ariaLabel="Adresse de facturation"
														value={values.customerContextId}
														options={customerContexts.map((context) => ({
															value: context.id,
															label: context.label,
															description:
																billingAddressLines(context).join(' · ') ||
																'Adresse non renseignée',
														}))}
														onChange={(customerContextId) =>
															onChange({ customerContextId })
														}
														emptyLabel="Aucune adresse pour ce client."
													/>
												)}
											</Field>
											<Button
												type="button"
												variant="ghost"
												size="sm"
												onClick={close}
											>
												Fermer
											</Button>
										</div>
									)}
								>
									{selectedCustomer ? (
										<>
											<p className="font-semibold">
												{customerDisplayName(selectedCustomer)}
											</p>
											{selectedCustomerContext ? (
												billingAddressLines(selectedCustomerContext).map(
													(line) => (
														<p
															key={line}
															className="text-sm text-muted-foreground"
														>
															{line}
														</p>
													),
												)
											) : (
												<p className="text-sm text-warning">
													Adresse de facturation non renseignée
												</p>
											)}
											{[selectedCustomer.email, selectedCustomer.phone]
												.filter(Boolean)
												.join(' · ') ? (
												<p className="text-sm text-muted-foreground">
													{[selectedCustomer.email, selectedCustomer.phone]
														.filter(Boolean)
														.join(' · ')}
												</p>
											) : null}
											{selectedCustomer.registration_number ? (
												<p className="mt-1 text-xs text-muted-foreground">
													SIRET {selectedCustomer.registration_number}
												</p>
											) : null}
										</>
									) : (
										<p className="text-sm text-muted-foreground italic">
											Sélectionner un client
										</p>
									)}
								</EditablePaperField>
							</div>

							<div className="mt-8">
								<EditablePaperField
									label="Objet du devis"
									renderEditor={(close) => (
										<div className="space-y-4">
											<Input
												autoFocus
												value={values.title}
												onChange={(event) =>
													onChange({ title: event.target.value })
												}
												placeholder="Ex. Rénovation salle de bain"
											/>
											<Button
												type="button"
												variant="ghost"
												size="sm"
												onClick={close}
											>
												Fermer
											</Button>
										</div>
									)}
								>
									<h1 className="text-2xl font-bold tracking-tight">
										{values.title || 'Objet du devis'}
									</h1>
								</EditablePaperField>
							</div>
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
						<p className="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
							<UserRound className="size-4 shrink-0" />
							Aucun client n’existe encore dans cette organisation.
						</p>
					) : null}
				</div>
			</PageShell>
		</form>
	)
}
