import type { Endpoints } from '#/api/api.client'
import type { ResponseStatus } from '#/pages/survey/types'
import { RespondError } from '#/pages/survey-respond/lib/respond-errors'
import {
	forgetResumeToken,
	readResumeToken,
	saveResumeToken,
} from '#/pages/survey-respond/lib/resume-store'
import { authStore } from '#/store/auth.store'

export type RespondentSurvey =
	Endpoints.get_OpenPublicSurvey['responses'][200]['data']

export interface RespondentResponse {
	handle: string
	answers: Record<string, unknown>
	status: ResponseStatus
}

export interface RespondSource {
	loadSurvey: () => Promise<RespondentSurvey>
	restore: () => Promise<RespondentResponse | null>
	start: () => Promise<RespondentResponse>
	save: (handle: string, answers: Record<string, unknown>) => Promise<void>
	complete: (handle: string) => Promise<void>
	uploadPhoto: (handle: string, photo: Blob) => Promise<string>
	forget: () => void
}

interface SendInit {
	method?: string
	body?: BodyInit
	contentType?: string
	authenticated: boolean
}

function record(value: unknown): Record<string, unknown> | null {
	return typeof value === 'object' && value !== null
		? (value as Record<string, unknown>)
		: null
}

function stringField(
	value: Record<string, unknown> | null,
	key: string,
): string | null {
	const field = value?.[key]
	return typeof field === 'string' ? field : null
}

async function send<T>(path: string, init: SendInit): Promise<T> {
	const headers = new Headers({ Accept: 'application/json' })
	if (init.contentType) headers.set('Content-Type', init.contentType)
	if (init.authenticated) {
		const token = authStore.state.accessToken
		if (token) headers.set('Authorization', `Bearer ${token}`)
	}
	let response: Response
	try {
		response = await fetch(`${window.apiUrl ?? ''}${path}`, {
			method: init.method ?? 'GET',
			headers,
			body: init.body,
			credentials: init.authenticated ? 'include' : 'omit',
		})
	} catch {
		throw new RespondError(null, null, null)
	}
	const body: unknown = await response.json().catch(() => null)
	if (!response.ok) {
		const details = record(record(body)?.details)
		throw new RespondError(
			response.status,
			stringField(details, 'reason'),
			stringField(details, 'question_id'),
		)
	}
	return record(body)?.data as T
}

function json(value: unknown): Pick<SendInit, 'body' | 'contentType'> {
	return { body: JSON.stringify(value), contentType: 'application/json' }
}

function photoBody(photo: Blob): Pick<SendInit, 'body' | 'contentType'> {
	return { body: photo, contentType: photo.type || 'image/jpeg' }
}

type ResponseView =
	Endpoints.get_ResumePublicSurveyResponse['responses'][200]['data']

function asResponse(handle: string, view: ResponseView): RespondentResponse {
	return { handle, answers: view.answers ?? {}, status: view.status }
}

export function publicSource(slug: string): RespondSource {
	const base = `/api/v1/public/surveys/${encodeURIComponent(slug)}`
	const responsePath = (token: string) =>
		`${base}/responses/${encodeURIComponent(token)}`
	return {
		loadSurvey: () => send<RespondentSurvey>(base, { authenticated: false }),
		restore: async () => {
			const token = readResumeToken(slug)
			if (!token) return null
			const view = await send<ResponseView>(responsePath(token), {
				authenticated: false,
			})
			return asResponse(token, view)
		},
		start: async () => {
			const view = await send<
				Endpoints.post_StartPublicSurveyResponse['responses'][201]['data']
			>(`${base}/responses`, { method: 'POST', authenticated: false })
			saveResumeToken(slug, view.resume_token)
			return asResponse(view.resume_token, view)
		},
		save: async (token, answers) => {
			await send(responsePath(token), {
				method: 'PUT',
				authenticated: false,
				...json({ answers }),
			})
		},
		complete: async (token) => {
			await send(`${responsePath(token)}/complete`, {
				method: 'POST',
				authenticated: false,
			})
		},
		uploadPhoto: async (token, photo) => {
			const view = await send<{ key: string }>(
				`${responsePath(token)}/photos`,
				{
					method: 'POST',
					authenticated: false,
					...photoBody(photo),
				},
			)
			return view.key
		},
		forget: () => forgetResumeToken(slug),
	}
}

export function memberSource(
	organizationId: string,
	surveyId: string,
): RespondSource {
	const organization = `/api/v1/organizations/${encodeURIComponent(organizationId)}`
	const surveyPath = `${organization}/member-surveys/${encodeURIComponent(surveyId)}`
	const responsePath = (id: string) =>
		`${organization}/member-survey-responses/${encodeURIComponent(id)}`
	const start = async () => {
		const view = await send<ResponseView>(`${surveyPath}/responses`, {
			method: 'POST',
			authenticated: true,
		})
		return asResponse(view.id, view)
	}
	return {
		loadSurvey: () =>
			send<RespondentSurvey>(surveyPath, { authenticated: true }),
		restore: start,
		start,
		save: async (id, answers) => {
			await send(responsePath(id), {
				method: 'PUT',
				authenticated: true,
				...json({ answers }),
			})
		},
		complete: async (id) => {
			await send(`${responsePath(id)}/complete`, {
				method: 'POST',
				authenticated: true,
			})
		},
		uploadPhoto: async (id, photo) => {
			const view = await send<{ key: string }>(`${responsePath(id)}/photos`, {
				method: 'POST',
				authenticated: true,
				...photoBody(photo),
			})
			return view.key
		},
		forget: () => undefined,
	}
}
