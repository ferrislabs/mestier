import { authStore } from '#/store/auth.store'

/**
 * An authenticated fetch against the API's own base URL, followed by a
 * browser download of whatever binary body comes back.
 *
 * Not a `window.tanstackApi` call: the generated client only speaks the
 * `DataEnvelope<T>` JSON shape, and a PDF export is a raw binary body with
 * no envelope to parse. This attaches the bearer token the same way
 * `api.fetch.ts`'s own `fetcher` does — the only other place in the app
 * that builds a request by hand rather than going through the generated
 * client.
 */
export async function downloadAuthenticatedFile(
	path: string,
	filename: string,
): Promise<void> {
	const accessToken = authStore.state.accessToken
	const headers = new Headers()
	if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)

	const response = await fetch(`${window.apiUrl}${path}`, {
		headers,
		credentials: 'include',
	})

	if (!response.ok) {
		throw new Error(`HTTP ${response.status}: ${response.statusText}`)
	}

	const blob = await response.blob()
	const objectUrl = URL.createObjectURL(blob)
	try {
		const link = document.createElement('a')
		link.href = objectUrl
		link.download = filename
		document.body.appendChild(link)
		link.click()
		link.remove()
	} finally {
		URL.revokeObjectURL(objectUrl)
	}
}
