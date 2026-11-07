import { useRef } from 'react'
import type { Customer } from '#/hooks/use-customers'
import { cn } from '#/lib/utils'
import { Money } from '#/pages/customers/ui/money'
import type {
	Deal,
	DealQuote,
	DealStage,
	IsoDate,
} from '#/pages/deals/lib/deal-types'
import {
	dealsInStage,
	STAGE_LABELS,
	totalAmount,
} from '#/pages/deals/lib/pipeline'
import { dealQuotes } from '#/pages/deals/lib/pipeline-filters'
import { DealStageBadge } from '#/pages/deals/ui/deal-badges'
import { DealCard } from '#/pages/deals/ui/deal-card'
import { useBoardDrag } from '#/pages/deals/ui/use-board-drag'
import { useFlip } from '#/pages/deals/ui/use-flip'

const EMPTY_HINTS: Record<DealStage, string> = {
	new: 'Aucune nouvelle demande.',
	visit: 'Aucune visite en cours.',
	quote_sent: 'Aucun devis en attente de réponse.',
	negotiation: 'Aucune négociation en cours.',
	won: 'Aucune opportunité gagnée.',
	lost: 'Aucune opportunité perdue.',
}

export function PipelineBoard({
	deals,
	stages,
	customers,
	quotes,
	today,
	organizationSlug,
	canManage,
	isLoading,
	onOpen,
	onMove,
}: {
	deals: readonly Deal[]
	stages: readonly DealStage[]
	customers: readonly Customer[]
	quotes: readonly DealQuote[]
	today: IsoDate
	organizationSlug: string
	canManage: boolean
	isLoading?: boolean
	onOpen: (dealId: string) => void
	onMove: (dealId: string, stage: DealStage) => void
}) {
	const boardRef = useRef<HTMLDivElement>(null)
	const droppedIdRef = useRef<string | null>(null)
	const { draggedId, overStage, draggedHeight, start } = useBoardDrag({
		boardRef,
		skipIdRef: droppedIdRef,
		stageOf: (dealId) => deals.find((deal) => deal.id === dealId)?.stage,
		onDrop: (dealId, stage) => {
			if (canManage) onMove(dealId, stage)
		},
	})
	const dragged = deals.find((deal) => deal.id === draggedId)
	useFlip(boardRef, droppedIdRef)

	return (
		<div
			ref={boardRef}
			className="grid min-h-0 auto-cols-[minmax(16rem,1fr)] grid-flow-col gap-3 overflow-x-auto pb-2"
		>
			{stages.map((stage) => {
				const column = dealsInStage(deals, stage)
				const droppable =
					canManage && dragged !== undefined && dragged.stage !== stage
				return (
					<section
						key={stage}
						aria-label={STAGE_LABELS[stage]}
						data-stage={stage}
						className={cn(
							'flex min-h-[24rem] flex-col rounded-lg border border-transparent bg-muted/50 p-2 transition-colors',
							droppable && 'border-border border-dashed',
							overStage === stage && droppable && 'border-ring bg-accent/60',
						)}
					>
						<header className="grid gap-1 px-1.5 pt-1 pb-2.5">
							<div className="flex items-center gap-2">
								<h2 className="min-w-0">
									<DealStageBadge stage={stage} />
								</h2>
								<span className="ml-auto text-muted-foreground text-xs tabular-nums">
									{column.length}
								</span>
							</div>
							<p className="text-muted-foreground text-xs">
								<Money cents={totalAmount(column)} /> HT
							</p>
						</header>
						<div className="grid content-start gap-2">
							{droppable && overStage === stage && (
								<div
									aria-hidden
									style={{ height: draggedHeight }}
									className="rounded-md border border-ring border-dashed bg-accent/50"
								/>
							)}
							{isLoading ? (
								<p className="px-2 py-6 text-center text-muted-foreground text-xs">
									Chargement…
								</p>
							) : (
								column.map((deal) => (
									<DealCard
										key={deal.id}
										deal={deal}
										customer={customers.find(
											(customer) => customer.id === deal.customerId,
										)}
										quotes={dealQuotes(quotes, deal.id)}
										today={today}
										organizationSlug={organizationSlug}
										canManage={canManage}
										dragging={deal.id === draggedId}
										onOpen={() => onOpen(deal.id)}
										onMove={(next) => onMove(deal.id, next)}
										onPointerDown={(event) => {
											if (canManage) start(event, deal.id)
										}}
									/>
								))
							)}
							{!isLoading && column.length === 0 && (
								<p className="px-2 py-6 text-center text-muted-foreground text-xs">
									{EMPTY_HINTS[stage]}
								</p>
							)}
						</div>
					</section>
				)
			})}
		</div>
	)
}
