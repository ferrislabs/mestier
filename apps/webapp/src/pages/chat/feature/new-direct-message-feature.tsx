import { useNavigate } from '@tanstack/react-router'
import { useChatDirectory, useOpenDirectChannel } from '#/hooks/use-chat'
import { useGatewayUserId } from '#/hooks/use-gateway'
import { usePresenceStatuses } from '#/hooks/use-presence'
import { buildOrgPath } from '#/modules/org-path'
import { userHue } from '#/pages/chat/lib/format'
import { NewDirectMessageDialogUI } from '#/pages/chat/ui/new-direct-message-dialog-ui'

export interface NewDirectMessageFeatureProps {
	organizationId: string
	organizationSlug: string
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function NewDirectMessageFeature({
	organizationId,
	organizationSlug,
	open,
	onOpenChange,
}: NewDirectMessageFeatureProps) {
	const navigate = useNavigate()
	const currentUserId = useGatewayUserId()
	const directory = useChatDirectory(organizationId)
	const presenceStatuses = usePresenceStatuses()
	const openDirectChannel = useOpenDirectChannel(organizationId)

	const members = (directory.data ?? [])
		.filter((entry) => entry.user_id !== currentUserId)
		.map((entry) => ({
			id: entry.user_id,
			name: entry.display_name,
			hue: userHue(entry.user_id),
			presence: presenceStatuses.get(entry.user_id) ?? entry.presence,
		}))

	function handleSelect(userId: string) {
		openDirectChannel.mutate(
			{
				path: { organization_id: organizationId },
				body: { user_id: userId },
			} as never,
			{
				onSuccess: (response) => {
					const { channel_id } = (response as { data: { channel_id: string } })
						.data
					onOpenChange(false)
					void navigate({
						to: buildOrgPath(organizationSlug, `/discussions/${channel_id}`),
					})
				},
			},
		)
	}

	return (
		<NewDirectMessageDialogUI
			open={open}
			onOpenChange={onOpenChange}
			members={members}
			isSubmitting={openDirectChannel.isPending}
			onSelect={handleSelect}
		/>
	)
}
