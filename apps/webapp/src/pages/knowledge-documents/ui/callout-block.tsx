import { CircleCheck, Info, OctagonAlert, TriangleAlert } from 'lucide-react'
import type { ComponentType } from 'react'
import { cn } from '#/lib/utils'
import type { CalloutTone } from '#/pages/knowledge-documents/lib/document-api-types'

export const CALLOUT_TONES: readonly {
	tone: CalloutTone
	label: string
	icon: ComponentType<{ className?: string }>
	className: string
}[] = [
	{
		tone: 'info',
		label: 'Information',
		icon: Info,
		className: 'border-state-progress/40 bg-state-progress/10',
	},
	{
		tone: 'success',
		label: 'Succès',
		icon: CircleCheck,
		className: 'border-success/40 bg-success-soft',
	},
	{
		tone: 'warning',
		label: 'Attention',
		icon: TriangleAlert,
		className: 'border-warning/40 bg-warning-soft',
	},
	{
		tone: 'danger',
		label: 'Danger',
		icon: OctagonAlert,
		className: 'border-destructive/40 bg-destructive-soft',
	},
]

interface CalloutBlockProps {
	tone: CalloutTone
	editable: boolean
	contentRef: (node: HTMLElement | null) => void
	onToneChange: (tone: CalloutTone) => void
}

export function CalloutBlock({
	tone,
	editable,
	contentRef,
	onToneChange,
}: CalloutBlockProps) {
	const current =
		CALLOUT_TONES.find((entry) => entry.tone === tone) ?? CALLOUT_TONES[0]
	if (!current) return null
	const Icon = current.icon
	return (
		<div
			data-callout-tone={current.tone}
			className={cn(
				'flex w-full items-start gap-3 rounded-lg border px-4 py-3',
				current.className,
			)}
		>
			<span contentEditable={false} className="mt-0.5 shrink-0">
				<Icon aria-hidden className="size-5" />
				<span className="sr-only">{current.label}</span>
			</span>
			<div ref={contentRef} className="min-w-0 flex-1" />
			{editable ? (
				<span contentEditable={false} className="flex shrink-0 gap-1">
					{CALLOUT_TONES.map((entry) => {
						const ToneIcon = entry.icon
						return (
							<button
								key={entry.tone}
								type="button"
								aria-pressed={entry.tone === current.tone}
								aria-label={entry.label}
								title={entry.label}
								onClick={() => onToneChange(entry.tone)}
								className={cn(
									'rounded p-1 text-muted-foreground outline-none hover:bg-background/60 focus-visible:ring-[3px] focus-visible:ring-ring/50',
									entry.tone === current.tone &&
										'bg-background text-foreground',
								)}
							>
								<ToneIcon aria-hidden className="size-4" />
							</button>
						)
					})}
				</span>
			) : null}
		</div>
	)
}
