import { useState } from 'react'
import {
	useCategories,
	useChannels,
	useChatRoles,
	useCreateCategory,
	useCreateChannel,
} from '#/hooks/use-chat'
import type { ChannelKind } from '#/pages/chat/lib/channel-kinds'
import { NewChannelDialogUI } from '#/pages/chat/ui/new-channel-dialog-ui'

export interface NewChannelFeatureProps {
	organizationId: string
	open: boolean
	/** The category to create the channel in, when opened from one. */
	categoryId: string | null
	onOpenChange: (open: boolean) => void
}

export function NewChannelFeature({
	organizationId,
	open,
	categoryId,
	onOpenChange,
}: NewChannelFeatureProps) {
	const categories = useCategories(organizationId)
	const channels = useChannels(organizationId)
	const createChannel = useCreateChannel(organizationId)
	const createCategory = useCreateCategory(organizationId)

	const [kind, setKind] = useState<'channel' | 'category'>('channel')
	const [name, setName] = useState('')
	const [channelKind, setChannelKind] = useState<ChannelKind>('TEXT')
	const [isPrivate, setIsPrivate] = useState(false)
	const [selectedRoleIds, setSelectedRoleIds] = useState<Set<string>>(new Set())
	const roles = useChatRoles(organizationId, open && isPrivate)

	const parentCategory = (categories.data ?? []).find(
		(category) => category.id === categoryId,
	)

	function reset() {
		setName('')
		setKind('channel')
		setChannelKind('TEXT')
		setIsPrivate(false)
		setSelectedRoleIds(new Set())
	}

	function handleSubmit() {
		if (categoryId !== null || kind === 'channel') {
			const position = channels.data?.length ?? 0
			createChannel.mutate(
				{
					path: { organization_id: organizationId },
					body: {
						name,
						position,
						category_id: categoryId,
						channel_type: channelKind,
						allowed_role_ids: isPrivate ? [...selectedRoleIds] : null,
					},
				} as never,
				{
					onSuccess: () => {
						reset()
						onOpenChange(false)
					},
				},
			)
			return
		}

		const position = categories.data?.length ?? 0
		createCategory.mutate(
			{
				path: { organization_id: organizationId },
				body: { name, position },
			} as never,
			{
				onSuccess: () => {
					reset()
					onOpenChange(false)
				},
			},
		)
	}

	return (
		<NewChannelDialogUI
			open={open}
			onOpenChange={onOpenChange}
			kind={categoryId === null ? kind : 'channel'}
			onChangeKind={setKind}
			name={name}
			onChangeName={setName}
			parentCategoryName={parentCategory?.name}
			channelKind={channelKind}
			onChangeChannelKind={setChannelKind}
			isPrivate={isPrivate}
			onChangePrivate={setIsPrivate}
			roles={(roles.data ?? []).map((role) => ({
				id: role.role_id,
				name: role.name,
			}))}
			rolesLoading={roles.isLoading}
			selectedRoleIds={selectedRoleIds}
			onToggleRole={(roleId, selected) =>
				setSelectedRoleIds((previous) => {
					const next = new Set(previous)
					if (selected) next.add(roleId)
					else next.delete(roleId)
					return next
				})
			}
			onSubmit={handleSubmit}
			isSubmitting={createChannel.isPending || createCategory.isPending}
		/>
	)
}
