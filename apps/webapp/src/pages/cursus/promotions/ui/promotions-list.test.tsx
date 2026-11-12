import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Promotion } from '#/pages/cursus/promotions/feature/use-promotions'
import { PromotionsList } from '#/pages/cursus/promotions/ui/promotions-list'

const PROMOTION: Promotion = {
	id: 'promo-1',
	organization_id: 'org-1',
	training_id: 'training-1',
	label: 'CAP Menuiserie 2026',
	start_year: 2026,
	end_year: 2028,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

function props(canManage: boolean, promotions: Promotion[] = [PROMOTION]) {
	return {
		promotions,
		total: promotions.length,
		trainingNames: { 'training-1': 'CAP Menuisier' },
		trainings: [{ value: 'training-1', label: 'CAP Menuisier' }],
		trainingId: undefined,
		archived: 'active' as const,
		search: '',
		archiveError: null,
		isLoading: false,
		canManage,
		onSearchChange: vi.fn(),
		onTrainingChange: vi.fn(),
		onResetFilters: vi.fn(),
		onArchivedChange: vi.fn(),
		onOpen: vi.fn(),
		onCreate: vi.fn(),
		onEdit: vi.fn(),
		onArchive: vi.fn(),
	}
}

describe('PromotionsList', () => {
	it('renders label, training and years of each promotion', () => {
		render(<PromotionsList {...props(true)} />)

		expect(screen.getByText('CAP Menuiserie 2026')).toBeDefined()
		expect(screen.getByText('CAP Menuisier', { selector: 'td' })).toBeDefined()
		expect(screen.getByText('2026–2028')).toBeDefined()
	})

	it('opens the promotion when its row is clicked', async () => {
		const user = userEvent.setup()
		const p = props(true)
		render(<PromotionsList {...p} />)

		await user.click(screen.getByText('CAP Menuiserie 2026'))

		expect(p.onOpen).toHaveBeenCalledWith('promo-1')
	})

	it('hides creation, edit and archive without the catalog bit', () => {
		render(<PromotionsList {...props(false)} />)

		expect(
			screen.queryByRole('button', { name: /Nouvelle promotion/ }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: /Modifier/ })).toBeNull()
		expect(screen.queryByRole('button', { name: /Archiver/ })).toBeNull()
	})

	it('offers creation with the catalog bit', async () => {
		const user = userEvent.setup()
		const p = props(true)
		render(<PromotionsList {...p} />)

		await user.click(screen.getByRole('button', { name: /Nouvelle promotion/ }))

		expect(p.onCreate).toHaveBeenCalledOnce()
	})

	it('offers to reset the filters when nothing matches', async () => {
		const user = userEvent.setup()
		const p = { ...props(true, []), search: 'zzz' }
		render(<PromotionsList {...p} />)

		expect(screen.getByText('Aucune promotion ne correspond.')).toBeTruthy()
		await user.click(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		)

		expect(p.onResetFilters).toHaveBeenCalledOnce()
	})

	it('has no reset button without filters', () => {
		render(<PromotionsList {...props(true, [])} />)

		expect(
			screen.queryByRole('button', { name: 'Réinitialiser les filtres' }),
		).toBeNull()
	})
})
