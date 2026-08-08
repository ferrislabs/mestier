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
	type PaginationMetadata,
	type Quote,
	useCreateQuote,
	useDeleteQuote,
	useQuotes,
} from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { getQuoteListUrlState } from '#/pages/quotes/quote-list-url-state'
import {
	centsToEuros,
	emptyQuoteLine,
	eurosToCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
} from '#/pages/quotes/types'
import { QuoteCreateUI } from '#/pages/quotes/ui/quote-create-ui'

export function QuoteCreateFeature() {
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
						La création de devis nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<QuoteWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function QuoteWorkspace({
	organizationId,
	organizationSlug,
}: {
	organizationId: string
	organizationSlug: string
}) {
	const [initialQuoteListState] = useState(getQuoteListUrlState)
	const [quotePage, setQuotePage] = useState(initialQuoteListState.page)
	const [quotePageSize, setQuotePageSize] = useState(
		initialQuoteListState.pageSize,
	)
	const [lastCreated, setLastCreated] = useState<Quote | null>(null)
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId, {
		page: quotePage,
		perPage: quotePageSize,
	})
	const catalog = useReferenceCatalog(organizationId, {
		employees: false,
		equipment: false,
	})
	const catalogItems = useCatalogItems(
		catalog.serviceRates.data?.data ?? [],
		catalog.products.data?.data ?? [],
	)
	const createQuote = useCreateQuote(organizationId)
	const deleteQuote = useDeleteQuote(organizationId)
	const uploadFile = useUploadFile()

	const form = useForm({
		defaultValues: {
			title: '',
			customerId: '',
			customerContextId: '',
			lines: [emptyQuoteLine()],
		} satisfies QuoteFormValues,
		onSubmit: async ({ value }) => {
			const quote = await createQuote.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					title: value.title.trim(),
					customer_id: value.customerId,
					customer_context_id: value.customerContextId,
					lines: value.lines.map((line) => ({
						service_rate_id: line.serviceRateId || null,
						label: line.label.trim(),
						quantity: line.quantity.replace(',', '.').trim(),
						unit: line.unit,
						unit_price_cents: eurosToCents(line.unitPrice),
						notes: line.notes.trim() || null,
						photo_keys: line.photoKeys,
					})),
				},
			})
			setLastCreated(quote.data)
			form.reset()
		},
	})

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => (
				<QuoteWorkspaceWithValues
					organizationSlug={organizationSlug}
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
					customerContextsQueryEnabled={Boolean(values.customerId)}
					catalogItems={catalogItems}
					quotes={quotes.data?.data ?? []}
					quotesPagination={quotes.data?.pagination ?? null}
					quotePage={quotePage}
					quotePageSize={quotePageSize}
					lastCreated={lastCreated}
					isLoading={
						customers.isLoading ||
						catalog.serviceRates.isLoading ||
						catalog.products.isLoading ||
						quotes.isLoading
					}
					isCreating={createQuote.isPending}
					isUploading={uploadFile.isPending}
					deletingQuoteId={
						deleteQuote.variables?.path.quote_id && deleteQuote.isPending
							? deleteQuote.variables.path.quote_id
							: null
					}
					error={
						customers.error?.message ??
						catalog.serviceRates.error?.message ??
						catalog.products.error?.message ??
						quotes.error?.message ??
						createQuote.error?.message ??
						deleteQuote.error?.message ??
						uploadFile.error?.message ??
						null
					}
					refetch={() => {
						void customers.refetch()
						void catalog.serviceRates.refetch()
						void catalog.products.refetch()
						void quotes.refetch()
					}}
					onQuotePageChange={setQuotePage}
					onQuotePageSizeChange={(pageSize) => {
						setQuotePageSize(pageSize)
						setQuotePage(1)
					}}
					onQuoteDelete={(quote) =>
						deleteQuote.mutateAsync({
							path: { quote_id: quote.id },
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

interface QuoteWorkspaceWithValuesProps {
	values: QuoteFormValues
	form: QuoteFormApi
	organizationId: string
	customers: Customer[]
	customerContextsQueryEnabled: boolean
	catalogItems: CatalogItem[]
	quotes: Quote[]
	quotesPagination?: PaginationMetadata | null
	quotePage: number
	quotePageSize: number
	lastCreated: Quote | null
	isLoading: boolean
	isCreating: boolean
	isUploading: boolean
	deletingQuoteId: string | null
	error: string | null
	refetch: () => void
	onQuotePageChange: (page: number) => void
	onQuotePageSizeChange: (pageSize: number) => void
	onQuoteDelete: (quote: Quote) => Promise<unknown>
	uploadFile: (lineIndex: number, file: File) => Promise<void>
	organizationSlug: string
}

interface QuoteFormApi {
	state: { values: QuoteFormValues }
	setFieldValue: (
		field: keyof QuoteFormValues,
		value: QuoteFormValues[keyof QuoteFormValues],
	) => void
	handleSubmit: () => void | Promise<void>
}

function QuoteWorkspaceWithValues({
	organizationSlug,
	values,
	form,
	customers,
	customerContextsQueryEnabled,
	catalogItems,
	quotes,
	quotesPagination,
	quotePage,
	quotePageSize,
	lastCreated,
	isLoading,
	isCreating,
	isUploading,
	deletingQuoteId,
	error,
	refetch,
	onQuotePageChange,
	onQuotePageSizeChange,
	onQuoteDelete,
	uploadFile,
}: QuoteWorkspaceWithValuesProps) {
	const customerContexts = useCustomerContexts(
		values.customerId,
		customerContextsQueryEnabled,
	)

	const updateValues = (patch: Partial<QuoteFormValues>) => {
		if (patch.title !== undefined) {
			form.setFieldValue('title', patch.title)
		}
		if (patch.customerId !== undefined) {
			form.setFieldValue('customerId', patch.customerId)
		}
		if (patch.customerContextId !== undefined) {
			form.setFieldValue('customerContextId', patch.customerContextId)
		}
		if (patch.lines !== undefined) {
			form.setFieldValue('lines', patch.lines)
		}
	}

	const updateLine = (index: number, patch: Partial<QuoteLineFormValues>) => {
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
			emptyQuoteLine(`line-${Date.now()}-${form.state.values.lines.length}`),
		])
	}

	const removeLine = (index: number) => {
		const next = form.state.values.lines.filter((_line, itemIndex) => {
			return itemIndex !== index
		})
		form.setFieldValue('lines', next.length > 0 ? next : [emptyQuoteLine()])
	}

	return (
		<QuoteCreateUI
			organizationSlug={organizationSlug}
			values={values}
			customers={customers}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			quotes={quotes}
			quotesPagination={quotesPagination}
			quotePage={quotePage}
			quotePageSize={quotePageSize}
			lastCreated={lastCreated}
			error={error ?? customerContexts.error?.message ?? null}
			isLoading={isLoading}
			isCreating={isCreating}
			isUploading={isUploading}
			deletingQuoteId={deletingQuoteId}
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
			onQuotePageChange={onQuotePageChange}
			onQuotePageSizeChange={onQuotePageSizeChange}
			onQuoteDelete={onQuoteDelete}
			onSubmit={() => void form.handleSubmit()}
		/>
	)
}
