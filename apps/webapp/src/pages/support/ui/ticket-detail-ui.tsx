import {
	AlertCircle,
	Maximize2,
	PanelRight,
	PanelRightClose,
	SearchX,
	X,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { DialogTitle } from '#/components/ui/dialog'
import { formatRelative } from '#/pages/support/lib/format'
import type {
	SupportDetailUIProps,
	SupportTicketVM,
} from '#/pages/support/types'
import { BadgeChip } from '#/pages/support/ui/custom-field-chip'
import { TicketStatusBadge } from '#/pages/support/ui/status-badge'
import { TicketConversationUI } from '#/pages/support/ui/ticket-conversation-ui'
import { TicketProperties } from '#/pages/support/ui/ticket-properties'

export function TicketHeadline({ ticket }: { ticket: SupportTicketVM }) {
	return (
		<p className="flex flex-wrap items-center gap-x-2 gap-y-1 text-muted-foreground text-sm">
			<TicketStatusBadge status={ticket.status} />
			<span aria-hidden>·</span>
			<span>
				{ticket.requester.name} via{' '}
				{ticket.externalId ? 'l’API' : 'saisie manuelle'},{' '}
				{formatRelative(ticket.createdAt)}
			</span>
			{ticket.badges.map((badge) => (
				<BadgeChip key={badge.label} label={badge.label} tone={badge.tone} />
			))}
		</p>
	)
}

const HEADING_CLASS = 'truncate font-semibold text-lg leading-tight'

function Heading({
	modal,
	className,
	children,
}: {
	modal: boolean
	className: string
	children: ReactNode
}) {
	return modal ? (
		<DialogTitle className={className}>{children}</DialogTitle>
	) : (
		<h2 className={className}>{children}</h2>
	)
}

function EmbeddedHeader({
	ticket,
	modal,
	propertiesOpen,
	onToggleProperties,
	onOpenFullPage,
	onClose,
}: {
	modal: boolean
	ticket: SupportTicketVM
	propertiesOpen: boolean
	onToggleProperties: () => void
	onOpenFullPage: () => void
	onClose: (() => void) | undefined
}) {
	return (
		<header className="flex shrink-0 items-start gap-3 border-border border-b px-5 py-3">
			<div className="min-w-0 flex-1">
				<Heading modal={modal} className={HEADING_CLASS}>
					<span className="mr-2 font-mono font-normal text-muted-foreground text-sm">
						{ticket.numberLabel}
					</span>
					{ticket.subject}
				</Heading>
				<div className="mt-1">
					<TicketHeadline ticket={ticket} />
				</div>
			</div>
			<div className="flex shrink-0 items-center gap-1">
				<Button
					type="button"
					variant="ghost"
					size="icon-sm"
					onClick={onToggleProperties}
					aria-pressed={propertiesOpen}
					aria-label={
						propertiesOpen
							? 'Masquer les propriétés'
							: 'Afficher les propriétés'
					}
					title={
						propertiesOpen
							? 'Masquer les propriétés'
							: 'Afficher les propriétés'
					}
				>
					{propertiesOpen ? <PanelRightClose /> : <PanelRight />}
				</Button>
				<Button
					type="button"
					variant="ghost"
					size="icon-sm"
					title="Ouvrir en pleine page"
					aria-label="Ouvrir en pleine page"
					onClick={onOpenFullPage}
				>
					<Maximize2 />
				</Button>
				{onClose && (
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						onClick={onClose}
						aria-label="Fermer le ticket"
						title="Fermer"
					>
						<X />
					</Button>
				)}
			</div>
		</header>
	)
}

function MissingTicket({
	modal,
	onClose,
}: {
	modal: boolean
	onClose: (() => void) | undefined
}) {
	return (
		<div className="flex h-full flex-col items-center justify-center gap-2 px-6 text-center">
			<SearchX className="size-8 text-muted-foreground" aria-hidden />
			<Heading modal={modal} className="font-medium text-base">
				Ticket introuvable
			</Heading>
			<p className="max-w-xs text-muted-foreground text-sm">
				Ce ticket n’existe pas ou a été supprimé.
			</p>
			{onClose && (
				<Button variant="outline" size="sm" onClick={onClose}>
					Fermer
				</Button>
			)}
		</div>
	)
}

function LoadingTicket({ modal }: { modal: boolean }) {
	return (
		<div className="flex h-full flex-col items-center justify-center gap-2 px-6 text-center">
			<Heading modal={modal} className="sr-only">
				Ticket
			</Heading>
			<p className="text-muted-foreground text-sm">Chargement…</p>
		</div>
	)
}

function FailedTicket({
	modal,
	error,
	onRetry,
}: {
	modal: boolean
	error: string
	onRetry: () => void
}) {
	return (
		<div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center text-destructive">
			<Heading modal={modal} className="sr-only">
				Ticket
			</Heading>
			<AlertCircle className="size-6" aria-hidden />
			<p className="text-sm font-medium">{error}</p>
			<Button onClick={onRetry} variant="outline" size="sm">
				Réessayer
			</Button>
		</div>
	)
}

export function TicketDetailUI({
	variant,
	ticket,
	isLoading,
	error,
	onRetry,
	conversation,
	properties,
	propertiesOpen,
	onToggleProperties,
	onOpenFullPage,
	onClose,
}: SupportDetailUIProps) {
	const modal = variant === 'modal'

	if (isLoading) return <LoadingTicket modal={modal} />
	if (error)
		return <FailedTicket modal={modal} error={error} onRetry={onRetry} />
	if (!ticket || !conversation || !properties)
		return <MissingTicket modal={modal} onClose={onClose} />

	if (variant === 'page')
		return (
			<div className="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_20rem]">
				<TicketConversationUI {...conversation} layout="page" />
				<TicketProperties {...properties} variant="card" />
			</div>
		)

	return (
		<div className="flex h-full min-h-0 flex-col bg-background">
			<EmbeddedHeader
				modal={modal}
				ticket={ticket}
				propertiesOpen={propertiesOpen}
				onToggleProperties={onToggleProperties}
				onOpenFullPage={onOpenFullPage}
				onClose={onClose}
			/>
			<div className="flex min-h-0 flex-1">
				<TicketConversationUI {...conversation} layout="embedded" />
				{propertiesOpen && (
					<div className="w-80 shrink-0 overflow-y-auto border-border border-l bg-card">
						<TicketProperties {...properties} variant="plain" />
					</div>
				)}
			</div>
		</div>
	)
}
