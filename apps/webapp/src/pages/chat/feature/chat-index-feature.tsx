import { useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useChannels } from '#/hooks/use-chat'
import { buildOrgPath } from '#/modules/org-path'
import { ChatIndexUI } from '#/pages/chat/ui/chat-index-ui'

/**
 * `/chat` with no channel selected. Redirects to the first channel (by
 * position) as soon as the list resolves — a paste-able URL for "my chat"
 * should not be a permanently empty column when channels exist.
 */
export function ChatIndexFeature() {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const channels = useChannels(activeOrganization.id)

	const firstChannelId = [...(channels.data ?? [])].sort(
		(a, b) => a.position - b.position,
	)[0]?.id

	useEffect(() => {
		if (!firstChannelId) return
		void navigate({
			to: buildOrgPath(activeOrganization.slug, `/chat/${firstChannelId}`),
			replace: true,
		})
	}, [firstChannelId, navigate, activeOrganization.slug])

	if (channels.isLoading) return <ChatIndexUI state="loading" />
	if (!firstChannelId) return <ChatIndexUI state="empty" />
	return <ChatIndexUI state="redirecting" />
}
