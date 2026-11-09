import { usePermissions } from '#/hooks/use-permissions'
import type { NavTarget } from '#/modules/types'
import { useIsCursusParticipant } from '#/pages/cursus/me/feature/use-cursus-participation'

export type SectionVisibility = (
	target: Pick<NavTarget, 'requiredPermission' | 'visibleWhen'>,
) => boolean

export function isTargetVisible(
	target: Pick<NavTarget, 'requiredPermission' | 'visibleWhen'>,
	context: { granted: string[]; loaded: boolean; isCursusParticipant: boolean },
): boolean {
	if (target.visibleWhen === 'cursus-participant') {
		return context.isCursusParticipant
	}
	return (
		!target.requiredPermission ||
		(context.loaded && context.granted.includes(target.requiredPermission))
	)
}

export function useSectionVisibility(): SectionVisibility {
	const { data, isSuccess } = usePermissions()
	const isCursusParticipant = useIsCursusParticipant()
	const granted = data?.data.permissions ?? []

	return (target) =>
		isTargetVisible(target, {
			granted,
			loaded: isSuccess,
			isCursusParticipant,
		})
}
