import { useEffect, useState } from 'react'
import {
	useCategories,
	useChannel,
	useChannelPermissions,
	useChannelWebhooks,
	useCreateWebhook,
	useDeleteChannel,
	useDeleteTargetOverwrite,
	useDeleteWebhook,
	useUpdateChannel,
	useUpsertEveryoneOverwrite,
} from '#/hooks/use-chat'
import type {
	OverwriteBitName,
	TriState,
} from '#/pages/chat/lib/permission-bits'
import { withBitState } from '#/pages/chat/lib/permission-bits'
import { ChannelAdminSheetUI } from '#/pages/chat/ui/channel-admin-sheet-ui'

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
	const webhooks = useChannelWebhooks(channelId)
	const createWebhook = useCreateWebhook(channelId)
	const deleteWebhook = useDeleteWebhook(channelId)

	const [nameDraft, setNameDraft] = useState('')
	const [topicDraft, setTopicDraft] = useState('')
	const [categoryIdDraft, setCategoryIdDraft] = useState<string | null>(null)
	const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
	const [newWebhookName, setNewWebhookName] = useState('')
	const [createdWebhookToken, setCreatedWebhookToken] = useState<string | null>(
		null,
	)

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
				setDeleteDialogOpen(false)
				onOpenChange(false)
			},
		})
	}

	const everyoneOverwrite = overwrites.data?.find(
		(overwrite) => overwrite.target_type === 'everyone',
	)
	const roleAndMemberOverwrites = (overwrites.data ?? []).filter(
		(overwrite) => overwrite.target_type !== 'everyone',
	)

	function handleChangeEveryoneBit(bit: OverwriteBitName, state: TriState) {
		const { allow, deny } = withBitState(
			everyoneOverwrite?.allow ?? 0,
			everyoneOverwrite?.deny ?? 0,
			bit,
			state,
		)
		upsertEveryone.mutate({
			path: { channel_id: channelId },
			body: { allow, deny },
		} as never)
	}

	function handleDeleteTargetOverwrite(targetType: string, targetId: string) {
		deleteTargetOverwrite.mutate({
			path: {
				channel_id: channelId,
				target_type: targetType,
				target_id: targetId,
			},
		} as never)
	}

	function handleCreateWebhook() {
		createWebhook.mutate(
			{
				path: { channel_id: channelId },
				body: { name: newWebhookName },
			} as never,
			{
				onSuccess: (response) => {
					setCreatedWebhookToken(
						(response as { data: { token: string } }).data.token,
					)
					setNewWebhookName('')
				},
			},
		)
	}

	return (
		<ChannelAdminSheetUI
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
			deleteDialogOpen={deleteDialogOpen}
			onRequestDelete={() => setDeleteDialogOpen(true)}
			onCancelDelete={() => setDeleteDialogOpen(false)}
			onConfirmDelete={handleConfirmDelete}
			isDeleting={deleteChannel.isPending}
			isLoadingOverwrites={overwrites.isLoading}
			everyoneOverwrite={everyoneOverwrite}
			roleAndMemberOverwrites={roleAndMemberOverwrites}
			onChangeEveryoneBit={handleChangeEveryoneBit}
			onDeleteTargetOverwrite={handleDeleteTargetOverwrite}
			webhooks={webhooks.data ?? []}
			isLoadingWebhooks={webhooks.isLoading}
			newWebhookName={newWebhookName}
			onChangeNewWebhookName={setNewWebhookName}
			onCreateWebhook={handleCreateWebhook}
			isCreatingWebhook={createWebhook.isPending}
			createdWebhookToken={createdWebhookToken}
			onDismissCreatedToken={() => setCreatedWebhookToken(null)}
			onDeleteWebhook={(id) =>
				deleteWebhook.mutate({ path: { webhook_id: id } } as never)
			}
		/>
	)
}
