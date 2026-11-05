import { AlertCircle, ChevronRight } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { SupportTicketPageUIProps } from '#/pages/support/types'
import { TicketHeadline } from '#/pages/support/ui/ticket-detail-ui'

export function SupportTicketPageUI({
	ticket,
	isLoading,
	error,
	onRetry,
	onBack,
	detail,
}: SupportTicketPageUIProps) {
	if (!isLoading && !error && !ticket)
		return (
			<PageShell className="gap-0">
				<PageHeader
					className="mb-6"
					title="Ticket introuvable"
					description="Ce ticket n’existe pas ou a été supprimé."
					actions={
						<Button onClick={onBack} variant="outline">
							Voir les tickets
						</Button>
					}
				/>
			</PageShell>
		)

	return (
		<PageShell className="max-w-7xl gap-0">
			<nav aria-label="Emplacement du ticket" className="mb-2">
				<ol className="flex items-center gap-1 text-muted-foreground text-sm">
					<li>
						<button
							type="button"
							onClick={onBack}
							className="rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
						>
							Tickets
						</button>
					</li>
					<ChevronRight className="size-3.5" aria-hidden />
					<li aria-current="page" className="font-mono text-foreground">
						{ticket?.numberLabel ?? '…'}
					</li>
				</ol>
			</nav>
			<header className="mb-6">
				<h1 className="font-semibold text-2xl tracking-tight">
					{ticket?.subject ?? 'Ticket'}
				</h1>
				{ticket ? (
					<div className="mt-1">
						<TicketHeadline ticket={ticket} />
					</div>
				) : null}
			</header>
			{error && !ticket ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</SectionCard>
			) : (
				detail
			)}
		</PageShell>
	)
}
