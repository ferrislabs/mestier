import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { RowActions } from '#/pages/cursus/shared/ui/row-actions'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'

beforeAll(installRadixPolyfills)

describe('RowActions', () => {
	it('runs the chosen action from the row menu', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const onArchive = vi.fn()
		render(
			<RowActions
				label="CAP 2026"
				actions={[
					{ label: 'Modifier', onSelect: vi.fn() },
					{ label: 'Archiver', onSelect: onArchive },
				]}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Actions CAP 2026' }))
		await user.click(await screen.findByRole('menuitem', { name: 'Archiver' }))

		expect(onArchive).toHaveBeenCalledOnce()
	})

	it('renders nothing without actions', () => {
		render(<RowActions label="CAP 2026" actions={[]} />)

		expect(screen.queryByRole('button')).toBeNull()
	})
})
