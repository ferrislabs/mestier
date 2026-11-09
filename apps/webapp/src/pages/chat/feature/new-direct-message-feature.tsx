import { useNavigate } from '@tanstack/react-router'
import { type ReactNode, useState } from 'react'
import { useChatDirectory, useOpenDirectChannel } from '#/hooks/use-chat'
import { useGatewayUserId } from '#/hooks/use-gateway'
import { usePresenceStatuses } from '#/hooks/use-presence'
import { buildOrgPath } from '#/modules/org-path'
import { userHue } from '#/pages/chat/lib/format'
import { NewDirectMessagePopoverUI } from '#/pages/chat/ui/new-direct-message-popover-ui'

export interface NewDirectMessageFeatureProps {
	organizationId: string
	organizationSlug: string
	/** The button that opens the member picker. */
	children: ReactNode
}

export function NewDirectMessageFeature({
	organizationId,
	organizationSlug,
	children,
}: NewDirectMessageFeatureProps) {
	const navigate = useNavigate()
	const currentUserId = useGatewayUserId()
	const directory = useChatDirectory(organizationId)
	const presenceStatuses = usePresenceStatuses()
	const openDirectChannel = useOpenDirectChannel(organizationId)
	const [open, setOpen] = useState(false)

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
					setOpen(false)
					void navigate({
						to: buildOrgPath(organizationSlug, `/discussions/${channel_id}`),
					})
				},
			},
		)
	}

	return (
		<NewDirectMessagePopoverUI
			open={open}
			onOpenChange={setOpen}
			members={members}
			isSubmitting={openDirectChannel.isPending}
			onSelect={handleSelect}
		>
			{children}
		</NewDirectMessagePopoverUI>
	)
}
