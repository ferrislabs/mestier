import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle, Loader2 } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import {
	useInvoicesByProject,
	useProjectBillingSummary,
} from '#/hooks/use-invoices'
import { useHasPermission } from '#/hooks/use-permissions'
import { useProjects } from '#/hooks/use-projects'
import {
	type Quote,
	type QuoteStatus,
	useDeleteQuote,
	useQuote,
	useUpdateQuote,
} from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { buildOrgPath } from '#/modules/org-path'
import { computeBillingPipelineSteps } from '#/pages/quotes/lib/billing-pipeline'
import {
	centsToEuros,
	emptyQuoteLine,
	eurosToCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { QuoteEditUI } from '#/pages/quotes/ui/quote-edit-ui'

interface QuoteEditFeatureProps {
	quoteId: string
}

export function QuoteEditFeature({ quoteId }: QuoteEditFeatureProps) {
	const quote = useQuote(quoteId)

	if (quote.isLoading) return <QuoteEditLoading />

	if (quote.isError) {
		return (
			<QuoteEditError
				title="Impossible de charger le devis"
				message={quote.error.message}
				onRetry={() => void quote.refetch()}
			/>
		)
	}

	if (!quote.data?.data) {
		return (
			<QuoteEditError
				title="Devis introuvable"
				message="Aucun devis ne correspond à cet identifiant."
			/>
		)
	}

	return <QuoteEditWorkspace quote={quote.data.data} />
}

function QuoteEditWorkspace({ quote }: { quote: Quote }) {
	const navigate = useNavigate()
	const { activeOrganization } = useActiveOrganization()
	const customers = useCustomers(quote.organization_id)
	const catalog = useReferenceCatalog(quote.organization_id, {
		members: false,
		employeeProfiles: false,
		equipment: false,
	})
	const catalogItems = useCatalogItems(
		catalog.serviceRates.data?.data,
		catalog.products.data?.data,
	)
	const missingLegalIdentityFields =
		activeOrganization.missing_legal_identity_fields
	const canViewInvoices = useHasPermission('VIEW_INVOICES')
	const projectFromQuote = useProjects(quote.organization_id, {
		quoteId: quote.id,
		includeArchived: true,
	})
	const project = projectFromQuote.data?.data?.[0]
	const billingSummary = useProjectBillingSummary(
		project?.id ?? '',
		Boolean(project) && canViewInvoices,
	)
	const projectInvoices = useInvoicesByProject(
		project?.id ?? '',
		Boolean(project) && canViewInvoices,
	)
	const billingSteps = computeBillingPipelineSteps({
		quoteStatus: quote.status,
		project,
		billingSummary: billingSummary.data?.data,
		invoices: projectInvoices.data?.data,
	})
	// Without VIEW_INVOICES, only the quote/project steps are shown — billing
	// amounts and invoice statuses stay hidden entirely.
	const visibleBillingSteps = canViewInvoices
		? billingSteps
		: billingSteps.slice(0, 2)
	const updateQuote = useUpdateQuote()
	const deleteQuote = useDeleteQuote(quote.organization_id)
	const uploadFile = useUploadFile()
	const [status, setStatus] = useState<QuoteStatus>(quote.status)
	const [values, setValues] = useState<QuoteFormValues>(() =>
		quoteToForm(quote, catalogItems),
	)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

	// Reloads the form when the server's quote changes, and only then. The
	// catalogue is in the dependencies because `quoteToForm` reads it, but it
	// resolves on its own schedule: without this guard, a catalogue arriving
	// after the quote overwrote whatever the user had already typed.
	const loadedVersion = useRef<string | null>(null)
	useEffect(() => {
		const version = `${quote.id}:${quote.updated_at}`
		if (loadedVersion.current === version) return
		loadedVersion.current = version

		setStatus(quote.status)
		setValues(quoteToForm(quote, catalogItems))
	}, [catalogItems, quote])

	const updateValues = (patch: Partial<QuoteFormValues>) => {
		setValues((current) => ({ ...current, ...patch }))
	}

	const updateLine = (index: number, patch: Partial<QuoteLineFormValues>) => {
		setValues((current) => {
			const lines = [...current.lines]
			const line = lines[index]
			if (!line) return current
			lines[index] = { ...line, ...patch }
			return { ...current, lines }
		})
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
			vatRateBp:
				catalogItem.defaultVatRateBp !== null
					? String(catalogItem.defaultVatRateBp)
					: values.lines[index]?.vatRateBp || '',
			notes: catalogItem.description || values.lines[index]?.notes || '',
		})
	}

	const addLine = () => {
		setValues((current) => ({
			...current,
			lines: [
				...current.lines,
				emptyQuoteLine(`line-${Date.now()}-${current.lines.length}`),
			],
		}))
	}

	const removeLine = (index: number) => {
		setValues((current) => {
			const lines = current.lines.filter(
				(_line, lineIndex) => lineIndex !== index,
			)
			return { ...current, lines: lines.length ? lines : [emptyQuoteLine()] }
		})
	}

	const uploadLinePhoto = async (index: number, file: File) => {
		const uploaded = await uploadFile.mutateAsync(file)
		setValues((current) => {
			const lines = [...current.lines]
			const line = lines[index]
			if (!line) return current
			lines[index] = {
				...line,
				photoKeys: [...line.photoKeys, uploaded.data.key],
			}
			return { ...current, lines }
		})
	}

	const hasValidContent =
		Boolean(values.title.trim()) &&
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.every((line) => {
			const quantity = Number(line.quantity.replace(',', '.'))
			return (
				line.label.trim() &&
				Number.isFinite(quantity) &&
				quantity > 0 &&
				line.unitPrice.trim() &&
				eurosToCents(line.unitPrice) >= 0
			)
		})

	// Sending is refused server-side when the organization's legal identity
	// is incomplete (#310, enforced again on export by #314) — caught here
	// too, independently of whatever status happens to be locally selected,
	// so the warning and the disabled "Envoyer" button show up before a
	// request is even attempted.
	const identityIncompleteForSending =
		!quote.reference && missingLegalIdentityFields.length > 0
	const blockedBySentIdentity =
		status === 'SENT' && identityIncompleteForSending

	const canSave = hasValidContent && !blockedBySentIdentity
	const canSend = hasValidContent && !identityIncompleteForSending

	const totalCents = values.lines.reduce(
		(sum, line) => sum + quoteLineTotalCents(line),
		0,
	)
	const error =
		customers.error?.message ??
		customerContexts.error?.message ??
		catalog.serviceRates.error?.message ??
		catalog.products.error?.message ??
		updateQuote.error?.message ??
		deleteQuote.error?.message ??
		uploadFile.error?.message ??
		null

	const buildUpdateBody = (targetStatus: QuoteStatus) => ({
		title: values.title.trim(),
		customer_id: values.customerId,
		customer_context_id: values.customerContextId,
		status: targetStatus,
		lines: values.lines.map((line) => ({
			service_rate_id: line.serviceRateId || null,
			label: line.label.trim(),
			quantity: line.quantity.replace(',', '.').trim(),
			unit: line.unit,
			unit_price_cents: eurosToCents(line.unitPrice),
			vat_rate_bp: line.vatRateBp === '' ? null : Number(line.vatRateBp),
			notes: line.notes.trim() || null,
			photo_keys: line.photoKeys,
		})),
	})

	const saveQuote = async () => {
		if (!canSave) return
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: buildUpdateBody(status),
		})
	}

	// Built on the literal `'SENT'` rather than on `setStatus` + `saveQuote`:
	// `setStatus` only lands on the *next* render, so a save fired right
	// after it would still read the old `status` from this render's closure.
	const sendQuote = async () => {
		if (!canSend) return
		setStatus('SENT')
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: buildUpdateBody('SENT'),
		})
	}

	const deleteCurrentQuote = async () => {
		await deleteQuote.mutateAsync({
			path: { quote_id: quote.id },
		})
		await navigate({ to: buildOrgPath(activeOrganization.slug, '/crm/quotes') })
	}

	return (
		<QuoteEditUI
			quote={quote}
			values={values}
			status={status}
			customers={customers.data?.data ?? []}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			customUnits={(catalog.customUnits.data?.data ?? []).map(
				(customUnit) => customUnit.code,
			)}
			error={error}
			isLoading={
				customers.isLoading ||
				customerContexts.isLoading ||
				catalog.serviceRates.isLoading ||
				catalog.products.isLoading
			}
			isSaving={updateQuote.isPending}
			isDeleting={deleteQuote.isPending}
			isUploading={uploadFile.isPending}
			billingSteps={visibleBillingSteps}
			totalCents={totalCents}
			canSave={canSave}
			canSend={canSend}
			identityIncompleteForSending={identityIncompleteForSending}
			onChange={(patch) => {
				if (patch.customerId !== undefined) {
					updateValues({ customerId: patch.customerId, customerContextId: '' })
					return
				}
				updateValues(patch)
			}}
			onStatusChange={setStatus}
			onLineChange={updateLine}
			onSelectCatalogItem={selectCatalogItem}
			onAddLine={addLine}
			onRemoveLine={removeLine}
			onUploadLinePhoto={uploadLinePhoto}
			onSave={saveQuote}
			onSend={sendQuote}
			onDelete={deleteCurrentQuote}
		/>
	)
}

function quoteToForm(
	quote: Quote,
	catalogItems: CatalogItem[],
): QuoteFormValues {
	const lines: QuoteLineFormValues[] = quote.lines.map((line, index) => {
		const catalogItem = line.service_rate_id
			? catalogItems.find(
					(item) =>
						item.type === 'SERVICE' && item.sourceId === line.service_rate_id,
				)
			: undefined

		return {
			clientId: line.id || `line-${index + 1}`,
			catalogItemId: catalogItem?.id ?? '',
			catalogItemType: catalogItem?.type ?? 'CUSTOM',
			serviceRateId: line.service_rate_id ?? '',
			label: line.label,
			quantity: line.quantity,
			unit: line.unit,
			unitPrice: centsToEuros(line.unit_price_cents),
			vatRateBp: line.vat_rate_bp != null ? String(line.vat_rate_bp) : '',
			notes: line.notes ?? '',
			photoKeys: line.photo_keys,
		}
	})

	return {
		title: quote.title,
		customerId: quote.customer_id,
		customerContextId: quote.customer_context_id,
		lines: lines.length ? lines : [emptyQuoteLine()],
	}
}

function QuoteEditLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement du devis…
			</SectionCard>
		</PageShell>
	)
}

function QuoteEditError({
	title,
	message,
	onRetry,
}: {
	title: string
	message: string
	onRetry?: () => void
}) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<PageShell>
			<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
				<AlertCircle className="size-6 text-destructive" />
				<div>
					<p className="font-semibold">{title}</p>
					<p className="mt-1 text-sm text-muted-foreground">{message}</p>
				</div>
				<div className="flex gap-2">
					<Button asChild variant="outline">
						<Link to={buildOrgPath(activeOrganization.slug, '/crm/quotes')}>
							Retour aux devis
						</Link>
					</Button>
					{onRetry ? (
						<Button type="button" onClick={onRetry}>
							Réessayer
						</Button>
					) : null}
				</div>
			</SectionCard>
		</PageShell>
	)
}
