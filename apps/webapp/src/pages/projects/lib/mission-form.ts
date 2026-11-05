import {
	dateErrorField,
	dateOrNull,
	type FormErrors,
	groupErrors,
	personIdOf,
	personValueOf,
} from '#/pages/projects/lib/form-model'
import { validateMissionDraft } from '#/pages/projects/lib/missions'
import type {
	Mission,
	MissionDraft,
	MissionStatus,
} from '#/pages/projects/lib/model'

export type MissionForm = {
	title: string
	description: string
	status: MissionStatus
	startDate: string
	endDate: string
	responsibleId: string
}

export type MissionField = 'title' | 'startDate' | 'endDate'

export function emptyMissionForm(responsibleId: string | null): MissionForm {
	return {
		title: '',
		description: '',
		status: 'todo',
		startDate: '',
		endDate: '',
		responsibleId: personValueOf(responsibleId),
	}
}

export function missionFormOf(mission: Mission): MissionForm {
	return {
		title: mission.title,
		description: mission.description,
		status: mission.status,
		startDate: mission.startDate ?? '',
		endDate: mission.endDate ?? '',
		responsibleId: personValueOf(mission.responsibleId),
	}
}

export function toMissionDraft(form: MissionForm): MissionDraft {
	return {
		title: form.title,
		description: form.description,
		status: form.status,
		startDate: dateOrNull(form.startDate),
		endDate: dateOrNull(form.endDate),
		responsibleId: personIdOf(form.responsibleId),
	}
}

export function missionFormErrors(form: MissionForm): FormErrors<MissionField> {
	return groupErrors<MissionField>(validateMissionDraft(toMissionDraft(form)), [
		['Le titre', 'title'],
		['La fin', 'endDate'],
		['Une date', dateErrorField(form.startDate)],
	])
}
