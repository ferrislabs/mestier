import { RotateCcw } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { RequirePermission } from '#/components/require-permission'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { StatusBadge } from '#/components/ui/surface'
import type { RunStepTreeNode } from '#/pages/automation/lib/workflow-runs'
import {
	canReplay,
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface RunStepListProps {
	stepTree: RunStepTreeNode[]
	runStatus: string
	onReplay: (connectorId: string) => void
	isReplaying: boolean
	replayingConnectorId: string | null
	replayError: string | null
}

interface TreeHandlers {
	runStatus: string
	onRequestReplay: (connectorId: string) => void
	isReplaying: boolean
	replayingConnectorId: string | null
}

export function RunStepList({
	stepTree,
	runStatus,
	onReplay,
	isReplaying,
	replayingConnectorId,
	replayError,
}: RunStepListProps) {
	const [pendingReplayConnectorId, setPendingReplayConnectorId] = useState<
		string | null
	>(null)

	return (
		<div className="flex flex-col gap-3">
			{replayError ? (
				<p role="alert" className="text-sm text-destructive">
					{replayError}
				</p>
			) : null}

			<StepTree
				nodes={stepTree}
				runStatus={runStatus}
				onRequestReplay={setPendingReplayConnectorId}
				isReplaying={isReplaying}
				replayingConnectorId={replayingConnectorId}
			/>

			<AlertDialog
				open={pendingReplayConnectorId !== null}
				onOpenChange={(open) => {
					if (!open) setPendingReplayConnectorId(null)
				}}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>
							Relancer depuis {pendingReplayConnectorId} ?
						</AlertDialogTitle>
						<AlertDialogDescription>
							Cette étape et toutes celles en aval seront rejouées. Les étapes
							en amont ne seront pas ré-exécutées : leur résultat est relu tel
							quel.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Annuler</AlertDialogCancel>
						<AlertDialogAction
							onClick={() => {
								if (pendingReplayConnectorId) onReplay(pendingReplayConnectorId)
								setPendingReplayConnectorId(null)
							}}
						>
							Relancer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</div>
	)
}

function StepTree({
	nodes,
	...handlers
}: { nodes: RunStepTreeNode[] } & TreeHandlers) {
	if (nodes.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">Aucune étape enregistrée.</p>
		)
	}

	return (
		<div className="flex flex-col gap-2">
			{nodes.map((node) =>
				node.kind === 'step' ? (
					<StepRow key={node.step.id} step={node.step} {...handlers} />
				) : (
					<LoopGroup key={node.connectorId} node={node} {...handlers} />
				),
			)}
		</div>
	)
}

function LoopGroup({
	node,
	...handlers
}: {
	node: Extract<RunStepTreeNode, { kind: 'loop' }>
} & TreeHandlers) {
	return (
		<Collapsible className="rounded-lg border">
			<CollapsibleTrigger className="flex w-full items-center justify-between px-3 py-2 text-left text-sm font-medium">
				<span>
					Boucle {node.connectorId} · {node.iterations.length} itération
					{node.iterations.length > 1 ? 's' : ''}
				</span>
			</CollapsibleTrigger>
			<CollapsibleContent className="flex flex-col gap-2 border-t p-2">
				{node.iterations.map((iteration) => (
					<Collapsible key={iteration.path} className="rounded-md border">
						<CollapsibleTrigger className="flex w-full items-center justify-between px-2 py-1.5 text-left text-xs font-medium text-muted-foreground">
							Itération {iteration.index + 1}
						</CollapsibleTrigger>
						<CollapsibleContent className="flex flex-col gap-2 border-t p-2">
							<StepTree nodes={iteration.children} {...handlers} />
						</CollapsibleContent>
					</Collapsible>
				))}
			</CollapsibleContent>
		</Collapsible>
	)
}

function formatStepTimings(step: Schemas.RunStepResponse): string {
	if (!step.started_at) return 'Pas encore démarrée'
	if (!step.finished_at) return `Démarrée à ${step.started_at}`
	return `${step.started_at} → ${step.finished_at}`
}

function StepRow({
	step,
	runStatus,
	onRequestReplay,
	isReplaying,
	replayingConnectorId,
}: { step: Schemas.RunStepResponse } & TreeHandlers) {
	return (
		<div className="rounded-lg border p-3 text-sm">
			<div className="flex items-center justify-between gap-2">
				<span className="font-mono font-medium">{step.connector_id}</span>
				<StatusBadge tone={RUN_STATUS_TONE[step.status] ?? 'neutral'}>
					{RUN_STATUS_LABEL[step.status] ?? step.status}
				</StatusBadge>
			</div>

			<div className="mt-2 grid gap-2 sm:grid-cols-2">
				<StepField label="Entrée résolue" value={step.input} />
				<StepField label="Sortie" value={step.output} />
			</div>

			{step.error ? (
				<p className="mt-2 text-xs text-destructive">{step.error}</p>
			) : null}

			<p className="mt-2 text-xs text-muted-foreground">
				<span>
					{step.attempts} tentative{step.attempts > 1 ? 's' : ''}
				</span>{' '}
				· <span>{formatStepTimings(step)}</span>
			</p>

			{canReplay(runStatus) ? (
				<RequirePermission permission="MANAGE_AUTOMATION">
					<Button
						variant="outline"
						size="sm"
						className="mt-2"
						disabled={isReplaying && replayingConnectorId === step.connector_id}
						onClick={() => onRequestReplay(step.connector_id)}
					>
						<RotateCcw className="size-3.5" />
						Relancer depuis ici
					</Button>
				</RequirePermission>
			) : null}
		</div>
	)
}

function StepField({ label, value }: { label: string; value: unknown }) {
	return (
		<div className="min-w-0">
			<p className="text-xs font-medium text-muted-foreground">{label}</p>
			<pre className="mt-0.5 overflow-x-auto rounded bg-muted/40 p-2 font-mono text-xs">
				{value === undefined ? '—' : JSON.stringify(value, null, 2)}
			</pre>
		</div>
	)
}
