import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Textarea } from '#/components/ui/textarea'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'

export interface CloseValues {
	closedAt: string
	closureReasonId: string
	closureNote: string
}

interface CloseDialogProps {
	open: boolean
	title: string
	description: string
	submitLabel: string
	reasons: SelectOption[]
	initialDate: string
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: CloseValues) => void
}

export function CloseDialog({
	open,
	title,
	description,
	submitLabel,
	reasons,
	initialDate,
	isPending,
	error,
	onOpenChange,
	onSubmit,
}: CloseDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<CloseDialogBody
					title={title}
					description={description}
					submitLabel={submitLabel}
					reasons={reasons}
					initialDate={initialDate}
					isPending={isPending}
					error={error}
					onCancel={() => onOpenChange(false)}
					onSubmit={onSubmit}
				/>
			</DialogContent>
		</Dialog>
	)
}

interface CloseDialogBodyProps
	extends Omit<CloseDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function CloseDialogBody({
	title,
	description,
	submitLabel,
	reasons,
	initialDate,
	isPending,
	error,
	onCancel,
	onSubmit,
}: CloseDialogBodyProps) {
	const [closedAt, setClosedAt] = useState(initialDate)
	const [reasonId, setReasonId] = useState<string | null>(null)
	const [note, setNote] = useState('')
	const canSubmit = closedAt !== '' && reasonId !== null && !isPending

	function submit(event: FormEvent) {
		event.preventDefault()
		if (!canSubmit || reasonId === null) return
		onSubmit({ closedAt, closureReasonId: reasonId, closureNote: note })
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>{title}</DialogTitle>
				<DialogDescription>{description}</DialogDescription>
			</DialogHeader>
			<FormField id="close-date" label="Date de clôture" required>
				<Input
					id="close-date"
					type="date"
					value={closedAt}
					onChange={(event) => setClosedAt(event.target.value)}
				/>
			</FormField>
			<FormField id="close-reason" label="Motif" required>
				<OptionSelect
					id="close-reason"
					value={reasonId}
					options={reasons}
					placeholder="Choisir un motif"
					onChange={setReasonId}
				/>
			</FormField>
			<FormField id="close-note" label="Note">
				<Textarea
					id="close-note"
					value={note}
					onChange={(event) => setNote(event.target.value)}
				/>
			</FormField>
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="submit" disabled={!canSubmit}>
					{submitLabel}
				</Button>
			</DialogFooter>
		</form>
	)
}
