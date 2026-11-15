import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createRef } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { QuestionKind } from '#/pages/survey/types'
import { question } from '#/pages/survey-respond/test/respond-fixtures'
import {
	progressLabel,
	QuestionStep,
} from '#/pages/survey-respond/ui/question-step'

function renderStep(
	kind: QuestionKind,
	onChange = vi.fn(),
	extra: {
		description?: string | null
		compact?: boolean
		error?: string
	} = {},
) {
	render(
		<QuestionStep
			question={question('q', kind, {
				label: 'Libellé',
				options: ['Oui', 'Non'],
				max_photos: 3,
				description: extra.description,
			})}
			compact={extra.compact}
			draft={undefined}
			photos={[]}
			photoNotice={null}
			progress={{ position: 2, total: null }}
			error={extra.error ?? null}
			headingRef={createRef()}
			onChange={onChange}
			onSubmit={vi.fn()}
			onAddPhotos={vi.fn()}
			onRemovePhoto={vi.fn()}
		/>,
	)
	return onChange
}

describe('QuestionStep', () => {
	it.each([
		['TEXT', 'text'],
		['NUMBER', 'text'],
		['DATE', 'button'],
		['DATETIME', 'button'],
		['PHONE', 'tel'],
		['EMAIL', 'email'],
	] as const)('binds the label of a %s question to a %s input', (kind, type) => {
		renderStep(kind)
		expect(
			(screen.getByLabelText('Libellé') as HTMLInputElement).getAttribute(
				'type',
			),
		).toBe(type)
	})

	it('renders a long text as a textarea', () => {
		renderStep('LONG_TEXT')
		expect(screen.getByLabelText('Libellé').tagName).toBe('TEXTAREA')
	})

	it('renders single choices as keyboard operable radio cards', async () => {
		const onChange = renderStep('SINGLE_CHOICE')
		expect(screen.getByRole('group', { name: 'Libellé' })).toBeTruthy()
		await userEvent.click(screen.getByRole('radio', { name: 'Non' }))
		expect(onChange).toHaveBeenLastCalledWith('Non')
	})

	it('renders multiple choices as checkbox cards', async () => {
		const onChange = renderStep('MULTIPLE_CHOICE')
		screen.getByRole('checkbox', { name: 'Oui' }).focus()
		await userEvent.keyboard(' ')
		expect(onChange).toHaveBeenLastCalledWith(['Oui'])
	})

	it('offers to take or pick photos within the limit', () => {
		renderStep('PHOTO')
		expect(
			screen.getByLabelText('Prendre une photo').getAttribute('capture'),
		).toBe('environment')
		expect(screen.getByLabelText('Choisir des photos')).toBeTruthy()
		expect(screen.getByText('0 sur 3 photos au plus.')).toBeTruthy()
	})

	it('only announces a total when no branch lies ahead', () => {
		expect(progressLabel({ position: 3, total: null })).toBe('Question 3')
		expect(progressLabel({ position: 3, total: 5 })).toBe('Question 3 sur 5')
	})

	it('names the range of a screen holding a block', () => {
		expect(progressLabel({ position: 3, last: 5, total: 12 })).toBe(
			'Questions 3 à 5 sur 12',
		)
		expect(progressLabel({ position: 3, last: 5, total: null })).toBe(
			'Questions 3 à 5',
		)
		expect(progressLabel({ position: 3, last: 3, total: 4 })).toBe(
			'Question 3 sur 4',
		)
	})

	it('shows the description under the heading and describes the input with it', () => {
		renderStep('TEXT', vi.fn(), {
			description: 'Tel qu’inscrit\nsur le contrat',
		})
		const description = screen.getByText(/Tel qu’inscrit/)
		expect(description.textContent).toBe('Tel qu’inscrit\nsur le contrat')
		expect(description.className).toContain('text-sm')
		expect(description.className).toContain('text-muted-foreground')
		expect(description.className).toContain('whitespace-pre-line')
		expect(description.className).toContain('break-words')
		const heading = screen.getByRole('heading', { name: 'Libellé' })
		expect(
			heading.compareDocumentPosition(description) &
				Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
		expect(
			screen.getByLabelText('Libellé').getAttribute('aria-describedby'),
		).toBe(description.id)
	})

	it('describes a choice group with the description and the error', () => {
		renderStep('SINGLE_CHOICE', vi.fn(), {
			description: 'Une seule réponse',
			error: 'Choisissez une option.',
		})
		const description = screen.getByText('Une seule réponse')
		expect(
			screen
				.getByRole('group', { name: 'Libellé' })
				.getAttribute('aria-describedby'),
		).toBe(`${description.id} error-q`)
	})

	it('uses a smaller description on a compact block screen', () => {
		renderStep('TEXT', vi.fn(), { description: 'Le jour', compact: true })
		const description = screen.getByText('Le jour')
		expect(description.className).toContain('text-xs')
		expect(description.className).not.toContain('text-sm')
		expect(description.className).toContain('whitespace-pre-line')
	})

	it('renders nothing more without a description', () => {
		renderStep('TEXT', vi.fn(), { description: null })
		expect(
			screen.getByLabelText('Libellé').getAttribute('aria-describedby'),
		).toBeNull()
		expect(document.getElementById('description-q')).toBeNull()
	})
})
