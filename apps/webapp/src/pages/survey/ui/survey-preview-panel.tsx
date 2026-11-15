import { RotateCcw, TriangleAlert } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'

export interface PreviewProblem {
	key: string
	label: string | null
	message: string
}

export function SurveyPreviewPanel({
	problems,
	children,
	onRestart,
}: {
	problems: readonly PreviewProblem[]
	children: ReactNode
	onRestart: () => void
}) {
	const blocked = problems.length > 0
	return (
		<section aria-label="Aperçu" className="flex min-h-full flex-col">
			<div className="flex shrink-0 flex-wrap items-center gap-2 border-b bg-card px-3 py-2 sm:px-4">
				<p className="min-w-0 flex-1 text-muted-foreground text-xs">
					Le questionnaire tel que le répondant le verra, avec vos modifications
					non enregistrées. Rien n’est envoyé.
				</p>
				<Button
					type="button"
					variant="outline"
					size="sm"
					className="min-h-11 sm:min-h-8"
					disabled={blocked}
					onClick={onRestart}
				>
					<RotateCcw aria-hidden />
					Recommencer
				</Button>
			</div>
			{blocked ? (
				<section
					aria-label="Problèmes à corriger"
					className="mx-auto mt-6 grid w-full max-w-xl gap-3 rounded-xl border bg-card p-5"
				>
					<h2 className="flex items-center gap-2 font-medium">
						<TriangleAlert className="size-4 text-destructive" aria-hidden />
						Corrigez ces problèmes pour voir l’aperçu
					</h2>
					<ul className="grid gap-2 text-sm">
						{problems.map((problem) => (
							<li key={problem.key}>
								{problem.label ? (
									<span className="block font-medium">{problem.label}</span>
								) : null}
								<span className="block text-muted-foreground">
									{problem.message}
								</span>
							</li>
						))}
					</ul>
				</section>
			) : (
				<div className="flex flex-1 flex-col bg-background">{children}</div>
			)}
		</section>
	)
}
