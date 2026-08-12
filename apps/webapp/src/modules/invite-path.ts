/**
 * The one place that assembles an invitation link, so the sheet that shows
 * it and the route that consumes it (`/invite/$token`, `invite.$token.tsx`)
 * can never drift apart on the shape.
 */
export function buildInviteLink(token: string): string {
	return `${window.location.origin}/invite/${encodeURIComponent(token)}`
}
