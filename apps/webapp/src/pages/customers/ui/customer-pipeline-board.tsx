import { ArrowRight, GripVertical, ThumbsDown, ThumbsUp } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { StatusBadge } from '#/components/ui/surface'
import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import {
	customerDisplayName,
	customerPipelineStageLabel,
	customerStatusLabel,
} from '#/pages/customers/types'

export const CUSTOMER_PIPELINE_STAGES: CustomerPipelineStage[] = [
	'NEW',
	'CONTACTED',
	'QUALIFIED',
	'QUOTE_SENT',
	'WON',
	'LOST',
]

/**
 * The stages a card advances through with the sequential arrows. `WON` and
 * `LOST` are deliberately excluded: they are terminal, reached only through
 * the explicit "Marquer gagné"/"Marquer perdu" actions below, never through a
 * neighbour-arrow misclick.
 */
const PIPELINE_SEQUENCE_STAGES: CustomerPipelineStage[] = [
	'NEW',
	'CONTACTED',
	'QUALIFIED',
	'QUOTE_SENT',
]

function isTerminalPipelineStage(stage: CustomerPipelineStage): boolean {
	return stage === 'WON' || stage === 'LOST'
}

interface CustomerPipelineBoardProps {
	customers: Customer[]
	canMove: boolean
	isLoading?: boolean
	draggedCustomerId: string | null
	movingCustomerId?: string | null
	onDragStart: (customerId: string) => void
	onDragEnd: () => void
	onDropOnStage: (stage: CustomerPipelineStage) => void
	onMove: (customer: Customer, stage: CustomerPipelineStage) => void
	onOpenCustomer?: (customer: Customer) => void
}

export function CustomerPipelineBoard({
	customers,
	canMove,
	isLoading,
	draggedCustomerId,
	movingCustomerId,
	onDragStart,
	onDragEnd,
	onDropOnStage,
	onMove,
	onOpenCustomer,
}: CustomerPipelineBoardProps) {
	return (
		<div className="min-w-0 max-w-full overflow-x-auto overscroll-x-contain pb-2">
			<div className="grid w-max auto-cols-[16rem] grid-flow-col gap-3">
				{CUSTOMER_PIPELINE_STAGES.map((stage) => {
					const stageCustomers = customers.filter(
						(customer) => customer.pipeline_stage === stage,
					)

					return (
						<section
							key={stage}
							aria-label={`Pipeline ${customerPipelineStageLabel(stage)}`}
							className="flex min-h-[28rem] min-w-0 flex-col overflow-hidden rounded-lg border bg-card"
							onDragOver={(event) => event.preventDefault()}
							onDrop={() => onDropOnStage(stage)}
						>
							<div className="border-b px-3 py-3">
								<div className="flex items-center justify-between gap-2">
									<p className="min-w-0 truncate text-sm font-semibold">
										{customerPipelineStageLabel(stage)}
									</p>
									<StatusBadge tone={pipelineStageTone(stage)}>
										{stageCustomers.length}
									</StatusBadge>
								</div>
								<p className="mt-1 truncate text-xs text-muted-foreground">
									{pipelineStageDescription(stage)}
								</p>
							</div>

							<div className="flex flex-1 flex-col gap-2 bg-muted/25 p-2">
								{isLoading ? (
									<div className="flex flex-1 items-center justify-center rounded-md border border-dashed bg-card/70 p-4 text-center text-xs text-muted-foreground">
										Chargement…
									</div>
								) : stageCustomers.length === 0 ? (
									<div className="flex flex-1 items-center justify-center rounded-md border border-dashed bg-card/70 p-4 text-center text-xs text-muted-foreground">
										Aucune carte
									</div>
								) : (
									stageCustomers.map((customer) => (
										<PipelineCustomerCard
											key={customer.id}
											customer={customer}
											canMove={canMove}
											isDragging={draggedCustomerId === customer.id}
											isMoving={movingCustomerId === customer.id}
											onDragStart={() => onDragStart(customer.id)}
											onDragEnd={onDragEnd}
											onMove={onMove}
											onOpen={() => onOpenCustomer?.(customer)}
										/>
									))
								)}
							</div>
						</section>
					)
				})}
			</div>
		</div>
	)
}

interface PipelineCustomerCardProps {
	customer: Customer
	canMove: boolean
	isDragging: boolean
	isMoving: boolean
	onDragStart: () => void
	onDragEnd: () => void
	onMove: (customer: Customer, stage: CustomerPipelineStage) => void
	onOpen: () => void
}

function PipelineCustomerCard({
	customer,
	canMove,
	isDragging,
	isMoving,
	onDragStart,
	onDragEnd,
	onMove,
	onOpen,
}: PipelineCustomerCardProps) {
	const isTerminal = isTerminalPipelineStage(customer.pipeline_stage)
	const sequenceIndex = PIPELINE_SEQUENCE_STAGES.indexOf(
		customer.pipeline_stage,
	)
	const previousStage =
		sequenceIndex > 0 ? PIPELINE_SEQUENCE_STAGES[sequenceIndex - 1] : undefined
	const nextStage =
		sequenceIndex >= 0 && sequenceIndex < PIPELINE_SEQUENCE_STAGES.length - 1
			? PIPELINE_SEQUENCE_STAGES[sequenceIndex + 1]
			: undefined

	return (
		<article
			draggable={canMove}
			onDragStart={onDragStart}
			onDragEnd={onDragEnd}
			className={`group rounded-md border bg-card p-3 shadow-xs transition ${
				isDragging ? 'opacity-50 ring-2 ring-primary/30' : 'hover:shadow-md'
			}`}
		>
			<div className="flex items-start gap-2">
				<GripVertical className="mt-0.5 size-4 shrink-0 cursor-grab text-muted-foreground" />
				<button
					type="button"
					className="min-w-0 flex-1 text-left outline-none"
					onClick={onOpen}
				>
					<p className="truncate text-sm font-semibold">
						{customerDisplayName(customer)}
					</p>
					<p className="mt-0.5 truncate text-xs text-muted-foreground">
						{customer.email || customer.phone || 'Coordonnées à compléter'}
					</p>
				</button>
			</div>

			<div className="mt-3 flex items-center justify-between gap-2">
				<StatusBadge tone={customerStatusTone(customer.status)}>
					{customerStatusLabel(customer.status)}
				</StatusBadge>
				<div className="flex items-center gap-1 opacity-100 transition-opacity sm:opacity-0 sm:group-hover:opacity-100 sm:group-focus-within:opacity-100">
					{canMove && previousStage ? (
						<Button
							type="button"
							size="icon-sm"
							variant="ghost"
							disabled={isMoving}
							onClick={() => onMove(customer, previousStage)}
						>
							<ArrowRight className="rotate-180" />
							<span className="sr-only">Reculer</span>
						</Button>
					) : null}
					{canMove && nextStage ? (
						<Button
							type="button"
							size="icon-sm"
							variant="ghost"
							disabled={isMoving}
							onClick={() => onMove(customer, nextStage)}
						>
							<ArrowRight />
							<span className="sr-only">Avancer</span>
						</Button>
					) : null}
					{canMove && !isTerminal ? (
						<>
							<Button
								type="button"
								size="icon-sm"
								variant="ghost"
								title="Marquer gagné"
								disabled={isMoving}
								onClick={() => onMove(customer, 'WON')}
							>
								<ThumbsUp className="text-success" />
								<span className="sr-only">Marquer gagné</span>
							</Button>
							<Button
								type="button"
								size="icon-sm"
								variant="ghost"
								title="Marquer perdu"
								disabled={isMoving}
								onClick={() => onMove(customer, 'LOST')}
							>
								<ThumbsDown className="text-destructive" />
								<span className="sr-only">Marquer perdu</span>
							</Button>
						</>
					) : null}
				</div>
			</div>
		</article>
	)
}

function customerStatusTone(status: Customer['status']) {
	if (status === 'CLIENT') return 'success'
	if (status === 'ARCHIVED') return 'neutral'
	return 'warning'
}

export function pipelineStageTone(stage: CustomerPipelineStage) {
	if (stage === 'WON') return 'success'
	if (stage === 'LOST') return 'error'
	if (stage === 'QUOTE_SENT') return 'brand'
	if (stage === 'QUALIFIED') return 'warning'
	return 'neutral'
}

export function pipelineStageDescription(stage: CustomerPipelineStage): string {
	if (stage === 'NEW') return 'À qualifier'
	if (stage === 'CONTACTED') return 'Premier échange lancé'
	if (stage === 'QUALIFIED') return 'Besoin confirmé'
	if (stage === 'QUOTE_SENT') return 'Offre transmise'
	if (stage === 'WON') return 'Converti en client'
	return 'Opportunité perdue'
}
