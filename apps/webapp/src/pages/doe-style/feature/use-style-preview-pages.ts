import { useEffect, useState } from 'react'
import { usePreviewDoeStylePages } from '#/hooks/use-doe-models'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	type CompleteDoeStyle,
	isStyleValid,
	normalizeStyle,
} from '#/pages/doe-style/lib/style'

export const PREVIEW_DEBOUNCE_MS = 350

export interface StylePreviewPages {
	pages: string[]
	loading: boolean
	error: string | null
}

export function useStylePreviewPages(
	organizationId: string,
	style: CompleteDoeStyle,
	delay: number = PREVIEW_DEBOUNCE_MS,
): StylePreviewPages {
	const { mutateAsync } = usePreviewDoeStylePages()
	const [pages, setPages] = useState<string[]>([])
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	const valid = isStyleValid(style)
	const body = JSON.stringify(normalizeStyle(style))

	useEffect(() => {
		if (!valid || !organizationId) {
			setLoading(false)
			return
		}
		let stale = false
		const timer = setTimeout(() => {
			setLoading(true)
			mutateAsync({
				path: { organization_id: organizationId },
				body: { style: JSON.parse(body) },
			})
				.then((response) => {
					if (stale) return
					setPages(response.data.pages)
					setError(null)
					setLoading(false)
				})
				.catch((failure) => {
					if (stale) return
					setError(mutationErrorMessage(failure))
					setLoading(false)
				})
		}, delay)
		return () => {
			stale = true
			clearTimeout(timer)
		}
	}, [organizationId, valid, body, delay, mutateAsync])

	return { pages, loading, error }
}
