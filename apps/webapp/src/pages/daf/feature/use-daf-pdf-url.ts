import { useEffect, useState } from 'react'
import { fetchDafPdf } from '#/hooks/use-daf'

interface PdfState {
	url: string | null
	error: string | null
	isLoading: boolean
}

/** Fetches a DAF PDF with the session token and exposes it as an object URL. */
export function useDafPdfUrl(path: string | null): PdfState {
	const [state, setState] = useState<PdfState>({
		url: null,
		error: null,
		isLoading: true,
	})

	useEffect(() => {
		if (!path) return
		let objectUrl: string | null = null
		let cancelled = false
		setState({ url: null, error: null, isLoading: true })

		fetchDafPdf(path)
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
	}, [path])

	return state
}
