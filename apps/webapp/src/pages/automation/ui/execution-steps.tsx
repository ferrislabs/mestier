import { ChevronRight, type LucideIcon, Repeat } from 'lucide-react'
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

export interface ExecutionLoopEntry {
	kind: 'loop'
	id: string
	label: string
	iterations: { index: number; path: string; children: ExecutionStepItem[] }[]
}

export type ExecutionStepItem =
	| { kind: 'step'; step: ExecutionStepEntry }
	| ExecutionLoopEntry

function leaves(items: readonly ExecutionStepItem[]): ExecutionStepEntry[] {
	return items.flatMap((item) =>
		item.kind === 'step'
			? [item.step]
			: item.iterations.flatMap((iteration) => leaves(iteration.children)),
	)
}

const ORDER_TONES: Partial<Record<ExecutionStatus, string>> = {
	success:
		'border-(--auto-success) bg-(--auto-success)/12 text-(color:--auto-success-text)',
	error: 'border-state-alert bg-state-alert/12 text-state-alert',
	running: 'border-state-today bg-state-today/12 text-state-today',
	paused: 'border-dashed border-muted-foreground/60 text-muted-foreground',
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

function StepList({
	items,
	order,
	onOpen,
}: {
	items: readonly ExecutionStepItem[]
	order: ReadonlyMap<string, number>
	onOpen: (nodeId: string) => void
}) {
	return (
		<ol className="grid gap-0.5">
			{items.map((item) =>
				item.kind === 'step' ? (
					<li key={item.step.id} className="min-w-0">
						<StepButton
							step={item.step}
							onOpen={() => onOpen(item.step.nodeId)}
							marker={
								<span
									className={cn(
										'flex size-5 shrink-0 items-center justify-center rounded-full border font-semibold text-[10px] tabular-nums',
										ORDER_TONES[item.step.status],
									)}
								>
									{order.get(item.step.id)}
								</span>
							}
						/>
					</li>
				) : (
					<li key={item.id} className="min-w-0">
						<details
							open
							className="group rounded-md border border-border bg-card"
						>
							<summary className="flex cursor-pointer list-none items-center gap-2 px-2 py-1.5 outline-none focus-visible:ring-2 focus-visible:ring-ring [&::-webkit-details-marker]:hidden">
								<ChevronRight
									className="size-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-90"
									aria-hidden
								/>
								<Repeat
									className="size-4 shrink-0 text-muted-foreground"
									aria-hidden
								/>
								<span className="min-w-0 flex-1 truncate font-medium text-sm">
									Boucle « {item.label} »
								</span>
								<span className="text-muted-foreground text-xs tabular-nums">
									{item.iterations.length > 1
										? `${item.iterations.length} itérations`
										: '1 itération'}
								</span>
							</summary>
							<div className="grid gap-1 border-border border-t p-1">
								{item.iterations.map((iteration) => (
									<details
										key={iteration.path}
										open
										className="group/iteration rounded-sm"
									>
										<summary className="flex cursor-pointer list-none items-center gap-1.5 rounded-sm px-2 py-1 text-muted-foreground text-xs outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring [&::-webkit-details-marker]:hidden">
											<ChevronRight
												className="size-3.5 shrink-0 transition-transform group-open/iteration:rotate-90"
												aria-hidden
											/>
											Itération {iteration.index + 1}
										</summary>
										<div className="pl-2">
											<StepList
												items={iteration.children}
												order={order}
												onOpen={onOpen}
											/>
										</div>
									</details>
								))}
							</div>
						</details>
					</li>
				),
			)}
		</ol>
	)
}

export function ExecutionSteps({
	steps,
	pending,
	pendingTitle,
	onOpen,
}: {
	steps: readonly ExecutionStepItem[]
	pending: readonly ExecutionStepEntry[]
	pendingTitle: string
	onOpen: (nodeId: string) => void
}) {
	const executed = leaves(steps)
	const order = new Map(executed.map((step, index) => [step.id, index + 1]))
	const reachedNodes = new Set(executed.map((step) => step.nodeId)).size
	return (
		<aside
			aria-label="Déroulé de l’exécution"
			className="flex min-h-0 w-80 shrink-0 flex-col border-border border-l bg-card"
		>
			<header className="flex h-11 shrink-0 items-center justify-between border-border border-b px-4">
				<h3 className="font-semibold text-sm">Déroulé</h3>
				<span className="text-muted-foreground text-xs tabular-nums">
					{reachedNodes} / {reachedNodes + pending.length} nœuds
				</span>
			</header>
			<div className="min-h-0 flex-1 overflow-y-auto p-2">
				<StepList items={steps} order={order} onOpen={onOpen} />
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
