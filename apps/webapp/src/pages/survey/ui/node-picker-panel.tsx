import { X } from 'lucide-react'
import type { CSSProperties } from 'react'
import { Button } from '#/components/ui/button'
import { nodeChoices } from '#/pages/survey/lib/question-kinds'
import type { NodeChoice } from '#/pages/survey/types'
import '#/pages/automation/ui/automation.css'

export function NodePickerPanel({
	attachedTo,
	onPick,
	onClose,
}: {
	attachedTo: string | null
	onPick: (choice: NodeChoice) => void
	onClose: () => void
}) {
	const choices = nodeChoices()
	const questions = choices.filter(
		(entry) => entry.choice !== 'CONDITION' && entry.choice !== 'END',
	)
	const flow = choices.filter(
		(entry) => entry.choice === 'CONDITION' || entry.choice === 'END',
	)
	const card = (entry: (typeof choices)[number]) => {
		const Icon = entry.icon
		return (
			<li key={entry.choice}>
				<button
					type="button"
					aria-label={entry.title}
					aria-describedby={`node-choice-${entry.choice}`}
					onClick={() => onPick(entry.choice)}
					className="node-tone flex h-full w-full items-start gap-3 rounded-lg border bg-card p-3 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
					style={{ '--node-hue': entry.hue } as CSSProperties}
				>
					<span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-(--tone-bg) text-(--tone-fg)">
						<Icon className="size-4" aria-hidden />
					</span>
					<span className="min-w-0">
						<span className="block font-medium text-sm">{entry.title}</span>
						<span
							id={`node-choice-${entry.choice}`}
							className="block text-muted-foreground text-xs"
						>
							{entry.description}
						</span>
					</span>
				</button>
			</li>
		)
	}
	return (
		<section
			aria-label="Ajouter un nœud"
			className="absolute top-3 right-3 bottom-3 z-20 flex w-[380px] max-w-[calc(100%-1.5rem)] flex-col overflow-hidden rounded-xl border bg-background shadow-lg"
		>
			<header className="flex items-start justify-between gap-2 border-b px-4 py-3">
				<div>
					<h2 className="font-medium">Ajouter un nœud</h2>
					<p className="text-muted-foreground text-xs">
						{attachedTo
							? `Il sera relié à la sortie libre de « ${attachedTo} ».`
							: 'Reliez-le ensuite en tirant une liaison depuis une sortie.'}
					</p>
				</div>
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label="Fermer le sélecteur"
					onClick={onClose}
				>
					<X aria-hidden />
				</Button>
			</header>
			<div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
				<div>
					<h3 className="mb-2 font-medium text-muted-foreground text-xs uppercase">
						Questions
					</h3>
					<ul className="grid gap-2">{questions.map(card)}</ul>
				</div>
				<div>
					<h3 className="mb-2 font-medium text-muted-foreground text-xs uppercase">
						Parcours
					</h3>
					<ul className="grid gap-2">{flow.map(card)}</ul>
				</div>
			</div>
		</section>
	)
}
