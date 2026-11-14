import type { QuestionNode, SurveyAnswer } from '#/pages/survey/types'

export type Draft = string | string[]
export type Drafts = Readonly<Record<string, Draft>>
export type AnswerMap = Record<string, SurveyAnswer>

export const DEFAULT_MAX_PHOTOS = 5

export const MESSAGES = {
	required: 'Cette question est obligatoire.',
	choose: 'Choisissez une option.',
	chooseMany: 'Choisissez au moins une option.',
	unknownOption: 'Choisissez une option proposée.',
	photoRequired: 'Ajoutez au moins une photo.',
	photoPending: 'Attendez la fin de l’envoi des photos.',
	number: 'Saisissez un nombre.',
	date: 'Saisissez une date valide.',
	dateTime: 'Saisissez une date et une heure valides.',
	phone: 'Saisissez un numéro de téléphone valide, de 6 à 15 chiffres.',
	email: 'Saisissez une adresse e-mail valide, par exemple nom@exemple.fr.',
} as const

export function maxPhotos(node: QuestionNode): number {
	return node.max_photos ?? DEFAULT_MAX_PHOTOS
}

export function tooManyPhotosMessage(limit: number): string {
	return `Vous pouvez joindre au plus ${limit} photo${limit > 1 ? 's' : ''}.`
}

function text(draft: Draft | undefined): string {
	return typeof draft === 'string' ? draft : ''
}

function list(draft: Draft | undefined): string[] {
	return Array.isArray(draft) ? draft : []
}

function isListKind(node: QuestionNode): boolean {
	return node.kind === 'MULTIPLE_CHOICE' || node.kind === 'PHOTO'
}

export function isBlank(node: QuestionNode, draft: Draft | undefined): boolean {
	return isListKind(node) ? list(draft).length === 0 : text(draft).trim() === ''
}

function parseNumber(value: string): number | null {
	const normalized = value.trim().replace(',', '.')
	if (!/^[-+]?(\d+\.?\d*|\.\d+)(e[-+]?\d+)?$/i.test(normalized)) return null
	const parsed = Number(normalized)
	return Number.isFinite(parsed) ? parsed : null
}

function isDate(value: string): boolean {
	const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
	if (!match) return false
	const [year, month, day] = match.slice(1).map(Number) as [
		number,
		number,
		number,
	]
	const date = new Date(Date.UTC(year, month - 1, day))
	return (
		date.getUTCFullYear() === year &&
		date.getUTCMonth() === month - 1 &&
		date.getUTCDate() === day
	)
}

function isDateTime(value: string): boolean {
	const match = /^(\d{4}-\d{2}-\d{2})T(\d{2}):(\d{2})(:\d{2})?$/.exec(value)
	if (!match) return false
	const [date, hours, minutes] = match.slice(1) as [string, string, string]
	return (
		isDate(date) &&
		Number(hours) < 24 &&
		Number(minutes) < 60 &&
		!Number.isNaN(new Date(value).getTime())
	)
}

export function isPhone(value: string): boolean {
	const trimmed = value.trim()
	let digits = 0
	for (const [index, char] of [...trimmed].entries()) {
		if (char >= '0' && char <= '9') digits += 1
		else if (char === '+' && index === 0) continue
		else if (!' -()'.includes(char)) return false
	}
	return digits >= 6 && digits <= 15
}

export function isEmail(value: string): boolean {
	const normalized = value.trim()
	const parts = normalized.split('@')
	return (
		parts.length === 2 &&
		parts[0] !== '' &&
		parts[1] !== '' &&
		!/\s/.test(normalized)
	)
}

function formatError(
	node: QuestionNode,
	draft: Draft | undefined,
): string | null {
	const value = text(draft).trim()
	const options = node.options ?? []
	switch (node.kind) {
		case 'NUMBER':
			return parseNumber(value) === null ? MESSAGES.number : null
		case 'DATE':
			return isDate(value) ? null : MESSAGES.date
		case 'DATETIME':
			return isDateTime(value) ? null : MESSAGES.dateTime
		case 'PHONE':
			return isPhone(value) ? null : MESSAGES.phone
		case 'EMAIL':
			return isEmail(value) ? null : MESSAGES.email
		case 'SINGLE_CHOICE':
			return options.includes(value) ? null : MESSAGES.unknownOption
		case 'MULTIPLE_CHOICE': {
			const choices = list(draft)
			const valid =
				choices.every((choice) => options.includes(choice)) &&
				new Set(choices).size === choices.length
			return valid ? null : MESSAGES.unknownOption
		}
		case 'PHOTO':
			return list(draft).length > maxPhotos(node)
				? tooManyPhotosMessage(maxPhotos(node))
				: null
		default:
			return null
	}
}

function requiredMessage(node: QuestionNode): string {
	if (node.kind === 'SINGLE_CHOICE') return MESSAGES.choose
	if (node.kind === 'MULTIPLE_CHOICE') return MESSAGES.chooseMany
	if (node.kind === 'PHOTO') return MESSAGES.photoRequired
	return MESSAGES.required
}

export function validateDraft(
	node: QuestionNode,
	draft: Draft | undefined,
	pendingUploads: number,
): string | null {
	if (node.kind === 'PHOTO' && pendingUploads > 0) return MESSAGES.photoPending
	if (isBlank(node, draft)) return node.required ? requiredMessage(node) : null
	return formatError(node, draft)
}

export function toAnswer(
	node: QuestionNode,
	draft: Draft | undefined,
): SurveyAnswer | null {
	if (isBlank(node, draft) || formatError(node, draft) !== null) return null
	const value = text(draft).trim()
	switch (node.kind) {
		case 'TEXT':
		case 'LONG_TEXT':
			return { Text: value }
		case 'NUMBER':
			return { Number: parseNumber(value) as number }
		case 'DATE':
			return { Date: value }
		case 'DATETIME':
			return { DateTime: new Date(value).toISOString() }
		case 'PHONE':
			return { Phone: value }
		case 'EMAIL':
			return { Email: value }
		case 'SINGLE_CHOICE':
			return { Choice: value }
		case 'MULTIPLE_CHOICE':
			return { Choices: [...list(draft)] }
		case 'PHOTO':
			return { Photos: [...list(draft)] }
	}
}

export function buildAnswers(
	nodes: readonly QuestionNode[],
	drafts: Drafts,
): AnswerMap {
	const answers: AnswerMap = {}
	for (const node of nodes) {
		const answer = toAnswer(node, drafts[node.id])
		if (answer) answers[node.id] = answer
	}
	return answers
}

function pad(value: number): string {
	return String(value).padStart(2, '0')
}

function localDateTime(iso: string): string {
	const date = new Date(iso)
	if (Number.isNaN(date.getTime())) return ''
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`
}

function strings(value: unknown): string[] {
	return Array.isArray(value)
		? value.filter((item): item is string => typeof item === 'string')
		: []
}

function draftOf(node: QuestionNode, answer: unknown): Draft | null {
	if (typeof answer !== 'object' || answer === null) return null
	const [tag, value] = Object.entries(answer)[0] ?? []
	switch (tag) {
		case 'Text':
		case 'Phone':
		case 'Email':
		case 'Choice':
		case 'Date':
			return typeof value === 'string' ? value : null
		case 'Number':
			return typeof value === 'number' ? String(value) : null
		case 'DateTime':
			return typeof value === 'string' ? localDateTime(value) : null
		case 'Choices':
		case 'Photos':
			return isListKind(node) ? strings(value) : null
		default:
			return null
	}
}

export function draftsFromAnswers(
	nodes: readonly QuestionNode[],
	answers: Readonly<Record<string, unknown>>,
): Record<string, Draft> {
	const drafts: Record<string, Draft> = {}
	for (const node of nodes) {
		const draft = draftOf(node, answers[node.id])
		if (draft !== null) drafts[node.id] = draft
	}
	return drafts
}
