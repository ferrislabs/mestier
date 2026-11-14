import { act, render, screen } from '@testing-library/react'
import { toast } from 'sonner'
import { describe, expect, it } from 'vitest'
import { Toaster } from '#/components/ui/sonner'

describe('Toaster', () => {
	it('shows a toast at the bottom right', async () => {
		const { container } = render(<Toaster />)

		act(() => {
			toast.success('Questionnaire enregistré.')
		})

		expect(await screen.findByText('Questionnaire enregistré.')).toBeDefined()
		const region = container.ownerDocument.querySelector('[data-sonner-toaster]')
		expect(region?.getAttribute('data-y-position')).toBe('bottom')
		expect(region?.getAttribute('data-x-position')).toBe('right')
	})
})
