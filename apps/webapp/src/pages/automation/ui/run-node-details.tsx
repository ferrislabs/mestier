import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'
import { CirclePause, LoaderCircle, RotateCcw } from 'lucide-react'
import { useState } from 'react'
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
	formatItemCount,
	formatMilliseconds,
	formatResumeNote,
} from '#/pages/automation/lib/format'
import { toItems } from '#/pages/automation/lib/items'
import type { RunUpstreamEntry } from '#/pages/automation/lib/run-upstream'
import type { RunNodeView } from '#/pages/automation/lib/run-view'
import { DataViewer } from '#/pages/automation/ui/data-viewer'
import {
	type ExecutionParameterEntry,
	ExecutionParametersPanel,
} from '#/pages/automation/ui/execution-parameters-panel'
import { ExecutionStatusLabel } from '#/pages/automation/ui/execution-status'
import { ErrorBox } from '#/pages/automation/ui/ndv-error-box'
import type { NdvNavItem } from '#/pages/automation/ui/ndv-node-nav'
import { NdvEmptyState, NdvPanel } from '#/pages/automation/ui/ndv-panel'
import { NodeDetailsDialog } from '#/pages/automation/ui/node-details-dialog'
import { VariablesPanel } from '#/pages/automation/ui/variables-panel'

export interface RunNodeReplay {
	available: boolean
	isReplaying: boolean
	error: string | null
	onReplay: () => void
}

export interface RunNodeDetailsProps {
	runNumber: number
	versionLabel: string
	node: RunNodeView
	typeLabel: string
	icon: LucideIcon
	hue: number
	parameters: readonly ExecutionParameterEntry[]
	itemIndex: number
	itemCount: number
	onItemIndexChange: (index: number) => void
	triggerPayload: unknown
	triggerLabel: string
	sources: readonly string[]
	notice: string | null
	upstream: readonly RunUpstreamEntry[]
	notRunLabel: (entryId: string) => string
	previous: readonly NdvNavItem[]
	next: readonly NdvNavItem[]
	replay: RunNodeReplay
	credentialsHref: string
	onNavigate: (nodeId: string) => void
	onClose: () => void
}

function statusDetail(node: RunNodeView): string | undefined {
	const position = node.order === null ? null : `étape ${node.order}`
	if (node.durationMs === null) return position ?? undefined
	return [position, formatMilliseconds(node.durationMs)]
		.filter(Boolean)
		.join(' · ')
}

export function RunNodeDetails({
	runNumber,
	versionLabel,
	node,
	typeLabel,
	icon,
	hue,
	parameters,
	itemIndex,
	itemCount,
	onItemIndexChange,
	triggerPayload,
	triggerLabel,
	sources,
	notice,
	upstream,
	notRunLabel,
	previous,
	next,
	replay,
	credentialsHref,
	onNavigate,
	onClose,
}: RunNodeDetailsProps) {
	const [confirming, setConfirming] = useState(false)
	const step = node.step
	const outputItems = toItems(
		node.kind === 'trigger' ? triggerPayload : step?.output,
	)

	const input =
		node.kind === 'trigger' ? (
			<NdvEmptyState
				title="Point de départ"
				text={`Ce déclencheur a démarré l’exécution (${triggerLabel}). Les données qu’il a émises sont dans la colonne « Après l’exécution ».`}
			/>
		) : (
			<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
				{notice ? (
					<p className="rounded-md border border-border border-dashed bg-card px-3 py-2 text-sm">
						{notice}
					</p>
				) : (
					<p className="text-muted-foreground text-sm">
						Reçu de{' '}
						<span className="font-medium text-foreground">
							{sources.join(', ')}
						</span>
					</p>
				)}
				<VariablesPanel
					upstream={upstream}
					mode="execution"
					intro="Résultats des nœuds précédents pendant cette exécution. Un clic copie l’expression d’une clé."
					notRunLabel={(entry) => notRunLabel(entry.id)}
				/>
			</div>
		)

	const output = (() => {
		switch (node.status) {
			case 'success':
				return <DataViewer key={node.id} items={outputItems} />
			case 'error':
				return (
					<ErrorBox
						message={step?.error ?? 'Erreur inconnue'}
						hint="Le nœud a reçu les données de la colonne de gauche, puis a échoué : aucune donnée n’a été transmise aux nœuds suivants."
						action={
							<Button asChild variant="outline" size="sm">
								<Link to={credentialsHref}>Ouvrir les identifications</Link>
							</Button>
						}
					/>
				)
			case 'paused': {
				const resumeNote = formatResumeNote(step?.resume_at)
				return (
					<div className="flex flex-col items-center px-4 py-12 text-center">
						<CirclePause
							className="mb-3 size-8 text-muted-foreground"
							aria-hidden
						/>
						<p className="font-medium text-base">En pause</p>
						<p className="mt-1 max-w-xs text-muted-foreground text-sm">
							Le workflow reprendra à la fin de la pause, puis transmettra les
							éléments reçus.
						</p>
						{resumeNote && (
							<p className="mt-2 font-medium text-sm tabular-nums">
								{resumeNote}
							</p>
						)}
					</div>
				)
			}
			case 'running':
				return (
					<div className="flex flex-col items-center px-4 py-12 text-center">
						<LoaderCircle
							className="mb-3 size-8 animate-spin text-state-today"
							aria-hidden
						/>
						<p className="font-medium text-base">Exécution en cours</p>
						<p className="mt-1 max-w-xs text-muted-foreground text-sm">
							Le résultat s’affichera ici dès que le nœud aura terminé.
						</p>
					</div>
				)
			default:
				return (
					<NdvEmptyState
						title="Aucune donnée produite"
						text={
							node.status === 'waiting'
								? 'Ce nœud n’a pas encore été exécuté.'
								: 'Ce nœud n’a pas été exécuté pendant cette exécution.'
						}
					/>
				)
		}
	})()

	return (
		<>
			<NodeDetailsDialog
				open
				onOpenChange={(open) => {
					if (!open) onClose()
				}}
				title={node.label}
				description={`${typeLabel} · ${node.subtitle} · Exécution #${runNumber}`}
				icon={icon}
				hue={hue}
				status={
					<ExecutionStatusLabel
						status={node.status}
						detail={statusDetail(node)}
					/>
				}
				input={
					<NdvPanel
						title="Avant l’exécution"
						caption={
							node.kind === 'trigger' || node.order === null || !step
								? 'Entrée'
								: `Entrée · ${formatItemCount(toItems(step.input).length)}`
						}
						tone="side"
					>
						{input}
					</NdvPanel>
				}
				parameters={
					<ExecutionParametersPanel
						nodeLabel={node.label}
						version={versionLabel}
						entries={parameters}
						itemIndex={Math.min(itemIndex, Math.max(0, itemCount - 1))}
						itemCount={itemCount}
						onItemIndexChange={onItemIndexChange}
					/>
				}
				output={
					<NdvPanel
						title="Après l’exécution"
						caption={
							node.status === 'success'
								? `Sortie · ${formatItemCount(outputItems.length)}`
								: 'Sortie'
						}
						tone="side"
					>
						<div className="grid gap-4">
							{output}
							{replay.error ? (
								<p role="alert" className="text-sm text-state-alert">
									{replay.error}
								</p>
							) : null}
							{replay.available && node.kind === 'connector' ? (
								<RequirePermission permission="MANAGE_AUTOMATION">
									<div>
										<Button
											variant="outline"
											size="sm"
											disabled={replay.isReplaying}
											onClick={() => setConfirming(true)}
										>
											<RotateCcw className="size-3.5" />
											Relancer depuis ici
										</Button>
									</div>
								</RequirePermission>
							) : null}
						</div>
					</NdvPanel>
				}
				previous={previous}
				next={next}
				onNavigate={onNavigate}
			/>
			<AlertDialog open={confirming} onOpenChange={setConfirming}>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>
							Relancer depuis « {node.label} » ?
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
								setConfirming(false)
								replay.onReplay()
							}}
						>
							Relancer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</>
	)
}
