import { useState } from 'react'
import { nodeChoiceGroups } from '#/pages/survey/lib/node-catalogue'
import type { NodeChoice } from '#/pages/survey/types'
import {
	CREATOR_SEARCH_ID,
	SurveyNodeCreator,
} from '#/pages/survey/ui/survey-node-creator'

export function SurveyNodeCreatorFeature({
	attachedTo,
	onPick,
	onClose,
}: {
	attachedTo: string | null
	onPick: (choice: NodeChoice) => void
	onClose: () => void
}) {
	const [query, setQuery] = useState('')
	const [cursor, setCursor] = useState(0)
	const groups = nodeChoiceGroups(query)
	const flat = groups.flatMap((group) => group.entries)
	const index = flat.length === 0 ? -1 : Math.min(cursor, flat.length - 1)
	const active = index >= 0 ? flat[index].choice : null

	return (
		<SurveyNodeCreator
			attachedTo={attachedTo}
			query={query}
			groups={groups}
			active={active}
			onQueryChange={(next) => {
				setQuery(next)
				setCursor(0)
			}}
			onKeyDown={(event) => {
				if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
					event.preventDefault()
					if (flat.length === 0) return
					const step = event.key === 'ArrowDown' ? 1 : -1
					setCursor((index + step + flat.length) % flat.length)
					return
				}
				if (event.key === 'Enter') {
					const target = event.target
					if (target instanceof HTMLButtonElement) return
					event.preventDefault()
					if (active) onPick(active)
					return
				}
				const target = event.target
				if (
					event.key.length === 1 &&
					!event.metaKey &&
					!event.ctrlKey &&
					!event.altKey &&
					!(target instanceof HTMLInputElement)
				) {
					event.preventDefault()
					setQuery(`${query}${event.key}`)
					setCursor(0)
					document.getElementById(CREATOR_SEARCH_ID)?.focus()
				}
			}}
			onPick={onPick}
			onClose={onClose}
		/>
	)
}
