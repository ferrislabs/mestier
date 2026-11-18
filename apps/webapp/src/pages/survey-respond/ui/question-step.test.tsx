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

function renderStep(kind: QuestionKind, onChange = vi.fn()) {
	render(
		<QuestionStep
			question={question('q', kind, {
				label: 'Libellé',
				options: ['Oui', 'Non'],
				max_photos: 3,
			})}
			draft={undefined}
			photos={[]}
			photoNotice={null}
			progress={{ position: 2, total: null }}
			error={null}
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
})
