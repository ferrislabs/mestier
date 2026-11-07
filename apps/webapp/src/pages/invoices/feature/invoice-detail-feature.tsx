import { Link, Navigate, useNavigate } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomerContexts, useCustomers } from '#/hooks/use-customers'
import {
	type Invoice,
	useCancelInvoice,
	useDeleteInvoice,
	useDeletePayment,
	useInvoice,
	useInvoiceBalance,
	useInvoiceCreditNotes,
	useInvoicePayments,
	useInvoices,
	useIssueCreditNote,
	useIssueInvoice,
	useNoteInvoice,
	useRecordPayment,
	useRemindInvoice,
	useSendInvoice,
	useUpdateInvoice,
} from '#/hooks/use-invoices'
import { useProject } from '#/hooks/use-projects'
import { useQuote } from '#/hooks/use-quotes'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { buildOrgPath } from '#/modules/org-path'
import { invoiceActions } from '#/pages/invoices/lib/invoice-actions'
import {
	buildInvoiceHistory,
	buildInvoicePaper,
	invoiceActivity,
	savedInvoiceTotals,
} from '#/pages/invoices/lib/invoice-document'
import { issueDateBounds } from '#/pages/invoices/lib/invoice-issue'
import { parseTerms } from '#/pages/invoices/lib/invoice-new'
import { isValidInvoiceLine } from '#/pages/invoices/lib/new-invoice-form'
import {
	paymentRequest,
	paymentsAndRefunds,
	todayDate,
} from '#/pages/invoices/lib/payment-form'
import {
	centsToEuros,
	emptyInvoiceLine,
	eurosToCents,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceDisplayState,
} from '#/pages/invoices/types'
import {
	InvoiceDetailUI,
	type InvoiceView,
} from '#/pages/invoices/ui/invoice-detail-ui'
import {
	duplicateItem,
	moveItem,
	newLineId,
} from '#/pages/quotes/lib/doc-lines'
import { vatNoticeOf } from '#/pages/quotes/lib/paper-parties'
import {
	DetailError,
	DetailLoading,
	NotFoundState,
} from '#/pages/quotes/ui/doc/detail-shell'

interface InvoiceDetailFeatureProps {
	invoiceId: string
	view: InvoiceView
	onViewChange: (view: InvoiceView) => void
}

export function InvoiceDetailFeature({
	invoiceId,
	view,
	onViewChange,
}: InvoiceDetailFeatureProps) {
	const invoice = useInvoice(invoiceId)
	const { activeOrganization } = useActiveOrganization()

	if (invoice.isLoading)
		return <DetailLoading label="Chargement de la facture…" />

	if (invoice.isError) {
		return (
			<DetailError
				title="Impossible de charger la facture"
				message={invoice.error.message}
				backTo="/crm/factures"
				backLabel="Retour aux factures"
				onRetry={() => void invoice.refetch()}
			/>
		)
	}

	if (!invoice.data?.data) {
		return (
			<NotFoundState
				title="Cette facture est introuvable"
				hint="Aucune facture ne correspond à cet identifiant, ou l’adresse n’est plus valide."
			>
				<Button variant="outline" size="sm" asChild>
					<Link to={buildOrgPath(activeOrganization.slug, '/crm/factures')}>
						Retour aux factures
					</Link>
				</Button>
			</NotFoundState>
		)
	}

	if (invoice.data.data.kind === 'CREDIT_NOTE') {
		return (
			<Navigate
				to={buildOrgPath(
					activeOrganization.slug,
					`/crm/avoirs/${invoice.data.data.id}`,
				)}
				replace
			/>
		)
	}

	return (
		<InvoiceDetailWorkspace
			invoice={invoice.data.data}
			view={view}
			onViewChange={onViewChange}
		/>
	)
}

function InvoiceDetailWorkspace({
	invoice,
	view,
	onViewChange,
}: {
	invoice: Invoice
	view: InvoiceView
	onViewChange: (view: InvoiceView) => void
}) {
	const navigate = useNavigate()
	const { activeOrganization } = useActiveOrganization()
	const isDraft = invoice.status === 'DRAFT'
	const vatEnabled = activeOrganization.vat_status?.type === 'subject'

	const customers = useCustomers(invoice.organization_id)
	const [values, setValues] = useState<InvoiceFormValues>(() =>
		invoiceToForm(invoice),
	)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)
	const project = useProject(invoice.organization_id, invoice.project_id ?? '')
	const updateInvoice = useUpdateInvoice()
	const issueInvoice = useIssueInvoice()
	const cancelInvoice = useCancelInvoice()
	const deleteInvoice = useDeleteInvoice()
	const balance = useInvoiceBalance(invoice.id)
	const payments = useInvoicePayments(invoice.id)
	const creditNotes = useInvoiceCreditNotes(invoice.id)
	const recordPayment = useRecordPayment()
	const deletePayment = useDeletePayment(invoice.id, invoice.organization_id)
	const issueCreditNote = useIssueCreditNote()
	const sendInvoice = useSendInvoice()
	const remindInvoice = useRemindInvoice()
	const noteInvoice = useNoteInvoice()
	const allInvoices = useInvoices(invoice.organization_id, {
		page: 1,
		perPage: 100,
	})
	const linkedQuote = useQuote(
		invoice.quote_id ?? '',
		Boolean(invoice.quote_id),
	)
	const [isDownloading, setIsDownloading] = useState(false)
	const [downloadError, setDownloadError] = useState<string | null>(null)

	// Reloads the form when the server's invoice changes, and only then:
	// without the guard, a customers fetch resolving after the invoice would
	// overwrite what the user typed.
	const loadedVersion = useRef<string | null>(null)
	useEffect(() => {
		const version = `${invoice.id}:${invoice.updated_at}`
		if (loadedVersion.current === version) return
		loadedVersion.current = version
		setValues(invoiceToForm(invoice))
	}, [invoice])

	const updateValues = (patch: Partial<InvoiceFormValues>) => {
		setValues((current) =>
			patch.customerId !== undefined
				? { ...current, ...patch, customerContextId: '' }
				: { ...current, ...patch },
		)
	}

	const updateLine = (index: number, patch: Partial<InvoiceLineFormValues>) => {
		setValues((current) => {
			const lines = [...current.lines]
			const line = lines[index]
			if (!line) return current
			lines[index] = { ...line, ...patch }
			return { ...current, lines }
		})
	}

	const addLine = () => {
		const clientId = newLineId(values.lines.length)
		setValues((current) => ({
			...current,
			lines: [...current.lines, emptyInvoiceLine(clientId)],
		}))
		return clientId
	}

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
			lines: duplicateItem(current.lines, index, (line) => ({
				...line,
				clientId,
			})),
		}))
		return clientId
	}

	const removeLine = (index: number) => {
		setValues((current) => {
			const lines = current.lines.filter((_line, i) => i !== index)
			return { ...current, lines: lines.length ? lines : [emptyInvoiceLine()] }
		})
	}

	const canSave =
		isDraft &&
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.every(isValidInvoiceLine)

	const isDirty =
		isDraft && JSON.stringify(values) !== JSON.stringify(invoiceToForm(invoice))

	const saveInvoice = async () => {
		if (!canSave) return
		await updateInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
			body: {
				customer_id: values.customerId,
				customer_context_id: values.customerContextId,
				project_id: invoice.project_id ?? null,
				title: values.title.trim(),
				payment_terms_days: parseTerms(values.termsDays),
				legal_notice: values.legalNotice.trim() || null,
				notes: values.notes.trim() || null,
				operation_nature: values.operationNature || null,
				lines: values.lines.map((line) => ({
					label: line.label.trim(),
					quantity: line.quantity.replace(',', '.').trim(),
					unit_price_cents: eurosToCents(line.unitPrice),
					vat_rate_basis_points:
						line.vatRateBp === '' ? null : Number(line.vatRateBp),
					discount_percent: line.discountPercent,
					product_id: line.productId,
					service_rate_id: line.serviceRateId,
				})),
			},
		})
	}

	const attempt = async (work: () => Promise<unknown>): Promise<boolean> => {
		try {
			await work()
			return true
		} catch {
			return false
		}
	}

	const downloadPdf = async () => {
		setDownloadError(null)
		setIsDownloading(true)
		try {
			await downloadAuthenticatedFile(
				`/api/v1/invoices/${invoice.id}/pdf`,
				`${invoice.number ?? invoice.id}.pdf`,
			)
		} catch (err) {
			setDownloadError(
				err instanceof Error ? err.message : 'Téléchargement impossible',
			)
		} finally {
			setIsDownloading(false)
		}
	}

	const customer = customers.data?.data.find(
		(item) => item.id === invoice.customer_id,
	)
	const paperContext =
		values.customerId === invoice.customer_id
			? customerContexts.data?.data.find(
					(context) => context.id === invoice.customer_context_id,
				)
			: undefined
	const creditNoteList = creditNotes.data?.data ?? []
	const allPayments = payments.data?.data ?? []
	const { payments: paymentList, refunds } = paymentsAndRefunds(allPayments)
	const activity = invoiceActivity(invoice)
	const remainingCents = invoice.balance_cents
	const displayState = invoiceDisplayState(invoice)
	const actions = invoiceActions({
		status: invoice.status,
		grossCents: invoice.gross_cents,
		creditedCents: invoice.credited_cents,
		remainingCents,
		sentAt: activity.sentAt,
	})

	const error =
		customers.error?.message ??
		customerContexts.error?.message ??
		updateInvoice.error?.message ??
		cancelInvoice.error?.message ??
		deleteInvoice.error?.message ??
		balance.error?.message ??
		payments.error?.message ??
		creditNotes.error?.message ??
		deletePayment.error?.message ??
		downloadError

	return (
		<InvoiceDetailUI
			invoice={invoice}
			view={view}
			values={values}
			isDirty={isDirty}
			customer={customer}
			customers={customers.data?.data ?? []}
			customerContexts={customerContexts.data?.data ?? []}
			isContextsLoading={customerContexts.isLoading}
			project={
				project.data?.data
					? { id: project.data.data.id, name: project.data.data.name }
					: null
			}
			vatEnabled={vatEnabled}
			displayState={displayState}
			actions={actions}
			balance={balance.data?.data}
			remainingCents={remainingCents}
			refunds={refunds}
			quote={
				invoice.quote_id
					? {
							id: invoice.quote_id,
							reference: linkedQuote.data?.data.reference ?? null,
						}
					: null
			}
			issueBounds={issueDateBounds(allInvoices.data?.data ?? [], todayDate())}
			payments={paymentList}
			paymentsLoading={payments.isLoading}
			creditNotes={creditNoteList}
			creditNotesLoading={creditNotes.isLoading}
			activity={activity}
			paper={buildInvoicePaper({
				invoice,
				customer,
				context: paperContext,
				organization: activeOrganization,
			})}
			history={buildInvoiceHistory({
				invoice,
				payments: allPayments,
				creditNotes: creditNoteList,
			})}
			totals={savedInvoiceTotals(invoice)}
			vatNotice={
				invoice.vat_breakdown.length === 0
					? vatNoticeOf(activeOrganization)
					: null
			}
			error={error}
			canSave={canSave}
			isSaving={updateInvoice.isPending}
			isIssuing={issueInvoice.isPending}
			isCancelling={cancelInvoice.isPending}
			isRecordingPayment={recordPayment.isPending}
			isIssuingCreditNote={issueCreditNote.isPending}
			isDeletingPayment={deletePayment.isPending}
			isSending={sendInvoice.isPending}
			isReminding={remindInvoice.isPending}
			isDownloading={isDownloading}
			issueError={issueInvoice.error?.message ?? null}
			paymentError={recordPayment.error?.message ?? null}
			creditNoteError={issueCreditNote.error?.message ?? null}
			onViewChange={onViewChange}
			onChange={updateValues}
			onLineChange={updateLine}
			onAddLine={addLine}
			onMoveLine={moveLine}
			onDuplicateLine={duplicateLine}
			onRemoveLine={removeLine}
			onSave={() => void saveInvoice()}
			onIssue={(issueDate) =>
				attempt(() =>
					issueInvoice.mutateAsync({
						path: { invoice_id: invoice.id },
						body: { issued_on: issueDate },
					}),
				)
			}
			onDeleteDraft={() =>
				void attempt(async () => {
					await deleteInvoice.mutateAsync({
						path: { invoice_id: invoice.id },
					})
					await navigate({
						to: buildOrgPath(activeOrganization.slug, '/crm/factures'),
					})
				})
			}
			onRefund={(refundValues) =>
				attempt(() =>
					recordPayment.mutateAsync({
						path: { invoice_id: invoice.id },
						body: paymentRequest(refundValues, 'refund'),
					}),
				)
			}
			onCancel={() =>
				attempt(() =>
					cancelInvoice.mutateAsync({ path: { invoice_id: invoice.id } }),
				)
			}
			onRecordPayment={(paymentValues) =>
				attempt(() =>
					recordPayment.mutateAsync({
						path: { invoice_id: invoice.id },
						body: paymentRequest(paymentValues),
					}),
				)
			}
			onDeletePayment={(paymentId) =>
				void attempt(() =>
					deletePayment.mutateAsync({
						path: { invoice_payment_id: paymentId },
					}),
				)
			}
			onIssueCreditNote={({ reason, ...request }) =>
				attempt(async () => {
					const created = await issueCreditNote.mutateAsync({
						path: { invoice_id: invoice.id },
						body: { reason, ...request },
					})
					await navigate({
						to: buildOrgPath(
							activeOrganization.slug,
							'/crm/avoirs/$creditNoteId',
						),
						params: { creditNoteId: created.data.id },
					})
				})
			}
			onSend={async () => {
				await sendInvoice.mutateAsync({ path: { invoice_id: invoice.id } })
				return 'Facture marquée comme envoyée.'
			}}
			onRemind={async () => {
				await remindInvoice.mutateAsync({ path: { invoice_id: invoice.id } })
				return 'Relance enregistrée dans l’historique de la facture.'
			}}
			onSaveNote={(text) =>
				void noteInvoice.mutateAsync({
					path: { invoice_id: invoice.id },
					body: { note: text },
				})
			}
			onDownloadPdf={() => void downloadPdf()}
		/>
	)
}

function invoiceToForm(invoice: Invoice): InvoiceFormValues {
	const lines: InvoiceLineFormValues[] = invoice.lines.map((line, index) => ({
		clientId: line.id || `line-${index + 1}`,
		label: line.label,
		quantity: line.quantity,
		unitPrice: centsToEuros(line.unit_price_cents),
		vatRateBp:
			line.vat_rate_basis_points != null
				? String(line.vat_rate_basis_points)
				: '',
		discountPercent: line.discount_percent,
		productId: line.product_id ?? null,
		serviceRateId: line.service_rate_id ?? null,
	}))

	return {
		kind: invoice.kind,
		projectId: invoice.project_id ?? '',
		customerId: invoice.customer_id,
		customerContextId: invoice.customer_context_id,
		title: invoice.title,
		termsDays: String(invoice.payment_terms_days),
		legalNotice: invoice.legal_notice,
		notes: invoice.notes ?? '',
		operationNature: invoice.operation_nature ?? '',
		lines: lines.length ? lines : [emptyInvoiceLine()],
	}
}
