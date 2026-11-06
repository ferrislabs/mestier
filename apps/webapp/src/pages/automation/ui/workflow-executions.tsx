import { History, Loader2, TriangleAlert } from 'lucide-react'
import type { ReactNode } from 'react'
import type { Schemas } from '#/api/api.client'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import type { ExecutionStatus } from '#/pages/automation/lib/execution-status'
import type { NodePosition } from '#/pages/automation/lib/graph'
import type { RunView } from '#/pages/automation/lib/run-view'
import { ExecutionCanvas } from '#/pages/automation/ui/execution-canvas'
import {
	type ExecutionFilter,
	ExecutionList,
	type ExecutionListEntry,
} from '#/pages/automation/ui/execution-list'
import { ExecutionStatusLabel } from '#/pages/automation/ui/execution-status'
import {
	type ExecutionStepEntry,
	type ExecutionStepItem,
	ExecutionSteps,
} from '#/pages/automation/ui/execution-steps'

export interface ExecutionDetailHeader {
	runId: string
	number: number
	startedAt: string
	trigger: string
	status: ExecutionStatus
	duration: string | undefined
	versionLabel: string
	isCurrentVersion: boolean
}

export interface ExecutionDetailData {
	header: ExecutionDetailHeader
	graph: Schemas.GraphDto | null
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	view: RunView
	steps: ExecutionStepItem[]
	pending: ExecutionStepEntry[]
	failure: { label: string; message: string; nodeId: string } | null
}

function EmptyExecutions() {
	return (
		<div className="flex h-full items-center justify-center bg-background px-6">
			<div className="flex max-w-sm flex-col items-center text-center">
				<History className="mb-3 size-8 text-muted-foreground" aria-hidden />
				<p className="font-medium text-base">Aucune exécution pour le moment</p>
				<p className="mt-1 text-muted-foreground text-sm">
					Exécutez le workflow ou activez-le pour voir apparaître ses exécutions
					ici, nœud par nœud.
				</p>
			</div>
		</div>
	)
}

function ExecutionDetail({
	detail,
	onOpenNode,
}: {
	detail: ExecutionDetailData
	onOpenNode: (nodeId: string) => void
}) {
	const { header, failure } = detail
	return (
		<section
			aria-label={`Exécution #${header.number}`}
			className="flex min-h-0 min-w-0 flex-col"
		>
			<header className="flex flex-wrap items-center gap-x-4 gap-y-2 border-border border-b bg-card px-5 py-3">
				<div className="min-w-0">
					<h2 className="font-semibold text-base leading-tight">
						Exécution{' '}
						<span className="font-mono text-muted-foreground">
							#{header.number}
						</span>
					</h2>
					<p className="mt-0.5 text-muted-foreground text-sm">
						{header.startedAt} · {header.trigger}
					</p>
				</div>
				<ExecutionStatusLabel status={header.status} detail={header.duration} />
				<span
					className="ml-auto inline-flex items-center gap-1.5 rounded-md border border-border px-2 py-1 text-muted-foreground text-xs"
					title="Le workflow tel qu’il était au lancement de cette exécution"
				>
					<History className="size-3.5" aria-hidden />
					{header.versionLabel}
					<span className="font-medium text-foreground">
						{header.isCurrentVersion ? '· actuelle' : '· antérieure'}
					</span>
				</span>
			</header>
			{failure && (
				<div
					role="alert"
					className="flex items-center gap-3 border-state-alert/30 border-b bg-state-alert/8 px-5 py-2 text-sm"
				>
					<TriangleAlert
						className="size-4 shrink-0 text-state-alert"
						aria-hidden
					/>
					<p className="min-w-0 flex-1 truncate">
						<span className="font-medium text-state-alert">
							Échec sur « {failure.label} »
						</span>{' '}
						<span className="text-muted-foreground">: {failure.message}</span>
					</p>
					<Button
						variant="outline"
						size="sm"
						onClick={() => onOpenNode(failure.nodeId)}
					>
						Voir le nœud
					</Button>
				</div>
			)}
			<div className="flex min-h-0 flex-1">
				<div className="relative min-w-0 flex-1">
					{detail.graph ? (
						<ExecutionCanvas
							key={header.runId}
							graph={detail.graph}
							layout={detail.layout}
							descriptors={detail.descriptors}
							view={detail.view}
							onOpenNode={onOpenNode}
						/>
					) : (
						<div className="flex h-full items-center justify-center p-8 text-center text-muted-foreground text-sm">
							La version de ce run a été supprimée depuis — son graphe n’est
							plus disponible, mais ses étapes restent consultables.
						</div>
					)}
				</div>
				<ExecutionSteps
					steps={detail.steps}
					pending={detail.pending}
					pendingTitle={
						header.status === 'running' || header.status === 'waiting'
							? 'En attente'
							: 'Non appelés'
					}
					onOpen={onOpenNode}
				/>
			</div>
		</section>
	)
}

export interface WorkflowExecutionsProps {
	entries: readonly ExecutionListEntry[]
	total: number
	selectedNumber: number | null
	filter: ExecutionFilter
	isLoading: boolean
	error: string | null
	detail: ExecutionDetailData | null
	detailLoading: boolean
	detailError: string | null
	onSelect: (number: number) => void
	onFilterChange: (filter: ExecutionFilter) => void
	onOpenNode: (nodeId: string) => void
	onRunNow: () => void
	nodeDialog: ReactNode
}

function ExecutionsHeader({ onRunNow }: { onRunNow: () => void }) {
	return (
		<RequirePermission permission="MANAGE_AUTOMATION">
			<header className="flex shrink-0 items-center justify-end border-border border-b bg-card px-4 py-2">
				<Button size="sm" onClick={onRunNow}>
					Exécuter maintenant
				</Button>
			</header>
		</RequirePermission>
	)
}

function ExecutionsBody({
	entries,
	total,
	selectedNumber,
	filter,
	isLoading,
	error,
	detail,
	detailLoading,
	detailError,
	onSelect,
	onFilterChange,
	onOpenNode,
	nodeDialog,
}: Omit<WorkflowExecutionsProps, 'onRunNow'>) {
	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement des exécutions…
			</div>
		)
	}

	if (error) {
		return (
			<div
				role="alert"
				className="m-6 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
			>
				{error}
			</div>
		)
	}

	if (total === 0) return <EmptyExecutions />

	return (
		<div className="grid h-full min-h-0 grid-cols-[18rem_minmax(0,1fr)] bg-background">
			<ExecutionList
				entries={entries}
				total={total}
				selectedNumber={selectedNumber}
				onSelect={onSelect}
				filter={filter}
				onFilterChange={onFilterChange}
			/>
			{detail ? (
				<ExecutionDetail detail={detail} onOpenNode={onOpenNode} />
			) : detailError ? (
				<div
					role="alert"
					className="m-6 h-fit rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
				>
					Impossible de charger cette exécution : {detailError}
				</div>
			) : (
				<div className="flex items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					{detailLoading ? <Loader2 className="size-5 animate-spin" /> : null}
					{detailLoading ? 'Chargement de l’exécution…' : null}
				</div>
			)}
			{nodeDialog}
		</div>
	)
}

export function WorkflowExecutions({
	onRunNow,
	...body
}: WorkflowExecutionsProps) {
	return (
		<div className="flex h-full min-h-0 flex-col">
			<ExecutionsHeader onRunNow={onRunNow} />
			<div className="min-h-0 flex-1">
				<ExecutionsBody {...body} />
			</div>
		</div>
	)
}
