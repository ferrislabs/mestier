import { Plus, UserCheck, X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import type { AccountExtras } from '#/pages/customers/feature/mocks/account-extras'
import type { AccountForm } from '#/pages/customers/lib/account-form'
import {
	DEFAULT_FILTERS,
	type DealFilters,
	hasActiveFilters,
	visibleStages,
} from '#/pages/customers/lib/pipeline-filters'
import { CustomerCreateDialog } from '#/pages/customers/ui/customer-create-dialog'
import { CustomerPipelineBoard } from '#/pages/customers/ui/customer-pipeline-board'
import {
	PipelineStats,
	type PipelineSummary,
} from '#/pages/customers/ui/pipeline-stats'
import { PipelineToolbar } from '#/pages/customers/ui/pipeline-toolbar'

interface CustomerPipelineUIProps {
	customers: readonly Customer[]
	summary: PipelineSummary
	filters: DealFilters
	onFiltersChange: (filters: DealFilters) => void
	extrasOf: (customerId: string) => AccountExtras
	canMove: boolean
	error?: string | null
	isLoading?: boolean
	movingCustomerId?: string | null
	notice: string | null
	onDismissNotice: () => void
	onMovePipelineStage: (
		customer: Customer,
		pipelineStage: CustomerPipelineStage,
	) => void
	onOpenCustomer?: (customer: Customer) => void
	onRetry?: () => void
	creating: boolean
	onCreatingChange: (open: boolean) => void
	defaultTerms: number
	defaultOwnerId: string | null
	isCreating: boolean
	createError: string | null
	onCreate: (form: AccountForm) => Promise<void>
}

export function CustomerPipelineUI({
	customers,
	summary,
	filters,
	onFiltersChange,
	extrasOf,
	canMove,
	error,
	isLoading,
	movingCustomerId,
	notice,
	onDismissNotice,
	onMovePipelineStage,
	onOpenCustomer,
	onRetry,
	creating,
	onCreatingChange,
	defaultTerms,
	defaultOwnerId,
	isCreating,
	createError,
	onCreate,
}: CustomerPipelineUIProps) {
	return (
		<PageShell className="block max-w-none">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Pipeline"
				description="Suis tes opportunités de la première demande au client gagné et transforme tes prospects en clients."
				actions={
					<RequirePermission permission="MANAGE_CUSTOMERS">
						<CustomerCreateDialog
							open={creating}
							onOpenChange={onCreatingChange}
							defaultTerms={defaultTerms}
							defaultOwnerId={defaultOwnerId}
							isPending={isCreating}
							error={createError}
							onSubmit={onCreate}
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
				canReset={hasActiveFilters(filters)}
				onChange={(patch) => onFiltersChange({ ...filters, ...patch })}
				onReset={() => onFiltersChange(DEFAULT_FILTERS)}
			/>
			{error ? (
				<div className="mb-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] text-state-alert">
					<p className="font-medium">{error}</p>
					{onRetry ? (
						<Button type="button" variant="outline" size="sm" onClick={onRetry}>
							Réessayer
						</Button>
					) : null}
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
			<CustomerPipelineBoard
				customers={customers}
				stages={visibleStages(filters.closed)}
				extrasOf={extrasOf}
				canMove={canMove}
				isLoading={isLoading}
				movingCustomerId={movingCustomerId}
				onMove={onMovePipelineStage}
				onOpenCustomer={onOpenCustomer}
			/>
		</PageShell>
	)
}
