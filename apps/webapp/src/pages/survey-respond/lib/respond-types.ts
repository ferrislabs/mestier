import type { RefObject } from 'react'
import type { QuestionNode } from '#/pages/survey/types'
import type { Draft } from '#/pages/survey-respond/lib/answer-draft'

export type PhotoStatus = 'converting' | 'uploading' | 'done' | 'error'

export interface PhotoItem {
	id: string
	name: string
	status: PhotoStatus
	previewUrl: string | null
	key: string | null
	message: string | null
}

export type SaveState = 'idle' | 'saving' | 'saved' | 'error'

export type RespondPhase =
	| { name: 'loading' }
	| { name: 'unavailable' }
	| { name: 'retry'; message: string }
	| { name: 'expired' }
	| { name: 'blocked'; reason: string }
	| { name: 'answering' }
	| { name: 'completed'; message: string }

export interface StepProgress {
	position: number
	last?: number
	total: number | null
}

export interface ScreenItem {
	question: QuestionNode
	draft: Draft | undefined
	photos: readonly PhotoItem[]
	photoNotice: string | null
	error: string | null
}

export interface RespondentView {
	phase: RespondPhase
	preview: boolean
	survey: { title: string; description?: string | null } | null
	screen: readonly ScreenItem[]
	progress: StepProgress | null
	isFirst: boolean
	isLast: boolean
	busy: boolean
	actionError: string | null
	saveState: SaveState
	saveMessage: string | null
	headingRef: RefObject<HTMLHeadingElement | null>
	setDraft: (questionId: string, draft: Draft) => void
	next: () => void
	back: () => void
	retrySave: () => void
	addPhotos: (questionId: string, files: readonly File[]) => void
	removePhoto: (questionId: string, id: string) => void
	restart: () => void
	reload: () => void
}
