import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	Ban,
	CheckCircle2,
	FileText,
	Loader2,
} from 'lucide-react'
import type * as React from 'react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type { Project } from '#/hooks/use-projects'
import type { SupplierInvoice } from '#/hooks/use-supplier-invoices'
import { buildOrgPath } from '#/modules/org-path'
import {
	formatDate,
	formatMoney,
	supplierInvoiceSourceLabel,
	supplierInvoiceStatusLabel,
} from '#/pages/purchase/types'
import { SupplierInvoiceLineAllocationEditor } from '#/pages/purchase/ui/supplier-invoice-line-allocation-editor'

interface SupplierInvoiceDetailUIProps {
	organizationSlug: string
	invoice: SupplierInvoice
	projects: Project[]
	/** `undefined` while the signed link is still resolving, or when the
	 * invoice was entered by hand with no file behind it. */
	fileUrl: string | undefined
	onSaveNotes: (notes: string | null) => void
	isSavingNotes: boolean
	onConfirm: (notes: string | null) => void
	isConfirming: boolean
	onReject: (notes: string | null) => void
	isRejecting: boolean
	onSaveLineAllocations: (
		lineId: string,
		shares: { projectId: string; amountCents: number }[],
	) => void
	isSavingAllocations: boolean
}

/**
 * The screen where a supplier invoice becomes project cost (#340): the
 * document next to what was parsed from it, confirm/reject, and per-line
 * allocation to a project.
 */
export function SupplierInvoiceDetailUI({
	organizationSlug,
	invoice,
	projects,
	fileUrl,
	onSaveNotes,
	isSavingNotes,
	onConfirm,
	isConfirming,
	onReject,
	isRejecting,
	onSaveLineAllocations,
	isSavingAllocations,
}: SupplierInvoiceDetailUIProps) {
	const [notes, setNotes] = useState(invoice.notes ?? '')
	const [openLineId, setOpenLineId] = useState<string | null>(
		invoice.lines[0]?.id ?? null,
	)
	const canReview = invoice.status === 'RECEIVED'

	return (
		<PageShell>
			<PageHeader
				title={invoice.supplier_name}
				description={`Facture ${invoice.number} · reçue le ${formatDate(invoice.received_at)}`}
				actions={
					<div className="flex items-center gap-2">
						<Button asChild type="button" variant="outline">
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/purchase/supplier-invoices',
								)}
							>
								<ArrowLeft />
								Retour à la boîte de réception
							</Link>
						</Button>
						{canReview ? (
							<>
								<Button
									type="button"
									variant="outline"
									disabled={isRejecting}
									onClick={() => onReject(notes.trim() || null)}
								>
									{isRejecting ? <Loader2 className="animate-spin" /> : <Ban />}
									Rejeter
								</Button>
								<Button
									type="button"
									disabled={isConfirming}
									onClick={() => onConfirm(notes.trim() || null)}
								>
									{isConfirming ? (
										<Loader2 className="animate-spin" />
									) : (
										<CheckCircle2 />
									)}
									Confirmer
								</Button>
							</>
						) : null}
					</div>
				}
			/>

			<section>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<Metric label="Statut">
						<StatusBadge tone={statusTone(invoice.status)}>
							{supplierInvoiceStatusLabel(invoice.status)}
						</StatusBadge>
					</Metric>
					<Metric
						label="Origine"
						value={supplierInvoiceSourceLabel(invoice.source)}
					/>
					<Metric label="Émise le" value={formatDate(invoice.issued_on)} />
					<Metric
						label="Total TTC"
						value={formatMoney(invoice.gross_cents)}
						emphasize
					/>
				</div>
			</section>

			<div className="grid gap-6 lg:grid-cols-2">
				<SectionCard>
					<SectionHeader
						title="Document original"
						description="Ce qui a été reçu, à comparer avec ce qui a été lu."
					/>
					<div className="p-5">
						{fileUrl ? (
							<iframe
								src={fileUrl}
								title={`Facture ${invoice.number}`}
								className="h-[600px] w-full rounded-md border"
							/>
						) : (
							<div className="flex h-52 flex-col items-center justify-center gap-2 rounded-md border border-dashed text-sm text-muted-foreground">
								<FileText className="size-5" />
								<p>Aucun fichier associé à cette facture.</p>
							</div>
						)}
					</div>
				</SectionCard>

				<SectionCard>
					<SectionHeader
						title={`Lignes (${invoice.lines.length})`}
						description="Attribuez chaque ligne à un ou plusieurs chantiers, ou laissez-la de côté pour des frais généraux."
					/>
					<div className="divide-y">
						{invoice.lines.map((line) => (
							<SupplierInvoiceLineAllocationEditor
								key={`${line.id}-${line.allocations.map((allocation) => `${allocation.id}:${allocation.amount_cents}`).join(',')}`}
								line={line}
								projects={projects}
								isOpen={openLineId === line.id}
								isSaving={isSavingAllocations}
								onOpenChange={(open) => setOpenLineId(open ? line.id : null)}
								onSave={(shares) => onSaveLineAllocations(line.id, shares)}
							/>
						))}
					</div>
				</SectionCard>
			</div>

			<SectionCard>
				<SectionHeader
					title="Note de suivi"
					description="Visible uniquement en interne — n’affecte jamais les montants du document."
				/>
				<div className="space-y-3 p-5">
					<Textarea
						rows={3}
						value={notes}
						onChange={(event) => setNotes(event.target.value)}
						placeholder="Contexte utile pour la prochaine relecture…"
					/>
					<div className="flex justify-end">
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={isSavingNotes || (invoice.notes ?? '') === notes}
							onClick={() => onSaveNotes(notes.trim() || null)}
						>
							{isSavingNotes ? <Loader2 className="animate-spin" /> : null}
							Enregistrer la note
						</Button>
					</div>
				</div>
			</SectionCard>
		</PageShell>
	)
}

function Metric({
	label,
	value,
	children,
	emphasize,
}: {
	label: string
	value?: string
	children?: React.ReactNode
	emphasize?: boolean
}) {
	return (
		<div className="rounded-lg border bg-card p-4">
			<p className="text-xs text-muted-foreground">{label}</p>
			<div
				className={emphasize ? 'mt-1 text-lg font-bold' : 'mt-1 font-medium'}
			>
				{children ?? value}
			</div>
		</div>
	)
}

function statusTone(status: SupplierInvoice['status']) {
	if (status === 'CONFIRMED') return 'success' as const
	if (status === 'REJECTED') return 'error' as const
	return 'brand' as const
}

export namespace SupplierInvoiceDetailUI {
	export function Loading() {
		return (
			<PageShell>
				<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
					Chargement…
				</SectionCard>
			</PageShell>
		)
	}

	export function ErrorState({
		organizationSlug,
		message,
	}: {
		organizationSlug: string
		message: string
	}) {
		return (
			<PageShell>
				<SectionCard className="flex flex-col items-center gap-3 p-12 text-center">
					<AlertCircle className="size-6 text-destructive" />
					<p className="text-sm font-medium">{message}</p>
					<Button asChild type="button" variant="outline">
						<Link
							to={buildOrgPath(organizationSlug, '/purchase/supplier-invoices')}
						>
							<ArrowLeft />
							Retour à la boîte de réception
						</Link>
					</Button>
				</SectionCard>
			</PageShell>
		)
	}
}
