import { useState } from 'react'
import { useProposeIntervention } from '#/hooks/use-field'
import { proposeErrorMessage } from '#/pages/maintenance/lib/field-intervention-errors'
import { formatShortDate } from '#/pages/maintenance/lib/recap-lines'
import { FieldInterventionPanel } from '#/pages/maintenance/ui/field-intervention-panel'
import { todayIsoDate } from '#/pages/planning/types'

interface FieldInterventionFeatureProps {
	organizationId: string
	/** A task that carries a maintenance line. */
	taskId: string
}

/**
 * "Intervention faite" on one maintenance task of the field day. The visit is
 * only proposed, so nothing else on the screen changes.
 */
export function FieldInterventionFeature({
	organizationId,
	taskId,
}: FieldInterventionFeatureProps) {
	const propose = useProposeIntervention()
	const [isOpen, setIsOpen] = useState(false)
	const [date, setDate] = useState('')
	const [sentOn, setSentOn] = useState<string | null>(null)

	// Read when the form opens, not when the screen mounts: a phone left on
	// the day list overnight must not offer yesterday as "today".
	const today = todayIsoDate()

	function open() {
		propose.reset()
		setDate(today)
		setIsOpen(true)
	}

	function submit() {
		propose
			.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				body: { done_on: date },
			} as never)
			.then(() => {
				setSentOn(formatShortDate(date))
				setIsOpen(false)
			})
			.catch(() => {
				// Shown through `propose.error`; the form stays open to correct the date.
			})
	}

	return (
		<FieldInterventionPanel
			taskId={taskId}
			isOpen={isOpen}
			date={date}
			maxDate={today}
			isSubmitting={propose.isPending}
			error={proposeErrorMessage(propose.error)}
			sentOn={sentOn}
			onOpen={open}
			onCancel={() => setIsOpen(false)}
			onDateChange={setDate}
			onSubmit={submit}
		/>
	)
}
