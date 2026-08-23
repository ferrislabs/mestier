export interface TypingIndicatorUIProps {
	typingCount: number
}

/**
 * No names here either, for the same reason as `PresenceSummaryUI` — a
 * `user_id` from the gateway has no REST path to a display name. "Someone is
 * typing" is the honest version of "Alice is typing" this can show.
 */
export function TypingIndicatorUI({ typingCount }: TypingIndicatorUIProps) {
	if (typingCount === 0) return null

	return (
		<div
			className="flex items-center gap-1.5 px-4 py-1.5 text-xs italic text-muted-foreground"
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
