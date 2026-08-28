import { createContext, useContext, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { ChannelAdminFeature } from './channel-admin-feature'

interface ChannelAdminDialogContextValue {
	openChannelAdmin: (channelId: string) => void
}

const ChannelAdminDialogContext =
	createContext<ChannelAdminDialogContextValue | null>(null)

/**
 * Lifts the channel admin sheet up to the chat shell so it can be opened
 * from anywhere under it — the sidebar's per-channel menu, the active
 * channel's own header — rather than being re-mounted once per channel
 * page. See #372: before this, the sheet only existed inside
 * `ChatChannelFeature`, so managing a channel required first navigating
 * into it.
 */
export function ChannelAdminDialogProvider({
	children,
}: {
	children: React.ReactNode
}) {
	const { activeOrganization } = useActiveOrganization()
	const [channelId, setChannelId] = useState<string | null>(null)
	const [open, setOpen] = useState(false)

	const value = useMemo<ChannelAdminDialogContextValue>(
		() => ({
			openChannelAdmin: (id: string) => {
				setChannelId(id)
				setOpen(true)
			},
		}),
		[],
	)

	return (
		<ChannelAdminDialogContext.Provider value={value}>
			{children}
			{channelId ? (
				<ChannelAdminFeature
					channelId={channelId}
					organizationId={activeOrganization.id}
					open={open}
					onOpenChange={setOpen}
				/>
			) : null}
		</ChannelAdminDialogContext.Provider>
	)
}

export function useChannelAdminDialog() {
	const context = useContext(ChannelAdminDialogContext)
	if (!context) {
		throw new Error(
			'useChannelAdminDialog must be used inside ChannelAdminDialogProvider',
		)
	}
	return context
}
