import { Link } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { buildOrgPath } from '#/modules/org-path'
import {
	firstInvalidFieldId,
	NEW_QUOTE_FIELD_IDS,
	validateNewQuote,
} from '#/pages/quotes/lib/new-quote-form'
import { vatNoticeOf } from '#/pages/quotes/lib/paper-parties'
import { previewQuoteTotals } from '#/pages/quotes/lib/quote-document'
import { canChangeClient } from '#/pages/quotes/lib/quote-draft'
import {
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { CustomerPicker } from '#/pages/quotes/ui/doc/customer-picker'
import { LinesEditor } from '#/pages/quotes/ui/doc/lines-editor'
import { LockBanner } from '#/pages/quotes/ui/doc/primitives'
import { TotalsPanel } from '#/pages/quotes/ui/doc/totals-panel'
import {
	type QuoteOwner,
	QuoteTermsFields,
} from '#/pages/quotes/ui/quote-terms-fields'

interface QuoteNewUIProps {
	organizationSlug: string
	organization: Organization
	values: QuoteFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	photoUrls: Record<string, string | undefined>
	vatEnabled: boolean
	customUnits?: string[]
	owners?: readonly QuoteOwner[]
	error?: string | null
	isCreating?: boolean
	isUploading?: boolean
	isCustomerContextsLoading?: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onAddLine: () => string
	onAddCatalogItem: (item: CatalogItem) => string
	onMoveLine: (index: number, delta: -1 | 1) => void
	onDuplicateLine: (index: number) => string
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSubmit: () => void
}

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
	owners = [],
	error,
	isCreating,
	isUploading,
	isCustomerContextsLoading,
	onChange,
	onLineChange,
	onAddLine,
	onAddCatalogItem,
	onMoveLine,
	onDuplicateLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSubmit,
}: QuoteNewUIProps) {
	const [submitted, setSubmitted] = useState(false)
	const errors = submitted ? validateNewQuote(values) : {}
	const hasCustomer = values.customerId !== ''
	const clientLocked = hasCustomer && !canChangeClient(values.lines)
	const noAddress =
		hasCustomer && !isCustomerContextsLoading && customerContexts.length === 0

	const submit = (event: FormEvent) => {
		event.preventDefault()
		setSubmitted(true)
		const found = validateNewQuote(values)
		const firstId = firstInvalidFieldId(found)
		if (firstId) {
			document.getElementById(firstId)?.focus()
			return
		}
		onSubmit()
	}

	return (
		<PageShell className="max-w-5xl gap-0">
			<PageHeader
				className="mb-6"
				title="Nouveau devis"
				description="Choisis le client et l’objet, puis ajoute les lignes du devis."
			/>
			<form noValidate onSubmit={submit} className="grid gap-5">
				<div className="grid gap-5 rounded-lg border bg-card p-5">
					<FormField
						id={NEW_QUOTE_FIELD_IDS.customerId}
						label="Client"
						required
						error={errors.customerId}
						hint={
							clientLocked
								? 'Le client est verrouillé tant que le devis contient des lignes.'
								: customers.length === 0
									? 'Aucun client n’existe encore dans cette organisation.'
									: undefined
						}
					>
						<CustomerPicker
							id={NEW_QUOTE_FIELD_IDS.customerId}
							customers={customers}
							value={hasCustomer ? values.customerId : null}
							invalid={errors.customerId !== undefined}
							disabled={clientLocked}
							onChange={(customerId) => onChange({ customerId })}
						/>
					</FormField>
					{noAddress && (
						<LockBanner
							tone="warn"
							title="La fiche de ce client est incomplète"
						>
							<p>
								Il n’a pas d’adresse de facturation : le devis ne pourra pas
								être envoyé.
							</p>
						</LockBanner>
					)}
					<FormField
						id={NEW_QUOTE_FIELD_IDS.customerContextId}
						label="Adresse de facturation"
						required
						error={errors.customerContextId}
					>
						<BillingAddressField
							id={NEW_QUOTE_FIELD_IDS.customerContextId}
							value={values.customerContextId}
							addresses={customerContexts}
							hasCustomer={hasCustomer}
							isLoading={isCustomerContextsLoading}
							onChange={(customerContextId) => onChange({ customerContextId })}
						/>
					</FormField>
					<FormField
						id={NEW_QUOTE_FIELD_IDS.title}
						label="Objet du devis"
						required
						error={errors.title}
					>
						<Input
							id={NEW_QUOTE_FIELD_IDS.title}
							value={values.title}
							autoComplete="off"
							placeholder="Remplacement de 4 fenêtres"
							aria-invalid={errors.title !== undefined || undefined}
							aria-describedby={
								errors.title ? `${NEW_QUOTE_FIELD_IDS.title}-error` : undefined
							}
							onChange={(event) => onChange({ title: event.target.value })}
						/>
					</FormField>
					<QuoteTermsFields
						idPrefix="new-quote"
						values={values}
						owners={owners}
						onChange={onChange}
					/>
				</div>

				<section
					id={NEW_QUOTE_FIELD_IDS.lines}
					aria-labelledby="new-quote-lines-title"
					tabIndex={-1}
					className="grid gap-3 outline-none"
				>
					<div>
						<h2 id="new-quote-lines-title" className="font-semibold text-base">
							Lignes
						</h2>
						<p className="mt-0.5 text-muted-foreground text-sm">
							Les montants sont en euros hors taxes. La remise et la fréquence
							sont indicatives : elles ne modifient pas encore les totaux.
						</p>
					</div>
					<LinesEditor
						lines={values.lines}
						idPrefix="new-quote"
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
					{errors.lines && (
						<p role="alert" className="text-[12px] text-state-alert">
							{errors.lines}
						</p>
					)}
					<TotalsPanel
						totals={previewQuoteTotals(values, vatEnabled)}
						vatNotice={vatNoticeOf(organization)}
						notice="Estimation, non enregistrée"
						className="ml-auto w-full max-w-sm"
					/>
				</section>

				{error && (
					<p role="alert" className="text-sm text-state-alert">
						{error}
					</p>
				)}
				<div className="flex flex-wrap justify-end gap-2 border-t pt-4">
					<Button variant="outline" asChild>
						<Link to={buildOrgPath(organizationSlug, '/crm/devis')}>
							Annuler
						</Link>
					</Button>
					<RequirePermission permission="MANAGE_QUOTES">
						<Button type="submit" disabled={isCreating}>
							{isCreating ? <Loader2 className="animate-spin" /> : null}
							Créer le devis
						</Button>
					</RequirePermission>
				</div>
			</form>
		</PageShell>
	)
}
