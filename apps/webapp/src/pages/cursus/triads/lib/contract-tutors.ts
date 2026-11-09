import type { Schemas } from '#/api/api.client'

export interface TutorEntry {
	companyTutorId: string
	isMain: boolean
}

export function normaliseTutors(tutors: TutorEntry[]): TutorEntry[] {
	if (tutors.length === 0) return tutors
	const mainIndex = Math.max(
		tutors.findIndex((tutor) => tutor.isMain),
		0,
	)
	return tutors.map((tutor, index) => ({
		...tutor,
		isMain: index === mainIndex,
	}))
}

export function tutorsToEntries(
	tutors: readonly Schemas.ContractTutorResponse[],
): TutorEntry[] {
	return normaliseTutors(
		tutors
			.filter((tutor) => !tutor.archived_at)
			.map((tutor) => ({
				companyTutorId: tutor.company_tutor_id,
				isMain: tutor.is_main,
			})),
	)
}

export function addTutor(
	tutors: TutorEntry[],
	companyTutorId: string,
): TutorEntry[] {
	if (tutors.some((tutor) => tutor.companyTutorId === companyTutorId)) {
		return tutors
	}
	return normaliseTutors([...tutors, { companyTutorId, isMain: false }])
}

export function removeTutor(
	tutors: TutorEntry[],
	companyTutorId: string,
): TutorEntry[] {
	return normaliseTutors(
		tutors.filter((tutor) => tutor.companyTutorId !== companyTutorId),
	)
}

export function setMainTutor(
	tutors: TutorEntry[],
	companyTutorId: string,
): TutorEntry[] {
	return tutors.map((tutor) => ({
		...tutor,
		isMain: tutor.companyTutorId === companyTutorId,
	}))
}

export function toTutorsRequest(
	tutors: TutorEntry[],
): Schemas.SetContractTutorsRequest {
	return {
		tutors: normaliseTutors(tutors).map((tutor) => ({
			company_tutor_id: tutor.companyTutorId,
			is_main: tutor.isMain,
		})),
	}
}
