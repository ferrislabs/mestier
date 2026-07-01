import { authStore } from '#/store/auth.store'

function apiBase(): string {
	return window.apiUrl ?? ''
}

/**
 * Opens an invoice PDF in a new browser tab.
 * The PDF endpoint requires bearer auth and returns binary, so we
 * fetch it manually, convert to a blob URL, and open it.
 */
export async function openInvoicePdf(invoiceId: string): Promise<void> {
	const token = authStore.state.accessToken
	const url = `${apiBase()}/api/v1/invoices/${invoiceId}/pdf`

	const response = await fetch(url, {
		headers: token ? { Authorization: `Bearer ${token}` } : {},
		credentials: 'include',
	})

	if (!response.ok) {
		throw new Error(`PDF indisponible (HTTP ${response.status})`)
	}

	const blob = await response.blob()
	const objectUrl = URL.createObjectURL(blob)
	const tab = window.open(objectUrl, '_blank')

	// Revoke the object URL after a short delay to free memory.
	if (tab) {
		tab.addEventListener('load', () => URL.revokeObjectURL(objectUrl))
	} else {
		setTimeout(() => URL.revokeObjectURL(objectUrl), 60_000)
	}
}

/**
 * Builds the full absolute URL for a public share link.
 * `path` comes from POST /api/v1/invoices/{id}/share → data.path,
 * e.g. "/api/v1/public/documents/{token}".
 */
export function buildShareUrl(path: string): string {
	return `${apiBase()}${path}`
}
