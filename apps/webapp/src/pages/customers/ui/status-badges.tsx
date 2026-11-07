import {
	ACCOUNT_STATUS_LABELS,
	type AccountStatus,
} from '#/pages/customers/types'
import { StatusBadge, type Tone } from '#/pages/customers/ui/badges'

const ACCOUNT_TONES: Record<AccountStatus, Tone> = {
	prospect: 'warn',
	client: 'ok',
	archived: 'neutral',
}

export function AccountStatusBadge({ status }: { status: AccountStatus }) {
	return (
		<StatusBadge tone={ACCOUNT_TONES[status]}>
			{ACCOUNT_STATUS_LABELS[status]}
		</StatusBadge>
	)
}
