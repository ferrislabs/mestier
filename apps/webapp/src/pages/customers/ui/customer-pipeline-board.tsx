import { EllipsisVertical } from 'lucide-react'
import { type PointerEvent, useRef } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import { cn } from '#/lib/utils'
import type { AccountExtras } from '#/pages/customers/feature/mocks/account-extras'
import { findPerson } from '#/pages/customers/feature/mocks/people'
import {
	PIPELINE_STAGES,
	pipelineStageDescription,
} from '#/pages/customers/lib/pipeline-filters'
import {
	ACCOUNT_KIND_LABELS,
	accountStatusOf,
	customerPipelineStageLabel,
	kindOf,
	LEAD_SOURCE_LABELS,
} from '#/pages/customers/types'
import { hueOf, NameAvatar, PersonAvatar } from '#/pages/customers/ui/avatars'
import {
	AccountStatusBadge,
	StageBadge,
} from '#/pages/customers/ui/status-badges'
import { useBoardDrag } from '#/pages/customers/ui/use-board-drag'
import { useFlip } from '#/pages/customers/ui/use-flip'

export const CUSTOMER_PIPELINE_STAGES = PIPELINE_STAGES

const EMPTY_HINTS: Record<CustomerPipelineStage, string> = {
	NEW: 'Aucune nouvelle demande.',
	CONTACTED: 'Aucun prospect contacté.',
	QUALIFIED: 'Aucun prospect qualifié.',
	QUOTE_SENT: 'Aucun devis en attente de réponse.',
	WON: 'Aucune opportunité gagnée.',
	LOST: 'Aucune opportunité perdue.',
}

interface CustomerPipelineBoardProps {
	customers: readonly Customer[]
	stages?: readonly CustomerPipelineStage[]
	extrasOf: (customerId: string) => AccountExtras
	canMove: boolean
	isLoading?: boolean
	movingCustomerId?: string | null
	onMove: (customer: Customer, stage: CustomerPipelineStage) => void
	onOpenCustomer?: (customer: Customer) => void
}

export function CustomerPipelineBoard({
	customers,
	stages = PIPELINE_STAGES,
	extrasOf,
	canMove,
	isLoading,
	movingCustomerId,
	onMove,
	onOpenCustomer,
}: CustomerPipelineBoardProps) {
	const boardRef = useRef<HTMLDivElement>(null)
	const droppedIdRef = useRef<string | null>(null)
	const { draggedId, overStage, draggedHeight, start } = useBoardDrag({
		boardRef,
		skipIdRef: droppedIdRef,
		stageOf: (customerId) =>
			customers.find((customer) => customer.id === customerId)?.pipeline_stage,
		onDrop: (customerId, stage) => {
			const customer = customers.find((item) => item.id === customerId)
			if (customer && canMove) onMove(customer, stage)
		},
	})
	const dragged = customers.find((customer) => customer.id === draggedId)
	useFlip(boardRef, droppedIdRef)

	return (
		<div
			ref={boardRef}
			className="grid min-h-0 auto-cols-[minmax(16rem,1fr)] grid-flow-col gap-3 overflow-x-auto pb-2"
		>
			{stages.map((stage) => {
				const column = customers.filter(
					(customer) => customer.pipeline_stage === stage,
				)
				const droppable =
					canMove && dragged !== undefined && dragged.pipeline_stage !== stage
				return (
					<section
						key={stage}
						aria-label={`Pipeline ${customerPipelineStageLabel(stage)}`}
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
									<StageBadge stage={stage}>
										{customerPipelineStageLabel(stage)}
									</StageBadge>
								</h2>
								<span className="ml-auto text-muted-foreground text-xs tabular-nums">
									{column.length}
								</span>
							</div>
							<p className="text-muted-foreground text-xs">
								{pipelineStageDescription(stage)}
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
								column.map((customer) => (
									<PipelineCustomerCard
										key={customer.id}
										customer={customer}
										extras={extrasOf(customer.id)}
										canMove={canMove}
										dragging={customer.id === draggedId}
										isMoving={movingCustomerId === customer.id}
										onOpen={() => onOpenCustomer?.(customer)}
										onMove={(next) => onMove(customer, next)}
										onPointerDown={(event) => {
											if (canMove) start(event, customer.id)
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

interface PipelineCustomerCardProps {
	customer: Customer
	extras: AccountExtras
	canMove: boolean
	dragging: boolean
	isMoving: boolean
	onOpen: () => void
	onMove: (stage: CustomerPipelineStage) => void
	onPointerDown: (event: PointerEvent<HTMLElement>) => void
}

function PipelineCustomerCard({
	customer,
	extras,
	canMove,
	dragging,
	isMoving,
	onOpen,
	onMove,
	onPointerDown,
}: PipelineCustomerCardProps) {
	const person = findPerson(extras.ownerId)
	const kind = kindOf(customer, extras)
	return (
		<article
			data-flip-id={customer.id}
			onPointerDown={onPointerDown}
			draggable={false}
			className={cn(
				'relative grid gap-2 rounded-md border border-border bg-card p-3 shadow-xs transition-[box-shadow,opacity] focus-within:ring-2 focus-within:ring-ring hover:border-ring/60 hover:shadow-sm',
				canMove && 'cursor-grab',
				(dragging || isMoving) && 'opacity-40',
			)}
		>
			<div className="flex items-start gap-2">
				<NameAvatar name={customer.name} hue={hueOf(customer.id)} size="sm" />
				<div className="grid min-w-0 flex-1 justify-items-start gap-1">
					<p className="w-full truncate font-medium text-xs">
						{ACCOUNT_KIND_LABELS[kind]}
					</p>
					<AccountStatusBadge status={accountStatusOf(customer.status)} />
				</div>
				{canMove && (
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								type="button"
								variant="ghost"
								data-no-drag
								size="icon-xs"
								className="-mt-0.5 -mr-1 relative z-10 text-muted-foreground"
								aria-label={`Déplacer « ${customer.name} » vers…`}
								disabled={isMoving}
							>
								<EllipsisVertical aria-hidden />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuLabel>Déplacer vers…</DropdownMenuLabel>
							<DropdownMenuSeparator />
							{PIPELINE_STAGES.filter(
								(stage) => stage !== customer.pipeline_stage,
							).map((stage) => (
								<DropdownMenuItem key={stage} onSelect={() => onMove(stage)}>
									{customerPipelineStageLabel(stage)}
								</DropdownMenuItem>
							))}
						</DropdownMenuContent>
					</DropdownMenu>
				)}
			</div>
			<button
				type="button"
				onClick={onOpen}
				className="line-clamp-3 text-left font-medium text-sm leading-snug outline-none after:absolute after:inset-0 after:rounded-md"
			>
				{customer.name}
			</button>
			<p className="truncate text-muted-foreground text-xs">
				{customer.email || customer.phone || 'Coordonnées à compléter'}
			</p>
			<div className="flex items-center gap-2 text-muted-foreground text-xs">
				<PersonAvatar person={person} size="sm" />
				<span className="min-w-0 flex-1 truncate">
					{person?.name ?? 'Non assigné'}
				</span>
				{extras.source && (
					<span className="shrink-0">{LEAD_SOURCE_LABELS[extras.source]}</span>
				)}
			</div>
		</article>
	)
}
