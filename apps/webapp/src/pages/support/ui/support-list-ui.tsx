import { AlertCircle, Inbox } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { SupportListUIProps } from '#/pages/support/types'
import { TicketDetailPlaceholder } from '#/pages/support/ui/ticket-detail-placeholder'
import { TicketRow } from '#/pages/support/ui/ticket-row'

export function SupportListUI({
	header,
	tickets,
	selectedTicketId,
	isLoading,
	error,
	onRetry,
	emptyReason,
	truncationNotice,
	detail,
	onOpenRow,
}: SupportListUIProps) {
	const isEmpty = !isLoading && !error && tickets.length === 0

	return (
		<div className="grid min-h-0 flex-1 grid-cols-[minmax(20rem,24rem)_minmax(0,1fr)]">
			<section
				aria-label="Liste des tickets"
				className="flex min-h-0 flex-col border-border border-r bg-card"
			>
				{header}
				<div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden">
					{error ? (
						<div className="m-4 flex flex-col gap-3 rounded-md border border-destructive/30 bg-destructive-soft p-4 text-destructive">
							<div className="flex items-center gap-3">
								<AlertCircle className="size-5 shrink-0" />
								<p className="text-sm font-medium">{error}</p>
							</div>
							<Button
								onClick={onRetry}
								variant="outline"
								size="sm"
								className="self-start"
							>
								Réessayer
							</Button>
						</div>
					) : null}
					{truncationNotice ? (
						<output className="m-4 block rounded-md border border-dashed bg-muted/25 px-3 py-2 text-sm text-muted-foreground">
							{truncationNotice}
						</output>
					) : null}
					{isLoading ? (
						<p className="px-6 py-14 text-center text-muted-foreground text-sm">
							Chargement…
						</p>
					) : isEmpty ? (
						<div className="flex flex-col items-center px-6 py-14 text-center">
							<Inbox
								className="mb-3 size-8 text-muted-foreground"
								aria-hidden
							/>
							<p className="font-medium text-base">Aucun ticket</p>
							<p className="mt-1 text-muted-foreground text-sm">
								{emptyReason ?? 'Aucun ticket pour le moment.'}
							</p>
						</div>
					) : (
						<ul>
							{tickets.map((ticket) => (
								<li key={ticket.id}>
									<TicketRow
										ticket={ticket}
										selected={ticket.id === selectedTicketId}
										onOpen={() => onOpenRow(ticket.id)}
									/>
								</li>
							))}
						</ul>
					)}
				</div>
			</section>
			<section aria-label="Détail du ticket" className="min-h-0 min-w-0">
				{detail ?? <TicketDetailPlaceholder />}
			</section>
		</div>
	)
}
