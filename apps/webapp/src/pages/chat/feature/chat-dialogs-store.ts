import { Store } from '@tanstack/store'
import type { NewItemKind } from '#/pages/chat/lib/channel-kinds'

export interface NewChannelPreset {
	name: string
	kind: NewItemKind
}

interface ChatDialogsState {
	newChannelOpen: boolean
	/** The category a channel is being created in, from its own "+". */
	newChannelCategoryId: string | null
	/** What a duplicated channel starts from, when the dialog opens for it. */
	newChannelPreset: NewChannelPreset | null
	newDirectOpen: boolean
	adminChannelId: string | null
}

const initialState: ChatDialogsState = {
	newChannelOpen: false,
	newChannelCategoryId: null,
	newChannelPreset: null,
	newDirectOpen: false,
	adminChannelId: null,
}

/**
 * Which chat dialog is open. The channel list lives in the app's side nav, a
 * sibling of the chat route rather than a parent of it, so it cannot reach
 * the dialogs through React context: both sides share this store instead.
 * The dialogs themselves are mounted once, by the chat route.
 */
export const chatDialogsStore = new Store<ChatDialogsState>(initialState)

export function openNewChannel(categoryId?: string, preset?: NewChannelPreset) {
	chatDialogsStore.setState((prev) => ({
		...prev,
		newChannelOpen: true,
		newChannelCategoryId: categoryId ?? null,
		newChannelPreset: preset ?? null,
	}))
}

export function setNewChannelOpen(open: boolean) {
	chatDialogsStore.setState((prev) => ({ ...prev, newChannelOpen: open }))
}

export function openNewDirect() {
	chatDialogsStore.setState((prev) => ({ ...prev, newDirectOpen: true }))
}

export function setNewDirectOpen(open: boolean) {
	chatDialogsStore.setState((prev) => ({ ...prev, newDirectOpen: open }))
}

export function openChannelAdmin(channelId: string) {
	chatDialogsStore.setState((prev) => ({ ...prev, adminChannelId: channelId }))
}

export function closeChannelAdmin() {
	chatDialogsStore.setState((prev) => ({ ...prev, adminChannelId: null }))
}

export function resetChatDialogs() {
	chatDialogsStore.setState(() => initialState)
}
