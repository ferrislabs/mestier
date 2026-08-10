import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { CurrentTimeLine } from '#/pages/planning/ui/current-time-line'

const TZ = 'Europe/Paris'
const AMPLITUDE = { startMinute: 8 * 60, endMinute: 18 * 60 }
const WEEK_COLUMNS = [
	'2026-08-03',
	'2026-08-04',
	'2026-08-05',
	'2026-08-06',
	'2026-08-07',
	'2026-08-08',
	'2026-08-09',
]

describe('CurrentTimeLine — visibility', () => {
	it('renders nothing when the current day is outside the visible period', () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				amplitude={AMPLITUDE}
				orientation="horizontal"
				now={new Date('2026-09-01T11:00:00Z')}
			/>,
		)

		expect(screen.queryByTestId('current-time-line')).toBeNull()
	})

	it('renders nothing when the current time is outside the amplitude', () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				amplitude={AMPLITUDE}
				orientation="horizontal"
				now={new Date('2026-08-05T04:00:00Z')} // 06:00 Paris, avant 08:00
			/>,
		)

		expect(screen.queryByTestId('current-time-line')).toBeNull()
	})

	it('renders the line when both day and time fall in the period and the amplitude', () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				amplitude={AMPLITUDE}
				orientation="horizontal"
				now={new Date('2026-08-05T11:00:00Z')} // 13:00 Paris
			/>,
		)

		expect(screen.getByTestId('current-time-line')).toBeDefined()
	})
})

describe('CurrentTimeLine — position horizontale (vue jour)', () => {
	it('positions the line proportionally along the amplitude', () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-05"
				windowTo="2026-08-05"
				amplitude={AMPLITUDE}
				orientation="horizontal"
				now={new Date('2026-08-05T11:00:00Z')} // 13:00 Paris → 50%
			/>,
		)

		const line = screen.getByTestId('current-time-line')
		expect(line.style.left).toBe('50%')
	})
})

describe('CurrentTimeLine — position verticale (vues semaine/mois)', () => {
	it("positions the line in the current day's column", () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				amplitude={AMPLITUDE}
				orientation="vertical"
				columns={WEEK_COLUMNS}
				now={new Date('2026-08-05T11:00:00Z')} // 2026-08-05 → index 2 sur 7
			/>,
		)

		const line = screen.getByTestId('current-time-line')
		expect(line.style.left).toBe(`${((2 + 0.5) / 7) * 100}%`)
	})

	it('renders nothing if the current day is absent from the given columns', () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				amplitude={AMPLITUDE}
				orientation="vertical"
				columns={[]}
				now={new Date('2026-08-05T11:00:00Z')}
			/>,
		)

		expect(screen.queryByTestId('current-time-line')).toBeNull()
	})
})

describe('CurrentTimeLine — no network call', () => {
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

	it('fires no fetch on render', () => {
		render(
			<CurrentTimeLine
				timeZone={TZ}
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				amplitude={AMPLITUDE}
				orientation="horizontal"
				now={new Date('2026-08-05T11:00:00Z')}
			/>,
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
