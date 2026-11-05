import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'
import type { ExecutionStatus } from '#/pages/automation/lib/execution-status'
import { ExecutionStatusIcon } from '#/pages/automation/ui/execution-status'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

export interface ExecutionStepEntry {
	id: string
	nodeId: string
	label: string
	icon: LucideIcon
	hue: number
	status: ExecutionStatus
	detail: string
}

const ORDER_TONES: Partial<Record<ExecutionStatus, string>> = {
	success:
		'border-(--auto-success) bg-(--auto-success)/12 text-(color:--auto-success-text)',
	error: 'border-state-alert bg-state-alert/12 text-state-alert',
	running: 'border-state-today bg-state-today/12 text-state-today',
}

const DETAIL_TONES: Partial<Record<ExecutionStatus, string>> = {
	error: 'text-state-alert',
	running: 'text-state-today',
}

function StepButton({
	step,
	marker,
	onOpen,
}: {
	step: ExecutionStepEntry
	marker: ReactNode
	onOpen: () => void
}) {
	const muted =
		step.status === 'skipped' ||
		step.status === 'waiting' ||
		step.status === 'cancelled'
	return (
		<button
			type="button"
			onClick={onOpen}
			className="flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
		>
			{marker}
			<KindIconTile
				icon={step.icon}
				hue={step.hue}
				size="sm"
				className={cn(muted && 'opacity-50 grayscale')}
			/>
			<span className="min-w-0 flex-1">
				<span
					className={cn(
						'block truncate font-medium text-sm',
						muted && 'text-muted-foreground',
					)}
				>
					{step.label}
				</span>
				<span
					className={cn(
						'block truncate text-muted-foreground text-xs tabular-nums',
						DETAIL_TONES[step.status],
					)}
				>
					{step.detail}
				</span>
			</span>
		</button>
	)
}

export function ExecutionSteps({
	steps,
	pending,
	pendingTitle,
	onOpen,
}: {
	steps: readonly ExecutionStepEntry[]
	pending: readonly ExecutionStepEntry[]
	pendingTitle: string
	onOpen: (nodeId: string) => void
}) {
	return (
		<aside
			aria-label="Déroulé de l’exécution"
			className="flex min-h-0 w-80 shrink-0 flex-col border-border border-l bg-card"
		>
			<header className="flex h-11 shrink-0 items-center justify-between border-border border-b px-4">
				<h3 className="font-semibold text-sm">Déroulé</h3>
				<span className="text-muted-foreground text-xs tabular-nums">
					{steps.length} / {steps.length + pending.length} nœuds
				</span>
			</header>
			<div className="min-h-0 flex-1 overflow-y-auto p-2">
				<ol className="grid gap-0.5">
					{steps.map((step, index) => (
						<li key={step.id} className="min-w-0">
							<StepButton
								step={step}
								onOpen={() => onOpen(step.nodeId)}
								marker={
									<span
										className={cn(
											'flex size-5 shrink-0 items-center justify-center rounded-full border font-semibold text-[10px] tabular-nums',
											ORDER_TONES[step.status],
										)}
									>
										{index + 1}
									</span>
								}
							/>
						</li>
					))}
				</ol>
				{pending.length > 0 && (
					<section aria-label={pendingTitle} className="mt-3">
						<h4 className="px-2 pb-1 font-medium text-muted-foreground text-xs">
							{pendingTitle}
						</h4>
						<ul className="grid gap-0.5">
							{pending.map((step) => (
								<li key={step.id} className="min-w-0">
									<StepButton
										step={step}
										onOpen={() => onOpen(step.nodeId)}
										marker={
											<ExecutionStatusIcon
												status={step.status}
												className="size-5"
											/>
										}
									/>
								</li>
							))}
						</ul>
					</section>
				)}
			</div>
		</aside>
	)
}
