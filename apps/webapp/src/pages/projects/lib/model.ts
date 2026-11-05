export type IsoDate = string

export interface Person {
	id: string
	name: string
	role: string
	hue: number
}

export type ProjectStatus = 'active' | 'archived'

export type MissionStatus = 'todo' | 'in_progress' | 'done'

export interface Mission {
	id: string
	projectId: string
	title: string
	description: string
	status: MissionStatus
	startDate: IsoDate | null
	endDate: IsoDate | null
	responsibleId: string | null
	position: number
	createdAt: IsoDate
}

export type MissionDraft = Pick<
	Mission,
	'title' | 'description' | 'status' | 'startDate' | 'endDate' | 'responsibleId'
>

export interface ProjectExtras {
	place: string
	description: string
	startDate: IsoDate | null
	endDate: IsoDate | null
	ownerId: string | null
}

export const EMPTY_EXTRAS: ProjectExtras = {
	place: '',
	description: '',
	startDate: null,
	endDate: null,
	ownerId: null,
}
