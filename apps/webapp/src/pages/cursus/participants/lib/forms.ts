import type { QueryClient } from '@tanstack/react-query'

export function emptyToNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function formatDate(iso: string | null | undefined): string {
	if (!iso) return '—'
	const date = new Date(iso)
	if (Number.isNaN(date.getTime())) return iso
	return date.toLocaleDateString('fr-FR')
}

export function invalidateCursus(queryClient: QueryClient, segment: string) {
	return queryClient.invalidateQueries({
		predicate: (query) => {
			const meta = query.queryKey[0]
			if (typeof meta !== 'object' || meta === null) return false
			const id = (meta as { _id?: unknown })._id
			return typeof id === 'string' && id.includes(`/cursus/${segment}`)
		},
	})
}

export function conflictMessage(
	error: unknown,
	message: string,
): string | null {
	if (error == null) return null
	if (
		typeof error === 'object' &&
		'status' in error &&
		(error as { status?: unknown }).status === 409
	) {
		return message
	}
	return null
}
