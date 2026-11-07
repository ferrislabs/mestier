import { Link } from '@tanstack/react-router'
import { CalendarClock, EllipsisVertical } from 'lucide-react'
import type { PointerEvent } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import type { Customer } from '#/hooks/use-customers'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { findPerson } from '#/pages/customers/feature/mocks/people'
import { formatShortDate } from '#/pages/customers/lib/format'
import { accountStatusOf, LEAD_SOURCE_LABELS } from '#/pages/customers/types'
import { hueOf, NameAvatar, PersonAvatar } from '#/pages/customers/ui/avatars'
import { Money } from '#/pages/customers/ui/money'
import { AccountStatusBadge } from '#/pages/customers/ui/status-badges'
import type {
	Deal,
	DealQuote,
	DealStage,
	IsoDate,
} from '#/pages/deals/lib/deal-types'
import { STAGE_LABELS, STAGES } from '#/pages/deals/lib/pipeline'
import { isOverdue, latestQuote } from '#/pages/deals/lib/pipeline-filters'
import { DealQuoteStatusBadge } from '#/pages/deals/ui/deal-badges'

export function DealCard({
	deal,
	customer,
	quotes,
	today,
	organizationSlug,
	canManage,
	dragging,
	onOpen,
	onMove,
	onPointerDown,
}: {
	deal: Deal
	customer: Customer | undefined
	quotes: readonly DealQuote[]
	today: IsoDate
	organizationSlug: string
	canManage: boolean
	dragging: boolean
	onOpen: () => void
	onMove: (stage: DealStage) => void
	onPointerDown: (event: PointerEvent<HTMLElement>) => void
}) {
	const overdue = isOverdue(deal, today)
	const latest = latestQuote(quotes)
	const person = findPerson(deal.ownerId)
	return (
		<article
			data-flip-id={deal.id}
			onPointerDown={onPointerDown}
			draggable={false}
			className={cn(
				'relative grid gap-2 rounded-md border border-border bg-card p-3 shadow-xs transition-[box-shadow,opacity] focus-within:ring-2 focus-within:ring-ring hover:border-ring/60 hover:shadow-sm',
				canManage && 'cursor-grab',
				dragging && 'opacity-40',
			)}
		>
			<div className="flex items-start gap-2">
				{customer ? (
					<NameAvatar name={customer.name} hue={hueOf(customer.id)} size="sm" />
				) : null}
				<div className="grid min-w-0 flex-1 justify-items-start gap-1">
					<p className="w-full truncate font-medium text-xs">
						{customer?.name ?? 'Client introuvable'}
					</p>
					{customer && (
						<AccountStatusBadge status={accountStatusOf(customer.status)} />
					)}
				</div>
				{canManage && (
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								type="button"
								variant="ghost"
								data-no-drag
								size="icon-xs"
								className="-mt-0.5 -mr-1 relative z-10 text-muted-foreground"
								aria-label={`Déplacer « ${deal.title} » vers…`}
							>
								<EllipsisVertical aria-hidden />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuLabel>Déplacer vers…</DropdownMenuLabel>
							<DropdownMenuSeparator />
							{STAGES.filter((stage) => stage !== deal.stage).map((stage) => (
								<DropdownMenuItem key={stage} onSelect={() => onMove(stage)}>
									{STAGE_LABELS[stage]}
								</DropdownMenuItem>
							))}
						</DropdownMenuContent>
					</DropdownMenu>
				)}
			</div>
			<button
				type="button"
				onClick={onOpen}
				className="line-clamp-3 text-left font-medium text-sm leading-snug outline-none after:absolute after:inset-0 after:rounded-md"
			>
				{deal.title}
			</button>
			<div className="flex flex-wrap items-center justify-between gap-x-2 gap-y-1 text-xs">
				<span className="font-semibold text-sm">
					<Money cents={deal.amount} />
					<span className="font-normal text-muted-foreground text-xs"> HT</span>
				</span>
				{deal.expectedCloseDate ? (
					<span
						className={cn(
							'flex items-center gap-1 tabular-nums',
							overdue
								? 'font-medium text-state-alert'
								: 'text-muted-foreground',
						)}
					>
						<CalendarClock className="size-3.5" aria-hidden />
						<time dateTime={deal.expectedCloseDate}>
							{formatShortDate(deal.expectedCloseDate)}
						</time>
						{overdue && <span className="sr-only">(date dépassée)</span>}
					</span>
				) : (
					<span className="text-muted-foreground">Sans date de clôture</span>
				)}
			</div>
			{deal.stage === 'lost' && deal.lostReason !== '' && (
				<p className="text-muted-foreground text-xs">
					Motif : {deal.lostReason}
				</p>
			)}
			<div className="flex items-center gap-2 text-muted-foreground text-xs">
				<PersonAvatar person={person} size="sm" />
				<span className="min-w-0 flex-1 truncate">
					{person?.name ?? 'Non assigné'}
				</span>
				{deal.source && (
					<span className="shrink-0">{LEAD_SOURCE_LABELS[deal.source]}</span>
				)}
			</div>
			{latest && (
				<div className="flex flex-wrap items-center gap-x-2 gap-y-1 border-t pt-2 text-xs">
					<Link
						to={buildOrgPath(organizationSlug, '/crm/devis/$quoteId')}
						params={{ quoteId: latest.id }}
						className="relative z-10 font-medium hover:underline"
					>
						{latest.number ?? 'Brouillon'}
					</Link>
					<DealQuoteStatusBadge status={latest.status} />
					{quotes.length > 1 && (
						<span className="text-muted-foreground">{quotes.length} devis</span>
					)}
				</div>
			)}
		</article>
	)
}
