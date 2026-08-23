import { useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { useGatewayEvent } from '#/hooks/use-gateway'

const ORG_CATEGORIES_PATH =
	'/api/v1/chat/organizations/{organization_id}/categories'
const ORG_CHANNELS_PATH =
	'/api/v1/chat/organizations/{organization_id}/channels'
const CHANNEL_PATH = '/api/v1/chat/channels/{channel_id}'

export type Category = Schemas.CategoryResponse
export type Channel = Schemas.ChannelResponse

function categoriesKey(organizationId: string) {
	return window.tanstackApi.get(ORG_CATEGORIES_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

function channelsKey(organizationId: string) {
	return window.tanstackApi.get(ORG_CHANNELS_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

export function useCategories(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(ORG_CATEGORIES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

/**
 * TEXT channels only — threads (`channel_type === 'THREAD'`) are fetched per
 * parent channel (see #326), not listed at the organization level. The
 * server already scopes this to `list_by_organization`, which does not
 * return threads; the filter here is a cheap, harmless belt-and-braces.
 */
export function useChannels(organizationId: string) {
	const query = useQuery({
		...window.tanstackApi.get(ORG_CHANNELS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})

	return {
		...query,
		data: query.data?.filter((channel) => channel.channel_type === 'TEXT'),
	}
}

/** A single channel by id — used for the channel header once one is active. */
export function useChannel(channelId: string) {
	return useQuery({
		...window.tanstackApi.get(CHANNEL_PATH, {
			path: { channel_id: channelId },
		}).queryOptions,
		enabled: Boolean(channelId),
	})
}

/**
 * Subscribes the org's category/channel list queries to the gateway so a
 * create, rename or delete from any tab updates every other tab's sidebar
 * without a refetch. Mount once, near the top of the chat page.
 */
export function useChatListGatewaySync(organizationId: string) {
	const queryClient = useQueryClient()

	useGatewayEvent('CATEGORY_CREATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, categoriesKey(organizationId), event.data)
	})
	useGatewayEvent('CATEGORY_UPDATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, categoriesKey(organizationId), event.data)
	})
	useGatewayEvent('CATEGORY_DELETE', (event) => {
		if (event.data.organization_id !== organizationId) return
		removeFromList(
			queryClient,
			categoriesKey(organizationId),
			event.data.category_id,
		)
	})

	useGatewayEvent('CHANNEL_CREATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, channelsKey(organizationId), event.data)
	})
	useGatewayEvent('CHANNEL_UPDATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, channelsKey(organizationId), event.data)
	})
	useGatewayEvent('CHANNEL_DELETE', (event) => {
		if (event.data.organization_id !== organizationId) return
		removeFromList(
			queryClient,
			channelsKey(organizationId),
			event.data.channel_id,
		)
	})
}

function upsertInList<T extends { id: string }>(
	queryClient: ReturnType<typeof useQueryClient>,
	key: readonly unknown[],
	item: T,
) {
	queryClient.setQueryData<T[]>(key, (old) => {
		if (!old) return [item]
		const index = old.findIndex((existing) => existing.id === item.id)
		if (index === -1) return [...old, item]
		const next = [...old]
		next[index] = item
		return next
	})
}

function removeFromList(
	queryClient: ReturnType<typeof useQueryClient>,
	key: readonly unknown[],
	id: string,
) {
	queryClient.setQueryData<{ id: string }[]>(key, (old) =>
		old?.filter((item) => item.id !== id),
	)
}
