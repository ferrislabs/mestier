import { AlertCircle } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { SupportBoardUIProps } from '#/pages/support/types'
import { TicketBoard } from '#/pages/support/ui/ticket-board'

export function SupportBoardUI({
	title,
	description,
	isLoading,
	error,
	onRetry,
	announcement,
	toolbar,
	modal,
	emptyReason,
	truncationNotice,
	...boardProps
}: SupportBoardUIProps) {
	const cardCount = boardProps.columns.reduce(
		(total, column) => total + column.tickets.length,
		0,
	)
	const isEmpty = !isLoading && !error && cardCount === 0

	return (
		<div className="min-h-0 flex-1 overflow-y-auto">
			<PageShell className="max-w-none gap-0">
				<PageHeader title={title} description={description} className="mb-6" />
				{toolbar}

				{error ? (
					<SectionCard className="mb-4 flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
						<div className="flex items-center gap-3">
							<AlertCircle className="size-5 shrink-0" />
							<p className="text-sm font-medium">{error}</p>
						</div>
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					</SectionCard>
				) : null}

				{truncationNotice ? (
					<output className="mb-4 block rounded-md border border-dashed bg-muted/25 px-3 py-2 text-sm text-muted-foreground">
						{truncationNotice}
					</output>
				) : null}

				{isEmpty && emptyReason ? (
					<output className="mb-4 block rounded-md border border-dashed bg-muted/25 px-3 py-2 text-sm text-muted-foreground">
						{emptyReason}
					</output>
				) : null}

				<TicketBoard {...boardProps} isLoading={isLoading} />
			</PageShell>
			{modal}
			<output aria-live="polite" className="sr-only">
				{announcement}
			</output>
		</div>
	)
}
