import {
	Select,
	SelectContent,
	SelectItem,
	SelectSeparator,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import { SUPPORT_COLUMNS } from '#/pages/support/lib/columns'
import { UNASSIGNED_FILTER } from '#/pages/support/lib/filters'
import { SUPPORT_PRIORITIES } from '#/pages/support/lib/priorities'
import {
	ACTIVE_STATUS_FILTER,
	ALL_FILTER,
	type StatusFilterValue,
	type TicketFilterValues,
} from '#/pages/support/lib/ticket-filters'
import type {
	SupportFilterOption,
	SupportMemberVM,
} from '#/pages/support/types'
import { MemberAvatar } from '#/pages/support/ui/member-avatar'
import { PriorityIcon } from '#/pages/support/ui/priority-badge'

function CountBadge({ count }: { count: number | undefined }) {
	return (
		<span className="ml-auto pl-3 text-muted-foreground text-xs tabular-nums">
			{count ?? 0}
		</span>
	)
}

export function AssigneeFilterSelect({
	id,
	value,
	onChange,
	counts,
	members,
	disabled,
	className,
}: {
	id?: string
	value: string
	onChange: (assignee: string) => void
	counts: ReadonlyMap<string | null, number>
	members: readonly SupportMemberVM[]
	disabled?: boolean
	className?: string
}) {
	return (
		<Select value={value} onValueChange={onChange} disabled={disabled}>
			<SelectTrigger
				id={id}
				aria-label={id ? undefined : 'Filtrer par assigné'}
				className={cn('min-w-[210px]', className)}
			>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={ALL_FILTER}>Tout le monde</SelectItem>
				<SelectItem value={UNASSIGNED_FILTER}>
					Non assignés
					<CountBadge count={counts.get(null)} />
				</SelectItem>
				{members.length > 0 && <SelectSeparator />}
				{members.map((member) => (
					<SelectItem key={member.id} value={member.id}>
						<MemberAvatar member={member} size="sm" />
						{member.name}
						<CountBadge count={counts.get(member.id)} />
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}

export function StatusFilterSelect({
	id,
	value,
	onChange,
	className,
}: {
	id?: string
	value: StatusFilterValue
	onChange: (status: StatusFilterValue) => void
	className?: string
}) {
	return (
		<Select
			value={value}
			onValueChange={(next) => onChange(next as StatusFilterValue)}
		>
			<SelectTrigger
				id={id}
				aria-label={id ? undefined : 'Filtrer par statut'}
				className={cn('min-w-[200px]', className)}
			>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={ACTIVE_STATUS_FILTER}>Tickets non fermés</SelectItem>
				<SelectItem value="all">Tous les statuts</SelectItem>
				<SelectSeparator />
				{SUPPORT_COLUMNS.map((entry) => (
					<SelectItem key={entry.status} value={entry.status}>
						{entry.label}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}

export function PriorityFilterSelect({
	id,
	value,
	onChange,
	className,
}: {
	id?: string
	value: TicketFilterValues['priority']
	onChange: (priority: TicketFilterValues['priority']) => void
	className?: string
}) {
	return (
		<Select
			value={value}
			onValueChange={(next) => onChange(next as TicketFilterValues['priority'])}
		>
			<SelectTrigger
				id={id}
				aria-label={id ? undefined : 'Filtrer par priorité'}
				className={cn('min-w-[180px]', className)}
			>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={ALL_FILTER}>Toutes les priorités</SelectItem>
				<SelectSeparator />
				{SUPPORT_PRIORITIES.map((priority) => (
					<SelectItem key={priority.id} value={priority.id}>
						<PriorityIcon priority={priority.id} />
						{priority.label}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}

export function BadgeFilterSelect({
	id,
	value,
	onChange,
	badges,
	className,
}: {
	id?: string
	value: string
	onChange: (badge: string) => void
	badges: readonly SupportFilterOption[]
	className?: string
}) {
	return (
		<Select value={value} onValueChange={onChange}>
			<SelectTrigger
				id={id}
				aria-label={id ? undefined : 'Filtrer par badge'}
				className={cn('min-w-[180px]', className)}
			>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={ALL_FILTER}>Tous les badges</SelectItem>
				{badges.length > 0 && <SelectSeparator />}
				{badges.map((badge) => (
					<SelectItem key={badge.id} value={badge.id}>
						{badge.label}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}
