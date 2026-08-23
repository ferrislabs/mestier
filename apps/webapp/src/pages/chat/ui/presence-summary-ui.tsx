import { Circle } from 'lucide-react'

export interface PresenceSummaryUIProps {
	onlineCount: number
}

/**
 * "N online" — not a member list with names, because there is no REST
 * mapping from a chat `user_id` to a display name (`MemberResponse` never
 * carries `users.id`/`users.sub`, by design — see
 * `libs/handlers-member/src/response.rs`). A count is the honest subset of
 * "who is online" this can show without guessing at an identity.
 */
export function PresenceSummaryUI({ onlineCount }: PresenceSummaryUIProps) {
	if (onlineCount === 0) return null

	return (
		<div className="flex items-center gap-1.5 px-4 pb-2 text-xs text-muted-foreground">
			<Circle className="size-2 fill-success text-success" />
			<span>
				{onlineCount}{' '}
				{onlineCount === 1 ? 'personne en ligne' : 'personnes en ligne'}
			</span>
		</div>
	)
}
