import { RotateCcw, TriangleAlert, X } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from '#/components/ui/dialog'

export interface PreviewProblem {
	key: string
	label: string | null
	message: string
}

export function SurveyPreviewDialog({
	open,
	problems,
	children,
	onOpenChange,
	onRestart,
}: {
	open: boolean
	problems: readonly PreviewProblem[]
	children: ReactNode
	onOpenChange: (open: boolean) => void
	onRestart: () => void
}) {
	const blocked = problems.length > 0
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				className="top-0 left-0 flex h-dvh w-screen max-w-none translate-x-0 translate-y-0 flex-col gap-0 rounded-none border-0 p-0 sm:max-w-none"
			>
				<div className="flex shrink-0 flex-wrap items-center gap-2 border-b bg-card px-3 py-2 sm:px-4">
					<div className="min-w-0 flex-1">
						<DialogTitle className="text-base">Aperçu</DialogTitle>
						<DialogDescription className="text-xs">
							Le questionnaire tel que le répondant le verra, avec vos
							modifications non enregistrées. Rien n’est envoyé.
						</DialogDescription>
					</div>
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
					<Button
						type="button"
						variant="ghost"
						size="icon"
						className="size-11 sm:size-8"
						aria-label="Fermer l’aperçu"
						onClick={() => onOpenChange(false)}
					>
						<X aria-hidden />
					</Button>
				</div>
				<div className="min-h-0 flex-1 overflow-y-auto bg-muted/40 p-3 sm:p-6">
					{blocked ? (
						<section
							aria-label="Problèmes à corriger"
							className="mx-auto grid max-w-xl gap-3 rounded-xl border bg-card p-5"
						>
							<h2 className="flex items-center gap-2 font-medium">
								<TriangleAlert
									className="size-4 text-destructive"
									aria-hidden
								/>
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
						<div className="mx-auto h-full w-full overflow-y-auto overflow-x-hidden rounded-lg border bg-background">
							{children}
						</div>
					)}
				</div>
			</DialogContent>
		</Dialog>
	)
}
