import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { OperationNature } from '#/hooks/use-invoices'
import {
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceKindLabel,
	invoiceLineTotalCents,
	operationNatureLabel,
} from '#/pages/invoices/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { CustomerPicker } from '#/pages/quotes/ui/doc/customer-picker'
import { LinesEditor } from '#/pages/quotes/ui/doc/lines-editor'
import { AutoTextarea, FormCard } from '#/pages/quotes/ui/doc/primitives'
import { type DocTotals, TotalsPanel } from '#/pages/quotes/ui/doc/totals-panel'

const OPERATION_NATURES: OperationNature[] = ['GOODS', 'SERVICES', 'BOTH']
const NO_VALUE = 'none'

interface InvoiceDraftFormProps {
	idPrefix: string
	values: InvoiceFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	isContextsLoading: boolean
	vatEnabled: boolean
	totals: DocTotals
	vatNotice: string | null
	unsavedNotice: string | null
	onChange: (patch: Partial<InvoiceFormValues>) => void
	onLineChange: (index: number, patch: Partial<InvoiceLineFormValues>) => void
	onAddLine: () => string
	onMoveLine: (index: number, delta: -1 | 1) => void
	onDuplicateLine: (index: number) => string
	onRemoveLine: (index: number) => void
}

export function InvoiceDraftForm({
	idPrefix,
	values,
	customers,
	customerContexts,
	isContextsLoading,
	vatEnabled,
	totals,
	vatNotice,
	unsavedNotice,
	onChange,
	onLineChange,
	onAddLine,
	onMoveLine,
	onDuplicateLine,
	onRemoveLine,
}: InvoiceDraftFormProps) {
	const hasCustomer = values.customerId !== ''
	return (
		<div className="grid gap-4">
			<FormCard
				id={`${idPrefix}-recipient-title`}
				title="Destinataire"
				description="Le client et l’adresse de facturation sont figés à l’émission."
			>
				<FormField id={`${idPrefix}-customer`} label="Client" required>
					<CustomerPicker
						id={`${idPrefix}-customer`}
						customers={customers}
						value={hasCustomer ? values.customerId : null}
						onChange={(customerId) => onChange({ customerId })}
					/>
				</FormField>
				<FormField
					id={`${idPrefix}-address`}
					label="Adresse de facturation"
					required
				>
					<BillingAddressField
						id={`${idPrefix}-address`}
						value={values.customerContextId}
						addresses={customerContexts}
						hasCustomer={hasCustomer}
						isLoading={isContextsLoading}
						onChange={(customerContextId) => onChange({ customerContextId })}
					/>
				</FormField>
			</FormCard>
			<FormCard id={`${idPrefix}-terms-title`} title="Type et conditions">
				<div className="@container grid gap-4">
					<div className="grid gap-4 @md:grid-cols-3">
						<FormField id={`${idPrefix}-kind`} label="Type">
							<p
								id={`${idPrefix}-kind`}
								className="flex h-9 items-center text-sm"
							>
								{invoiceKindLabel(values.kind)}
							</p>
						</FormField>
						<FormField id={`${idPrefix}-due`} label="Échéance">
							<Input
								id={`${idPrefix}-due`}
								type="date"
								value={values.dueAt}
								onChange={(event) => onChange({ dueAt: event.target.value })}
							/>
						</FormField>
						<FormField id={`${idPrefix}-nature`} label="Nature de l’opération">
							<Select
								value={values.operationNature || NO_VALUE}
								onValueChange={(nature) =>
									onChange({
										operationNature:
											nature === NO_VALUE ? '' : (nature as OperationNature),
									})
								}
							>
								<SelectTrigger id={`${idPrefix}-nature`} className="w-full">
									<SelectValue placeholder="Non renseignée" />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={NO_VALUE}>Non renseignée</SelectItem>
									{OPERATION_NATURES.map((nature) => (
										<SelectItem key={nature} value={nature}>
											{operationNatureLabel(nature)}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</FormField>
					</div>
				</div>
			</FormCard>
			<FormCard
				id={`${idPrefix}-lines-title`}
				title="Lignes"
				description="Les montants sont en euros hors taxes."
			>
				<LinesEditor
					lines={values.lines}
					idPrefix={idPrefix}
					vatEnabled={vatEnabled}
					totalOf={invoiceLineTotalCents}
					onPatch={onLineChange}
					onMove={onMoveLine}
					onDuplicate={onDuplicateLine}
					onRemove={onRemoveLine}
					onAdd={onAddLine}
				/>
				<TotalsPanel
					totals={totals}
					vatNotice={vatNotice}
					notice={unsavedNotice}
					className="ml-auto w-full max-w-sm"
				/>
			</FormCard>
			<FormCard id={`${idPrefix}-notes-title`} title="Notes">
				<FormField
					id={`${idPrefix}-notes`}
					label="Notes"
					hint="Affichées sur la facture, sous le détail des lignes."
				>
					<AutoTextarea
						id={`${idPrefix}-notes`}
						value={values.notes}
						onChange={(event) => onChange({ notes: event.target.value })}
						className="min-h-20"
					/>
				</FormField>
			</FormCard>
		</div>
	)
}
