import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useUnreadMentionCount } from '#/hooks/use-chat'

/** The count of mentions not read yet, next to "Activité" in the side nav. */
export function ActivityBadgeFeature() {
	const { activeOrganization } = useActiveOrganization()
	const count = useUnreadMentionCount(activeOrganization.id)
	if (count === 0) return null
	return (
		<span
			role="img"
			aria-label={`${count} mentions non lues`}
			className="tabular ml-auto flex h-5 min-w-5 shrink-0 items-center justify-center rounded-full bg-nav-primary-foreground px-1.5 font-semibold text-nav-primary text-xs"
		>
			{count > 99 ? '99+' : count}
		</span>
	)
}
