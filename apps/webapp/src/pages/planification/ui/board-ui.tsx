import { AlertCircle, KanbanSquare } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import {
	BoardColumns,
	type BoardColumnsProps,
	type BoardColumnVM,
} from '#/pages/planification/ui/board-columns'

export interface BoardUIProps
	extends Omit<BoardColumnsProps, 'columns' | 'isLoading'> {
	organizationName: string
	columns: BoardColumnVM[]
	isLoading: boolean
	error: string | null
	onRetry: () => void
	/**
	 * What just happened, for screen readers — the keyboard path moves cards
	 * without any visible pointer, so the move has to be said out loud.
	 */
	announcement: string
	/**
	 * The filter bar, mounted by the feature layer. Rendered above the
	 * columns and inside the same card, so it stays on screen when the
	 * filtered board comes back empty — a bar you cannot reach is a board
	 * you cannot get out of.
	 */
	toolbar?: ReactNode
	/**
	 * Why the board is empty, when there is more to say than "nothing to
	 * do" — a filter that matches nothing, or a project that no longer
	 * exists. `null` falls back to the unfiltered wording.
	 */
	emptyReason?: string | null
	/** The task create/edit sheet, mounted by the feature layer. */
	taskSheet?: ReactNode
}

export function BoardUI({
	organizationName,
	columns,
	isLoading,
	error,
	onRetry,
	announcement,
	toolbar,
	emptyReason = null,
	taskSheet,
	...boardProps
}: BoardUIProps) {
	const cardCount = columns.reduce(
		(total, column) => total + column.cards.length,
		0,
	)
	const isEmpty = !isLoading && !error && cardCount === 0

	return (
		<PageShell className="max-w-none overflow-x-hidden">
			<PageHeader
				title="Tableau"
				description="Cinq colonnes, une carte par tâche. Déplacez une carte pour la faire avancer, ou planifiez-la sans quitter l’écran."
				eyebrow={organizationName}
				leading={
					<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft text-primary">
						<KanbanSquare className="size-6" />
					</div>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</SectionCard>
			) : null}

			<SectionCard className="flex min-w-0 flex-col gap-4 overflow-hidden p-4">
				<div className="min-w-0">
					<div className="flex flex-wrap items-center gap-2">
						<h2 className="font-semibold">Tâches</h2>
						<StatusBadge tone="neutral">
							{cardCount} carte{cardCount > 1 ? 's' : ''}
						</StatusBadge>
					</div>
					<p className="mt-1 text-sm text-muted-foreground">
						Sélectionnez une carte puis utilisez les flèches : gauche/droite
						pour changer de colonne, haut/bas pour la réordonner.
					</p>
				</div>

				{toolbar}

				{isEmpty ? (
					<div className="flex flex-col items-center justify-center gap-2 rounded-md border border-dashed bg-muted/25 p-10 text-center">
						<p className="text-sm font-semibold">Aucune tâche</p>
						<p className="text-sm text-muted-foreground">
							{emptyReason ??
								'Le tableau est vide : rien n’est encore à faire, ni en cours, ni terminé.'}
						</p>
					</div>
				) : (
					<BoardColumns
						{...boardProps}
						columns={columns}
						isLoading={isLoading}
					/>
				)}
			</SectionCard>

			<output aria-live="polite" className="sr-only">
				{announcement}
			</output>

			{taskSheet}
		</PageShell>
	)
}
