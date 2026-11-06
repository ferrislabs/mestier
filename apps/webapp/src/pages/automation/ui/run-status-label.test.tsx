import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { RunStatusLabel } from '#/pages/automation/ui/run-status-label'

describe('RunStatusLabel', () => {
	it.each([
		['succeeded', 'Succès'],
		['failed', 'Échec'],
		['running', 'En cours'],
		['pending', 'En attente'],
		['cancelled', 'Annulé'],
	])('reads the run status %s as %s', (status, label) => {
		render(<RunStatusLabel status={status} />)

		expect(screen.getByText(label)).toBeDefined()
	})

	it.each([
		['in_flight', 'En cours'],
		['dead', 'Échec'],
		['skipped', 'Non appelé'],
		['waiting', 'En pause'],
	])('reads the step status %s as %s', (status, label) => {
		render(<RunStatusLabel status={status} />)

		expect(screen.getByText(label)).toBeDefined()
	})

	it('says a workflow has never run', () => {
		render(<RunStatusLabel status="never" />)

		expect(screen.getByText('Jamais exécuté')).toBeDefined()
	})
})
