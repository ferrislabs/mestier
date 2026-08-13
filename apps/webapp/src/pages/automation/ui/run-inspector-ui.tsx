import { Link } from '@tanstack/react-router'
import { ArrowLeft, Loader2, RotateCcw } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import {
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface StepRow {
	id: string
	connectorId: string
	status: string
	attempts: number
	error: string | null
	input: unknown
	output: unknown
	startedAt: string | null
	finishedAt: string | null
}

export type StepBlock =
	| { kind: 'step'; step: StepRow }
	| { kind: 'iteration'; path: string; label: string; steps: StepRow[] }

export interface RunSummary {
	id: string
	status: string
	startedAt: string | null
	finishedAt: string | null
	error: string | null
}

export interface RunInspectorUIProps {
	organizationName: string
	organizationSlug: string
	workflowId: string
	isLoading: boolean
	error: string | null
	run: RunSummary | null
	blocks: StepBlock[]
	canReplayRun: boolean
	replayingConnectorId: string | null
	onReplay: (step: StepRow) => void
}

export function RunInspectorUI({
	organizationName,
	organizationSlug,
	workflowId,
	isLoading,
	error,
	run,
	blocks,
	canReplayRun,
	replayingConnectorId,
	onReplay,
}: RunInspectorUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Détail de l’exécution"
				description={run ? `Run ${run.id}` : undefined}
				actions={
					<Link
						to={buildOrgPath(organizationSlug, '/automation/$workflowId/runs')}
						params={{ workflowId }}
						className="inline-flex items-center gap-1.5 text-sm font-medium text-primary hover:underline"
					>
						<ArrowLeft className="size-4" />
						Retour aux runs
					</Link>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement de l’exécution…
				</SectionCard>
			) : run === null ? (
				<SectionCard className="flex min-h-40 items-center justify-center p-8 text-sm text-muted-foreground">
					Exécution introuvable.
				</SectionCard>
			) : (
				<>
					<RunSummaryCard run={run} />
					<StepsSection
						blocks={blocks}
						canReplayRun={canReplayRun}
						replayingConnectorId={replayingConnectorId}
						onReplay={onReplay}
					/>
				</>
			)}
		</PageShell>
	)
}

function RunSummaryCard({ run }: { run: RunSummary }) {
	return (
		<SectionCard>
			<SectionHeader
				title="Résumé"
				actions={
					<StatusBadge tone={RUN_STATUS_TONE[run.status] ?? 'neutral'}>
						{RUN_STATUS_LABEL[run.status] ?? run.status}
					</StatusBadge>
				}
			/>
			<div className="grid grid-cols-1 gap-4 p-5 sm:grid-cols-3">
				<Field label="Démarré">{formatTimestamp(run.startedAt)}</Field>
				<Field label="Terminé">{formatTimestamp(run.finishedAt)}</Field>
				<Field label="Erreur">
					{run.error ? (
						<span className="text-destructive">{run.error}</span>
					) : (
						'—'
					)}
				</Field>
			</div>
		</SectionCard>
	)
}

function Field({
	label,
	children,
}: {
	label: string
	children: React.ReactNode
}) {
	return (
		<div>
			<p className="text-xs font-semibold uppercase text-muted-foreground">
				{label}
			</p>
			<p className="mt-1 text-sm">{children}</p>
		</div>
	)
}

interface StepsSectionProps {
	blocks: StepBlock[]
	canReplayRun: boolean
	replayingConnectorId: string | null
	onReplay: (step: StepRow) => void
}

function StepsSection({
	blocks,
	canReplayRun,
	replayingConnectorId,
	onReplay,
}: StepsSectionProps) {
	const stepCount = blocks.reduce(
		(count, block) => count + (block.kind === 'step' ? 1 : block.steps.length),
		0,
	)

	return (
		<SectionCard>
			<SectionHeader
				title={`Étapes (${stepCount})`}
				description="Dans l’ordre du graphe — les itérations d’une boucle sont regroupées."
			/>
			<div className="flex flex-col gap-3 p-5">
				{blocks.length === 0 ? (
					<p className="text-sm text-muted-foreground">
						Aucune étape enregistrée pour cette exécution.
					</p>
				) : (
					blocks.map((block) =>
						block.kind === 'step' ? (
							<StepCard
								key={block.step.id}
								step={block.step}
								canReplayRun={canReplayRun}
								replayingConnectorId={replayingConnectorId}
								onReplay={onReplay}
							/>
						) : (
							<IterationGroupCard
								key={`${block.path}:${block.steps[0]?.id ?? ''}`}
								label={block.label}
								steps={block.steps}
								canReplayRun={canReplayRun}
								replayingConnectorId={replayingConnectorId}
								onReplay={onReplay}
							/>
						),
					)
				)}
			</div>
		</SectionCard>
	)
}

interface IterationGroupCardProps {
	label: string
	steps: StepRow[]
	canReplayRun: boolean
	replayingConnectorId: string | null
	onReplay: (step: StepRow) => void
}

function IterationGroupCard({
	label,
	steps,
	canReplayRun,
	replayingConnectorId,
	onReplay,
}: IterationGroupCardProps) {
	const summary = summarizeIterationStatus(steps)

	return (
		<details className="rounded-lg border" open>
			<summary className="flex cursor-pointer items-center justify-between gap-3 px-4 py-3 text-sm font-medium">
				<span>{label}</span>
				<span className="flex items-center gap-2 text-xs text-muted-foreground">
					{steps.length} étape{steps.length > 1 ? 's' : ''}
					<StatusBadge tone={summary.tone}>{summary.label}</StatusBadge>
				</span>
			</summary>
			<div className="flex flex-col gap-3 border-t p-4">
				{steps.map((step) => (
					<StepCard
						key={step.id}
						step={step}
						canReplayRun={canReplayRun}
						replayingConnectorId={replayingConnectorId}
						onReplay={onReplay}
					/>
				))}
			</div>
		</details>
	)
}

/** Rolls a group's steps into one badge for its collapsed header — the worst
 * outcome wins, so a single failed iteration step is never hidden behind a
 * "succeeded" summary. */
function summarizeIterationStatus(steps: StepRow[]): {
	tone: 'error' | 'warning' | 'success' | 'neutral'
	label: string
} {
	if (steps.some((step) => step.status === 'failed')) {
		return { tone: 'error', label: RUN_STATUS_LABEL.failed ?? 'Échoué' }
	}
	if (
		steps.some((step) => step.status === 'running' || step.status === 'pending')
	) {
		return { tone: 'warning', label: 'En cours' }
	}
	if (steps.every((step) => step.status === 'succeeded')) {
		return { tone: 'success', label: RUN_STATUS_LABEL.succeeded ?? 'Réussi' }
	}
	return { tone: 'neutral', label: 'Mixte' }
}

interface StepCardProps {
	step: StepRow
	canReplayRun: boolean
	replayingConnectorId: string | null
	onReplay: (step: StepRow) => void
}

function StepCard({
	step,
	canReplayRun,
	replayingConnectorId,
	onReplay,
}: StepCardProps) {
	const isReplaying = replayingConnectorId === step.connectorId

	return (
		<div className="rounded-lg border p-4">
			<div className="flex flex-wrap items-start justify-between gap-3">
				<div className="min-w-0">
					<p className="truncate font-mono text-sm font-medium">
						{step.connectorId}
					</p>
					<div className="mt-1 flex items-center gap-2">
						<StatusBadge tone={RUN_STATUS_TONE[step.status] ?? 'neutral'}>
							{RUN_STATUS_LABEL[step.status] ?? step.status}
						</StatusBadge>
						<span className="text-xs text-muted-foreground">
							{step.attempts} tentative{step.attempts > 1 ? 's' : ''}
						</span>
					</div>
				</div>
				{canReplayRun ? (
					<Button
						variant="outline"
						size="sm"
						disabled={isReplaying}
						onClick={() => onReplay(step)}
					>
						{isReplaying ? <Loader2 className="animate-spin" /> : <RotateCcw />}
						Relancer depuis ici
					</Button>
				) : null}
			</div>

			{step.error ? (
				<p className="mt-3 text-sm text-destructive">{step.error}</p>
			) : null}

			<div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
				<JsonBlock label="Entrée" value={step.input} />
				<JsonBlock label="Sortie" value={step.output} />
			</div>

			<div className="mt-2 flex flex-wrap gap-4 text-xs text-muted-foreground">
				<span>Démarré : {formatTimestamp(step.startedAt)}</span>
				<span>Terminé : {formatTimestamp(step.finishedAt)}</span>
			</div>
		</div>
	)
}

function JsonBlock({ label, value }: { label: string; value: unknown }) {
	return (
		<div>
			<p className="mb-1 text-xs font-semibold uppercase text-muted-foreground">
				{label}
			</p>
			{value === undefined ? (
				<p className="text-xs text-muted-foreground italic">—</p>
			) : (
				<pre className="max-h-48 overflow-auto rounded-md bg-muted/50 p-2 text-xs">
					{JSON.stringify(value, null, 2)}
				</pre>
			)}
		</div>
	)
}

function formatTimestamp(iso: string | null): string {
	if (!iso) return '—'
	return new Date(iso).toLocaleString('fr-FR')
}
