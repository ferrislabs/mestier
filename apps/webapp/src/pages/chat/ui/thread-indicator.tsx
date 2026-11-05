import { ChevronRight } from 'lucide-react'

export function ThreadIndicator({ onOpen }: { onOpen: () => void }) {
	return (
		<button
			type="button"
			onClick={onOpen}
			className="group/thread -ml-1.5 mt-1.5 flex max-w-full items-center gap-2 rounded-md border border-transparent px-1.5 py-1 text-left text-xs outline-none transition-colors hover:border-border hover:bg-card focus-visible:ring-2 focus-visible:ring-ring"
		>
			<span className="font-semibold text-accent-foreground">
				Fil de discussion
			</span>
			<ChevronRight
				aria-hidden
				className="size-3.5 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover/thread:opacity-100 group-focus-visible/thread:opacity-100"
			/>
		</button>
	)
}
