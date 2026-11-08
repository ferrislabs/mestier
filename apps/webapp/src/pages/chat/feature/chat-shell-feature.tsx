import { Outlet } from '@tanstack/react-router'
import { ChatDialogsFeature } from './chat-dialogs-feature'

/**
 * The chat layout: the active channel's own route through `Outlet`, and the
 * dialogs the side nav and the channel header open. The channel list itself is
 * the module's side nav (`ChatSidebarFeature`), where the rework puts it.
 */
export function ChatShellFeature() {
	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<Outlet />
			<ChatDialogsFeature />
		</div>
	)
}
