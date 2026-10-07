import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { DoeModelStarter, StarterModel } from '#/hooks/use-doe-models'

interface DoeModelStartersDialogProps {
	open: boolean
	starters: DoeModelStarter[]
	isLoading: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onCreate: (starter: StarterModel) => void
}

export function DoeModelStartersDialog({
	open,
	starters,
	isLoading,
	isPending,
	error,
	onOpenChange,
	onCreate,
}: DoeModelStartersDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-w-lg">
				<DialogHeader>
					<DialogTitle>Créer depuis un modèle de départ</DialogTitle>
					<DialogDescription>
						Le modèle est copié dans votre organisation. Le modifier ne change
						pas le modèle de départ.
					</DialogDescription>
				</DialogHeader>
				{isLoading ? (
					<div className="flex items-center justify-center gap-2 p-6 text-sm text-muted-foreground">
						<Loader2 className="size-4 animate-spin" />
						Chargement…
					</div>
				) : (
					<ul className="space-y-3">
						{starters.map((starter) => (
							<li
								key={starter.starter}
								className="flex items-start justify-between gap-4 rounded-md border p-4"
							>
								<div className="min-w-0">
									<p className="font-medium">{starter.name}</p>
									{starter.description ? (
										<p className="mt-1 text-sm text-muted-foreground">
											{starter.description}
										</p>
									) : null}
									<p className="mt-1 text-xs text-muted-foreground">
										{starter.sections.length} thèmes :{' '}
										{starter.sections
											.map((section) => section.title)
											.join(', ')}
									</p>
								</div>
								<Button
									type="button"
									disabled={isPending}
									onClick={() => onCreate(starter.starter)}
								>
									{isPending ? <Loader2 className="animate-spin" /> : null}
									Créer depuis {starter.name}
								</Button>
							</li>
						))}
					</ul>
				)}
				{error ? <p className="text-sm text-destructive">{error}</p> : null}
			</DialogContent>
		</Dialog>
	)
}
