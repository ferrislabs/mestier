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

		const lastElement = elements[elements.length - 1]
		const lastId = lastElement.id

		// The rootMargin below ties detection to a fixed strip near the top
		// of the viewport, independent of scroll position. When the document
		// is only slightly taller than the viewport, or the last section is
		// short, that section's rect can sit below the strip for the entire
		// scrollable range and never be reported as intersecting — the nav
		// then gets stuck on whichever earlier section still spans the
		// strip. Reaching the bottom of the page is unambiguous regardless
		// of section geometry, so both writers below consult this single
		// check before deciding activeId, instead of one inferring it from
		// intersection state that can be wrong right at the bottom.
		const isAtBottom = () => {
			const scrollHeight = document.documentElement.scrollHeight
			// The page doesn't scroll at all: every section is already
			// fully on screen, so there's nothing to scroll "toward" and
			// forcing any single section active here would be arbitrary.
			// Left to the observer on purpose, not an oversight.
			if (scrollHeight <= window.innerHeight + BOTTOM_THRESHOLD_PX) {
				return false
			}
			return (
				window.scrollY + window.innerHeight >=
				scrollHeight - BOTTOM_THRESHOLD_PX
			)
		}

		const observer = new IntersectionObserver(
			(entries) => {
				// The observer's own reading can be wrong right at the
				// bottom of a short page (see comment above): re-check the
				// same authoritative condition before trusting entries, so
				// whichever writer runs last still lands on the same value.
				if (isAtBottom()) {
					setActiveId(lastId)
					return
				}

				const visible = entries
					.filter((entry) => entry.isIntersecting)
					.sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)

				// A section that is almost entirely scrolled past can still be
				// intersecting — only its trailing edge trails through the band —
				// at the same time as the section the reader has just reached.
				// Sorting by top puts that scrolled-past section first and the
				// just-entered one last: take the last entry, since it is the
				// one whose top edge most recently crossed into the band.
				const mostRecentlyEntered = visible[visible.length - 1]?.target.id
				if (mostRecentlyEntered) setActiveId(mostRecentlyEntered)
			},
			{ rootMargin: '-80px 0px -60% 0px' },
		)

		for (const element of elements) observer.observe(element)

		const handleScroll = () => {
			if (isAtBottom()) setActiveId(lastId)
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
