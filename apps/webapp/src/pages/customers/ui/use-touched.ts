import { useState } from 'react'

export function useTouched() {
	const [touched, setTouched] = useState<ReadonlySet<string>>(new Set())
	const [submitted, setSubmitted] = useState(false)
	return {
		isShown: (field: string) => submitted || touched.has(field),
		touch: (...fields: string[]) =>
			setTouched((current) => new Set([...current, ...fields])),
		revealAll: () => setSubmitted(true),
	}
}
