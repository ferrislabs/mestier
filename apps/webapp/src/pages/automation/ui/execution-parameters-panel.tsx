import { ChevronLeft, ChevronRight, Lock } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'

export type ExecutionParameterEntry = {
	id: string
	label: string
	value: string
	resolved: string | null
}

function ValueBox({ value, mono }: { value: string; mono?: boolean }) {
	return (
		<p
			className={cn(
				'whitespace-pre-wrap break-words rounded-md border border-border bg-muted/40 px-3 py-2 text-sm',
				mono && 'font-mono text-xs',
				value === '' && 'text-muted-foreground italic',
			)}
		>
			{value === '' ? 'Vide' : value}
		</p>
	)
}

export function ExecutionParametersPanel({
	nodeLabel,
	version,
	entries,
	itemIndex,
	itemCount,
	onItemIndexChange,
}: {
	nodeLabel: string
	version: string
	entries: readonly ExecutionParameterEntry[]
	itemIndex: number
	itemCount: number
	onItemIndexChange: (index: number) => void
}) {
	const hasExpressions = entries.some((entry) => entry.resolved !== null)
	return (
		<NdvPanel
			title="Paramètres"
			caption="Au moment de l’exécution"
			tone="center"
		>
			<div className="grid gap-4">
				<p className="flex items-start gap-2 rounded-md bg-muted px-3 py-2 text-muted-foreground text-xs">
					<Lock className="mt-px size-3.5 shrink-0" aria-hidden />
					<span>
						Lecture seule : ces valeurs sont celles de la{' '}
						<span className="font-medium text-foreground">
							{version.charAt(0).toLowerCase()}
							{version.slice(1)}
						</span>
						, utilisée par cette exécution.
					</span>
				</p>
				<div className="grid gap-2">
					<p className="font-medium text-sm">Nom du nœud</p>
					<ValueBox value={nodeLabel} />
				</div>
				{hasExpressions && itemCount > 1 && (
					<div className="flex items-center justify-between gap-2 rounded-md border border-border px-3 py-1.5">
						<span className="text-muted-foreground text-xs">
							Expressions résolues pour l’élément{' '}
							<span className="font-medium text-foreground tabular-nums">
								{itemIndex + 1} sur {itemCount}
							</span>
						</span>
						<span className="flex gap-1">
							<Button
								variant="ghost"
								size="icon-sm"
								aria-label="Élément précédent"
								disabled={itemIndex === 0}
								onClick={() => onItemIndexChange(itemIndex - 1)}
							>
								<ChevronLeft />
							</Button>
							<Button
								variant="ghost"
								size="icon-sm"
								aria-label="Élément suivant"
								disabled={itemIndex >= itemCount - 1}
								onClick={() => onItemIndexChange(itemIndex + 1)}
							>
								<ChevronRight />
							</Button>
						</span>
					</div>
				)}
				{entries.length === 0 ? (
					<p className="text-muted-foreground text-sm">
						Ce nœud n’a pas de paramètre.
					</p>
				) : (
					entries.map((entry) => (
						<div key={entry.id} className="grid gap-2">
							<div className="flex items-center justify-between gap-2">
								<p className="font-medium text-sm">{entry.label}</p>
								{entry.resolved !== null && (
									<span className="rounded-sm bg-accent px-1.5 py-0.5 font-mono text-[10px] text-accent-foreground">
										Expression
									</span>
								)}
							</div>
							{entry.resolved === null ? (
								<ValueBox value={entry.value} />
							) : (
								<div className="grid gap-1.5">
									<ValueBox value={entry.value} mono />
									<p className="text-muted-foreground text-xs">
										Valeur utilisée
									</p>
									<ValueBox value={entry.resolved} />
								</div>
							)}
						</div>
					))
				)}
			</div>
		</NdvPanel>
	)
}
