import { useNavigate } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	useInvoicesByProject,
	useProjectBillingSummary,
} from '#/hooks/use-invoices'
import { useHasPermission } from '#/hooks/use-permissions'
import { useProjects } from '#/hooks/use-projects'
import {
	type Quote,
	type QuoteStatus,
	useCreateQuote,
	useDeleteQuote,
	useQuote,
	useUpdateQuote,
} from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { buildOrgPath } from '#/modules/org-path'
import { invoiceKindLabel } from '#/pages/invoices/types'
import { computeBillingPipelineSteps } from '#/pages/quotes/lib/billing-pipeline'
import {
	duplicateItem,
	moveItem,
	newLineId,
} from '#/pages/quotes/lib/doc-lines'
import { isValidQuoteLine } from '#/pages/quotes/lib/new-quote-form'
import {
	buildQuoteHistory,
	buildQuotePaper,
	previewQuoteTotals,
	savedQuoteTotals,
} from '#/pages/quotes/lib/quote-document'
import {
	copyQuoteLine,
	lineFromCatalogItem,
} from '#/pages/quotes/lib/quote-lines'
import {
	centsToEuros,
	emptyQuoteLine,
	eurosToCents,
	formatCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { DetailError, DetailLoading } from '#/pages/quotes/ui/doc/detail-shell'
import { QuoteEditUI } from '#/pages/quotes/ui/quote-edit-ui'

interface QuoteEditFeatureProps {
	quoteId: string
}

export function QuoteEditFeature({ quoteId }: QuoteEditFeatureProps) {
	const quote = useQuote(quoteId)

	if (quote.isLoading) return <DetailLoading label="Chargement du devis…" />

	if (quote.isError) {
		return (
			<DetailError
				title="Impossible de charger le devis"
				message={quote.error.message}
				backTo="/crm/quotes"
				backLabel="Retour aux devis"
				onRetry={() => void quote.refetch()}
			/>
		)
	}

	if (!quote.data?.data) {
		return (
			<DetailError
				title="Devis introuvable"
				message="Aucun devis ne correspond à cet identifiant."
				backTo="/crm/quotes"
				backLabel="Retour aux devis"
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
	const vatEnabled = activeOrganization.vat_status?.type === 'subject'
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
	const linkedInvoices = canViewInvoices
		? (projectInvoices.data?.data ?? [])
				.filter((invoice) => invoice.kind !== 'CREDIT_NOTE')
				.map((invoice) => ({
					id: invoice.id,
					label: invoice.number ?? 'Brouillon',
					kindLabel: invoiceKindLabel(invoice.kind),
					status: invoice.status,
					dueAt: invoice.due_at ?? null,
					grossCents: invoice.gross_cents,
				}))
		: []
	const updateQuote = useUpdateQuote()
	const createQuote = useCreateQuote(quote.organization_id)
	const deleteQuote = useDeleteQuote(quote.organization_id)
	const uploadFile = useUploadFile()
	const [values, setValues] = useState<QuoteFormValues>(() =>
		quoteToForm(quote, catalogItems),
	)
	const [isDownloading, setIsDownloading] = useState(false)
	const [downloadError, setDownloadError] = useState<string | null>(null)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)
	const photoPreviews = useFileUrls(
		values.lines.flatMap((line) => line.photoKeys),
	)
	const photoUrls = Object.fromEntries(
		photoPreviews.map((preview) => [preview.key, preview.url]),
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

	const appendLine = (line: QuoteLineFormValues) => {
		setValues((current) => ({ ...current, lines: [...current.lines, line] }))
		return line.clientId
	}

	const addLine = () =>
		appendLine(emptyQuoteLine(newLineId(values.lines.length)))

	const addCatalogItem = (item: CatalogItem) =>
		appendLine(lineFromCatalogItem(item, newLineId(values.lines.length)))

	const moveLine = (index: number, delta: -1 | 1) => {
		setValues((current) => ({
			...current,
			lines: moveItem(current.lines, index, delta),
		}))
	}

	const duplicateLine = (index: number) => {
		const clientId = newLineId(values.lines.length)
		setValues((current) => ({
			...current,
			lines: duplicateItem(current.lines, index, (line) =>
				copyQuoteLine(line, clientId),
			),
		}))
		return clientId
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
		values.lines.every(isValidQuoteLine)

	// Sending is refused server-side when the organization's legal identity
	// is incomplete (#310, enforced again on export by #314) — caught here
	// too, so the warning and the disabled "Envoyer" button show up before a
	// request is even attempted.
	const identityIncompleteForSending =
		!quote.reference && missingLegalIdentityFields.length > 0

	const canSave = hasValidContent
	const canSend = hasValidContent && !identityIncompleteForSending

	const savedValues = quoteToForm(quote, catalogItems)
	const isDirty = JSON.stringify(values) !== JSON.stringify(savedValues)
	const previewTotals = previewQuoteTotals(values, vatEnabled)
	const totals = isDirty ? previewTotals : savedQuoteTotals(quote)
	const savedCustomer = customers.data?.data.find(
		(customer) => customer.id === quote.customer_id,
	)
	const paper = buildQuotePaper({
		quote,
		values,
		customer: customers.data?.data.find(
			(customer) => customer.id === values.customerId,
		),
		context: customerContexts.data?.data.find(
			(context) => context.id === values.customerContextId,
		),
		organization: activeOrganization,
		totals,
	})
	const totalLines = values.lines.reduce(
		(sum, line) => sum + quoteLineTotalCents(line),
		0,
	)
	const error =
		customers.error?.message ??
		customerContexts.error?.message ??
		catalog.serviceRates.error?.message ??
		catalog.products.error?.message ??
		updateQuote.error?.message ??
		createQuote.error?.message ??
		deleteQuote.error?.message ??
		uploadFile.error?.message ??
		downloadError

	const buildUpdateBody = (targetStatus: QuoteStatus) => ({
		title: values.title.trim(),
		customer_id: values.customerId,
		customer_context_id: values.customerContextId,
		status: targetStatus,
		lines: quoteLineBodies(values.lines),
	})

	const saveQuote = async () => {
		if (!canSave) return
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: buildUpdateBody(quote.status),
		})
	}

	// Every transition goes through the same write as a save, on the literal
	// target: the form is saved with it, as the old inline status select did.
	const transitionTo = async (target: QuoteStatus) => {
		if (!hasValidContent) return
		if (target === 'SENT' && identityIncompleteForSending) return
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: buildUpdateBody(target),
		})
	}

	const deleteCurrentQuote = async () => {
		await deleteQuote.mutateAsync({
			path: { quote_id: quote.id },
		})
		await navigate({ to: buildOrgPath(activeOrganization.slug, '/crm/quotes') })
	}

	const duplicateCurrentQuote = async () => {
		if (!hasValidContent) return
		const created = await createQuote.mutateAsync({
			path: { organization_id: quote.organization_id },
			body: {
				title: values.title.trim(),
				customer_id: values.customerId,
				customer_context_id: values.customerContextId,
				lines: quoteLineBodies(values.lines),
			},
		})
		await navigate({
			to: buildOrgPath(activeOrganization.slug, '/crm/quotes/$quoteId'),
			params: { quoteId: created.data.id },
		})
	}

	const downloadPdf = async () => {
		setDownloadError(null)
		setIsDownloading(true)
		try {
			await downloadAuthenticatedFile(
				`/api/v1/quotes/${quote.id}/pdf`,
				`${quote.reference ?? quote.id}.pdf`,
			)
		} catch (err) {
			setDownloadError(
				err instanceof Error ? err.message : 'Téléchargement impossible',
			)
		} finally {
			setIsDownloading(false)
		}
	}

	return (
		<QuoteEditUI
			quote={quote}
			values={values}
			customer={savedCustomer}
			customers={customers.data?.data ?? []}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			photoUrls={photoUrls}
			vatEnabled={vatEnabled}
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
			isDuplicating={createQuote.isPending}
			isUploading={uploadFile.isPending}
			isDownloading={isDownloading}
			billingSteps={visibleBillingSteps}
			linkedInvoices={linkedInvoices}
			paper={paper}
			history={buildQuoteHistory(quote)}
			totals={savedQuoteTotals(quote)}
			unsavedNotice={
				isDirty ? `Aperçu non enregistré : ${formatCents(totalLines)} HT` : null
			}
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
			onLineChange={updateLine}
			onAddLine={addLine}
			onAddCatalogItem={addCatalogItem}
			onMoveLine={moveLine}
			onDuplicateLine={duplicateLine}
			onRemoveLine={removeLine}
			onUploadLinePhoto={uploadLinePhoto}
			onSave={saveQuote}
			onTransition={transitionTo}
			onDelete={deleteCurrentQuote}
			onDuplicate={duplicateCurrentQuote}
			onDownloadPdf={downloadPdf}
		/>
	)
}

function quoteLineBodies(lines: QuoteLineFormValues[]) {
	return lines.map((line) => ({
		service_rate_id: line.serviceRateId || null,
		label: line.label.trim(),
		quantity: line.quantity.replace(',', '.').trim(),
		unit: line.unit,
		unit_price_cents: eurosToCents(line.unitPrice),
		vat_rate_bp: line.vatRateBp === '' ? null : Number(line.vatRateBp),
		notes: line.notes.trim() || null,
		photo_keys: line.photoKeys,
	}))
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
