import type { ReactNode } from 'react'
import { type PermissionName, useHasPermission } from '#/hooks/use-permissions'
import { useRhActorId } from '#/pages/hr/ui/rh/shared/rh-context'

export type RequestKind = 'absence' | 'extra'

const BIT_FOR_OTHERS: Record<RequestKind, PermissionName> = {
	absence: 'MANAGE_REFERENCE',
	extra: 'MANAGE_PLANNING',
}

export function useCanRequestForOthers(kind: RequestKind): boolean {
	return useHasPermission(BIT_FOR_OTHERS[kind])
}

export function useOwnRequestsOnly(kind: RequestKind): boolean {
	const planning = useHasPermission('MANAGE_PLANNING')
	const reference = useHasPermission('MANAGE_REFERENCE')
	return kind === 'absence' ? !(planning || reference) : !planning
}

export function useCanRequestFor(
	kind: RequestKind,
	memberId: string | undefined,
): boolean {
	const forOthers = useCanRequestForOthers(kind)
	const actorId = useRhActorId()
	if (forOthers) return true
	return memberId === undefined || (actorId !== '' && memberId === actorId)
}

export function CanRequest({
	kind,
	memberId,
	children,
}: {
	kind: RequestKind
	memberId?: string
	children: ReactNode
}) {
	return useCanRequestFor(kind, memberId) ? children : null
}
