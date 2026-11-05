export interface TypingIndicatorUIProps {
	typingCount: number
}

export function TypingIndicatorUI({ typingCount }: TypingIndicatorUIProps) {
	if (typingCount === 0) return null

	return (
		<div
			className="flex shrink-0 items-center gap-1.5 px-4 py-1.5 text-muted-foreground text-xs italic"
			aria-live="polite"
		>
			<span className="flex gap-0.5">
				<span className="size-1 animate-bounce rounded-full bg-muted-foreground [animation-delay:-0.3s]" />
				<span className="size-1 animate-bounce rounded-full bg-muted-foreground [animation-delay:-0.15s]" />
				<span className="size-1 animate-bounce rounded-full bg-muted-foreground" />
			</span>
			{typingCount === 1
				? 'Quelqu’un écrit…'
				: `${typingCount} personnes écrivent…`}
		</div>
	)
}
