import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ParticipantDetailUI,
	type ParticipantDetailUIProps,
} from '#/pages/cursus/participants/ui/participant-detail-ui'

const PARTICIPANT: Schemas.ParticipantDetailResponse = {
	id: 'p-1',
	organization_id: 'org-1',
	last_name: 'Martin',
	first_name: 'Léa',
	email: 'lea@example.fr',
	status: 'pending',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
	apprentice: null,
	company_tutors: [
		{
			id: 'ct-1',
			participant_id: 'p-1',
			host_company_id: 'hc-1',
			host_company_name: 'Menuiserie Roux',
			job_title: 'Chef d’atelier',
			created_at: '2026-01-01T00:00:00Z',
			updated_at: '2026-01-01T00:00:00Z',
		},
	],
}

function renderDetail(overrides: Partial<ParticipantDetailUIProps> = {}) {
	const props: ParticipantDetailUIProps = {
		participant: PARTICIPANT,
		isLoading: false,
		loadError: null,
		canManage: true,
		isBusy: false,
		hostCompanies: [{ id: 'hc-1', name: 'Menuiserie Roux' }],
		errors: { identity: null, apprentice: null, tutors: null, status: null },
		onSaveIdentity: vi.fn(),
		onSaveApprentice: vi.fn(),
		onAddTutor: vi.fn(),
		onRemoveTutor: vi.fn(),
		onInvite: vi.fn(),
		onDeactivate: vi.fn(),
		onReactivate: vi.fn(),
		...overrides,
	}
	render(<ParticipantDetailUI {...props} />)
	return props
}

describe('ParticipantDetailUI', () => {
	it('invites a pending participant', async () => {
		const props = renderDetail()

		await userEvent.click(screen.getByRole('button', { name: 'Inviter' }))

		expect(props.onInvite).toHaveBeenCalled()
	})

	it('offers to resend an invitation already sent', () => {
		renderDetail({
			participant: {
				...PARTICIPANT,
				invitation_expires_at: '2026-12-01T00:00:00Z',
			},
		})

		expect(
			screen.getByRole('button', { name: "Renvoyer l'invitation" }),
		).toBeTruthy()
	})

	it('offers to reactivate an inactive participant', () => {
		renderDetail({ participant: { ...PARTICIPANT, status: 'inactive' } })

		expect(screen.getByRole('button', { name: 'Réactiver' })).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Désactiver' })).toBeNull()
	})

	it('shows the tutor removal error', () => {
		renderDetail({
			errors: {
				identity: null,
				apprentice: null,
				tutors: 'Cette fiche tuteur est liée à un contrat.',
				status: null,
			},
		})

		expect(
			screen.getByText('Cette fiche tuteur est liée à un contrat.'),
		).toBeTruthy()
	})

	it('hides every write control without the manage bit', () => {
		renderDetail({ canManage: false })

		for (const name of [
			'Inviter',
			'Désactiver',
			"Enregistrer l'identité",
			'Retirer la fiche Menuiserie Roux',
			'Ajouter une fiche tuteur',
		]) {
			expect(screen.queryByRole('button', { name })).toBeNull()
		}
	})
})
