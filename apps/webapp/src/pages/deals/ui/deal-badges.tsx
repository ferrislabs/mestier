import { cn } from '#/lib/utils'
import { StatusBadge, type Tone } from '#/pages/customers/ui/badges'
import type { DealQuoteStatus, DealStage } from '#/pages/deals/lib/deal-types'
import { STAGE_LABELS } from '#/pages/deals/lib/pipeline'

const STAGE_TONES: Record<DealStage, Tone> = {
	new: 'neutral',
	visit: 'info',
	quote_sent: 'info',
	negotiation: 'warn',
	won: 'ok',
	lost: 'alert',
}

export function DealStageBadge({
	stage,
	className,
}: {
	stage: DealStage
	className?: string
}) {
	return (
		<StatusBadge tone={STAGE_TONES[stage]} className={cn(className)}>
			{STAGE_LABELS[stage]}
		</StatusBadge>
	)
}

const QUOTE_LABELS: Record<DealQuoteStatus, string> = {
	draft: 'Brouillon',
	sent: 'Envoyé',
	accepted: 'Accepté',
	declined: 'Refusé',
	cancelled: 'Annulé',
	superseded: 'Remplacé',
}

const QUOTE_TONES: Record<DealQuoteStatus, Tone> = {
	draft: 'neutral',
	sent: 'info',
	accepted: 'ok',
	declined: 'alert',
	cancelled: 'neutral',
	superseded: 'neutral',
}

export function DealQuoteStatusBadge({ status }: { status: DealQuoteStatus }) {
	return (
		<StatusBadge tone={QUOTE_TONES[status]}>{QUOTE_LABELS[status]}</StatusBadge>
	)
}
