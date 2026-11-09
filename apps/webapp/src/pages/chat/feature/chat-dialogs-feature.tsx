import { useStore } from '@tanstack/react-store'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { ChannelAdminFeature } from './channel-admin-feature'
import {
	chatDialogsStore,
	closeChannelAdmin,
	setNewChannelOpen,
	setNewDirectOpen,
} from './chat-dialogs-store'
import { NewChannelFeature } from './new-channel-feature'
import { NewDirectMessageFeature } from './new-direct-message-feature'

/** The chat dialogs, mounted once by the chat route and driven by
 * `chatDialogsStore` from the side nav and from the channel header. */
export function ChatDialogsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const {
		newChannelOpen,
		newChannelCategoryId,
		newChannelPreset,
		newDirectOpen,
		adminChannelId,
	} = useStore(chatDialogsStore)

	return (
		<>
			<NewChannelFeature
				organizationId={activeOrganization.id}
				open={newChannelOpen}
				categoryId={newChannelCategoryId}
				preset={newChannelPreset}
				onOpenChange={setNewChannelOpen}
			/>
			<NewDirectMessageFeature
				organizationId={activeOrganization.id}
				organizationSlug={activeOrganization.slug}
				open={newDirectOpen}
				onOpenChange={setNewDirectOpen}
			/>
			{adminChannelId ? (
				<ChannelAdminFeature
					channelId={adminChannelId}
					organizationId={activeOrganization.id}
					open
					onOpenChange={(open) => {
						if (!open) closeChannelAdmin()
					}}
				/>
			) : null}
		</>
	)
}
