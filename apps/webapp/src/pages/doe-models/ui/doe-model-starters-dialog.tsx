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
			<DialogContent className="sm:max-w-xl">
				<DialogHeader>
					<DialogTitle>Créer depuis un modèle de départ</DialogTitle>
					<DialogDescription>
						Le modèle est copié dans votre organisation. Le modifier ne change
						pas le modèle de départ.
					</DialogDescription>
				</DialogHeader>
				{isLoading ? (
					<div className="flex items-center justify-center gap-2 p-6 text-sm text-muted-foreground">
						<Loader2 className="size-4 animate-spin" aria-hidden />
						Chargement…
					</div>
				) : (
					<ul className="grid gap-3">
						{starters.map((starter) => (
							<li
								key={starter.starter}
								className="grid gap-3 rounded-lg border bg-card p-4"
							>
								<div className="grid gap-1">
									<p className="font-medium">{starter.name}</p>
									{starter.description ? (
										<p className="text-[13px] text-muted-foreground">
											{starter.description}
										</p>
									) : null}
									<p className="text-[12px] text-muted-foreground">
										{starter.sections.length}{' '}
										{starter.sections.length > 1 ? 'thèmes' : 'thème'} :{' '}
										{starter.sections
											.map((section) => section.title)
											.join(', ')}
									</p>
								</div>
								<div className="flex sm:justify-end">
									<Button
										type="button"
										size="sm"
										className="max-sm:w-full"
										disabled={isPending}
										onClick={() => onCreate(starter.starter)}
									>
										{isPending ? <Loader2 className="animate-spin" /> : null}
										Créer depuis {starter.name}
									</Button>
								</div>
							</li>
						))}
					</ul>
				)}
				{error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				) : null}
			</DialogContent>
		</Dialog>
	)
}
