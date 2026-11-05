import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'
import type {
	SupportDetailUIProps,
	SupportDraftReviewsVM,
} from '#/pages/support/types'
import { ALIX, entryVM, ticketVM } from '#/pages/support/ui/support-fixtures'
import { TicketDetailUI } from '#/pages/support/ui/ticket-detail-ui'
import { TicketModalUI } from '#/pages/support/ui/ticket-modal-ui'

beforeAll(installRadixPolyfills)

const NO_REVIEWS = new Map<string, SupportDraftReviewsVM>()

function detailProps(
	overrides: Partial<SupportDetailUIProps> = {},
): SupportDetailUIProps {
	const ticket = ticketVM({ assignee: ALIX })
	return {
		variant: 'pane',
		ticket,
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		conversation: {
			entries: [entryVM()],
			isLoading: false,
			error: null,
			onRetry: vi.fn(),
			canManage: true,
			mode: 'reply',
			onModeChange: vi.fn(),
			text: '',
			onTextChange: vi.fn(),
			nextStatus: 'waiting_customer',
			onNextStatusChange: vi.fn(),
			editingDraftId: null,
			pendingEntryId: null,
			isSubmitting: false,
			composerError: null,
			reviewsByEntry: NO_REVIEWS,
			onEditDraft: vi.fn(),
			onCancelDraft: vi.fn(),
			onSendDraft: vi.fn(),
			onDeleteDraft: vi.fn(),
			onToggleApproval: vi.fn(),
			onCommentDraft: vi.fn(),
			onSend: vi.fn(),
			onSaveDraft: vi.fn(),
			onAddNote: vi.fn(),
			onOpenAttachment: vi.fn(),
		},
		properties: {
			ticket,
			canManage: true,
			statusOptions: [{ status: 'new', label: 'Backlog' }],
			members: [ALIX],
			knownBadges: [],
			otherTickets: [],
			error: null,
			onOpenOtherTicket: vi.fn(),
			onStatusChange: vi.fn(),
			onPriorityChange: vi.fn(),
			onAssigneeChange: vi.fn(),
			onAddBadge: vi.fn(),
			onRemoveBadge: vi.fn(),
		},
		propertiesOpen: true,
		onToggleProperties: vi.fn(),
		onOpenFullPage: vi.fn(),
		onClose: vi.fn(),
		...overrides,
	}
}

function renderDetail(overrides: Partial<SupportDetailUIProps> = {}) {
	const props = detailProps(overrides)
	render(<TicketDetailUI {...props} />)
	return props
}

describe('TicketDetailUI — volet', () => {
	it('titles the pane with the number, the subject and the headline', () => {
		renderDetail()

		const heading = screen.getByRole('heading', { level: 2 })
		expect(heading.textContent).toBe('#1Fuite sous l’évier')
		expect(screen.getByText(/Claire Fontaine via saisie manuelle/)).toBeTruthy()
	})

	it('shows the thread, the composer and the properties', () => {
		renderDetail()

		expect(screen.getByRole('list', { name: 'Fil de discussion' })).toBeTruthy()
		expect(screen.getByRole('region', { name: 'Écrire' })).toBeTruthy()
		expect(
			screen.getByRole('complementary', { name: 'Propriétés du ticket' }),
		).toBeTruthy()
	})

	it('hides the properties panel and offers to show it again', async () => {
		const props = renderDetail({ propertiesOpen: false })

		expect(screen.queryByRole('complementary')).toBeNull()
		await userEvent.click(
			screen.getByRole('button', { name: 'Afficher les propriétés' }),
		)

		expect(props.onToggleProperties).toHaveBeenCalledTimes(1)
	})

	it('offers to hide the properties when they are shown', () => {
		renderDetail()

		expect(
			screen
				.getByRole('button', { name: 'Masquer les propriétés' })
				.getAttribute('aria-pressed'),
		).toBe('true')
	})

	it('opens the full page and closes the ticket', async () => {
		const props = renderDetail()

		await userEvent.click(screen.getByLabelText('Ouvrir en pleine page'))
		await userEvent.click(screen.getByLabelText('Fermer le ticket'))

		expect(props.onOpenFullPage).toHaveBeenCalledTimes(1)
		expect(props.onClose).toHaveBeenCalledTimes(1)
	})

	it('has no close button without onClose', () => {
		renderDetail({ onClose: undefined })

		expect(screen.queryByLabelText('Fermer le ticket')).toBeNull()
	})

	it('hides the composer for a reader', () => {
		const base = detailProps()
		renderDetail({
			conversation: base.conversation && {
				...base.conversation,
				canManage: false,
			},
		})

		expect(screen.queryByRole('region', { name: 'Écrire' })).toBeNull()
	})

	it('says the thread is empty', () => {
		const base = detailProps()
		renderDetail({
			conversation: base.conversation && { ...base.conversation, entries: [] },
		})

		expect(screen.getByText('Aucun message pour le moment.')).toBeTruthy()
	})

	it('shows a loading hint for the thread', () => {
		const base = detailProps()
		renderDetail({
			conversation: base.conversation && {
				...base.conversation,
				isLoading: true,
			},
		})

		expect(screen.getByText('Chargement…')).toBeTruthy()
	})

	it('shows the thread error with a retry', async () => {
		const base = detailProps()
		const onRetry = vi.fn()
		renderDetail({
			conversation: base.conversation && {
				...base.conversation,
				error: 'Le fil est indisponible.',
				onRetry,
			},
		})

		expect(screen.getByText('Le fil est indisponible.')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(onRetry).toHaveBeenCalledTimes(1)
	})
})

describe('TicketDetailUI — états', () => {
	it('says the ticket is unknown, with a way to close', async () => {
		const props = renderDetail({
			ticket: null,
			conversation: null,
			properties: null,
		})

		expect(screen.getByText('Ticket introuvable')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Fermer' }))

		expect(props.onClose).toHaveBeenCalledTimes(1)
	})

	it('shows a loading hint while the ticket loads', () => {
		renderDetail({
			isLoading: true,
			ticket: null,
			conversation: null,
			properties: null,
		})

		expect(screen.getByText('Chargement…')).toBeTruthy()
	})

	it('shows the error with a retry when the ticket cannot be read', async () => {
		const props = renderDetail({
			error: 'Le ticket est indisponible.',
			ticket: null,
			conversation: null,
			properties: null,
		})

		expect(screen.getByText('Le ticket est indisponible.')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})
})

describe('TicketDetailUI — page', () => {
	it('puts the properties in a card next to the conversation', () => {
		renderDetail({ variant: 'page' })

		expect(
			screen.queryByRole('button', { name: 'Masquer les propriétés' }),
		).toBeNull()
		expect(screen.getByRole('complementary').className).toContain('rounded-lg')
	})
})

describe('TicketModalUI', () => {
	it('shows its content in a dialog and closes on Escape', async () => {
		const onClose = vi.fn()
		render(
			<TicketModalUI onClose={onClose}>
				<TicketDetailUI {...detailProps({ variant: 'modal' })} />
			</TicketModalUI>,
		)

		const dialog = screen.getByRole('dialog')
		expect(
			within(dialog).getByRole('heading', { name: '#1Fuite sous l’évier' }),
		).toBeTruthy()
		expect(
			within(dialog).getByText(
				'Conversation, notes internes et propriétés du ticket.',
			),
		).toBeTruthy()

		await userEvent.keyboard('{Escape}')

		expect(onClose).toHaveBeenCalledTimes(1)
	})

	it('keeps the dialog open on Escape while a message is being typed', async () => {
		const onClose = vi.fn()
		const base = detailProps({ variant: 'modal' })
		render(
			<TicketModalUI onClose={onClose}>
				<TicketDetailUI
					{...base}
					conversation={
						base.conversation && { ...base.conversation, text: 'En cours' }
					}
				/>
			</TicketModalUI>,
		)

		screen.getByRole('textbox', { name: 'Réponse au client' }).focus()
		await userEvent.keyboard('{Escape}')

		expect(onClose).not.toHaveBeenCalled()
	})
})
