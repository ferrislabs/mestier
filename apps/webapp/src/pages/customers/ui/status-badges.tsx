import type { ReactNode } from 'react'
import type { CustomerPipelineStage } from '#/hooks/use-customers'
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

const STAGE_TONES: Record<CustomerPipelineStage, Tone> = {
	NEW: 'neutral',
	CONTACTED: 'info',
	QUALIFIED: 'warn',
	QUOTE_SENT: 'info',
	WON: 'ok',
	LOST: 'alert',
}

export function StageBadge({
	stage,
	children,
}: {
	stage: CustomerPipelineStage
	children: ReactNode
}) {
	return <StatusBadge tone={STAGE_TONES[stage]}>{children}</StatusBadge>
}
