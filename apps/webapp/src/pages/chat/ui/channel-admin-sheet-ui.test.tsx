import { render, screen } from '@testing-library/react'
import type { UserEvent } from '@testing-library/user-event'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { ChannelAdminSheetUIProps } from './channel-admin-sheet-ui'
import { ChannelAdminSheetUI } from './channel-admin-sheet-ui'

/** Radix `Tabs` only keeps the active panel in the accessible tree — tests
 * touching the "permissions" or "webhooks" tab must switch to it first. */
async function switchToTab(user: UserEvent, label: string) {
	await user.click(screen.getByRole('tab', { name: label }))
}

function channel(overrides: Partial<Record<string, unknown>> = {}) {
	return {
		id: 'ch-1',
		organization_id: 'org-1',
		channel_type: 'TEXT' as const,
		name: 'general',
		topic: null,
		position: 0,
		category_id: null,
		parent_id: null,
		origin_message_id: null,
		archived: false,
		created_at: '',
		updated_at: '',
		...overrides,
	}
}

function baseProps(
	overrides: Partial<ChannelAdminSheetUIProps> = {},
): ChannelAdminSheetUIProps {
	return {
		open: true,
		onOpenChange: vi.fn(),
		channel: channel(),
		categories: [],
		nameDraft: 'general',
		topicDraft: '',
		categoryIdDraft: null,
		onChangeName: vi.fn(),
		onChangeTopic: vi.fn(),
		onChangeCategory: vi.fn(),
		onSaveGeneral: vi.fn(),
		isSavingGeneral: false,
		deleteDialogOpen: false,
		onRequestDelete: vi.fn(),
		onCancelDelete: vi.fn(),
		onConfirmDelete: vi.fn(),
		isDeleting: false,
		isLoadingOverwrites: false,
		everyoneOverwrite: undefined,
		roleAndMemberOverwrites: [],
		onChangeEveryoneBit: vi.fn(),
		onDeleteTargetOverwrite: vi.fn(),
		webhooks: [],
		isLoadingWebhooks: false,
		newWebhookName: '',
		onChangeNewWebhookName: vi.fn(),
		onCreateWebhook: vi.fn(),
		isCreatingWebhook: false,
		createdWebhookToken: null,
		onDismissCreatedToken: vi.fn(),
		onDeleteWebhook: vi.fn(),
		...overrides,
	}
}

describe('ChannelAdminSheetUI — general', () => {
	it('calls onSaveGeneral when the save button is clicked', async () => {
		const user = userEvent.setup()
		const onSaveGeneral = vi.fn()
		render(<ChannelAdminSheetUI {...baseProps({ onSaveGeneral })} />)

		await user.click(screen.getByText('Enregistrer'))

		expect(onSaveGeneral).toHaveBeenCalledTimes(1)
	})

	it('opens a confirmation naming the channel before deleting', async () => {
		const user = userEvent.setup()
		const onRequestDelete = vi.fn()
		render(<ChannelAdminSheetUI {...baseProps({ onRequestDelete })} />)

		await user.click(screen.getByText('Supprimer le canal'))

		expect(onRequestDelete).toHaveBeenCalledTimes(1)
	})

	it('names what is lost in the delete confirmation', () => {
		render(
			<ChannelAdminSheetUI
				{...baseProps({
					deleteDialogOpen: true,
					channel: channel({ name: 'random' }),
				})}
			/>,
		)

		expect(screen.getAllByText(/#random/).length).toBeGreaterThan(0)
		expect(screen.getByText(/tous ses messages/)).toBeDefined()
	})

	it('confirms deletion via the alert dialog action', async () => {
		const user = userEvent.setup()
		const onConfirmDelete = vi.fn()
		render(
			<ChannelAdminSheetUI
				{...baseProps({ deleteDialogOpen: true, onConfirmDelete })}
			/>,
		)

		await user.click(screen.getByText('Supprimer'))

		expect(onConfirmDelete).toHaveBeenCalledTimes(1)
	})
})

describe('ChannelAdminSheetUI — permissions', () => {
	it('shows the everyone overwrite as inherited with no overwrite', async () => {
		const user = userEvent.setup()
		render(<ChannelAdminSheetUI {...baseProps()} />)
		await switchToTab(user, 'Permissions')

		const buttons = screen
			.getAllByText('Hérité')
			.map((el) => el.closest('button'))
		for (const button of buttons) {
			expect(button?.getAttribute('aria-pressed')).toBe('true')
		}
	})

	it('calls onChangeEveryoneBit with the clicked state', async () => {
		const user = userEvent.setup()
		const onChangeEveryoneBit = vi.fn()
		render(<ChannelAdminSheetUI {...baseProps({ onChangeEveryoneBit })} />)
		await switchToTab(user, 'Permissions')

		const allowButtons = screen.getAllByText('Autorisé')
		await user.click(allowButtons[0] as HTMLElement)

		expect(onChangeEveryoneBit).toHaveBeenCalledWith('VIEW_CHANNEL', 'allow')
	})

	it('reflects an existing everyone overwrite', async () => {
		const user = userEvent.setup()
		render(
			<ChannelAdminSheetUI
				{...baseProps({
					everyoneOverwrite: {
						target_type: 'everyone',
						target_id: null,
						allow: 0,
						deny: 32,
						created_at: '',
						updated_at: '',
					},
				})}
			/>,
		)
		await switchToTab(user, 'Permissions')

		const denyButtons = screen
			.getAllByText('Refusé')
			.map((el) => el.closest('button'))
		expect(denyButtons[0]?.getAttribute('aria-pressed')).toBe('true')
	})

	it('lists existing role/member overwrites with a way to remove them', async () => {
		const user = userEvent.setup()
		const onDeleteTargetOverwrite = vi.fn()
		render(
			<ChannelAdminSheetUI
				{...baseProps({
					roleAndMemberOverwrites: [
						{
							target_type: 'role',
							target_id: 'role-1',
							allow: 32,
							deny: 0,
							created_at: '',
							updated_at: '',
						},
					],
					onDeleteTargetOverwrite,
				})}
			/>,
		)
		await switchToTab(user, 'Permissions')

		expect(screen.getByText(/role-1/)).toBeDefined()
		await user.click(screen.getByLabelText('Retirer ce remplacement'))
		expect(onDeleteTargetOverwrite).toHaveBeenCalledWith('role', 'role-1')
	})
})

describe('ChannelAdminSheetUI — webhooks', () => {
	it('disables webhook creation until a name is entered', async () => {
		const user = userEvent.setup()
		render(<ChannelAdminSheetUI {...baseProps({ newWebhookName: '' })} />)
		await switchToTab(user, 'Webhooks')

		const button = screen.getByText('Créer').closest('button')
		expect(button?.hasAttribute('disabled')).toBe(true)
	})

	it('shows the created token once, with a dismiss action', async () => {
		const user = userEvent.setup()
		const onDismissCreatedToken = vi.fn()
		render(
			<ChannelAdminSheetUI
				{...baseProps({
					createdWebhookToken: 'secret-token-123',
					onDismissCreatedToken,
				})}
			/>,
		)
		await switchToTab(user, 'Webhooks')

		expect(screen.getByText('secret-token-123')).toBeDefined()
		await user.click(screen.getByText('J’ai copié le jeton'))
		expect(onDismissCreatedToken).toHaveBeenCalledTimes(1)
	})

	it('lists webhooks with a revoke action', async () => {
		const user = userEvent.setup()
		const onDeleteWebhook = vi.fn()
		render(
			<ChannelAdminSheetUI
				{...baseProps({
					webhooks: [
						{
							id: 'wh-1',
							organization_id: 'org-1',
							channel_id: 'ch-1',
							name: 'CI bot',
							avatar_url: null,
							created_by: 'me',
							created_at: '',
							updated_at: '',
						},
					],
					onDeleteWebhook,
				})}
			/>,
		)
		await switchToTab(user, 'Webhooks')

		await user.click(screen.getByLabelText('Révoquer CI bot'))
		expect(onDeleteWebhook).toHaveBeenCalledWith('wh-1')
	})
})
