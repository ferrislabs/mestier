import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { firstEvent, searchEvents } from '#/pages/automation/lib/event-search'
import { MestierEventDialog } from '#/pages/automation/ui/mestier-event-dialog'

export function EventPickerFeature({
	events,
	initialEvent,
	confirmLabel,
	onConfirm,
	onCancel,
}: {
	events: readonly Schemas.EventDescriptorResponse[]
	initialEvent?: string
	confirmLabel: string
	onConfirm: (event: Schemas.EventDescriptorResponse) => void
	onCancel: () => void
}) {
	const [query, setQuery] = useState('')
	const [selectedName, setSelectedName] = useState(
		initialEvent ?? firstEvent(events)?.name ?? '',
	)
	const selected = events.find((event) => event.name === selectedName) ?? null

	return (
		<MestierEventDialog
			open
			onOpenChange={(open) => {
				if (!open) onCancel()
			}}
			query={query}
			onQueryChange={setQuery}
			groups={searchEvents(events, query)}
			selected={selected}
			onSelect={setSelectedName}
			onConfirm={() => {
				if (selected) onConfirm(selected)
			}}
			confirmLabel={confirmLabel}
		/>
	)
}
