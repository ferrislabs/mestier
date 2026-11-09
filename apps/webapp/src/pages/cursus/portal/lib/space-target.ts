import type { useNavigate } from '@tanstack/react-router'
import type { CursusSpace } from '#/pages/cursus/me/feature/use-my-cursus'

export function navigateToSpace(
	navigate: ReturnType<typeof useNavigate>,
	space: Pick<CursusSpace, 'slug' | 'is_member'>,
	replace = false,
) {
	if (space.is_member) {
		return navigate({
			to: '/o/$organizationSlug/cursus/mes-trinomes',
			params: { organizationSlug: space.slug },
			replace,
		})
	}
	return navigate({
		to: '/portail/$organizationSlug',
		params: { organizationSlug: space.slug },
		replace,
	})
}
