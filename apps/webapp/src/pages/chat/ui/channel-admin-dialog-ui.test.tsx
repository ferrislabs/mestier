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

const EVERYONE = {
	key: 'everyone',
	type: 'everyone' as const,
	id: null,
	name: '@everyone',
	allow: 0,
	deny: 0,
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
		onConfirmDelete: vi.fn(),
		isDeleting: false,
		isLoadingOverwrites: false,
		isPrivate: false,
		onTogglePrivate: vi.fn(),
		permissionTargets: [EVERYONE],
		selectedPermissionKey: 'everyone',
		onSelectPermissionKey: vi.fn(),
		addableRoles: [],
		onAddRole: vi.fn(),
		onChangeTargetBit: vi.fn(),
		onRemoveTarget: vi.fn(),
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

	it('deletes the channel only on the second click', async () => {
		const user = userEvent.setup()
		const onConfirmDelete = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ onConfirmDelete })} />,
		)

		await user.click(screen.getByText('Supprimer le canal'))
		expect(onConfirmDelete).not.toHaveBeenCalled()

		await user.click(screen.getByText('Confirmer la suppression'))
		expect(onConfirmDelete).toHaveBeenCalledTimes(1)
	})
})

describe('ChannelAdminDialogUI — permissions', () => {
	it('shows every permission of @everyone as inherited without an overwrite', async () => {
		const user = userEvent.setup()
		renderWithPermissions(<ChannelAdminDialogUI {...baseProps()} />)
		await switchToTab(user, 'Permissions')

		for (const name of [
			'Hériter : Voir le canal',
			'Hériter : Envoyer des messages',
		]) {
			expect(
				screen.getByRole('button', { name }).getAttribute('aria-pressed'),
			).toBe('true')
		}
	})

	it('calls onChangeTargetBit with the target, the bit and the clicked state', async () => {
		const user = userEvent.setup()
		const onChangeTargetBit = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ onChangeTargetBit })} />,
		)
		await switchToTab(user, 'Permissions')

		await user.click(
			screen.getByRole('button', { name: 'Autoriser : Voir le canal' }),
		)

		expect(onChangeTargetBit).toHaveBeenCalledWith(
			'everyone',
			'VIEW_CHANNEL',
			'allow',
		)
	})

	it('reflects the stored bits of the selected target', async () => {
		const user = userEvent.setup()
		renderWithPermissions(
			<ChannelAdminDialogUI
				{...baseProps({ permissionTargets: [{ ...EVERYONE, deny: 32 }] })}
			/>,
		)
		await switchToTab(user, 'Permissions')

		expect(
			screen
				.getByRole('button', { name: 'Refuser : Voir le canal' })
				.getAttribute('aria-pressed'),
		).toBe('true')
	})

	it('toggles the private switch', async () => {
		const user = userEvent.setup()
		const onTogglePrivate = vi.fn()
		renderWithPermissions(
			<ChannelAdminDialogUI {...baseProps({ onTogglePrivate })} />,
		)
		await switchToTab(user, 'Permissions')

		await user.click(screen.getByRole('switch', { name: 'Canal privé' }))

		expect(onTogglePrivate).toHaveBeenCalledWith(true)
	})

	it('selects a role and lets it be removed', async () => {
		const user = userEvent.setup()
		const onSelectPermissionKey = vi.fn()
		const onRemoveTarget = vi.fn()
		const role = {
			key: 'role:r1',
			type: 'role' as const,
			id: 'r1',
			name: 'Chefs de chantier',
			allow: 32,
			deny: 0,
		}
		renderWithPermissions(
			<ChannelAdminDialogUI
				{...baseProps({
					permissionTargets: [EVERYONE, role],
					selectedPermissionKey: 'role:r1',
					onSelectPermissionKey,
					onRemoveTarget,
				})}
			/>,
		)
		await switchToTab(user, 'Permissions')

		await user.click(screen.getByRole('button', { name: '@everyone' }))
		expect(onSelectPermissionKey).toHaveBeenCalledWith('everyone')

		await user.click(
			screen.getByRole('button', { name: 'Retirer Chefs de chantier' }),
		)
		expect(onRemoveTarget).toHaveBeenCalledWith('role:r1')
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
		await user.click(
			screen.getByRole('button', { name: 'Suppression de webhook' }),
		)
		expect(onDeleteWebhook).not.toHaveBeenCalled()
		await user.click(
			screen.getByRole('button', { name: 'Cliquer à nouveau pour confirmer' }),
		)
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
