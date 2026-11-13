import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { StepIndicator } from '#/pages/cursus/shared/ui/step-indicator'
import { StepperFooter } from '#/pages/cursus/shared/ui/stepper-footer'
import { useSteps } from '#/pages/cursus/shared/ui/use-steps'

const STEPS = ['Identité', 'Contact', 'Apprenti'] as const

function Harness({ onSubmit }: { onSubmit: () => void }) {
	const steps = useSteps(STEPS.length)
	return (
		<form
			onSubmit={(event) => {
				event.preventDefault()
				if (steps.isLast) onSubmit()
				else steps.next()
			}}
		>
			<StepIndicator
				steps={STEPS}
				current={steps.index}
				onSelect={steps.goTo}
			/>
			<input aria-label="Champ" />
			<StepperFooter
				isFirst={steps.isFirst}
				isLast={steps.isLast}
				submitLabel="Créer"
				isPending={false}
				onCancel={vi.fn()}
				onBack={steps.back}
			/>
		</form>
	)
}

function currentStep() {
	return document.querySelector('[aria-current="step"]')?.textContent ?? ''
}

describe('StepIndicator with useSteps', () => {
	it('moves forward and back', async () => {
		const user = userEvent.setup()
		render(<Harness onSubmit={vi.fn()} />)

		expect(currentStep()).toContain('Identité')
		expect(screen.getByRole('button', { name: 'Annuler' })).toBeTruthy()
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(currentStep()).toContain('Contact')
		expect(screen.queryByRole('button', { name: 'Annuler' })).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Précédent' }))
		expect(currentStep()).toContain('Identité')
	})

	it('lets done steps be selected but not future ones', async () => {
		const user = userEvent.setup()
		render(<Harness onSubmit={vi.fn()} />)

		expect(screen.queryByRole('button', { name: /Contact/ })).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(currentStep()).toContain('Apprenti')
		await user.click(screen.getByRole('button', { name: /Identité/ }))
		expect(currentStep()).toContain('Identité')
		expect(screen.queryByRole('button', { name: /Apprenti/ })).toBeNull()
	})

	it('submits only on the last step and Enter goes to the next one', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(<Harness onSubmit={onSubmit} />)

		await user.type(screen.getByLabelText('Champ'), '{Enter}')
		expect(currentStep()).toContain('Contact')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(onSubmit).not.toHaveBeenCalled()
		await user.click(screen.getByRole('button', { name: 'Créer' }))
		expect(onSubmit).toHaveBeenCalledTimes(1)
	})
})
