import { AlertCircle, ArrowLeft, LifeBuoy } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { SupportTicketPageUIProps } from '#/pages/support/types'

export function SupportTicketPageUI({
	organizationName,
	ticket,
	isLoading,
	error,
	onRetry,
	onBack,
	thread,
	composer,
	sidePanel,
}: SupportTicketPageUIProps) {
	return (
		<PageShell className="max-w-none overflow-x-hidden">
			<PageHeader
				title={ticket ? `${ticket.numberLabel} ${ticket.subject}` : 'Ticket'}
				eyebrow={organizationName}
				className="sm:items-center sm:justify-start"
				leading={
					<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft text-primary">
						<LifeBuoy className="size-6" />
					</div>
				}
				actions={
					<Button onClick={onBack} variant="outline" size="sm">
						<ArrowLeft />
						Retour aux tickets
					</Button>
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

			{isLoading ? (
				<SectionCard className="flex items-center justify-center p-10 text-sm text-muted-foreground">
					Chargement…
				</SectionCard>
			) : ticket ? (
				<div className="grid min-w-0 gap-6 lg:grid-cols-[minmax(0,1fr)_20rem]">
					<SectionCard className="min-w-0 overflow-hidden">
						{thread}
						{composer}
					</SectionCard>
					<div className="min-w-0">{sidePanel}</div>
				</div>
			) : error ? null : (
				<SectionCard className="flex items-center justify-center p-10 text-sm text-muted-foreground">
					Ticket introuvable.
				</SectionCard>
			)}
		</PageShell>
	)
}
