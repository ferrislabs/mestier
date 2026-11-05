import { StatusBadge, type Tone } from '#/pages/customers/ui/badges'
import {
	STRIPE_STATUS_LABELS,
	type StripeEventStatus,
} from '#/pages/stripe/types'

const STRIPE_TONES: Record<StripeEventStatus, Tone> = {
	processed: 'ok',
	to_review: 'warn',
	ignored: 'neutral',
}

export function StripeStatusBadge({ status }: { status: StripeEventStatus }) {
	return (
		<StatusBadge tone={STRIPE_TONES[status]}>
			{STRIPE_STATUS_LABELS[status]}
		</StatusBadge>
	)
}
