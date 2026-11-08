import { Plus } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type {
	HistoryFilters,
	sortPending,
} from '#/pages/hr/lib/rh/absences-view'
import type { Absence, RhState } from '#/pages/hr/lib/rh/types'
import {
	AbsenceFormDialog,
	type AbsenceFormInitial,
} from '#/pages/hr/ui/rh/absences/absence-form-dialog'
import { AbsenceTypesTab } from '#/pages/hr/ui/rh/absences/absence-types-tab'
import { HistorySection } from '#/pages/hr/ui/rh/absences/history-section'
import { PendingRequests } from '#/pages/hr/ui/rh/absences/pending-requests'
import { RejectDialog } from '#/pages/hr/ui/rh/absences/reject-dialog'

export type AbsencesTab = 'demandes' | 'motifs'

export interface AbsencesPageProps {
	state: RhState
	tab: AbsencesTab
	onTabChange: (tab: AbsencesTab) => void
	pending: ReturnType<typeof sortPending>
	historyStatus: HistoryFilters['status']
	historyMemberId: string
	onHistoryFiltersChange: (patch: {
		status: HistoryFilters['status']
		memberId: string
	}) => void
	onApprove: (absence: Absence) => void
	onAskReject: (absence: Absence) => void
	onCancel: (absence: Absence) => void
	rejecting: { absence: Absence | null; open: boolean }
	onRejectOpenChange: (open: boolean) => void
	onRejectConfirm: (absence: Absence, comment: string) => Promise<void>
	creating: boolean
	initial?: AbsenceFormInitial
	onCreatingChange: (open: boolean, initial?: AbsenceFormInitial) => void
	saveError: string | null
	notice: string
}

export function AbsencesPage({
	state,
	tab,
	onTabChange,
	pending,
	historyStatus,
	historyMemberId,
	onHistoryFiltersChange,
	onApprove,
	onAskReject,
	onCancel,
	rejecting,
	onRejectOpenChange,
	onRejectConfirm,
	creating,
	initial,
	onCreatingChange,
	saveError,
	notice,
}: AbsencesPageProps) {
	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Absences"
				description={
					pending.length > 0
						? `${pending.length} demande${pending.length > 1 ? 's' : ''} à valider`
						: 'Aucune demande en attente.'
				}
				actions={
					<RequirePermission permission="MANAGE_REFERENCE">
						<Button
							type="button"
							size="sm"
							onClick={() => onCreatingChange(true)}
						>
							<Plus />
							Poser une absence
						</Button>
					</RequirePermission>
				}
			/>
			{saveError ? (
				<div className="mb-6 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{saveError}
				</div>
			) : null}
			<Tabs
				value={tab}
				onValueChange={(value) => onTabChange(value as AbsencesTab)}
				className="gap-6"
			>
				<TabsList aria-label="Sections des absences">
					<TabsTrigger value="demandes">
						Demandes
						{pending.length > 0 && (
							<span className="rounded-full bg-state-today/15 px-1.5 text-[11px] text-state-today leading-4 tabular">
								{pending.length}
								<span className="sr-only"> à valider</span>
							</span>
						)}
					</TabsTrigger>
					<TabsTrigger value="motifs">Motifs</TabsTrigger>
				</TabsList>
				<TabsContent value="demandes">
					{pending.length > 0 && (
						<PendingRequests
							state={state}
							requests={pending}
							onApprove={onApprove}
							onReject={onAskReject}
						/>
					)}
					<HistorySection
						state={state}
						status={historyStatus}
						memberId={historyMemberId}
						onFiltersChange={onHistoryFiltersChange}
						onApprove={onApprove}
						onReject={onAskReject}
						onCancel={onCancel}
						onCreate={() => onCreatingChange(true)}
					/>
				</TabsContent>
				<TabsContent value="motifs">
					<AbsenceTypesTab />
				</TabsContent>
			</Tabs>
			<RejectDialog
				absence={rejecting.absence}
				open={rejecting.open}
				onOpenChange={onRejectOpenChange}
				onConfirm={onRejectConfirm}
			/>
			<AbsenceFormDialog
				open={creating}
				initial={initial}
				onOpenChange={(open) => onCreatingChange(open)}
			/>
			<output className="sr-only">{notice}</output>
		</PageShell>
	)
}
