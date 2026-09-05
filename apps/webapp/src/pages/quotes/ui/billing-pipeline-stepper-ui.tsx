import { Check, Circle, Dot, X } from 'lucide-react'
import type * as React from 'react'
import { cn } from '#/lib/utils'
import type { BillingPipelineStep } from '#/pages/quotes/lib/billing-pipeline'

const ICON_BY_STATE: Record<
	BillingPipelineStep['state'],
	React.ComponentType<{ className?: string }>
> = {
	done: Check,
	current: Dot,
	pending: Circle,
	blocked: Circle,
	stopped: X,
}

const MARKER_TONE_BY_STATE: Record<BillingPipelineStep['state'], string> = {
	done: 'border-success bg-success-soft text-success',
	current: 'border-primary bg-brand-soft text-primary',
	pending: 'border-muted-foreground/30 text-muted-foreground/50',
	blocked: 'border-muted-foreground/15 text-muted-foreground/30',
	stopped: 'border-destructive bg-destructive-soft text-destructive',
}

const LABEL_TONE_BY_STATE: Record<BillingPipelineStep['state'], string> = {
	done: 'text-foreground',
	current: 'text-foreground font-medium',
	pending: 'text-muted-foreground',
	blocked: 'text-muted-foreground/50',
	stopped: 'text-destructive',
}

interface BillingPipelineStepperUiProps {
	steps: BillingPipelineStep[]
}

/** Pure horizontal stepper for a quote's billing progress. No hooks, no
 * fetch — every step is already resolved by `computeBillingPipelineSteps`. */
export function BillingPipelineStepperUi({
	steps,
}: BillingPipelineStepperUiProps) {
	return (
		<ol className="flex flex-col gap-4 sm:flex-row sm:items-start sm:gap-0">
			{steps.map((step, index) => {
				const Icon = ICON_BY_STATE[step.state]
				const isLast = index === steps.length - 1
				return (
					<li
						key={step.id}
						className="flex flex-1 items-start gap-3 sm:flex-col sm:items-stretch sm:gap-2"
					>
						<div className="flex items-center sm:w-full">
							<span
								className={cn(
									'flex size-7 shrink-0 items-center justify-center rounded-full border-2',
									MARKER_TONE_BY_STATE[step.state],
								)}
							>
								<Icon className="size-4" />
							</span>
							{!isLast ? (
								<span
									className={cn(
										'hidden h-0.5 flex-1 sm:ml-2 sm:block',
										step.state === 'done'
											? 'bg-success/40'
											: 'bg-muted-foreground/15',
									)}
								/>
							) : null}
						</div>
						<div className="flex flex-col">
							<span className={cn('text-sm', LABEL_TONE_BY_STATE[step.state])}>
								{step.label}
							</span>
							{step.optional && step.state !== 'done' ? (
								<span className="text-xs text-muted-foreground/70">
									Optionnel
								</span>
							) : null}
						</div>
					</li>
				)
			})}
		</ol>
	)
}
