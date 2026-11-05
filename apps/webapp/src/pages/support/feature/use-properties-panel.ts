import { useCallback, useState } from 'react'

const STORAGE_KEY = 'mestier:support:properties-panel'

function readOpen(): boolean {
	try {
		return localStorage.getItem(STORAGE_KEY) !== 'closed'
	} catch {
		return true
	}
}

export function usePropertiesPanel(): [boolean, () => void] {
	const [open, setOpen] = useState(readOpen)
	const toggle = useCallback(() => {
		setOpen((current) => {
			const next = !current
			try {
				localStorage.setItem(STORAGE_KEY, next ? 'open' : 'closed')
			} catch {}
			return next
		})
	}, [])
	return [open, toggle]
}
