import { Handle, Position } from '@xyflow/react'
import type { LucideIcon } from 'lucide-react'
import {
	Check,
	LoaderCircle,
	Plus,
	TriangleAlert,
	Unlink,
	X,
	Zap,
} from 'lucide-react'
import { type CSSProperties, Fragment } from 'react'
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from '#/components/ui/tooltip'
import { cn } from '#/lib/utils'
import type { ExecutionStatus } from '#/pages/automation/lib/execution-status'
import '#/pages/automation/ui/automation.css'

export type RunStatus = 'success' | 'error'

export type NodeCardOutput = {
	id: string | null
	label?: string
	open: boolean
}

export type NodeCardExecution = { status: ExecutionStatus; note: string }

const EXECUTION_FRAME: Record<ExecutionStatus, string> = {
	success: 'border-2 border-(--auto-success)',
	error: 'border-2 border-state-alert',
	running:
		'border-2 border-state-today shadow-[0_0_0_4px] shadow-state-today/20',
	paused: 'border-dashed border-muted-foreground/50',
	waiting: 'border-dashed border-muted-foreground/50',
	skipped: 'border-border bg-muted shadow-none',
	cancelled: 'border-border bg-muted shadow-none',
}

const EXECUTION_NOTE: Record<ExecutionStatus, string> = {
	success: 'text-(color:--auto-success-text)',
	error: 'text-state-alert',
	running: 'text-state-today',
	paused: 'text-muted-foreground',
	waiting: 'text-muted-foreground',
	skipped: 'text-muted-foreground',
	cancelled: 'text-muted-foreground',
}

function outputOffset(index: number, count: number): string {
	return `${((index + 1) / (count + 1)) * 100}%`
}

function StatusBadge({ status }: { status: RunStatus }) {
	const isSuccess = status === 'success'
	return (
		<span
			role="img"
			aria-label={isSuccess ? 'Exécuté avec succès' : 'Exécution en échec'}
			className={cn(
				'absolute -top-2 -right-2 flex size-[20px] items-center justify-center rounded-full border-2 border-background',
				isSuccess
					? 'bg-(--auto-success) text-(color:--auto-success-foreground)'
					: 'bg-state-alert text-primary-foreground',
			)}
		>
			{isSuccess ? (
				<Check className="size-3" strokeWidth={3.5} />
			) : (
				<X className="size-3" strokeWidth={3.5} />
			)}
		</span>
	)
}

function RunningBadge() {
	return (
		<span
			role="img"
			aria-label="Exécution en cours"
			className="absolute -top-2 -right-2 flex size-[20px] items-center justify-center rounded-full border-2 border-background bg-state-today text-primary-foreground"
		>
			<LoaderCircle className="size-3 animate-spin" strokeWidth={3} />
		</span>
	)
}

function ExecutionBadge({ status }: { status: ExecutionStatus }) {
	if (status === 'success' || status === 'error')
		return <StatusBadge status={status} />
	if (status === 'running') return <RunningBadge />
	return null
}

function SetupBadge() {
	return (
		<span
			role="img"
			aria-label="À configurer"
			className="absolute -top-2 -right-2 flex size-[20px] items-center justify-center rounded-full border-2 border-background bg-state-today text-primary-foreground"
		>
			<TriangleAlert className="size-3" strokeWidth={2.75} />
		</span>
	)
}

function BindingBadge({ issues }: { issues: readonly string[] }) {
	return (
		<Tooltip>
			<TooltipTrigger asChild>
				<span
					role="img"
					aria-label={`À corriger : ${issues.join(' ')}`}
					className="absolute -top-2 -right-2 flex size-[20px] items-center justify-center rounded-full border-2 border-background bg-state-alert text-primary-foreground"
				>
					<Unlink className="size-3" strokeWidth={2.75} />
				</span>
			</TooltipTrigger>
			<TooltipContent side="top" className="max-w-xs">
				<ul className="grid gap-1">
					{issues.map((issue) => (
						<li key={issue}>{issue}</li>
					))}
				</ul>
			</TooltipContent>
		</Tooltip>
	)
}

export function AutomationNodeCard({
	label,
	subtitle,
	icon: Icon,
	hue,
	trigger,
	status,
	needsSetup,
	disabled,
	bindingIssues,
	selected,
	hasInput,
	outputs,
	execution = null,
	onOpen,
	onAddAfter,
}: {
	label: string
	subtitle: string
	icon: LucideIcon
	hue: number
	trigger: boolean
	status: RunStatus | null
	needsSetup: boolean
	disabled: boolean
	bindingIssues: readonly string[]
	selected: boolean
	hasInput: boolean
	outputs: readonly NodeCardOutput[]
	execution?: NodeCardExecution | null
	onOpen: () => void
	onAddAfter: (outputId: string | null) => void
}) {
	const dimmed =
		execution?.status === 'skipped' || execution?.status === 'cancelled'
	return (
		<div
			className={cn(
				'node-tone relative size-[100px]',
				dimmed && 'opacity-55 grayscale',
			)}
			style={{ '--node-hue': hue } as CSSProperties}
		>
			{hasInput && (
				<Handle
					type="target"
					position={Position.Left}
					className="automation-handle"
				/>
			)}
			<button
				type="button"
				aria-label={`${label}, ${subtitle}${disabled ? ', désactivé' : ''}. Appuyez sur Entrée pour ouvrir les détails`}
				onKeyDown={(event) => {
					if (event.key !== 'Enter') return
					event.preventDefault()
					onOpen()
				}}
				className={cn(
					'flex size-full cursor-grab items-center justify-center border bg-card shadow-sm outline-none transition-shadow focus-visible:ring-2 focus-visible:ring-ring active:cursor-grabbing',
					trigger
						? 'rounded-l-[calc(var(--radius)*3.5)] rounded-r-lg'
						: 'rounded-lg',
					(status === 'error' || (bindingIssues.length > 0 && !needsSetup)) &&
						'border-2 border-state-alert',
					needsSetup && 'border-2 border-state-today border-dashed',
					!execution &&
						status !== 'error' &&
						!needsSetup &&
						bindingIssues.length === 0 &&
						'border-border',
					selected &&
						status !== 'error' &&
						!needsSetup &&
						bindingIssues.length === 0 &&
						'border-ring',
					selected && 'ring-2 ring-ring/40',
					disabled && 'border-dashed bg-muted',
					execution && EXECUTION_FRAME[execution.status],
					execution && 'cursor-pointer active:cursor-pointer',
				)}
			>
				<Icon
					className={cn(
						'size-[42px] text-(color:--tone-fg)',
						disabled && 'opacity-40 grayscale',
						(execution?.status === 'waiting' ||
							execution?.status === 'paused') &&
							'opacity-60',
					)}
					strokeWidth={1.5}
					aria-hidden
				/>
			</button>
			{trigger && (
				<Zap
					className="absolute top-1/2 -left-[22px] size-4 -translate-y-1/2 fill-state-today text-state-today"
					aria-label="Déclencheur"
					role="img"
				/>
			)}
			{execution ? (
				<ExecutionBadge status={execution.status} />
			) : needsSetup ? (
				<SetupBadge />
			) : bindingIssues.length > 0 ? (
				<BindingBadge issues={bindingIssues} />
			) : (
				status && <StatusBadge status={status} />
			)}
			{outputs.map((output, index) => {
				const top = outputOffset(index, outputs.length)
				return (
					<Fragment key={output.id ?? 'default'}>
						<Handle
							id={output.id ?? undefined}
							type="source"
							position={Position.Right}
							className="automation-handle"
							style={{ top }}
						/>
						{output.label && (
							<span
								className="absolute left-full ml-2.5 -translate-y-full rounded-sm bg-background/90 px-1 pb-0.5 font-medium text-[11px] text-muted-foreground leading-none"
								style={{ top }}
							>
								{output.label}
							</span>
						)}
						{output.open && (
							<div
								className="absolute left-full flex -translate-y-1/2 items-center"
								style={{ top }}
							>
								<span className="ml-1 h-0 w-[28px] border-muted-foreground/70 border-t-2 border-dashed" />
								<button
									type="button"
									aria-label={`Ajouter un nœud après « ${label} »${output.label ? ` (${output.label})` : ''}`}
									onClick={() => onAddAfter(output.id)}
									className="nodrag nopan flex size-[24px] items-center justify-center rounded-md border-2 border-muted-foreground/70 border-dashed bg-card text-muted-foreground outline-none transition-colors hover:border-ring hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
								>
									<Plus className="size-3.5" aria-hidden />
								</button>
							</div>
						)}
					</Fragment>
				)
			})}
			<div className="absolute top-full left-1/2 mt-3 w-[184px] -translate-x-1/2 text-center">
				<p className="line-clamp-2 text-balance font-medium text-[13px] text-foreground leading-tight">
					{label}
				</p>
				{disabled && (
					<p className="mt-0.5 font-medium text-[11px] text-muted-foreground">
						Désactivé
					</p>
				)}
				<p className="mt-0.5 line-clamp-1 text-[11px] text-muted-foreground">
					{subtitle}
				</p>
				{execution && execution.note !== '' && (
					<p
						className={cn(
							'mt-1 font-medium text-[11px] tabular-nums',
							EXECUTION_NOTE[execution.status],
						)}
					>
						{execution.note}
					</p>
				)}
			</div>
		</div>
	)
}
