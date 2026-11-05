import { AlertCircle } from 'lucide-react'
import type { ReactNode } from 'react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import {
	BoardColumns,
	type BoardColumnsProps,
	type BoardColumnVM,
} from '#/pages/planification/ui/board-columns'

export interface BoardUIProps
	extends Omit<BoardColumnsProps, 'columns' | 'isLoading'> {
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
	/**
	 * Said when the API returned fewer cards than match the filters — it
	 * clamps every listing to 100 rows and does not signal it. Rendered
	 * beside the columns rather than in place of them: the cards that did
	 * come back are still the ones being worked on. `null` when the board
	 * is showing everything.
	 */
	truncationNotice?: string | null
	/** The task create/edit sheet, mounted by the feature layer. */
	taskSheet?: ReactNode
}

export function BoardUI({
	columns,
	isLoading,
	error,
	onRetry,
	announcement,
	toolbar,
	emptyReason = null,
	truncationNotice = null,
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
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-4 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-4 shrink-0" />
						<p className="text-[13px] font-medium">{error}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</SectionCard>
			) : null}

			<div className="flex min-w-0 flex-col gap-4">
				{toolbar}

				<p className="text-xs text-muted-foreground">
					<span className="tabular font-medium text-foreground">
						{cardCount} carte{cardCount > 1 ? 's' : ''}
					</span>
					{' · '}
					Sélectionnez une carte puis utilisez les flèches : gauche/droite pour
					changer de colonne, haut/bas pour la réordonner.
				</p>

				{truncationNotice ? (
					<output className="block rounded-lg border border-dashed bg-card px-3 py-2 text-[13px] text-muted-foreground">
						{truncationNotice}
					</output>
				) : null}

				{isEmpty ? (
					<EmptyState
						title="Aucune tâche"
						hint={
							emptyReason ??
							'Le tableau est vide : rien n’est encore à faire, ni en cours, ni terminé.'
						}
					/>
				) : (
					<BoardColumns
						{...boardProps}
						columns={columns}
						isLoading={isLoading}
					/>
				)}
			</div>

			<output aria-live="polite" className="sr-only">
				{announcement}
			</output>

			{taskSheet}
		</PageShell>
	)
}
