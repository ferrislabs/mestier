import { AlertCircle, LifeBuoy } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import type { SupportBoardUIProps } from '#/pages/support/types'
import { TicketColumns } from '#/pages/support/ui/ticket-columns'

export function SupportBoardUI({
	organizationName,
	columns,
	isLoading,
	error,
	onRetry,
	announcement,
	toolbar,
	emptyReason,
	truncationNotice,
	...boardProps
}: SupportBoardUIProps) {
	const cardCount = columns.reduce(
		(total, column) => total + column.cards.length,
		0,
	)
	const isEmpty = !isLoading && !error && cardCount === 0

	return (
		<PageShell className="max-w-none overflow-x-hidden">
			<PageHeader
				title="Support"
				description="Quatre colonnes, un ticket par demande. Déplacez un ticket pour changer son statut."
				eyebrow={organizationName}
				className="sm:items-center sm:justify-start"
				leading={
					<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft text-primary">
						<LifeBuoy className="size-6" />
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
				<div className="flex flex-wrap items-center gap-2">
					<h2 className="font-semibold">Tickets</h2>
					<StatusBadge tone="neutral">
						{cardCount} ticket{cardCount > 1 ? 's' : ''}
					</StatusBadge>
				</div>

				{toolbar}

				{truncationNotice ? (
					<output className="block rounded-md border border-dashed bg-muted/25 px-3 py-2 text-sm text-muted-foreground">
						{truncationNotice}
					</output>
				) : null}

				{isEmpty ? (
					<div className="flex flex-col items-center justify-center gap-2 rounded-md border border-dashed bg-muted/25 p-10 text-center">
						<p className="text-sm font-semibold">Aucun ticket</p>
						<p className="text-sm text-muted-foreground">
							{emptyReason ?? 'Aucun ticket pour le moment.'}
						</p>
					</div>
				) : (
					<TicketColumns
						{...boardProps}
						columns={columns}
						isLoading={isLoading}
					/>
				)}
			</SectionCard>

			<output aria-live="polite" className="sr-only">
				{announcement}
			</output>
		</PageShell>
	)
}
