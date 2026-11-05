import { SearchX, Workflow } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'

function EmptyCard({
	icon,
	title,
	hint,
	children,
}: {
	icon: ReactNode
	title: string
	hint: string
	children?: ReactNode
}) {
	return (
		<div className="flex flex-col items-center rounded-lg border border-border border-dashed bg-card px-6 py-14 text-center">
			{icon}
			<p className="font-medium text-base">{title}</p>
			<p className="mt-1 max-w-md text-muted-foreground text-sm">{hint}</p>
			{children}
		</div>
	)
}

export function WorkflowListEmpty({
	query,
	tag,
	onReset,
}: {
	query: string
	tag: string | null
	onReset: () => void
}) {
	const trimmedQuery = query.trim()
	const title = trimmedQuery
		? `Aucun workflow ne correspond à « ${trimmedQuery} »`
		: 'Aucun workflow à afficher'
	const hint = tag
		? `Aucun workflow n’a le tag « ${tag} » avec cette recherche. Essayez un autre terme ou retirez le filtre.`
		: 'Vérifiez l’orthographe ou essayez un autre terme.'
	return (
		<EmptyCard
			icon={
				<SearchX className="mb-3 size-8 text-muted-foreground" aria-hidden />
			}
			title={title}
			hint={hint}
		>
			<Button variant="outline" size="sm" className="mt-4" onClick={onReset}>
				Réinitialiser les filtres
			</Button>
		</EmptyCard>
	)
}

export function WorkflowListNoneYet() {
	return (
		<EmptyCard
			icon={
				<Workflow className="mb-3 size-8 text-muted-foreground" aria-hidden />
			}
			title="Aucun workflow"
			hint="Un workflow automatise une suite d’actions déclenchées par un événement de l’organisation — créez le premier pour commencer."
		/>
	)
}
