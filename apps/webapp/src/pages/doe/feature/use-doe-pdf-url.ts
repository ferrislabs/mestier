import { useEffect, useState } from 'react'
import { doePdfPath } from '#/hooks/use-doe'
import { authStore } from '#/store/auth.store'

interface PdfState {
	url: string | null
	error: string | null
	isLoading: boolean
}

export function useDoePdfUrl(organizationId: string, doeId: string): PdfState {
	const [state, setState] = useState<PdfState>({
		url: null,
		error: null,
		isLoading: true,
	})

	useEffect(() => {
		if (!organizationId || !doeId) return
		let objectUrl: string | null = null
		let cancelled = false
		setState({ url: null, error: null, isLoading: true })

		const headers = new Headers()
		const accessToken = authStore.state.accessToken
		if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)

		fetch(`${window.apiUrl}${doePdfPath(organizationId, doeId)}`, {
			headers,
			credentials: 'include',
		})
			.then((response) => {
				if (!response.ok) {
					throw new Error(`HTTP ${response.status}: ${response.statusText}`)
				}
				return response.blob()
			})
			.then((blob) => {
				if (cancelled) return
				objectUrl = URL.createObjectURL(blob)
				setState({ url: objectUrl, error: null, isLoading: false })
			})
			.catch((failure: Error) => {
				if (!cancelled)
					setState({ url: null, error: failure.message, isLoading: false })
			})

		return () => {
			cancelled = true
			if (objectUrl) URL.revokeObjectURL(objectUrl)
		}
	}, [organizationId, doeId])

	return state
}
