import { useState } from 'react'

export interface Steps {
	index: number
	isFirst: boolean
	isLast: boolean
	next: () => void
	back: () => void
	goTo: (index: number) => void
}

export function useSteps(count: number): Steps {
	const [index, setIndex] = useState(0)
	return {
		index,
		isFirst: index === 0,
		isLast: index === count - 1,
		next: () => setIndex((current) => Math.min(current + 1, count - 1)),
		back: () => setIndex((current) => Math.max(current - 1, 0)),
		goTo: (target) =>
			setIndex((current) =>
				target >= 0 && target < current ? target : current,
			),
	}
}
