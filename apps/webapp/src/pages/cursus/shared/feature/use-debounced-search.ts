import { useEffect, useRef, useState } from 'react'

export const SEARCH_DEBOUNCE_MS = 300

export function useDebouncedSearch(
	initial: string,
	onCommit?: (query: string) => void,
	delay = SEARCH_DEBOUNCE_MS,
) {
	const [text, setText] = useState(initial)
	const [query, setQuery] = useState(initial.trim())
	const committed = useRef(initial.trim())
	const commit = useRef(onCommit)
	commit.current = onCommit

	useEffect(() => {
		const next = text.trim()
		const timer = setTimeout(() => {
			if (next === committed.current) return
			committed.current = next
			setQuery(next)
			commit.current?.(next)
		}, delay)
		return () => clearTimeout(timer)
	}, [text, delay])

	const reset = () => {
		committed.current = ''
		setText('')
		setQuery('')
	}

	return { text, query, setText, reset }
}
