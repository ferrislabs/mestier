import { useEffect, useState } from 'react'

const QUERY = '(min-width: 1024px)'

const matches = () =>
	typeof window.matchMedia === 'function' && window.matchMedia(QUERY).matches

export function useWideScreen() {
	const [wide, setWide] = useState(matches)
	useEffect(() => {
		if (typeof window.matchMedia !== 'function') return
		const list = window.matchMedia(QUERY)
		const onChange = () => setWide(list.matches)
		list.addEventListener('change', onChange)
		onChange()
		return () => list.removeEventListener('change', onChange)
	}, [])
	return wide
}
