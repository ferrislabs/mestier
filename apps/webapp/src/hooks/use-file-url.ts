import { useQueries } from '@tanstack/react-query'

const FILE_URL_PATH = '/api/v1/files/url'

/** Matches the five-minute link lifetime the API issues, less a safety margin. */
const LINK_STALE_MS = 4 * 60 * 1000

export interface FilePreview {
	key: string
	url: string | undefined
}

/**
 * Resolves storage keys into short-lived readable urls.
 *
 * One query per key rather than one for the batch: keys are stable, so a line
 * that gains a photo reuses the urls already fetched for the others instead of
 * re-signing the whole set. Results are cached just under the link lifetime,
 * which is what stops a quote being reopened from re-signing every photo.
 */
export function useFileUrls(keys: string[]): FilePreview[] {
	return useQueries({
		queries: keys.map((key) => ({
			...window.tanstackApi.get(FILE_URL_PATH, { query: { key } }).queryOptions,
			staleTime: LINK_STALE_MS,
			gcTime: LINK_STALE_MS,
			retry: false,
		})),
		combine: (results) =>
			results.map((result, index) => ({
				key: keys[index] as string,
				url: (result.data as { data?: { url?: string } } | undefined)?.data
					?.url,
			})),
	})
}
