import { WifiOff } from 'lucide-react'
import { useGatewayConnectionState } from '#/hooks/use-gateway'

/**
 * The one visible sign that the realtime connection dropped. Silence here is
 * the failure mode this issue exists to avoid — a chat that stops receiving
 * without saying so. Only surfaces during `reconnecting`: the first
 * `connecting` (page load) and `closed` (signed out) states are normal and
 * would just add noise.
 */
export function GatewayStatusBanner() {
	const state = useGatewayConnectionState()

	if (state !== 'reconnecting') return null

	return (
		<div className="flex items-center justify-center gap-2 bg-amber-100 px-3 py-1.5 text-xs font-medium text-amber-900 dark:bg-amber-950 dark:text-amber-200">
			<WifiOff className="size-3.5" />
			Reconnexion en cours…
		</div>
	)
}
