import { cn } from '#/lib/utils'
import { DELETED_LABEL } from '#/pages/knowledge-documents/ui/deleted-element'

interface MentionChipProps {
	label: string
	missing: boolean
	href: string
	onNavigate: () => void
}

export function MentionChip({
	label,
	missing,
	href,
	onNavigate,
}: MentionChipProps) {
	if (missing) {
		return (
			<span className="rounded bg-muted px-1.5 py-0.5 text-muted-foreground italic">
				{DELETED_LABEL}
			</span>
		)
	}
	return (
		<a
			href={href}
			onClick={(event) => {
				if (event.metaKey || event.ctrlKey || event.shiftKey) return
				event.preventDefault()
				onNavigate()
			}}
			className={cn(
				'rounded bg-primary/10 px-1.5 py-0.5 font-medium text-primary no-underline hover:bg-primary/15',
			)}
		>
			@{label}
		</a>
	)
}
