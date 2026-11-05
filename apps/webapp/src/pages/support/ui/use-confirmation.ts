import { useCallback, useEffect, useState } from 'react'

const CONFIRMATION_MS = 6000

export interface Confirmation {
	armed: boolean
	press: (action: () => void) => void
	cancel: () => void
}

export function useConfirmation(timeoutMs = CONFIRMATION_MS): Confirmation {
	const [armed, setArmed] = useState(false)

	useEffect(() => {
		if (!armed) return
		const timer = window.setTimeout(() => setArmed(false), timeoutMs)
		return () => window.clearTimeout(timer)
	}, [armed, timeoutMs])

	const cancel = useCallback(() => setArmed(false), [])
	const press = useCallback(
		(action: () => void) => {
			if (!armed) {
				setArmed(true)
				return
			}
			setArmed(false)
			action()
		},
		[armed],
	)

	return { armed, press, cancel }
}
