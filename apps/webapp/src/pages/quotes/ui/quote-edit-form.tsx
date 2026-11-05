import { Link } from '@tanstack/react-router'
import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import {
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { CustomerPicker } from '#/pages/quotes/ui/doc/customer-picker'
import { LinesEditor } from '#/pages/quotes/ui/doc/lines-editor'
import { FormCard, LockBanner } from '#/pages/quotes/ui/doc/primitives'

interface QuoteEditFormProps {
	organizationSlug: string
	idPrefix: string
	values: QuoteFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	photoUrls: Record<string, string | undefined>
	vatEnabled: boolean
	customUnits: string[]
	isLoading: boolean
	isUploading: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onAddLine: () => string
	onAddCatalogItem: (item: CatalogItem) => string
	onMoveLine: (index: number, delta: -1 | 1) => void
	onDuplicateLine: (index: number) => string
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
}

export function QuoteEditForm({
	organizationSlug,
	idPrefix,
	values,
	customers,
	customerContexts,
	catalogItems,
	photoUrls,
	vatEnabled,
	customUnits,
	isLoading,
	isUploading,
	onChange,
	onLineChange,
	onAddLine,
	onAddCatalogItem,
	onMoveLine,
	onDuplicateLine,
	onRemoveLine,
	onUploadLinePhoto,
}: QuoteEditFormProps) {
	const hasCustomer = values.customerId !== ''
	const noAddress = hasCustomer && !isLoading && customerContexts.length === 0
	return (
		<div className="grid gap-5">
			<FormCard
				id={`${idPrefix}-client-title`}
				title="Client et destinataire"
				description="Les coordonnées affichées sur le devis viennent de la fiche du client."
			>
				<FormField
					id={`${idPrefix}-customer`}
					label="Client"
					required
					hint="Changer de client réinitialise l’adresse de facturation."
				>
					<CustomerPicker
						id={`${idPrefix}-customer`}
						customers={customers}
						value={hasCustomer ? values.customerId : null}
						onChange={(customerId) => onChange({ customerId })}
					/>
				</FormField>
				{noAddress && (
					<LockBanner tone="warn" title="Ce client n’a aucune adresse">
						<p>
							Le devis ne pourra pas être envoyé sans adresse de facturation.{' '}
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/crm/customers/$customerId',
								)}
								params={{ customerId: values.customerId }}
								className="rounded-sm font-medium text-foreground underline underline-offset-2 outline-none focus-visible:ring-2 focus-visible:ring-ring"
							>
								Ajoute une adresse sur la fiche du client.
							</Link>
						</p>
					</LockBanner>
				)}
				<FormField
					id={`${idPrefix}-billing-address`}
					label="Adresse de facturation"
					required
				>
					<BillingAddressField
						id={`${idPrefix}-billing-address`}
						value={values.customerContextId}
						addresses={customerContexts}
						hasCustomer={hasCustomer}
						isLoading={isLoading}
						onChange={(customerContextId) => onChange({ customerContextId })}
					/>
				</FormField>
			</FormCard>

			<FormCard id={`${idPrefix}-infos-title`} title="Informations du devis">
				<FormField id={`${idPrefix}-title`} label="Objet" required>
					<Input
						id={`${idPrefix}-title`}
						value={values.title}
						autoComplete="off"
						placeholder="Remplacement de 4 fenêtres"
						onChange={(event) => onChange({ title: event.target.value })}
					/>
				</FormField>
			</FormCard>

			<section
				aria-labelledby={`${idPrefix}-lines-title`}
				className="grid gap-3"
			>
				<div>
					<h2
						id={`${idPrefix}-lines-title`}
						className="font-semibold text-base"
					>
						Lignes
					</h2>
					<p className="mt-0.5 text-muted-foreground text-sm">
						Les montants sont en euros hors taxes.
					</p>
				</div>
				<LinesEditor
					lines={values.lines}
					idPrefix={idPrefix}
					vatEnabled={vatEnabled}
					totalOf={quoteLineTotalCents}
					customUnits={customUnits}
					catalog={{ items: catalogItems, onPick: onAddCatalogItem }}
					details={{
						photoUrls,
						isUploading,
						onUpload: (index, file) => void onUploadLinePhoto(index, file),
					}}
					onPatch={onLineChange}
					onMove={onMoveLine}
					onDuplicate={onDuplicateLine}
					onRemove={onRemoveLine}
					onAdd={onAddLine}
				/>
			</section>
		</div>
	)
}
