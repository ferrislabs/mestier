import { useState } from 'react'
import {
	useCategories,
	useChannels,
	useCreateCategory,
	useCreateChannel,
} from '#/hooks/use-chat'
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

	const parentCategory = (categories.data ?? []).find(
		(category) => category.id === categoryId,
	)

	function reset() {
		setName('')
		setKind('channel')
	}

	function handleSubmit() {
		if (categoryId !== null || kind === 'channel') {
			const position = channels.data?.length ?? 0
			createChannel.mutate(
				{
					path: { organization_id: organizationId },
					body: { name, position, category_id: categoryId },
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
			onSubmit={handleSubmit}
			isSubmitting={createChannel.isPending || createCategory.isPending}
		/>
	)
}
