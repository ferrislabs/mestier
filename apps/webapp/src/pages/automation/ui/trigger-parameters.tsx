import { X } from 'lucide-react'
import { useId } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import type { CredentialStatus } from '#/pages/automation/lib/credential-meta'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import {
	findEvent,
	scheduleOf,
	triggerEventNames,
	triggerKindOf,
	webhookOf,
} from '#/pages/automation/lib/trigger-events'
import type { TriggerFieldErrors } from '#/pages/automation/lib/validation'
import { ScheduleParameters } from '#/pages/automation/ui/schedule-parameters'
import { WebhookParameters } from '#/pages/automation/ui/webhook-parameters'

const EVENT_HINT = 'Le workflow reçoit le payload contractuel de cet événement.'

function EventCard({
	name,
	event,
	several,
	onChange,
	onRemove,
}: {
	name: string
	event: Schemas.EventDescriptorResponse | undefined
	several: boolean
	onChange: () => void
	onRemove: () => void
}) {
	return (
		<div className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2">
			<span className="min-w-0 flex-1">
				<span className="block truncate font-medium text-sm">
					{event?.label ?? name}
				</span>
				<span className="block truncate font-mono text-muted-foreground text-xs">
					{event ? `${event.name} · version ${event.version}` : name}
				</span>
			</span>
			<Button
				type="button"
				aria-label={
					several ? `Changer l’événement ${name}` : 'Changer l’événement'
				}
				variant="outline"
				size="sm"
				onClick={onChange}
			>
				Changer
			</Button>
			{several && (
				<Button
					type="button"
					aria-label={`Retirer l’événement ${name}`}
					variant="ghost"
					size="icon-sm"
					onClick={onRemove}
				>
					<X />
				</Button>
			)}
		</div>
	)
}

function EventParameters({
	trigger,
	events,
	onChooseEvent,
	onRemoveEvent,
}: {
	trigger: Schemas.PlacedTriggerDto
	events: readonly Schemas.EventDescriptorResponse[]
	onChooseEvent: (replacing: string | null) => void
	onRemoveEvent: (name: string) => void
}) {
	const controlId = useId()
	const names = triggerEventNames(trigger)
	return (
		<div className="grid gap-2">
			<Label htmlFor={controlId}>Événement</Label>
			<div id={controlId} className="grid gap-2">
				{names.length === 0 ? (
					<div className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2">
						<span className="block min-w-0 flex-1 truncate font-medium text-sm">
							Aucun événement choisi
						</span>
						<Button
							type="button"
							aria-label="Choisir l’événement"
							variant="outline"
							size="sm"
							onClick={() => onChooseEvent(null)}
						>
							Choisir
						</Button>
					</div>
				) : (
					<>
						{names.map((name) => (
							<EventCard
								key={name}
								name={name}
								event={findEvent(events, name)}
								several={names.length > 1}
								onChange={() => onChooseEvent(name)}
								onRemove={() => onRemoveEvent(name)}
							/>
						))}
						<div>
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={() => onChooseEvent(null)}
							>
								Ajouter un événement
							</Button>
						</div>
					</>
				)}
			</div>
			<p className="text-muted-foreground text-xs">{EVENT_HINT}</p>
		</div>
	)
}

function messagesFor(
	issues: readonly NodeIssue[],
	parameter: string,
	server: string | null,
): string[] {
	const own = issues
		.filter((issue) => issue.parameter === parameter)
		.map((issue) => issue.message)
	return [...new Set(server ? [...own, server] : own)]
}

export interface TriggerParametersProps {
	trigger: Schemas.PlacedTriggerDto
	events: readonly Schemas.EventDescriptorResponse[]
	origin: string
	credentials: Schemas.CredentialResponse[]
	credentialHint: string
	credentialStatusOf: (credentialId: string) => CredentialStatus
	issues: readonly NodeIssue[]
	errors: TriggerFieldErrors
	onChooseEvent: (replacing: string | null) => void
	onRemoveEvent: (name: string) => void
	onChange: (kind: Schemas.TriggerKindDto) => void
	onCreateCredential: () => void
}

export function TriggerParameters({
	trigger,
	events,
	origin,
	credentials,
	credentialHint,
	credentialStatusOf,
	issues,
	errors,
	onChooseEvent,
	onRemoveEvent,
	onChange,
	onCreateCredential,
}: TriggerParametersProps) {
	const kind = triggerKindOf(trigger.kind)
	const webhook = webhookOf(trigger)
	const schedule = scheduleOf(trigger)

	if (webhook) {
		return (
			<WebhookParameters
				rule={webhook}
				origin={origin}
				credentials={credentials}
				credentialHint={credentialHint}
				credentialStatusOf={credentialStatusOf}
				messages={{
					path: messagesFor(issues, 'path', errors.path),
					credential: errors.credential,
					contract: errors.contract ? [errors.contract] : [],
					keys: errors.keys,
				}}
				onChange={(rule) => onChange({ Webhook: rule })}
				onCreateCredential={onCreateCredential}
			/>
		)
	}
	if (schedule) {
		return (
			<ScheduleParameters
				rule={schedule}
				messages={{
					time: messagesFor(issues, 'time', errors.time),
					weekday: errors.weekday ? [errors.weekday] : [],
					timezone: errors.timezone ? [errors.timezone] : [],
				}}
				onChange={(rule) => onChange({ Schedule: rule })}
			/>
		)
	}
	if (kind !== 'events') return null
	return (
		<EventParameters
			trigger={trigger}
			events={events}
			onChooseEvent={onChooseEvent}
			onRemoveEvent={onRemoveEvent}
		/>
	)
}
