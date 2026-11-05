import { useCallback, useEffect, useState } from 'react'

const NOTICE_DURATION_MS = 8000

export function useNotice() {
	const [message, setMessage] = useState<string | null>(null)
	useEffect(() => {
		if (message === null) return
		const timer = setTimeout(() => setMessage(null), NOTICE_DURATION_MS)
		return () => clearTimeout(timer)
	}, [message])
	const clear = useCallback(() => setMessage(null), [])
	return { message, show: setMessage, clear }
}
