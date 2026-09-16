import { TriangleAlert } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Field } from '#/components/ui/field'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Textarea } from '#/components/ui/textarea'

export interface RunNowDialogProps {
	open: boolean
	events: Schemas.EventDescriptorResponse[]
	triggerEventNames: string[]
	isStarting: boolean
	startError: string | null
	onOpenChange: (open: boolean) => void
	onConfirm: (payload: unknown) => void
}

function stringifyPayload(value: unknown): string {
	return JSON.stringify(value ?? {}, null, 2)
}

function subscribedEventsOf(
	events: Schemas.EventDescriptorResponse[],
	triggerEventNames: string[],
): Schemas.EventDescriptorResponse[] {
	return triggerEventNames
		.map((name) => events.find((event) => event.name === name))
		.filter(
			(event): event is Schemas.EventDescriptorResponse => event !== undefined,
		)
}

export function RunNowDialog({
	open,
	events,
	triggerEventNames,
	isStarting,
	startError,
	onOpenChange,
	onConfirm,
}: RunNowDialogProps) {
	const [wasOpen, setWasOpen] = useState(false)
	const [selectedEventName, setSelectedEventName] = useState<string | null>(
		null,
	)
	const [payloadText, setPayloadText] = useState('')
	const [parseError, setParseError] = useState<string | null>(null)

	const subscribedEvents = subscribedEventsOf(events, triggerEventNames)

	if (open && !wasOpen) {
		setWasOpen(true)
		const initial = subscribedEvents[0] ?? null
		setSelectedEventName(initial?.name ?? null)
		setPayloadText(stringifyPayload(initial?.payload_example))
		setParseError(null)
	} else if (!open && wasOpen) {
		setWasOpen(false)
	}

	function handleSelectEvent(name: string) {
		setSelectedEventName(name)
		const found = events.find((event) => event.name === name)
		setPayloadText(stringifyPayload(found?.payload_example))
		setParseError(null)
	}

	function handleConfirm() {
		const trimmed = payloadText.trim()
		try {
			const parsed = trimmed === '' ? null : JSON.parse(trimmed)
			setParseError(null)
			onConfirm(parsed)
		} catch {
			setParseError('Cette charge utile n’est pas un JSON valide.')
		}
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>Exécuter maintenant</DialogTitle>
					<DialogDescription>
						Démarre une exécution réelle de ce workflow — pas un test.
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-4 overflow-y-auto py-4">
					<div
						role="alert"
						className="flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-700"
					>
						<TriangleAlert className="mt-0.5 size-4 shrink-0" />
						<span>
							Les connecteurs de ce workflow agiront pour de vrai — appels HTTP,
							écritures dans Odoo. Cette exécution ne peut pas être annulée une
							fois lancée.
						</span>
					</div>

					{subscribedEvents.length > 0 ? (
						<Field label="Événement déclencheur" htmlFor="run-now-event">
							<Select
								value={selectedEventName ?? undefined}
								onValueChange={handleSelectEvent}
							>
								<SelectTrigger id="run-now-event" className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{subscribedEvents.map((event) => (
										<SelectItem key={event.name} value={event.name}>
											{event.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</Field>
					) : (
						<p className="text-sm text-muted-foreground">
							Aucun événement configuré pour ce workflow — la charge utile part
							vide.
						</p>
					)}

					<Field label="Charge utile (JSON)" htmlFor="run-now-payload">
						<Textarea
							id="run-now-payload"
							rows={10}
							value={payloadText}
							onChange={(event) => setPayloadText(event.target.value)}
							className="font-mono text-xs"
						/>
					</Field>

					{parseError ? (
						<p className="text-sm text-destructive">{parseError}</p>
					) : null}
					{startError ? (
						<p className="text-sm text-destructive">{startError}</p>
					) : null}
				</div>

				<DialogFooter className="border-t pt-4">
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<RequirePermission permission="MANAGE_AUTOMATION">
						<Button disabled={isStarting} onClick={handleConfirm}>
							{isStarting ? 'Exécution…' : 'Exécuter maintenant'}
						</Button>
					</RequirePermission>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
