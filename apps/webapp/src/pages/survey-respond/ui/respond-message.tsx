import type { LucideIcon } from 'lucide-react'
import type { ReactNode, RefObject } from 'react'
import { cn } from '#/lib/utils'

export type MessageTone = 'success' | 'neutral' | 'warning'

const TONES: Record<MessageTone, string> = {
	success: 'bg-success-soft text-success',
	neutral: 'bg-muted text-muted-foreground',
	warning: 'bg-warning-soft text-warning',
}

export function RespondMessage({
	icon: Icon,
	tone,
	title,
	message,
	action,
	headingRef,
}: {
	icon: LucideIcon
	tone: MessageTone
	title: string
	message: string
	action?: ReactNode
	headingRef?: RefObject<HTMLHeadingElement | null>
}) {
	if (tone === 'success') {
		return (
			<section className="mx-auto my-6 flex w-full max-w-md flex-1 flex-col items-center gap-5 self-start rounded-2xl border bg-card px-6 py-10 text-center shadow-xs motion-safe:animate-in motion-safe:fade-in motion-safe:zoom-in-95 motion-safe:duration-300">
				<span className="flex size-16 items-center justify-center rounded-full bg-success-soft text-success ring-8 ring-success-soft/50">
					<Icon className="size-8" aria-hidden />
				</span>
				<div className="flex flex-col gap-2">
					<h2
						ref={headingRef}
						tabIndex={-1}
						className="text-xl font-semibold tracking-tight outline-none"
					>
						{title}
					</h2>
					<p className="text-sm text-muted-foreground whitespace-pre-line break-words">
						{message}
					</p>
				</div>
				{action}
			</section>
		)
	}
	return (
		<section className="flex flex-1 flex-col items-start gap-5 py-8 motion-safe:animate-in motion-safe:fade-in motion-safe:duration-300">
			<span
				className={cn(
					'flex size-14 items-center justify-center rounded-2xl',
					TONES[tone],
				)}
			>
				<Icon className="size-7" aria-hidden />
			</span>
			<div className="flex flex-col gap-2">
				<h2
					ref={headingRef}
					tabIndex={-1}
					className="text-2xl font-semibold tracking-tight outline-none"
				>
					{title}
				</h2>
				<p className="text-base text-muted-foreground whitespace-pre-line break-words">
					{message}
				</p>
			</div>
			{action}
		</section>
	)
}
