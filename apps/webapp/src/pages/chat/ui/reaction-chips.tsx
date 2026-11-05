import { cn } from '#/lib/utils'

export interface ReactionChipVM {
	emoji: string
	count: number
	reactedByMe: boolean
}

export function ReactionChips({
	reactions,
	onToggle,
}: {
	reactions: readonly ReactionChipVM[]
	onToggle: (emoji: string, currentlyReacted: boolean) => void
}) {
	if (reactions.length === 0) return null
	return (
		<ul aria-label="Réactions" className="mt-1.5 flex flex-wrap gap-1.5">
			{reactions.map((reaction) => (
				<li key={reaction.emoji}>
					<button
						type="button"
						aria-pressed={reaction.reactedByMe}
						aria-label={`${reaction.reactedByMe ? 'Retirer' : 'Ajouter'} la réaction ${reaction.emoji}`}
						onClick={() => onToggle(reaction.emoji, reaction.reactedByMe)}
						className={cn(
							'inline-flex h-6 items-center gap-1 rounded-full border px-2 text-xs outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring',
							reaction.reactedByMe
								? 'border-highlight bg-highlight/15 text-foreground'
								: 'border-border bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground',
						)}
					>
						<span aria-hidden>{reaction.emoji}</span>
						<span className="tabular font-medium">{reaction.count}</span>
					</button>
				</li>
			))}
		</ul>
	)
}
