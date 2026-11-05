import { cn } from '#/lib/utils'
import type { SupportMemberVM } from '#/pages/support/types'
import { MemberAvatar } from '#/pages/support/ui/member-avatar'

export function AssigneeStack({
	assignee,
	showNames = true,
	className,
}: {
	assignee: SupportMemberVM | null
	showNames?: boolean
	className?: string
}) {
	return (
		<span
			className={cn('flex min-w-0 items-center gap-1.5', className)}
			title={assignee?.name ?? ''}
		>
			<span className="flex shrink-0 -space-x-1.5">
				<MemberAvatar
					member={assignee}
					size="sm"
					className={assignee ? 'ring-2 ring-card' : undefined}
				/>
			</span>
			{showNames && (
				<span
					className={cn(
						'truncate',
						assignee === null && 'text-muted-foreground',
					)}
				>
					{assignee?.name ?? 'Non assigné'}
				</span>
			)}
		</span>
	)
}
