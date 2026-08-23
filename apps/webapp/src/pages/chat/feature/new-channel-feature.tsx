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
	onOpenChange: (open: boolean) => void
}

export function NewChannelFeature({
	organizationId,
	open,
	onOpenChange,
}: NewChannelFeatureProps) {
	const categories = useCategories(organizationId)
	const channels = useChannels(organizationId)
	const createChannel = useCreateChannel(organizationId)
	const createCategory = useCreateCategory(organizationId)

	const [kind, setKind] = useState<'channel' | 'category'>('channel')
	const [name, setName] = useState('')
	const [categoryId, setCategoryId] = useState<string | null>(null)

	function reset() {
		setName('')
		setCategoryId(null)
	}

	function handleSubmit() {
		if (kind === 'channel') {
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
			kind={kind}
			onChangeKind={setKind}
			name={name}
			onChangeName={setName}
			categories={categories.data ?? []}
			categoryId={categoryId}
			onChangeCategoryId={setCategoryId}
			onSubmit={handleSubmit}
			isSubmitting={createChannel.isPending || createCategory.isPending}
		/>
	)
}
