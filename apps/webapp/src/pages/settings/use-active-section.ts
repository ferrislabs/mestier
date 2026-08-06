import { useEffect, useState } from 'react'

// Tolerance for float rounding when comparing scroll offsets to the
// document's scrollable height (browsers can report either side with
// sub-pixel drift at 100%/125% zoom).
const BOTTOM_THRESHOLD_PX = 2

// Fraction of the viewport that stays below the detection band's lower
// edge. Mirrors the `-60%` bottom rootMargin this hook used to hand
// IntersectionObserver: the band's lower edge sits at `innerHeight * (1 -
// BAND_BOTTOM_VIEWPORT_RATIO)`. Keeping that edge partway down the
// viewport, instead of right at its top, is what stops the highlight from
// flickering when two headings are briefly stacked near the top together.
const BAND_BOTTOM_VIEWPORT_RATIO = 0.6

// Height of the sticky app header that sections must not scroll under.
// Tailwind classes can't read this constant, so `anchor-nav.tsx`
// (`sticky top-20`) and `settings-layout.tsx` (`scroll-mt-20`) encode the
// same 80px separately — both must be updated together with this value.
export const SETTINGS_HEADER_OFFSET_PX = 80

export function useActiveSection(ids: string[]): string {
	const [activeId, setActiveId] = useState(ids[0] ?? '')
	const key = ids.join('|')

	useEffect(() => {
		const sectionIds = key ? key.split('|') : []
		const elements = sectionIds
			.map((id) => document.getElementById(id))
			.filter((element): element is HTMLElement => element !== null)

		if (elements.length === 0) return

		const lastId = elements[elements.length - 1].id

		// Reaching the bottom of the page is unambiguous regardless of section
		// geometry, so this check is authoritative and is consulted before the
		// band-based decision below, from the single call site that decides
		// activeId. Without it, a short final section — one that can never be
		// scrolled up past the band's lower edge, because the page runs out of
		// room to scroll first — would leave the nav stuck on an earlier
		// section even once the reader has scrolled all the way down.
		const isAtBottom = () => {
			const scrollHeight = document.documentElement.scrollHeight
			// The page doesn't scroll at all: every section is already fully
			// on screen, so there's nothing to scroll "toward" and forcing any
			// single section active here would be arbitrary. Left to the
			// band-based check below on purpose, not an oversight.
			if (scrollHeight <= window.innerHeight + BOTTOM_THRESHOLD_PX) {
				return false
			}
			return (
				window.scrollY + window.innerHeight >=
				scrollHeight - BOTTOM_THRESHOLD_PX
			)
		}

		// Picks the section the reader is currently on by measuring every
		// section's position directly, instead of relying on which targets an
		// IntersectionObserver happens to report as changed. A callback-driven
		// `entries` list only ever contains targets whose intersection status
		// just flipped, so deriving the answer from it silently drops every
		// section that hasn't crossed a boundary since the previous callback —
		// which, at any given moment, is most of them.
		const computeActiveId = () => {
			if (isAtBottom()) return lastId

			const bandBottom = window.innerHeight * (1 - BAND_BOTTOM_VIEWPORT_RATIO)

			// The reader is "on" the last section whose top edge has already
			// scrolled up to or past the band's lower edge. `elements` runs top
			// to bottom, so later qualifying sections override earlier ones,
			// leaving whichever qualifying section is furthest down the page —
			// i.e. the one most recently reached.
			let candidateId = elements[0].id
			for (const element of elements) {
				if (element.getBoundingClientRect().top <= bandBottom) {
					candidateId = element.id
				}
			}
			return candidateId
		}

		const handleUpdate = () => setActiveId(computeActiveId())

		handleUpdate()
		window.addEventListener('scroll', handleUpdate, { passive: true })
		window.addEventListener('resize', handleUpdate, { passive: true })

		return () => {
			window.removeEventListener('scroll', handleUpdate)
			window.removeEventListener('resize', handleUpdate)
		}
	}, [key])

	return activeId
}
