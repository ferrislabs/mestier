import { Link, Navigate } from '@tanstack/react-router'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomerContexts, useCustomers } from '#/hooks/use-customers'
import { useInvoice } from '#/hooks/use-invoices'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { buildOrgPath } from '#/modules/org-path'
import { CreditNoteDetailUI } from '#/pages/credit-notes/ui/credit-note-detail-ui'
import {
	buildInvoicePaper,
	creditNoteKindLabel,
	savedInvoiceTotals,
} from '#/pages/invoices/lib/invoice-document'
import { invoiceDisplayState } from '#/pages/invoices/types'
import { vatNoticeOf } from '#/pages/quotes/lib/paper-parties'
import {
	DetailError,
	DetailLoading,
	NotFoundState,
} from '#/pages/quotes/ui/doc/detail-shell'

export function CreditNoteDetailFeature({
	creditNoteId,
}: {
	creditNoteId: string
}) {
	const { activeOrganization } = useActiveOrganization()
	const note = useInvoice(creditNoteId)
	const source = useInvoice(
		note.data?.data.source_invoice_id ?? '',
		Boolean(note.data?.data.source_invoice_id),
	)
	const customers = useCustomers(activeOrganization.id)
	const contexts = useCustomerContexts(
		note.data?.data.customer_id ?? '',
		Boolean(note.data?.data.customer_id),
	)
	const [isDownloading, setIsDownloading] = useState(false)
	const [downloadError, setDownloadError] = useState<string | null>(null)

	if (note.isLoading) return <DetailLoading label="Chargement de l’avoir…" />

	if (note.isError) {
		return (
			<DetailError
				title="Impossible de charger l’avoir"
				message={note.error.message}
				backTo="/crm/avoirs"
				backLabel="Retour aux avoirs"
				onRetry={() => void note.refetch()}
			/>
		)
	}

	const creditNote = note.data?.data
	if (!creditNote) {
		return (
			<NotFoundState
				title="Cet avoir est introuvable"
				hint="Il n’existe pas ou l’adresse n’est plus valide."
			>
				<Button variant="outline" size="sm" asChild>
					<Link to={buildOrgPath(activeOrganization.slug, '/crm/avoirs')}>
						Retour aux avoirs
					</Link>
				</Button>
			</NotFoundState>
		)
	}

	if (creditNote.kind !== 'CREDIT_NOTE') {
		return (
			<Navigate
				to={buildOrgPath(
					activeOrganization.slug,
					`/crm/factures/${creditNote.id}`,
				)}
				replace
			/>
		)
	}

	const sourceInvoice = source.data?.data
	const customer = customers.data?.data.find(
		(item) => item.id === creditNote.customer_id,
	)
	const downloadPdf = async () => {
		setDownloadError(null)
		setIsDownloading(true)
		try {
			await downloadAuthenticatedFile(
				`/api/v1/invoices/${creditNote.id}/pdf`,
				`${creditNote.number ?? creditNote.id}.pdf`,
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
		<CreditNoteDetailUI
			note={creditNote}
			source={sourceInvoice}
			sourceState={sourceInvoice ? invoiceDisplayState(sourceInvoice) : null}
			sourceRemainingCents={sourceInvoice?.balance_cents ?? null}
			customer={customer}
			kindLabel={creditNoteKindLabel(creditNote)}
			paper={buildInvoicePaper({
				invoice: creditNote,
				customer,
				context: contexts.data?.data.find(
					(context) => context.id === creditNote.customer_context_id,
				),
				organization: activeOrganization,
				reference: sourceInvoice
					? `Avoir sur la facture ${sourceInvoice.number ?? 'en brouillon'}`
					: null,
			})}
			totals={savedInvoiceTotals(creditNote)}
			vatNotice={
				creditNote.vat_breakdown.length === 0
					? vatNoticeOf(activeOrganization)
					: null
			}
			error={downloadError}
			isDownloading={isDownloading}
			onDownloadPdf={() => void downloadPdf()}
		/>
	)
}
