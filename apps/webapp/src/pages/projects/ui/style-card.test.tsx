import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { completeStyle } from '#/pages/doe-style/lib/style'
import { StyleCard } from '#/pages/projects/ui/style-card'

function renderCard(hasOwnStyle: boolean) {
	const style = completeStyle(undefined)
	const onClear = vi.fn(() => Promise.resolve())
	render(
		<StyleCard
			draft={style}
			stored={style}
			hasOwnStyle={hasOwnStyle}
			editor={null}
			isSaving={false}
			error={null}
			onReplace={() => {}}
			onSubmit={() => Promise.resolve()}
			onClear={onClear}
			onDirtyChange={() => {}}
		/>,
	)
	return {
		onClear,
		button: screen.getByRole('button', {
			name: 'Revenir au style de l’organisation',
		}),
	}
}

describe('StyleCard', () => {
	it('removes the project style when the project has one', async () => {
		const { onClear, button } = renderCard(true)

		await userEvent.click(button)

		expect(onClear).toHaveBeenCalledOnce()
	})

	it('has nothing to remove when the project follows its model or organization', () => {
		const { button } = renderCard(false)

		expect((button as HTMLButtonElement).disabled).toBe(true)
	})
})
