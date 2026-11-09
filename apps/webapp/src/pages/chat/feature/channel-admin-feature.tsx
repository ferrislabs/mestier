import { useEffect, useState } from 'react'
import {
	useCategories,
	useChannel,
	useChannelPermissions,
	useChannelWebhooks,
	useChatDirectory,
	useChatRoles,
	useCreateWebhook,
	useDeleteChannel,
	useDeleteTargetOverwrite,
	useDeleteWebhook,
	useUpdateChannel,
	useUpdateWebhook,
	useUpsertTargetOverwrite,
	useUpsertEveryoneOverwrite,
} from '#/hooks/use-chat'
import {
	buildPermissionTargets,
	EVERYONE_KEY,
	type PermissionTargetType,
} from '#/pages/chat/lib/permission-targets'
import {
	OVERWRITE_BITS,
	type OverwriteBitName,
	readBitState,
	type TriState,
	withBitState,
} from '#/pages/chat/lib/permission-bits'
import { ChannelAdminDialogUI } from '#/pages/chat/ui/channel-admin-dialog-ui'

export interface ChannelAdminFeatureProps {
	channelId: string
	organizationId: string
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function ChannelAdminFeature({
	channelId,
	organizationId,
	open,
	onOpenChange,
}: ChannelAdminFeatureProps) {
	const channel = useChannel(channelId)
	const categories = useCategories(organizationId)
	const updateChannel = useUpdateChannel(organizationId)
	const deleteChannel = useDeleteChannel(organizationId)
	const overwrites = useChannelPermissions(channelId)
	const upsertEveryone = useUpsertEveryoneOverwrite(channelId)
	const deleteTargetOverwrite = useDeleteTargetOverwrite(channelId)
	const upsertTarget = useUpsertTargetOverwrite(channelId)
	const roles = useChatRoles(organizationId, open)
	const [selectedPermissionKey, setSelectedPermissionKey] =
		useState(EVERYONE_KEY)
	const webhooks = useChannelWebhooks(channelId)
	const createWebhook = useCreateWebhook(channelId)
	const deleteWebhook = useDeleteWebhook(channelId)
	const updateWebhook = useUpdateWebhook(channelId)
	const directory = useChatDirectory(organizationId)

	const [nameDraft, setNameDraft] = useState('')
	const [topicDraft, setTopicDraft] = useState('')
	const [categoryIdDraft, setCategoryIdDraft] = useState<string | null>(null)
	const [newWebhookName, setNewWebhookName] = useState('')
	const [createdWebhookToken, setCreatedWebhookToken] = useState<string | null>(
		null,
	)
	const [createdWebhookId, setCreatedWebhookId] = useState<string | null>(null)
	const webhookCallUrlBase = `${window.apiUrl || window.location.origin}/api/v1/chat/webhooks`

	// Re-seed the drafts from the server every time the sheet opens, so an
	// edit abandoned by closing without saving never lingers into the next
	// open.
	useEffect(() => {
		if (open && channel.data) {
			setNameDraft(channel.data.name)
			setTopicDraft(channel.data.topic ?? '')
			setCategoryIdDraft(channel.data.category_id ?? null)
		}
	}, [open, channel.data])

	function handleSaveGeneral() {
		updateChannel.mutate({
			path: { channel_id: channelId },
			body: {
				name: nameDraft,
				topic: topicDraft.length > 0 ? topicDraft : null,
				category_id: categoryIdDraft,
				position: channel.data?.position ?? 0,
			},
		} as never)
	}

	function handleConfirmDelete() {
		deleteChannel.mutate({ path: { channel_id: channelId } } as never, {
			onSuccess: () => {
				onOpenChange(false)
			},
		})
	}

	const permissionTargets = buildPermissionTargets(
		overwrites.data ?? [],
		(id) => roles.data?.find((role) => role.role_id === id)?.name,
		(id) => directory.data?.find((entry) => entry.user_id === id)?.display_name,
	)
	const everyoneTarget = permissionTargets[0]
	const isPrivate =
		readBitState(
			everyoneTarget?.allow ?? 0,
			everyoneTarget?.deny ?? 0,
			'VIEW_CHANNEL',
		) === 'deny'
	const addableRoles = (roles.data ?? [])
		.filter(
			(role) =>
				!permissionTargets.some(
					(target) => target.key === `role:${role.role_id}`,
				),
		)
		.map((role) => ({ id: role.role_id, name: role.name }))
	const addableMembers = (directory.data ?? [])
		.filter(
			(entry) =>
				!permissionTargets.some(
					(target) => target.key === `member:${entry.user_id}`,
				),
		)
		.map((entry) => ({ id: entry.user_id, name: entry.display_name }))

	function saveTarget(
		type: PermissionTargetType,
		id: string | null,
		allow: number,
		deny: number,
	) {
		if (type === 'everyone') {
			upsertEveryone.mutate({
				path: { channel_id: channelId },
				body: { allow, deny },
			} as never)
			return
		}
		upsertTarget.mutate({
			path: { channel_id: channelId, target_type: type, target_id: id },
			body: { allow, deny },
		} as never)
	}

	function handleChangeTargetBit(
		key: string,
		bit: OverwriteBitName,
		state: TriState,
	) {
		const target = permissionTargets.find((candidate) => candidate.key === key)
		if (!target) return
		const { allow, deny } = withBitState(target.allow, target.deny, bit, state)
		saveTarget(target.type, target.id, allow, deny)
	}

	function handleTogglePrivate(next: boolean) {
		handleChangeTargetBit(
			EVERYONE_KEY,
			'VIEW_CHANNEL',
			next ? 'deny' : 'inherit',
		)
	}

	function handleAddTarget(type: 'role' | 'member', id: string) {
		saveTarget(type, id, OVERWRITE_BITS.VIEW_CHANNEL, 0)
		setSelectedPermissionKey(`${type}:${id}`)
	}

	function handleRemoveTarget(key: string) {
		const target = permissionTargets.find((candidate) => candidate.key === key)
		if (!target?.id) return
		deleteTargetOverwrite.mutate({
			path: {
				channel_id: channelId,
				target_type: target.type,
				target_id: target.id,
			},
		} as never)
		setSelectedPermissionKey(EVERYONE_KEY)
	}

	function handleCreateWebhook() {
		createWebhook.mutate(
			{
				path: { channel_id: channelId },
				body: { name: newWebhookName },
			} as never,
			{
				onSuccess: (response) => {
					const created = (response as { data: { id: string; token: string } })
						.data
					setCreatedWebhookToken(created.token)
					setCreatedWebhookId(created.id)
					setNewWebhookName('')
				},
			},
		)
	}

	return (
		<ChannelAdminDialogUI
			open={open}
			onOpenChange={onOpenChange}
			channel={channel.data}
			categories={categories.data ?? []}
			nameDraft={nameDraft}
			topicDraft={topicDraft}
			categoryIdDraft={categoryIdDraft}
			onChangeName={setNameDraft}
			onChangeTopic={setTopicDraft}
			onChangeCategory={setCategoryIdDraft}
			onSaveGeneral={handleSaveGeneral}
			isSavingGeneral={updateChannel.isPending}
			onConfirmDelete={handleConfirmDelete}
			isDeleting={deleteChannel.isPending}
			isLoadingOverwrites={overwrites.isLoading}
			isPrivate={isPrivate}
			onTogglePrivate={handleTogglePrivate}
			permissionTargets={permissionTargets}
			selectedPermissionKey={selectedPermissionKey}
			onSelectPermissionKey={setSelectedPermissionKey}
			addableRoles={addableRoles}
			addableMembers={addableMembers}
			onAddTarget={handleAddTarget}
			onChangeTargetBit={handleChangeTargetBit}
			onRemoveTarget={handleRemoveTarget}
			webhooks={webhooks.data ?? []}
			isLoadingWebhooks={webhooks.isLoading}
			newWebhookName={newWebhookName}
			onChangeNewWebhookName={setNewWebhookName}
			onCreateWebhook={handleCreateWebhook}
			isCreatingWebhook={createWebhook.isPending}
			createdWebhookToken={createdWebhookToken}
			webhookCallUrlBase={webhookCallUrlBase}
			createdWebhookCallUrl={
				createdWebhookToken && createdWebhookId
					? `${webhookCallUrlBase}/${createdWebhookId}/${createdWebhookToken}`
					: null
			}
			onDismissCreatedToken={() => setCreatedWebhookToken(null)}
			creatorName={(userId) =>
				directory.data?.find((entry) => entry.user_id === userId)?.display_name
			}
			onRenameWebhook={(id, name) =>
				updateWebhook.mutate({
					path: { webhook_id: id },
					body: { name },
				} as never)
			}
			onDeleteWebhook={(id) =>
				deleteWebhook.mutate({ path: { webhook_id: id } } as never)
			}
		/>
	)
}
