import { Handle, Position } from '@xyflow/react'
import { Plus } from 'lucide-react'
import '#/pages/automation/ui/automation.css'

export function FirstQuestionPlaceholder({ onAdd }: { onAdd: () => void }) {
	return (
		<section
			aria-label="Première question à ajouter"
			className="relative size-[100px]"
		>
			<Handle
				type="target"
				position={Position.Left}
				isConnectable={false}
				className="automation-handle opacity-0"
			/>
			<button
				type="button"
				onClick={onAdd}
				aria-label="Ajoutez votre première question"
				className="nodrag nopan flex size-full items-center justify-center rounded-lg border-2 border-muted-foreground/60 border-dashed bg-card/70 text-muted-foreground outline-none transition-colors hover:border-ring hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
			>
				<Plus className="size-8" aria-hidden />
			</button>
			<div className="pointer-events-none absolute top-full left-1/2 mt-3 grid w-[184px] -translate-x-1/2 gap-1 text-center">
				<p className="text-balance font-medium text-[13px] text-foreground leading-tight">
					Ajoutez votre première question
				</p>
				<p className="text-balance text-[11px] text-muted-foreground leading-snug">
					Elle sera reliée au départ du questionnaire.
				</p>
			</div>
		</section>
	)
}
