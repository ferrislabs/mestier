import { isIsoDate } from '#/pages/projects/lib/dates'

export const NO_PERSON = 'aucun'

export type FormErrors<K extends string> = {
	byField: Partial<Record<K, string>>
	other: string[]
}

export type ErrorRule<K extends string> = readonly [prefix: string, field: K]

export function groupErrors<K extends string>(
	messages: readonly string[],
	rules: readonly ErrorRule<K>[],
): FormErrors<K> {
	const byField: Partial<Record<K, string>> = {}
	const other: string[] = []
	for (const message of messages) {
		const rule = rules.find(([prefix]) => message.startsWith(prefix))
		if (!rule) other.push(message)
		else if (byField[rule[1]] === undefined) byField[rule[1]] = message
	}
	return { byField, other }
}

export function hasErrors<K extends string>(errors: FormErrors<K>): boolean {
	return Object.keys(errors.byField).length + errors.other.length > 0
}

export function dateErrorField(startDate: string): 'startDate' | 'endDate' {
	const start = startDate.trim()
	return start !== '' && !isIsoDate(start) ? 'startDate' : 'endDate'
}

export function dateOrNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function personIdOf(value: string): string | null {
	return value === NO_PERSON ? null : value
}

export function personValueOf(id: string | null): string {
	return id ?? NO_PERSON
}
