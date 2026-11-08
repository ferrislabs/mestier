import { Link } from '@tanstack/react-router'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Member } from '#/pages/hr/lib/rh/types'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

export function MemberLink({ member }: { member: Member }) {
	const orgPath = useRhOrgPath()
	return (
		<Link
			to={orgPath('/rh/membres/$memberId')}
			params={{ memberId: member.id }}
			className="-mx-1.5 flex items-center gap-2.5 rounded-md px-1.5 py-1 outline-none transition-colors hover:bg-accent/60 focus-visible:ring-2 focus-visible:ring-ring"
		>
			<MemberAvatar member={member} size="md" />
			<span className="min-w-0">
				<span className="block truncate font-medium text-[13px]">
					{fullName(member)}
				</span>
				{member.jobTitle && (
					<span className="block truncate text-[12px] text-muted-foreground">
						{member.jobTitle}
					</span>
				)}
			</span>
		</Link>
	)
}
