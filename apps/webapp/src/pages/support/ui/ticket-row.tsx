import { cn } from '#/lib/utils'
import { supportStatusLabel } from '#/pages/support/lib/columns'
import { formatRelative } from '#/pages/support/lib/format'
import type { SupportTicketVM } from '#/pages/support/types'
import { AssigneeStack } from '#/pages/support/ui/assignee-stack'
import {
	BadgeChip,
	CustomFieldChip,
} from '#/pages/support/ui/custom-field-chip'
import { PersonAvatar } from '#/pages/support/ui/member-avatar'
import { PriorityBadge } from '#/pages/support/ui/priority-badge'
import { StatusDot } from '#/pages/support/ui/status-badge'

export function TicketRow({
	ticket,
	selected,
	onOpen,
}: {
	ticket: SupportTicketVM
	selected: boolean
	onOpen: () => void
}) {
	return (
		<button
			type="button"
			onClick={onOpen}
			aria-current={selected ? 'true' : undefined}
			className={cn(
				'relative flex w-full gap-3 border-border border-b px-4 py-3 text-left outline-none transition-colors hover:bg-accent/60 focus-visible:bg-accent',
				selected && 'bg-accent',
			)}
		>
			{selected && (
				<span
					aria-hidden
					className="absolute inset-y-0 left-0 w-0.5 bg-primary"
				/>
			)}
			<PersonAvatar
				name={ticket.requester.name}
				hue={null}
				size="lg"
				className="mt-0.5"
			/>
			<span className="grid min-w-0 flex-1 grid-cols-[minmax(0,1fr)] gap-0.5">
				<span className="flex items-baseline justify-between gap-2">
					<span className="truncate font-medium text-sm">
						{ticket.requester.name}
					</span>
					<span className="shrink-0 text-muted-foreground text-xs tabular-nums">
						{formatRelative(ticket.updatedAt)}
					</span>
				</span>
				<span className="flex items-baseline gap-1.5 text-sm">
					<span className="shrink-0 font-mono text-muted-foreground text-xs">
						{ticket.numberLabel}
					</span>
					<span className="truncate">{ticket.subject}</span>
				</span>
				{ticket.preview !== '' && (
					<span className="flex items-center gap-1 text-muted-foreground text-xs">
						<span className="truncate">{ticket.preview}</span>
					</span>
				)}
				<span className="mt-1 flex flex-wrap items-center gap-1">
					<span className="mr-1 inline-flex items-center gap-1.5 text-muted-foreground text-xs">
						<StatusDot status={ticket.status} />
						{supportStatusLabel(ticket.status)}
					</span>
					<PriorityBadge priority={ticket.priority} />
					{ticket.badges.map((badge) => (
						<BadgeChip
							key={badge.label}
							label={badge.label}
							tone={badge.tone}
						/>
					))}
					{ticket.displayFields.map((field) => (
						<CustomFieldChip
							key={field.key}
							fieldKey={field.key}
							value={field.value}
						/>
					))}
					<AssigneeStack
						assignee={ticket.assignee}
						showNames={false}
						className="ml-auto"
					/>
				</span>
			</span>
		</button>
	)
}
