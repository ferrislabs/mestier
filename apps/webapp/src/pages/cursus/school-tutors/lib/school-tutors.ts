import type { Schemas } from '#/api/api.client'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	conflictMessage,
	emptyToNull,
} from '#/pages/cursus/participants/lib/forms'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'

export type SchoolTutor = Schemas.SchoolTutorResponse

export const NOT_A_SCHOOL_TUTOR = 'E_NOT_A_SCHOOL_TUTOR'

export interface SchoolTutorValues {
	memberId: string
	jobTitle: string
}

export const EMPTY_SCHOOL_TUTOR: SchoolTutorValues = {
	memberId: '',
	jobTitle: '',
}

export function toCreateSchoolTutorRequest(
	values: SchoolTutorValues,
): Schemas.CreateSchoolTutorRequest {
	return { member_id: values.memberId, job_title: emptyToNull(values.jobTitle) }
}

export function toJobTitleRequest(
	jobTitle: string,
): Schemas.UpdateSchoolTutorRequest {
	return { job_title: emptyToNull(jobTitle) }
}

export function availableMembers(
	members: readonly { id: string; display_name: string }[],
	declared: readonly { member_id: string }[],
): SelectOption[] {
	const taken = new Set(declared.map((tutor) => tutor.member_id))
	return members
		.filter((member) => !taken.has(member.id))
		.map((member) => ({ value: member.id, label: member.display_name }))
}

export function schoolTutorOptions(
	active: readonly { member_id: string; member_name: string }[],
	current: { id: string; name: string } | null,
): SelectOption[] {
	const options = active.map((tutor) => ({
		value: tutor.member_id,
		label: tutor.member_name,
	}))
	if (current && !options.some((option) => option.value === current.id)) {
		options.push({ value: current.id, label: current.name })
	}
	return options
}

export function apiErrorCode(error: unknown): string | null {
	if (typeof error !== 'object' || error === null || !('data' in error)) {
		return null
	}
	const data = (error as { data?: unknown }).data
	if (typeof data !== 'object' || data === null) return null
	const code = (data as { code?: unknown }).code
	return typeof code === 'string' ? code : null
}

export function createSchoolTutorError(error: unknown): string | null {
	return (
		conflictMessage(error, 'Ce membre est déjà déclaré comme tuteur école.') ??
		mutationErrorMessage(error)
	)
}

export interface TutorFieldErrors {
	field: string | null
	general: string | null
}

export function triadTutorErrors(error: unknown): TutorFieldErrors {
	if (apiErrorCode(error) === NOT_A_SCHOOL_TUTOR) {
		return {
			field: "Ce membre n'est pas un tuteur école actif.",
			general: null,
		}
	}
	return { field: null, general: mutationErrorMessage(error) }
}
