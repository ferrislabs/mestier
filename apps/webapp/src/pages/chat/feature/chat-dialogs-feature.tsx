import { useStore } from '@tanstack/react-store'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { ChannelAdminFeature } from './channel-admin-feature'
import {
	chatDialogsStore,
	closeChannelAdmin,
	setNewChannelOpen,
} from './chat-dialogs-store'
import { NewChannelFeature } from './new-channel-feature'

/** The chat dialogs, mounted once by the chat route and driven by
 * `chatDialogsStore` from the side nav and from the channel header. */
export function ChatDialogsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const {
		newChannelOpen,
		newChannelCategoryId,
		newChannelPreset,
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
