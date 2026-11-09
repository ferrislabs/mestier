import { screen } from '@testing-library/react'
import type { UserEvent } from '@testing-library/user-event'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { renderWithPermissions } from '#/test/with-permissions'
import type { ChannelAdminDialogUIProps } from './channel-admin-dialog-ui'
import { ChannelAdminDialogUI } from './channel-admin-dialog-ui'

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
	overrides: Partial<ChannelAdminDialogUIProps> = {},
): ChannelAdminDialogUIProps {
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
		onRenameWebhook: vi.fn(),
		webhookCallUrlBase: 'https://api.test/api/v1/chat/webhooks',
		createdWebhookCallUrl: null,
		creatorName: () => undefined,
		...overrides,
	}
}

describe('ChannelAdminDialogUI — general', () => {
	it('calls onSaveGeneral when the save button is clicked', async () => {
		const user = userEvent.setup()
		const onSaveGeneral = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ onSaveGeneral })} />,
		)

		await user.click(screen.getByText('Enregistrer'))

		expect(onSaveGeneral).toHaveBeenCalledTimes(1)
	})

	it('opens a confirmation naming the channel before deleting', async () => {
		const user = userEvent.setup()
		const onRequestDelete = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ onRequestDelete })} />,
		)

		await user.click(screen.getByText('Supprimer le canal'))

		expect(onRequestDelete).toHaveBeenCalledTimes(1)
	})

	it('names what is lost in the delete confirmation', () => {
		renderWithPermissions(
			<ChannelAdminDialogUI
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
		renderWithPermissions(
			<ChannelAdminDialogUI
				{...baseProps({ deleteDialogOpen: true, onConfirmDelete })}
			/>,
		)

		await user.click(screen.getByText('Supprimer'))

		expect(onConfirmDelete).toHaveBeenCalledTimes(1)
	})
})

describe('ChannelAdminDialogUI — permissions', () => {
	it('shows the everyone overwrite as inherited with no overwrite', async () => {
		const user = userEvent.setup()
		renderWithPermissions(<ChannelAdminDialogUI {...baseProps()} />)
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
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ onChangeEveryoneBit })} />,
		)
		await switchToTab(user, 'Permissions')

		const allowButtons = screen.getAllByText('Autorisé')
		await user.click(allowButtons[0] as HTMLElement)

		expect(onChangeEveryoneBit).toHaveBeenCalledWith('VIEW_CHANNEL', 'allow')
	})

	it('reflects an existing everyone overwrite', async () => {
		const user = userEvent.setup()
		renderWithPermissions(
			<ChannelAdminDialogUI
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
		renderWithPermissions(
			<ChannelAdminDialogUI
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

describe('ChannelAdminDialogUI — webhooks', () => {
	it('disables webhook creation until a name is entered', async () => {
		const user = userEvent.setup()
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ newWebhookName: '' })} />,
		)
		await switchToTab(user, 'Intégrations')

		const button = screen.getByText('Créer').closest('button')
		expect(button?.hasAttribute('disabled')).toBe(true)
	})

	it('shows the created token once, with a dismiss action', async () => {
		const user = userEvent.setup()
		const onDismissCreatedToken = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI
				{...baseProps({
					createdWebhookToken: 'secret-token-123',
					onDismissCreatedToken,
				})}
			/>,
		)
		await switchToTab(user, 'Intégrations')

		expect(screen.getByText('secret-token-123')).toBeDefined()
		await user.click(screen.getByText('J’ai copié le jeton'))
		expect(onDismissCreatedToken).toHaveBeenCalledTimes(1)
	})

	it('shows the full call URL of a webhook just created, and the template for the others', async () => {
		const user = userEvent.setup()
		renderWithPermissions(
			<ChannelAdminDialogUI
				{...baseProps({
					createdWebhookToken: 'tok',
					createdWebhookCallUrl:
						'https://api.test/api/v1/chat/webhooks/wh-9/tok',
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
				})}
			/>,
		)
		await switchToTab(user, 'Intégrations')

		expect(
			screen.getByText('https://api.test/api/v1/chat/webhooks/wh-9/tok'),
		).toBeDefined()
		await user.click(screen.getByText('CI bot'))
		expect(
			(screen.getByLabelText('URL à appeler (POST)') as HTMLInputElement).value,
		).toBe('https://api.test/api/v1/chat/webhooks/wh-1/{jeton}')
	})

	it('lists webhooks with a revoke action', async () => {
		const user = userEvent.setup()
		const onDeleteWebhook = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI
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
		await switchToTab(user, 'Intégrations')

		await user.click(screen.getByText('CI bot'))
		await user.click(screen.getByLabelText('Révoquer CI bot'))
		expect(onDeleteWebhook).toHaveBeenCalledWith('wh-1')
	})
})

describe('ChannelAdminDialogUI permission gating (#407)', () => {
	it('shows the permissions and webhooks tabs to a fully-privileged caller', () => {
		renderWithPermissions(<ChannelAdminDialogUI {...baseProps()} />, {
			permissions: ['MANAGE_CHANNELS', 'MANAGE_WEBHOOKS'],
		})
		expect(screen.getByRole('tab', { name: 'Permissions' })).toBeDefined()
		expect(screen.getByRole('tab', { name: 'Intégrations' })).toBeDefined()
	})

	it('hides the permissions tab without MANAGE_CHANNELS', () => {
		// `channel/permissions/list.rs` requires `channel.manage`, so the tab
		// would open onto a 403 rather than an empty list.
		renderWithPermissions(<ChannelAdminDialogUI {...baseProps()} />, {
			permissions: ['MANAGE_WEBHOOKS'],
		})
		expect(screen.queryByRole('tab', { name: 'Permissions' })).toBeNull()
	})

	it('hides the webhooks tab without MANAGE_WEBHOOKS', () => {
		// `webhook/list.rs` requires `webhook.manage` — same reasoning.
		renderWithPermissions(<ChannelAdminDialogUI {...baseProps()} />, {
			permissions: ['MANAGE_CHANNELS'],
		})
		expect(screen.queryByRole('tab', { name: 'Intégrations' })).toBeNull()
	})

	it('hides the save and delete controls without MANAGE_CHANNELS', () => {
		renderWithPermissions(<ChannelAdminDialogUI {...baseProps()} />, {
			permissions: ['VIEW_CHANNEL'],
		})
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
		expect(
			screen.queryByRole('button', { name: /supprimer le canal/i }),
		).toBeNull()
	})
})
