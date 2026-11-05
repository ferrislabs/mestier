export interface PresenceSummaryUIProps {
	onlineCount: number
}

export function PresenceSummaryUI({ onlineCount }: PresenceSummaryUIProps) {
	if (onlineCount === 0) return null

	return (
		<div className="flex items-center gap-1.5 px-4 pb-2 text-muted-foreground text-xs">
			<span
				aria-hidden
				className="inline-block size-2 shrink-0 rounded-full bg-state-progress"
			/>
			<span>
				{onlineCount}{' '}
				{onlineCount === 1 ? 'personne en ligne' : 'personnes en ligne'}
			</span>
		</div>
	)
}
