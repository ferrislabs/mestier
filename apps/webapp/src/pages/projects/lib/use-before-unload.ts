import { useEffect } from 'react'

export function useBeforeUnload(active: boolean) {
	useEffect(() => {
		if (!active) return
		function guard(event: BeforeUnloadEvent) {
			event.preventDefault()
		}
		window.addEventListener('beforeunload', guard)
		return () => window.removeEventListener('beforeunload', guard)
	}, [active])
}
