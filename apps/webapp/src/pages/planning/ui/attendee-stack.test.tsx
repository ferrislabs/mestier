import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { CalendarAttendeeVM } from '../lib/build-calendar-model'
import { AttendeeStack } from './attendee-stack'

function attendee(
	overrides: Partial<CalendarAttendeeVM> = {},
): CalendarAttendeeVM {
	return { id: 'e-1', name: 'Marie Leroy', initials: 'ML', ...overrides }
}

describe('AttendeeStack', () => {
	it('renders nothing for a task with no one assigned', () => {
		const { container } = render(<AttendeeStack attendees={[]} />)

		expect(container.firstChild).toBeNull()
	})

	it('shows each attendee by their initials', () => {
		render(
			<AttendeeStack
				attendees={[
					attendee({ id: 'e-1', initials: 'ML', name: 'Marie Leroy' }),
					attendee({ id: 'e-2', initials: 'JD', name: 'Jean Dupont' }),
				]}
			/>,
		)

		expect(screen.getByText('ML')).toBeDefined()
		expect(screen.getByText('JD')).toBeDefined()
	})

	it('caps the default size at three, folding the rest into a count', () => {
		render(
			<AttendeeStack
				attendees={[
					attendee({ id: 'e-1', initials: 'ML' }),
					attendee({ id: 'e-2', initials: 'JD' }),
					attendee({ id: 'e-3', initials: 'PB' }),
					attendee({ id: 'e-4', initials: 'RT' }),
				]}
			/>,
		)

		expect(screen.getByText('ML')).toBeDefined()
		expect(screen.getByText('JD')).toBeDefined()
		expect(screen.getByText('PB')).toBeDefined()
		expect(screen.queryByText('RT')).toBeNull()
		expect(screen.getByText('+1')).toBeDefined()
	})

	it('caps the compact size at two — a month-view row has less room', () => {
		render(
			<AttendeeStack
				size="sm"
				attendees={[
					attendee({ id: 'e-1', initials: 'ML' }),
					attendee({ id: 'e-2', initials: 'JD' }),
					attendee({ id: 'e-3', initials: 'PB' }),
				]}
			/>,
		)

		expect(screen.getByText('ML')).toBeDefined()
		expect(screen.getByText('JD')).toBeDefined()
		expect(screen.queryByText('PB')).toBeNull()
		expect(screen.getByText('+1')).toBeDefined()
	})
})
