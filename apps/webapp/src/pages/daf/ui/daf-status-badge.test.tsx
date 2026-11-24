import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { DafStatusBadge } from '#/pages/daf/ui/daf-status-badge'

describe('DafStatusBadge', () => {
	it.each([
		['pending', 'En attente', 'amber'],
		['accepted', 'Acceptée', 'success'],
		['rejected_with_comments', 'Refus avec obs.', 'orange'],
		['rejected', 'Refus ferme', 'destructive'],
	] as const)('renders %s as %s', (status, label, tone) => {
		render(<DafStatusBadge status={status} />)
		expect(screen.getByText(label).className).toContain(tone)
	})
})
