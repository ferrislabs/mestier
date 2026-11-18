import { CircleCheck, TriangleAlert } from 'lucide-react'
import type { GraphIssue } from '#/pages/survey/lib/graph-validation'

export function GraphProblems({
	issues,
	serverIssue,
	nodeLabel,
	onSelect,
}: {
	issues: readonly GraphIssue[]
	serverIssue: GraphIssue | null
	nodeLabel: (nodeId: string) => string
	onSelect: (nodeId: string) => void
}) {
	const all = serverIssue ? [serverIssue, ...issues] : [...issues]
	return (
		<section aria-label="Problèmes du questionnaire" className="grid gap-3 p-4">
			<h2 className="font-medium">Vérification</h2>
			{all.length === 0 ? (
				<p className="flex items-center gap-2 text-sm text-success">
					<CircleCheck className="size-4" aria-hidden />
					Le parcours est complet : chaque chemin mène à une fin.
				</p>
			) : (
				<>
					<p className="text-muted-foreground text-xs">
						{all.length === 1
							? 'Un problème empêche l’enregistrement et l’ouverture.'
							: `${all.length} problèmes empêchent l’enregistrement et l’ouverture.`}
					</p>
					<ul className="grid gap-1.5">
						{all.map((item) => {
							const nodeId = item.nodeId
							const content = (
								<>
									<TriangleAlert
										className="mt-0.5 size-3.5 shrink-0 text-destructive"
										aria-hidden
									/>
									<span>
										{nodeId ? (
											<span className="block font-medium">
												{nodeLabel(nodeId)}
											</span>
										) : null}
										<span className="block text-muted-foreground">
											{item.message}
										</span>
									</span>
								</>
							)
							return (
								<li key={`${item.code}:${nodeId ?? 'graph'}`}>
									{nodeId ? (
										<button
											type="button"
											className="flex w-full gap-2 rounded-md px-2 py-1.5 text-left text-xs hover:bg-accent"
											onClick={() => onSelect(nodeId)}
										>
											{content}
										</button>
									) : (
										<div className="flex gap-2 px-2 py-1.5 text-xs">
											{content}
										</div>
									)}
								</li>
							)
						})}
					</ul>
				</>
			)}
		</section>
	)
}
