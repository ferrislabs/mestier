import { Lock } from 'lucide-react'
import { cn } from '#/lib/utils'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'

export interface ExecutionParameterEntry {
	id: string
	label: string
	value: string
	expression: boolean
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
}: {
	nodeLabel: string
	version: string
	entries: readonly ExecutionParameterEntry[]
}) {
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
				{entries.length === 0 ? (
					<p className="text-muted-foreground text-sm">
						Ce nœud n’a pas de paramètre.
					</p>
				) : (
					entries.map((entry) => (
						<div key={entry.id} className="grid gap-2">
							<div className="flex items-center justify-between gap-2">
								<p className="font-medium text-sm">{entry.label}</p>
								{entry.expression && (
									<span className="rounded-sm bg-accent px-1.5 py-0.5 font-mono text-[10px] text-accent-foreground">
										Expression
									</span>
								)}
							</div>
							<ValueBox value={entry.value} mono={entry.expression} />
						</div>
					))
				)}
			</div>
		</NdvPanel>
	)
}
