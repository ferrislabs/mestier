import type { Schemas } from '#/api/api.client'
import {
	type TriggerCopyKind,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import { resolveTriggerExample } from '#/pages/automation/lib/data-tree'
import {
	type ScheduleRule,
	scheduleExample,
	scheduleSubtitle,
} from '#/pages/automation/lib/schedule'
import {
	sampleBody,
	type WebhookRule,
} from '#/pages/automation/lib/webhook-contract'

export function triggerKindOf(kind: Schemas.TriggerKindDto): TriggerCopyKind {
	if (kind === 'Manual') return 'manual'
	if ('Events' in kind) return 'events'
	if ('Webhook' in kind) return 'webhook'
	return 'schedule'
}

export function webhookOf(
	trigger: Schemas.PlacedTriggerDto,
): WebhookRule | null {
	return typeof trigger.kind === 'object' && 'Webhook' in trigger.kind
		? trigger.kind.Webhook
		: null
}

export function scheduleOf(
	trigger: Schemas.PlacedTriggerDto,
): ScheduleRule | null {
	return typeof trigger.kind === 'object' && 'Schedule' in trigger.kind
		? trigger.kind.Schedule
		: null
}

export function triggerEventNames(trigger: Schemas.PlacedTriggerDto): string[] {
	return typeof trigger.kind === 'object' && 'Events' in trigger.kind
		? trigger.kind.Events
		: []
}

export function isManualTrigger(trigger: Schemas.PlacedTriggerDto): boolean {
	return trigger.kind === 'Manual'
}

export function isEventsTrigger(trigger: Schemas.PlacedTriggerDto): boolean {
	return triggerKindOf(trigger.kind) === 'events'
}

export function findEvent(
	events: readonly Schemas.EventDescriptorResponse[],
	name: string,
): Schemas.EventDescriptorResponse | undefined {
	return events.find((event) => event.name === name)
}

export interface TriggerPresentation {
	label: string
	subtitle: string
}

export function triggerPresentation(
	trigger: Schemas.PlacedTriggerDto,
	events: readonly Schemas.EventDescriptorResponse[],
): TriggerPresentation {
	const kind = triggerKindOf(trigger.kind)
	if (kind === 'manual' || kind === 'webhook') {
		const { label, subtitle } = triggerCopy(kind)
		return { label, subtitle }
	}
	if (kind === 'schedule') {
		const { label } = triggerCopy('schedule')
		const schedule = scheduleOf(trigger)
		return {
			label,
			subtitle: schedule ? scheduleSubtitle(schedule) : '',
		}
	}
	const eventsLabel = triggerCopy('events').label
	const names = triggerEventNames(trigger)
	if (names.length === 0) {
		return {
			label: eventsLabel,
			subtitle: 'Cliquez pour choisir l’événement',
		}
	}
	if (names.length === 1) {
		const name = names[0]
		return {
			label: findEvent(events, name)?.label ?? name,
			subtitle: `Event Mestier · ${name}`,
		}
	}
	return {
		label: eventsLabel,
		subtitle: `${names.length} événements`,
	}
}

export function triggerExample(
	trigger: Schemas.PlacedTriggerDto,
	events: readonly Schemas.EventDescriptorResponse[],
	now: Date = new Date(),
): unknown {
	const webhook = webhookOf(trigger)
	if (webhook) return sampleBody(webhook.contract ?? [])
	const schedule = scheduleOf(trigger)
	if (schedule) return scheduleExample(schedule, now)
	return resolveTriggerExample(events, triggerEventNames(trigger))
}
