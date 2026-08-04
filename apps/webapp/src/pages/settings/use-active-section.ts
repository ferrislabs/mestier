import { useEffect, useState } from 'react'

export function useActiveSection(ids: string[]): string {
	const [activeId, setActiveId] = useState(ids[0] ?? '')
	const key = ids.join('|')

	useEffect(() => {
		const sectionIds = key ? key.split('|') : []
		const elements = sectionIds
			.map((id) => document.getElementById(id))
			.filter((element): element is HTMLElement => element !== null)

		if (elements.length === 0) return

		const observer = new IntersectionObserver(
			(entries) => {
				const visible = entries
					.filter((entry) => entry.isIntersecting)
					.sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)

				const first = visible[0]?.target.id
				if (first) setActiveId(first)
			},
			{ rootMargin: '-80px 0px -60% 0px' },
		)

		for (const element of elements) observer.observe(element)
		return () => observer.disconnect()
	}, [key])

	return activeId
}
