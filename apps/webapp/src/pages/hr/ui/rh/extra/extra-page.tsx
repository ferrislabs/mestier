import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type {
	ExtraPeriod,
	ExtraStatus,
	IsoDate,
	RhState,
} from '#/pages/hr/lib/rh/types'
import {
	ExtraFormDialog,
	type ExtraFormInitial,
} from '#/pages/hr/ui/rh/extra/extra-form-dialog'
import { HistorySection } from '#/pages/hr/ui/rh/extra/history-section'
import { MonthSummary } from '#/pages/hr/ui/rh/extra/month-summary'
import { PendingExtras } from '#/pages/hr/ui/rh/extra/pending-extras'
import { RejectDialog } from '#/pages/hr/ui/rh/extra/reject-dialog'
import { SurchargesTab } from '#/pages/hr/ui/rh/extra/surcharges-tab'

export type ExtraTab = 'declarations' | 'majorations'

export interface ExtraPageProps {
	state: RhState
	today: IsoDate
	month: IsoDate
	onMonthChange: (month: IsoDate) => void
	tab: ExtraTab
	onTabChange: (tab: ExtraTab) => void
	pending: readonly ExtraPeriod[]
	status: ExtraStatus | 'all'
	memberId: string
	onFiltersChange: (patch: {
		status: ExtraStatus | 'all'
		memberId: string
	}) => void
	onApprove: (extra: ExtraPeriod) => void
	onAskReject: (extra: ExtraPeriod) => void
	onCancel: (extra: ExtraPeriod) => void
	rejecting: { extra: ExtraPeriod | null; open: boolean }
	onRejectOpenChange: (open: boolean) => void
	onRejectConfirm: (extra: ExtraPeriod, comment: string) => void
	creating: boolean
	initial?: ExtraFormInitial
	onCreatingChange: (open: boolean, initial?: ExtraFormInitial) => void
	notice: string
}

export function ExtraPage({
	state,
	today,
	month,
	onMonthChange,
	tab,
	onTabChange,
	pending,
	status,
	memberId,
	onFiltersChange,
	onApprove,
	onAskReject,
	onCancel,
	rejecting,
	onRejectOpenChange,
	onRejectConfirm,
	creating,
	initial,
	onCreatingChange,
	notice,
}: ExtraPageProps) {
	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Heures extra"
				description={
					pending.length > 0
						? `${pending.length} déclaration${pending.length > 1 ? 's' : ''} à valider`
						: 'Aucune déclaration en attente.'
				}
				actions={
					<Button
						type="button"
						size="sm"
						onClick={() => onCreatingChange(true)}
					>
						<Plus />
						Déclarer des heures
					</Button>
				}
			/>
			<Tabs
				value={tab}
				onValueChange={(value) => onTabChange(value as ExtraTab)}
				className="gap-6"
			>
				<TabsList aria-label="Sections des heures extra">
					<TabsTrigger value="declarations">
						Déclarations
						{pending.length > 0 && (
							<span className="tabular rounded-full bg-state-today/15 px-1.5 text-[11px] text-state-today leading-4">
								{pending.length}
								<span className="sr-only"> à valider</span>
							</span>
						)}
					</TabsTrigger>
					<TabsTrigger value="majorations">Majorations</TabsTrigger>
				</TabsList>
				<TabsContent value="declarations">
					<MonthSummary
						state={state}
						month={month}
						today={today}
						onMonthChange={onMonthChange}
					/>
					{pending.length > 0 && (
						<PendingExtras
							state={state}
							requests={pending}
							onApprove={onApprove}
							onReject={onAskReject}
						/>
					)}
					<HistorySection
						state={state}
						status={status}
						memberId={memberId}
						onFiltersChange={onFiltersChange}
						onApprove={onApprove}
						onReject={onAskReject}
						onCancel={onCancel}
						onCreate={() => onCreatingChange(true)}
					/>
				</TabsContent>
				<TabsContent value="majorations">
					<SurchargesTab />
				</TabsContent>
			</Tabs>
			<RejectDialog
				extra={rejecting.extra}
				open={rejecting.open}
				onOpenChange={onRejectOpenChange}
				onConfirm={onRejectConfirm}
			/>
			<ExtraFormDialog
				open={creating}
				initial={initial}
				onOpenChange={(open) => onCreatingChange(open)}
			/>
			<output className="sr-only">{notice}</output>
		</PageShell>
	)
}
