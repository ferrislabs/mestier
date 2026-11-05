import type { Project } from '#/hooks/use-projects'
import { isIsoDate } from '#/pages/projects/lib/dates'
import {
	dateErrorField,
	dateOrNull,
	type FormErrors,
	groupErrors,
	personIdOf,
	personValueOf,
} from '#/pages/projects/lib/form-model'
import type { ProjectExtras } from '#/pages/projects/lib/model'
import { CLIENT_INTERNAL } from '#/pages/projects/lib/project-list'
import type { ProjectFormValues } from '#/pages/projects/types'

export const NO_QUOTE = 'aucun'

export interface ProjectFullForm extends ProjectFormValues {
	place: string
	description: string
	startDate: string
	endDate: string
	ownerId: string
}

export type ProjectField = 'name' | 'startDate' | 'endDate'

export function emptyFullForm(
	ownerId: string | null,
	customerId: string,
): ProjectFullForm {
	return {
		name: '',
		customerId,
		quoteId: '',
		customerContextId: '',
		place: '',
		description: '',
		startDate: '',
		endDate: '',
		ownerId: personValueOf(ownerId),
	}
}

export function fullFormOf(
	project: Project,
	extras: ProjectExtras,
): ProjectFullForm {
	return {
		name: project.name,
		customerId: project.customer_id ?? '',
		quoteId: project.quote_id ?? '',
		customerContextId: project.customer_context_id ?? '',
		place: extras.place,
		description: extras.description,
		startDate: extras.startDate ?? '',
		endDate: extras.endDate ?? '',
		ownerId: personValueOf(extras.ownerId),
	}
}

export function extrasOfForm(form: ProjectFullForm): ProjectExtras {
	return {
		place: form.place.trim(),
		description: form.description.trim(),
		startDate: dateOrNull(form.startDate),
		endDate: dateOrNull(form.endDate),
		ownerId: personIdOf(form.ownerId),
	}
}

export function validateProjectFields(form: ProjectFullForm): string[] {
	const errors: string[] = []
	if (form.name.trim() === '') errors.push('Le nom du projet est obligatoire.')
	const start = dateOrNull(form.startDate)
	const end = dateOrNull(form.endDate)
	for (const date of [start, end]) {
		if (date !== null && !isIsoDate(date))
			errors.push('Une date n’est pas valide.')
	}
	if (start !== null && end !== null && end < start)
		errors.push('La fin ne peut pas précéder le début.')
	return errors
}

export function projectFormErrors(
	form: ProjectFullForm,
): FormErrors<ProjectField> {
	return groupErrors<ProjectField>(validateProjectFields(form), [
		['Le nom', 'name'],
		['La fin', 'endDate'],
		['Une date', dateErrorField(form.startDate)],
	])
}

export function customerSelectValue(customerId: string): string {
	return customerId === '' ? CLIENT_INTERNAL : customerId
}

export function customerIdOfSelect(value: string): string {
	return value === CLIENT_INTERNAL ? '' : value
}

export function quoteSelectValue(quoteId: string): string {
	return quoteId === '' ? NO_QUOTE : quoteId
}

export function quoteIdOfSelect(value: string): string {
	return value === NO_QUOTE ? '' : value
}

export type NewProjectSearch = { client?: string }

export function parseNewSearch(raw: Record<string, unknown>): NewProjectSearch {
	const client = typeof raw.client === 'string' ? raw.client.trim() : ''
	return client === '' ? {} : { client }
}

function trimmedForm(form: ProjectFullForm): ProjectFullForm {
	return {
		...form,
		name: form.name.trim(),
		place: form.place.trim(),
		description: form.description.trim(),
		startDate: form.startDate.trim(),
		endDate: form.endDate.trim(),
	}
}

export function sameFullForm(a: ProjectFullForm, b: ProjectFullForm): boolean {
	return JSON.stringify(trimmedForm(a)) === JSON.stringify(trimmedForm(b))
}

export function canSaveFullForm(
	form: ProjectFullForm,
	stored: ProjectFullForm,
): boolean {
	return !sameFullForm(form, stored) && validateProjectFields(form).length === 0
}

export function datesError(form: ProjectFullForm): string | undefined {
	const messages = validateProjectFields({ ...form, name: 'x' })
	return messages.length > 0 ? [...new Set(messages)].join(' ') : undefined
}
