import { Plus, UserCheck, X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Customer } from '#/hooks/use-customers'
import type {
	Deal,
	DealDraft,
	DealQuote,
	DealStage,
	IsoDate,
} from '#/pages/deals/lib/deal-types'
import type { PipelineSummary } from '#/pages/deals/lib/pipeline'
import {
	type DealFilters,
	dealQuotes,
	visibleStages,
} from '#/pages/deals/lib/pipeline-filters'
import { DealCreateDialog } from '#/pages/deals/ui/deal-create-dialog'
import { DealDialog } from '#/pages/deals/ui/deal-dialog'
import { LostReasonDialog } from '#/pages/deals/ui/lost-reason-dialog'
import { PipelineBoard } from '#/pages/deals/ui/pipeline-board'
import { PipelineStats } from '#/pages/deals/ui/pipeline-stats'
import { PipelineToolbar } from '#/pages/deals/ui/pipeline-toolbar'

export function PipelineUI({
	deals,
	customers,
	quotes,
	summary,
	filters,
	canReset,
	today,
	organizationSlug,
	defaultOwnerId,
	canManage,
	isLoading,
	error,
	notice,
	creating,
	selected,
	losing,
	onRetry,
	onFiltersChange,
	onReset,
	onDismissNotice,
	onCreatingChange,
	onCreate,
	onOpen,
	onCloseSelected,
	onMove,
	onSave,
	onDelete,
	onCancelLoss,
	onConfirmLoss,
}: {
	deals: readonly Deal[]
	customers: readonly Customer[]
	quotes: readonly DealQuote[]
	summary: PipelineSummary
	filters: DealFilters
	canReset: boolean
	today: IsoDate
	organizationSlug: string
	defaultOwnerId: string
	canManage: boolean
	isLoading: boolean
	error: string | null
	notice: string | null
	creating: boolean
	selected: Deal | undefined
	losing: Deal | undefined
	onRetry: () => void
	onFiltersChange: (patch: Partial<DealFilters>) => void
	onReset: () => void
	onDismissNotice: () => void
	onCreatingChange: (open: boolean) => void
	onCreate: (draft: DealDraft) => void
	onOpen: (dealId: string) => void
	onCloseSelected: () => void
	onMove: (dealId: string, stage: DealStage) => void
	onSave: (dealId: string, draft: DealDraft) => void
	onDelete: (dealId: string) => void
	onCancelLoss: () => void
	onConfirmLoss: (dealId: string, reason: string) => void
}) {
	return (
		<PageShell className="block max-w-none">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Pipeline"
				description="Suis tes opportunités de la première demande au devis accepté et transforme tes prospects en clients."
				actions={
					<RequirePermission permission="MANAGE_CUSTOMERS">
						<DealCreateDialog
							open={creating}
							onOpenChange={onCreatingChange}
							customers={customers}
							defaultOwnerId={defaultOwnerId}
							organizationSlug={organizationSlug}
							onCreate={onCreate}
							trigger={
								<Button size="sm">
									<Plus aria-hidden /> Nouvelle opportunité
								</Button>
							}
						/>
					</RequirePermission>
				}
			/>
			<PipelineStats summary={summary} />
			<PipelineToolbar
				filters={filters}
				canReset={canReset}
				onChange={onFiltersChange}
				onReset={onReset}
			/>
			{error ? (
				<div className="mb-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] text-state-alert">
					<p className="font-medium">{error}</p>
					<Button type="button" variant="outline" size="sm" onClick={onRetry}>
						Réessayer
					</Button>
				</div>
			) : null}
			{notice && (
				<output className="fixed bottom-6 left-1/2 z-50 flex max-w-[calc(100vw-2rem)] -translate-x-1/2 items-center gap-2 rounded-md border border-state-progress/30 bg-card px-3 py-2 text-sm shadow-lg">
					<UserCheck
						className="size-4 shrink-0 text-state-progress"
						aria-hidden
					/>
					<span className="min-w-0 flex-1 font-medium">{notice}</span>
					<Button
						type="button"
						variant="ghost"
						size="icon-xs"
						aria-label="Fermer la notification"
						onClick={onDismissNotice}
					>
						<X aria-hidden />
					</Button>
				</output>
			)}
			<PipelineBoard
				deals={deals}
				stages={visibleStages(filters.closed)}
				customers={customers}
				quotes={quotes}
				today={today}
				organizationSlug={organizationSlug}
				canManage={canManage}
				isLoading={isLoading}
				onOpen={onOpen}
				onMove={onMove}
			/>
			{selected && (
				<DealDialog
					key={selected.id}
					deal={selected}
					customers={customers}
					quotes={dealQuotes(quotes, selected.id)}
					organizationSlug={organizationSlug}
					canManage={canManage}
					onClose={onCloseSelected}
					onMove={onMove}
					onSave={(draft) => onSave(selected.id, draft)}
					onDelete={() => onDelete(selected.id)}
				/>
			)}
			{losing && (
				<LostReasonDialog
					key={losing.id}
					deal={losing}
					onCancel={onCancelLoss}
					onConfirm={(reason) => onConfirmLoss(losing.id, reason)}
				/>
			)}
		</PageShell>
	)
}
