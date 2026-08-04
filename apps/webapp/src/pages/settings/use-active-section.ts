import { useEffect, useState } from 'react'

// Tolerance for float rounding when comparing scroll offsets to the
// document's scrollable height (browsers can report either side with
// sub-pixel drift at 100%/125% zoom).
const BOTTOM_THRESHOLD_PX = 2

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

		// The rootMargin above ties detection to a fixed strip near the top
		// of the viewport, independent of scroll position. When the document
		// is only slightly taller than the viewport, or the last section is
		// short, that section's rect can sit below the strip for the entire
		// scrollable range and never be reported as intersecting — the nav
		// then gets stuck on whichever earlier section still spans the
		// strip. Reaching the bottom of the page is unambiguous regardless
		// of section geometry, so it gets its own check: once there is
		// nowhere further down to scroll, the last section is the one being
		// read.
		const lastElement = elements[elements.length - 1]
		const lastId = lastElement.id

		const handleScroll = () => {
			const scrollHeight = document.documentElement.scrollHeight
			const isScrollable =
				scrollHeight > window.innerHeight + BOTTOM_THRESHOLD_PX
			if (!isScrollable) return

			const atBottom =
				window.scrollY + window.innerHeight >=
				scrollHeight - BOTTOM_THRESHOLD_PX
			if (atBottom) setActiveId(lastId)
		}

		handleScroll()
		window.addEventListener('scroll', handleScroll, { passive: true })

		return () => {
			observer.disconnect()
			window.removeEventListener('scroll', handleScroll)
		}
	}, [key])

	return activeId
}
