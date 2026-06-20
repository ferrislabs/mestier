import { useForm } from '@tanstack/react-form'
import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import { type Quote, useCreateQuote, useQuotes } from '#/hooks/use-quotes'
import {
	type ServiceRate,
	useReferenceCatalog,
} from '#/hooks/use-reference-catalog'
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
		/>
	)
}

function QuoteWorkspace({ organizationId }: { organizationId: string }) {
	const [lastCreated, setLastCreated] = useState<Quote | null>(null)
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId)
	const catalog = useReferenceCatalog(organizationId)
	const createQuote = useCreateQuote(organizationId)
	const uploadFile = useUploadFile()

	const form = useForm({
		defaultValues: {
			customerId: '',
			customerContextId: '',
			lines: [emptyQuoteLine()],
		} satisfies QuoteFormValues,
		onSubmit: async ({ value }) => {
			const quote = await createQuote.mutateAsync({
				path: { organization_id: organizationId },
				body: {
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
			setLastCreated(quote)
			form.reset()
		},
	})

	return (
		<form.Subscribe selector={(state) => state.values}>
			{(values) => (
				<QuoteWorkspaceWithValues
					values={values}
					form={form}
					organizationId={organizationId}
					customers={customers.data?.data ?? []}
					customerContextsQueryEnabled={Boolean(values.customerId)}
					serviceRates={catalog.serviceRates.data?.data ?? []}
					quotes={quotes.data?.data ?? []}
					lastCreated={lastCreated}
					isLoading={
						customers.isLoading ||
						catalog.serviceRates.isLoading ||
						quotes.isLoading
					}
					isCreating={createQuote.isPending}
					isUploading={uploadFile.isPending}
					error={
						customers.error?.message ??
						catalog.serviceRates.error?.message ??
						quotes.error?.message ??
						createQuote.error?.message ??
						uploadFile.error?.message ??
						null
					}
					refetch={() => {
						void customers.refetch()
						void catalog.serviceRates.refetch()
						void quotes.refetch()
					}}
					uploadFile={async (lineIndex, file) => {
						const uploaded = await uploadFile.mutateAsync(file)
						const lines = [...form.state.values.lines]
						const line = lines[lineIndex]
						if (!line) return
						lines[lineIndex] = {
							...line,
							photoKeys: [...line.photoKeys, uploaded.key],
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
	form: ReturnType<typeof useForm<QuoteFormValues>>
	organizationId: string
	customers: Customer[]
	customerContextsQueryEnabled: boolean
	serviceRates: ServiceRate[]
	quotes: Quote[]
	lastCreated: Quote | null
	isLoading: boolean
	isCreating: boolean
	isUploading: boolean
	error: string | null
	refetch: () => void
	uploadFile: (lineIndex: number, file: File) => Promise<void>
}

function QuoteWorkspaceWithValues({
	values,
	form,
	customers,
	customerContextsQueryEnabled,
	serviceRates,
	quotes,
	lastCreated,
	isLoading,
	isCreating,
	isUploading,
	error,
	refetch,
	uploadFile,
}: QuoteWorkspaceWithValuesProps) {
	const customerContexts = useCustomerContexts(
		values.customerId,
		customerContextsQueryEnabled,
	)

	const updateValues = (patch: Partial<QuoteFormValues>) => {
		for (const key of Object.keys(patch) as (keyof QuoteFormValues)[]) {
			form.setFieldValue(key, patch[key] as never)
		}
	}

	const updateLine = (index: number, patch: Partial<QuoteLineFormValues>) => {
		const lines = [...form.state.values.lines]
		const current = lines[index]
		if (!current) return
		lines[index] = { ...current, ...patch }
		form.setFieldValue('lines', lines)
	}

	const selectServiceRate = (index: number, serviceRateId: string) => {
		const serviceRate = serviceRates.find((item) => item.id === serviceRateId)
		if (!serviceRate) {
			updateLine(index, { serviceRateId })
			return
		}
		updateLine(index, {
			serviceRateId,
			label: serviceRate.label,
			unit: serviceRate.unit,
			unitPrice: centsToEuros(serviceRate.rate_cents),
		})
	}

	const addLine = () => {
		form.setFieldValue('lines', [
			...form.state.values.lines,
			emptyQuoteLine(`line-${Date.now()}-${form.state.values.lines.length}`),
		])
	}

	const removeLine = (index: number) => {
		const next = form.state.values.lines.filter((_, itemIndex) => {
			return itemIndex !== index
		})
		form.setFieldValue('lines', next.length > 0 ? next : [emptyQuoteLine()])
	}

	return (
		<QuoteCreateUI
			values={values}
			customers={customers}
			customerContexts={customerContexts.data?.data ?? []}
			serviceRates={serviceRates}
			quotes={quotes}
			lastCreated={lastCreated}
			error={error ?? customerContexts.error?.message ?? null}
			isLoading={isLoading}
			isCreating={isCreating}
			isUploading={isUploading}
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
			onSelectServiceRate={selectServiceRate}
			onAddLine={addLine}
			onRemoveLine={removeLine}
			onUploadLinePhoto={uploadFile}
			onSubmit={() => void form.handleSubmit()}
		/>
	)
}
