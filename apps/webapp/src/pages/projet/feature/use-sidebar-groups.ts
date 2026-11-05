import { useCallback, useState } from 'react'

function readCollapsed(storageKey: string): ReadonlySet<string> {
	try {
		const stored = localStorage.getItem(storageKey)
		const parsed: unknown = stored ? JSON.parse(stored) : []
		return new Set(Array.isArray(parsed) ? parsed.map(String) : [])
	} catch {
		return new Set()
	}
}

export function useSidebarGroups(
	storageKey: string,
): [(groupId: string) => boolean, (groupId: string) => void] {
	const [collapsed, setCollapsed] = useState(() => readCollapsed(storageKey))
	const toggle = useCallback(
		(groupId: string) => {
			setCollapsed((current) => {
				const next = new Set(current)
				if (next.has(groupId)) next.delete(groupId)
				else next.add(groupId)
				try {
					localStorage.setItem(storageKey, JSON.stringify([...next]))
				} catch {}
				return next
			})
		},
		[storageKey],
	)
	return [(groupId) => collapsed.has(groupId), toggle]
}
