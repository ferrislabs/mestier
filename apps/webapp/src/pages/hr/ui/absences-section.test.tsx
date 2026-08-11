import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { AbsenceListItem } from '#/pages/hr/lib/absences'
import { AbsencesSection } from '#/pages/hr/ui/absences-section'

function absence(overrides: Partial<AbsenceListItem> = {}): AbsenceListItem {
	return {
		id: 'ab-1',
		memberId: 'member-1',
		kind: 'LEAVE',
		allDay: true,
		range: { from: '2026-08-10', to: '2026-08-10' },
		startTime: '08:00',
		endTime: '18:00',
		note: '',
		...overrides,
	}
}

function baseProps() {
	return {
		absences: [] as AbsenceListItem[],
		isLoading: false,
		onCreate: vi.fn(),
		onSelect: vi.fn(),
	}
}

describe('AbsencesSection — states', () => {
	it('shows a loading state without crashing', () => {
		render(<AbsencesSection {...baseProps()} isLoading={true} />)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it('shows a message when there is no absence', () => {
		render(<AbsencesSection {...baseProps()} />)
		expect(screen.getByText('Aucune absence enregistrée.')).toBeDefined()
	})
})

describe('AbsencesSection — liste', () => {
	it('shows a single-day full-day absence with its kind', () => {
		render(<AbsencesSection {...baseProps()} absences={[absence()]} />)
		expect(screen.getByText(/Congé — 10\/08\/2026/)).toBeDefined()
	})

	it('shows the full range for a multi-day absence', () => {
		render(
			<AbsencesSection
				{...baseProps()}
				absences={[
					absence({ range: { from: '2026-08-10', to: '2026-08-12' } }),
				]}
			/>,
		)
		expect(screen.getByText(/10\/08\/2026 → 12\/08\/2026/)).toBeDefined()
	})

	it('shows the hours for a time-slot absence', () => {
		render(
			<AbsencesSection
				{...baseProps()}
				absences={[
					absence({
						kind: 'SICK',
						allDay: false,
						startTime: '09:00',
						endTime: '12:00',
					}),
				]}
			/>,
		)
		expect(screen.getByText(/09:00–12:00/)).toBeDefined()
	})

	it('shows the note when it is filled in', () => {
		render(
			<AbsencesSection
				{...baseProps()}
				absences={[absence({ note: 'Rendez-vous médical' })]}
			/>,
		)
		expect(screen.getByText('Rendez-vous médical')).toBeDefined()
	})

	it('calls onSelect with the id when an absence is clicked', async () => {
		const user = userEvent.setup()
		const onSelect = vi.fn()
		render(
			<AbsencesSection
				{...baseProps()}
				absences={[absence({ id: 'ab-42' })]}
				onSelect={onSelect}
			/>,
		)

		await user.click(screen.getByText(/Congé —/))
		expect(onSelect).toHaveBeenCalledWith('ab-42')
	})
})

describe('AbsencesSection — creation', () => {
	it('calls onCreate when Ajouter une absence is clicked', async () => {
		const user = userEvent.setup()
		const onCreate = vi.fn()
		render(<AbsencesSection {...baseProps()} onCreate={onCreate} />)

		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)
		expect(onCreate).toHaveBeenCalledTimes(1)
	})
})

describe('AbsencesSection — no network call', () => {
	let fetchSpy: ReturnType<typeof createFetchSpy>

	function createFetchSpy() {
		return vi.spyOn(global, 'fetch').mockImplementation(() => {
			throw new Error('le composant ui/ ne doit jamais appeler fetch')
		})
	}

	beforeEach(() => {
		fetchSpy = createFetchSpy()
	})

	afterEach(() => {
		fetchSpy.mockRestore()
	})

	it('fires no fetch on render nor during interactions', async () => {
		const user = userEvent.setup()
		render(<AbsencesSection {...baseProps()} absences={[absence()]} />)

		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)
		await user.click(screen.getByText(/Congé —/))

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
