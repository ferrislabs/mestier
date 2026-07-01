import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	type Customer,
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import {
	type Invoice,
	type PaginationMetadata,
	useCreateInvoice,
	useDeleteInvoice,
	useInvoices,
} from '#/hooks/use-invoices'
import {
	type LegalMentionTemplate,
	useLegalMentionTemplates,
} from '#/hooks/use-legal-mentions'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { getInvoiceListUrlState } from '#/pages/invoices/invoice-list-url-state'
import {
	centsToEuros,
	emptyInvoiceLine,
	eurosToCents,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
} from '#/pages/invoices/types'
import { InvoiceCreateUI } from '#/pages/invoices/ui/invoice-create-ui'

export function InvoiceCreateFeature() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						La création de factures nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<InvoiceWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function InvoiceWorkspace({ organizationId }: { organizationId: string }) {
	const [initialInvoiceListState] = useState(getInvoiceListUrlState)
	const [invoicePage, setInvoicePage] = useState(initialInvoiceListState.page)
	const [invoicePageSize, setInvoicePageSize] = useState(
		initialInvoiceListState.pageSize,
	)
	const [lastCreated, setLastCreated] = useState<Invoice | null>(null)
	const customers = useCustomers(organizationId)
	const legalMentionTemplates = useLegalMentionTemplates(organizationId)
	const invoices = useInvoices(organizationId, {
		page: invoicePage,
		perPage: invoicePageSize,
	})
	const catalog = useReferenceCatalog(organizationId, {
		employees: false,
		equipment: false,
	})
	const catalogItems = useCatalogItems(
		catalog.serviceRates.data?.data ?? [],
		catalog.products.data?.data ?? [],
	)
	const createInvoice = useCreateInvoice(organizationId)
	const deleteInvoice = useDeleteInvoice(organizationId)
	const uploadFile = useUploadFile()

	const form = useForm({
		defaultValues: {
			title: '',
			customerId: '',
			customerContextId: '',
			legalMentionTemplateIds: [],
			lines: [emptyInvoiceLine()],
		} satisfies InvoiceFormValues,
		onSubmit: async ({ value }) => {
			const invoice = await createInvoice.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					title: value.title.trim(),
					customer_id: value.customerId,
					customer_context_id: value.customerContextId,
					legal_mention_template_ids: value.legalMentionTemplateIds,
					lines: value.lines.map((line) => ({
						service_rate_id: line.serviceRateId || null,
						label: line.label.trim(),
						quantity: line.quantity.replace(',', '.').trim(),
						unit: line.unit,
						unit_price_cents: eurosToCents(line.unitPrice),
						vat_rate: line.vatRate.replace(',', '.').trim(),
						notes: line.notes.trim() || null,
						photo_keys: line.photoKeys,
					})),
				},
			})
			setLastCreated(invoice.data)
			form.reset()
		},
	})

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => (
				<InvoiceWorkspaceWithValues
					values={values}
					form={{
						state: form.state,
						setFieldValue: (field, value) => {
							form.setFieldValue(field, value as never)
						},
						handleSubmit: form.handleSubmit,
					}}
					organizationId={organizationId}
					customers={customers.data?.data ?? []}
					legalMentionTemplates={legalMentionTemplates.data?.data ?? []}
					isLegalMentionTemplatesLoading={legalMentionTemplates.isLoading}
					customerContextsQueryEnabled={Boolean(values.customerId)}
					catalogItems={catalogItems}
					invoices={invoices.data?.data ?? []}
					invoicesPagination={invoices.data?.pagination ?? null}
					invoicePage={invoicePage}
					invoicePageSize={invoicePageSize}
					lastCreated={lastCreated}
					isLoading={
						customers.isLoading ||
						catalog.serviceRates.isLoading ||
						catalog.products.isLoading ||
						invoices.isLoading
					}
					isCreating={createInvoice.isPending}
					isUploading={uploadFile.isPending}
					deletingInvoiceId={
						deleteInvoice.variables?.path.invoice_id && deleteInvoice.isPending
							? deleteInvoice.variables.path.invoice_id
							: null
					}
					error={
						customers.error?.message ??
						catalog.serviceRates.error?.message ??
						catalog.products.error?.message ??
						invoices.error?.message ??
						createInvoice.error?.message ??
						deleteInvoice.error?.message ??
						uploadFile.error?.message ??
						null
					}
					refetch={() => {
						void customers.refetch()
						void catalog.serviceRates.refetch()
						void catalog.products.refetch()
						void invoices.refetch()
					}}
					onInvoicePageChange={setInvoicePage}
					onInvoicePageSizeChange={(pageSize) => {
						setInvoicePageSize(pageSize)
						setInvoicePage(1)
					}}
					onInvoiceDelete={(invoice) =>
						deleteInvoice.mutateAsync({
							path: { invoice_id: invoice.id },
						})
					}
					uploadFile={async (lineIndex, file) => {
						const uploaded = await uploadFile.mutateAsync(file)
						const lines = [...form.state.values.lines]
						const line = lines[lineIndex]
						if (!line) return
						lines[lineIndex] = {
							...line,
							photoKeys: [...line.photoKeys, uploaded.data.key],
						}
						form.setFieldValue('lines', lines)
					}}
				/>
			)}
		</form.Subscribe>
	)
}

interface InvoiceWorkspaceWithValuesProps {
	values: InvoiceFormValues
	form: InvoiceFormApi
	organizationId: string
	customers: Customer[]
	legalMentionTemplates: LegalMentionTemplate[]
	isLegalMentionTemplatesLoading: boolean
	customerContextsQueryEnabled: boolean
	catalogItems: CatalogItem[]
	invoices: Invoice[]
	invoicesPagination?: PaginationMetadata | null
	invoicePage: number
	invoicePageSize: number
	lastCreated: Invoice | null
	isLoading: boolean
	isCreating: boolean
	isUploading: boolean
	deletingInvoiceId: string | null
	error: string | null
	refetch: () => void
	onInvoicePageChange: (page: number) => void
	onInvoicePageSizeChange: (pageSize: number) => void
	onInvoiceDelete: (invoice: Invoice) => Promise<unknown>
	uploadFile: (lineIndex: number, file: File) => Promise<void>
}

interface InvoiceFormApi {
	state: { values: InvoiceFormValues }
	setFieldValue: (
		field: keyof InvoiceFormValues,
		value: InvoiceFormValues[keyof InvoiceFormValues],
	) => void
	handleSubmit: () => void | Promise<void>
}

function InvoiceWorkspaceWithValues({
	values,
	form,
	customers,
	legalMentionTemplates,
	isLegalMentionTemplatesLoading,
	customerContextsQueryEnabled,
	catalogItems,
	invoices,
	invoicesPagination,
	invoicePage,
	invoicePageSize,
	lastCreated,
	isLoading,
	isCreating,
	isUploading,
	deletingInvoiceId,
	error,
	refetch,
	onInvoicePageChange,
	onInvoicePageSizeChange,
	onInvoiceDelete,
	uploadFile,
}: InvoiceWorkspaceWithValuesProps) {
	const customerContexts = useCustomerContexts(
		values.customerId,
		customerContextsQueryEnabled,
	)

	const updateValues = (patch: Partial<InvoiceFormValues>) => {
		if (patch.title !== undefined) {
			form.setFieldValue('title', patch.title)
		}
		if (patch.customerId !== undefined) {
			form.setFieldValue('customerId', patch.customerId)
		}
		if (patch.customerContextId !== undefined) {
			form.setFieldValue('customerContextId', patch.customerContextId)
		}
		if (patch.legalMentionTemplateIds !== undefined) {
			form.setFieldValue(
				'legalMentionTemplateIds',
				patch.legalMentionTemplateIds,
			)
		}
		if (patch.lines !== undefined) {
			form.setFieldValue('lines', patch.lines)
		}
	}

	const updateLine = (index: number, patch: Partial<InvoiceLineFormValues>) => {
		const lines = [...form.state.values.lines]
		const current = lines[index]
		if (!current) return
		lines[index] = { ...current, ...patch }
		form.setFieldValue('lines', lines)
	}

	const selectCatalogItem = (index: number, catalogItemId: string) => {
		const catalogItem = catalogItems.find((item) => item.id === catalogItemId)
		if (!catalogItem) {
			updateLine(index, {
				catalogItemId: '',
				catalogItemType: 'CUSTOM',
				serviceRateId: '',
			})
			return
		}

		updateLine(index, {
			catalogItemId: catalogItem.id,
			catalogItemType: catalogItem.type,
			serviceRateId: catalogItem.type === 'SERVICE' ? catalogItem.sourceId : '',
			label: catalogItem.label,
			unit: catalogItem.unit,
			unitPrice: centsToEuros(catalogItem.unitPriceCents),
			notes:
				catalogItem.description || form.state.values.lines[index]?.notes || '',
		})
	}

	const addLine = () => {
		form.setFieldValue('lines', [
			...form.state.values.lines,
			emptyInvoiceLine(`line-${Date.now()}-${form.state.values.lines.length}`),
		])
	}

	const removeLine = (index: number) => {
		const next = form.state.values.lines.filter((_line, itemIndex) => {
			return itemIndex !== index
		})
		form.setFieldValue('lines', next.length > 0 ? next : [emptyInvoiceLine()])
	}

	return (
		<InvoiceCreateUI
			values={values}
			customers={customers}
			legalMentionTemplates={legalMentionTemplates}
			isLegalMentionTemplatesLoading={isLegalMentionTemplatesLoading}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			invoices={invoices}
			invoicesPagination={invoicesPagination}
			invoicePage={invoicePage}
			invoicePageSize={invoicePageSize}
			lastCreated={lastCreated}
			error={error ?? customerContexts.error?.message ?? null}
			isLoading={isLoading}
			isCreating={isCreating}
			isUploading={isUploading}
			deletingInvoiceId={deletingInvoiceId}
			isCustomerContextsLoading={customerContexts.isLoading}
			onRetry={refetch}
			onChange={(patch) => {
				if (patch.customerId !== undefined) {
					updateValues({ customerId: patch.customerId, customerContextId: '' })
					return
				}
				updateValues(patch)
			}}
			onLineChange={updateLine}
			onSelectCatalogItem={selectCatalogItem}
			onAddLine={addLine}
			onRemoveLine={removeLine}
			onUploadLinePhoto={uploadFile}
			onInvoicePageChange={onInvoicePageChange}
			onInvoicePageSizeChange={onInvoicePageSizeChange}
			onInvoiceDelete={onInvoiceDelete}
			onSubmit={() => void form.handleSubmit()}
		/>
	)
}
