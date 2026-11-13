import { Check } from 'lucide-react'
import { cn } from '#/lib/utils'

export interface StepIndicatorProps {
	steps: readonly string[]
	current: number
	onSelect: (index: number) => void
}

export function StepIndicator({
	steps,
	current,
	onSelect,
}: StepIndicatorProps) {
	return (
		<ol aria-label="Étapes" className="flex items-center gap-2">
			{steps.map((label, index) => {
				const done = index < current
				const active = index === current
				const dot = (
					<span
						aria-hidden
						className={cn(
							'flex size-6 shrink-0 items-center justify-center rounded-full border text-xs font-medium tabular',
							active && 'border-primary bg-primary text-primary-foreground',
							done && 'border-primary text-primary',
							!active && !done && 'text-muted-foreground',
						)}
					>
						{done ? <Check className="size-3.5" /> : index + 1}
					</span>
				)
				const text = (
					<span
						className={cn(
							'truncate text-sm',
							active ? 'font-medium' : 'text-muted-foreground',
						)}
					>
						{label}
					</span>
				)
				return (
					<li
						key={label}
						className="flex min-w-0 flex-1 items-center gap-2 last:flex-none"
					>
						{done ? (
							<button
								type="button"
								className="flex min-w-0 items-center gap-2 rounded-md outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
								onClick={() => onSelect(index)}
							>
								{dot}
								{text}
							</button>
						) : (
							<span
								aria-current={active ? 'step' : undefined}
								className="flex min-w-0 items-center gap-2"
							>
								{dot}
								{text}
							</span>
						)}
						{index < steps.length - 1 ? (
							<span
								aria-hidden
								className={cn(
									'h-px min-w-4 flex-1',
									done ? 'bg-primary' : 'bg-border',
								)}
							/>
						) : null}
					</li>
				)
			})}
		</ol>
	)
}
