import {
	type CursusSpace,
	useMyCursusSpaces,
} from '#/pages/cursus/me/feature/use-my-cursus'

export function usePortalSpace(slug: string): {
	isLoading: boolean
	space: CursusSpace | undefined
} {
	const spaces = useMyCursusSpaces()
	return {
		isLoading: spaces.isLoading,
		space: spaces.data?.data.find((candidate) => candidate.slug === slug),
	}
}
