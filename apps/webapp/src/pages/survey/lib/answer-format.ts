import { formatDate, formatDateTime } from '#/pages/survey/lib/survey-format'

export type AnswerView =
	| { type: 'empty' }
	| { type: 'text'; text: string }
	| { type: 'choices'; choices: string[] }
	| { type: 'photos'; keys: string[] }

const NUMBER = new Intl.NumberFormat('fr-FR')

function strings(value: unknown): string[] {
	return Array.isArray(value)
		? value.filter((item): item is string => typeof item === 'string')
		: []
}

export function answerView(answer: unknown): AnswerView {
	if (typeof answer !== 'object' || answer === null) return { type: 'empty' }
	const [tag, value] = Object.entries(answer)[0] ?? []
	switch (tag) {
		case 'Text':
		case 'Phone':
		case 'Email':
		case 'Choice':
			return typeof value === 'string' && value !== ''
				? { type: 'text', text: value }
				: { type: 'empty' }
		case 'Number':
			return typeof value === 'number'
				? { type: 'text', text: NUMBER.format(value) }
				: { type: 'empty' }
		case 'Date':
			return typeof value === 'string'
				? { type: 'text', text: formatDate(value) }
				: { type: 'empty' }
		case 'DateTime':
			return typeof value === 'string'
				? { type: 'text', text: formatDateTime(value) }
				: { type: 'empty' }
		case 'Choices': {
			const choices = strings(value)
			return choices.length > 0
				? { type: 'choices', choices }
				: { type: 'empty' }
		}
		case 'Photos': {
			const keys = strings(value)
			return keys.length > 0 ? { type: 'photos', keys } : { type: 'empty' }
		}
		default:
			return { type: 'empty' }
	}
}

export function answerText(answer: unknown): string | null {
	const view = answerView(answer)
	if (view.type === 'text') return view.text
	if (view.type === 'choices') return view.choices.join(', ')
	return null
}
