import { Plus } from 'lucide-react'

export function EmptyCanvasPrompt({ onAdd }: { onAdd: () => void }) {
	return (
		<div className="pointer-events-none absolute inset-0 flex items-center justify-center">
			<button
				type="button"
				onClick={onAdd}
				className="group pointer-events-auto flex flex-col items-center gap-2 text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:text-foreground"
			>
				<span className="flex size-[100px] items-center justify-center rounded-lg border-2 border-current border-dashed bg-card/70 group-focus-visible:ring-2 group-focus-visible:ring-ring">
					<Plus className="size-8" aria-hidden />
				</span>
				<span className="font-medium text-sm">Ajouter la première étape</span>
			</button>
		</div>
	)
}
