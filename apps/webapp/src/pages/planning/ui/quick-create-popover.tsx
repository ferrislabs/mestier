import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Popover, PopoverAnchor, PopoverContent } from '#/components/ui/popover'
import { TIME_OPTIONS } from '#/pages/planning/lib/task-form'
import {
	type AssigneeOption,
	AssigneePicker,
} from '#/pages/planning/ui/assignee-picker'

export interface QuickCreateAnchor {
	x: number
	y: number
}

export interface QuickCreateDraft {
	title: string
	date: string
	startTime: string
	endTime: string
	assigneeResourceIds: string[]
}

export interface QuickCreatePopoverProps {
	/** `null` closes the popover. A new value — even one carrying the same
	 * date/time as before — reseeds the draft: each click starts fresh. */
	anchor: (QuickCreateAnchor & { date: string; startTime: string }) | null
	assigneeOptions: AssigneeOption[]
	isSaving?: boolean
	error?: string | null
	onCreate: (draft: QuickCreateDraft) => void
	onMoreOptions: (draft: QuickCreateDraft) => void
	onClose: () => void
}

/**
 * The calendar's fast path for a task: click empty time and get a small form
 * at that spot instead of the full sheet, the way Google Calendar's own
 * quick-add works. "Autres options" is the escalation hatch — anything this
 * form doesn't cover (client, project, recurrence, labels) still lives one
 * click away, carrying over what was already typed here rather than
 * discarding it.
 */
export function QuickCreatePopover({
	anchor,
	assigneeOptions,
	isSaving,
	error,
	onCreate,
	onMoreOptions,
	onClose,
}: QuickCreatePopoverProps) {
	const [title, setTitle] = useState('')
	const [startTime, setStartTime] = useState('')
	const [endTime, setEndTime] = useState('')
	const [assigneeResourceIds, setAssigneeResourceIds] = useState<string[]>([])
	// Re-seeds the three fields above whenever a new anchor point arrives —
	// keyed on the anchor object itself (a fresh object every click, even for
	// the same slot) so a second click on the exact same spot still clears a
	// half-typed title rather than reopening it. Seeded from `undefined`, not
	// from `anchor` itself — starting equal to the very first anchor would
	// skip seeding on mount, leaving the fields at their own blank defaults
	// for the first slot ever clicked.
	const [seededAnchor, setSeededAnchor] = useState<typeof anchor | undefined>(
		undefined,
	)

	if (anchor !== seededAnchor) {
		setSeededAnchor(anchor)
		setTitle('')
		setStartTime(anchor?.startTime ?? '')
		setEndTime(anchor ? addOneHour(anchor.startTime) : '')
		setAssigneeResourceIds([])
	}

	if (!anchor) return null

	const draft: QuickCreateDraft = {
		title: title.trim(),
		date: anchor.date,
		startTime,
		endTime,
		assigneeResourceIds,
	}
	const canSave = draft.title !== '' && startTime !== '' && endTime > startTime

	return (
		<Popover
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<PopoverAnchor asChild>
				<div
					style={{
						position: 'fixed',
						left: anchor.x,
						top: anchor.y,
						width: 0,
						height: 0,
					}}
				/>
			</PopoverAnchor>
			<PopoverContent
				align="start"
				className="w-80 rounded-none p-4 shadow-lg"
				onInteractOutside={(event) => {
					// The assignee picker below is its own Popover — its options
					// render in a separate Radix popper, positioned outside this
					// content's own DOM node. Without this, picking someone reads
					// as a click outside this popover and closes it before the
					// pick can register.
					const target = event.target as HTMLElement | null
					if (target?.closest('[data-radix-popper-content-wrapper]')) {
						event.preventDefault()
					}
				}}
			>
				<form
					className="space-y-3"
					onSubmit={(event) => {
						event.preventDefault()
						if (canSave) onCreate(draft)
					}}
				>
					<Input
						autoFocus
						placeholder="Ajouter un titre"
						value={title}
						onChange={(event) => setTitle(event.target.value)}
					/>

					<div className="flex items-center gap-2">
						<TimeSelect value={startTime} onChange={setStartTime} />
						<span className="text-sm text-muted-foreground">–</span>
						<TimeSelect value={endTime} onChange={setEndTime} />
					</div>

					<AssigneePicker
						options={assigneeOptions}
						selectedResourceIds={assigneeResourceIds}
						onToggle={(resourceId) =>
							setAssigneeResourceIds((current) =>
								current.includes(resourceId)
									? current.filter((id) => id !== resourceId)
									: [...current, resourceId],
							)
						}
					/>

					{error ? <p className="text-sm text-destructive">{error}</p> : null}

					<div className="flex items-center justify-between pt-1">
						<Button
							type="button"
							variant="ghost"
							size="sm"
							onClick={() => onMoreOptions(draft)}
						>
							Autres options
						</Button>
						<Button type="submit" size="sm" disabled={!canSave || isSaving}>
							Enregistrer
						</Button>
					</div>
				</form>
			</PopoverContent>
		</Popover>
	)
}

function TimeSelect({
	value,
	onChange,
}: {
	value: string
	onChange: (value: string) => void
}) {
	return (
		<select
			aria-label="Heure"
			value={value}
			onChange={(event) => onChange(event.target.value)}
			className="h-9 flex-1 rounded-md border border-input bg-card px-2 text-sm"
		>
			{TIME_OPTIONS.map((option) => (
				<option key={option} value={option}>
					{option}
				</option>
			))}
		</select>
	)
}

function addOneHour(time: string): string {
	const [hours = '0', minutes = '0'] = time.split(':')
	const totalMinutes = (Number(hours) * 60 + Number(minutes) + 60) % (24 * 60)
	const clampedHours = Math.floor(totalMinutes / 60)
	const remainingMinutes = totalMinutes % 60
	return `${String(clampedHours).padStart(2, '0')}:${String(remainingMinutes).padStart(2, '0')}`
}
