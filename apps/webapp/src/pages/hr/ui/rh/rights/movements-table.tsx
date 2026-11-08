import { RequirePermission } from '#/components/require-permission'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import type { Movement } from '#/pages/hr/lib/rh/balances'
import { formatFull } from '#/pages/hr/lib/rh/dates'
import {
	describeMovement,
	formatDay,
	formatSigned,
	MOVEMENT_NATURES,
} from '#/pages/hr/lib/rh/rights-model'
import type { IsoDate, LeaveAccount, RhState } from '#/pages/hr/lib/rh/types'
import { RemoveAdjustmentButton } from '#/pages/hr/ui/rh/rights/remove-adjustment-button'
import { AbsenceStatusBadge } from '#/pages/hr/ui/rh/shared/absence-status-badge'

function removeLabel(movement: Movement): string {
	const noun =
		movement.kind === 'carry_over' ? 'le report' : 'la régularisation'
	return `Supprimer ${noun} du ${formatFull(movement.date)}`
}

export function MovementsTable({
	state,
	account,
	movements,
	today,
	onRemove,
}: {
	state: RhState
	account: LeaveAccount
	movements: readonly Movement[]
	today: IsoDate
	onRemove: (movementId: string) => void | Promise<void>
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table className="table-fixed">
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="w-32 pl-4">Date</TableHead>
						<TableHead>Mouvement</TableHead>
						<TableHead className="w-32">Nature</TableHead>
						<TableHead className="w-20 text-right">Jours</TableHead>
						<TableHead className="w-52 pr-4">
							<span className="sr-only">Actions</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{movements.map((movement) => {
						const { title, detail } = describeMovement(
							state,
							account,
							movement,
							today,
						)
						const removable =
							movement.kind === 'adjustment' || movement.kind === 'carry_over'
						return (
							<TableRow key={movement.id}>
								<TableCell className="tabular py-2.5 pl-4 text-muted-foreground">
									{formatDay(movement.date)}
								</TableCell>
								<TableCell className="whitespace-normal">
									<span className="block font-medium">{title}</span>
									{detail && (
										<span className="block text-[12px] text-muted-foreground">
											{detail}
										</span>
									)}
								</TableCell>
								<TableCell>
									{movement.status ? (
										<AbsenceStatusBadge status={movement.status} />
									) : (
										<span className="inline-flex h-5 items-center rounded-sm bg-muted px-1.5 font-medium text-[11px] text-muted-foreground">
											{MOVEMENT_NATURES[movement.kind]}
										</span>
									)}
								</TableCell>
								<TableCell
									className={cn(
										'tabular text-right font-medium',
										movement.days > 0 && 'text-state-progress',
									)}
								>
									{formatSigned(movement.days)}
								</TableCell>
								<TableCell className="pr-4 text-right">
									{removable && (
										<RequirePermission permission="MANAGE_PLANNING">
											<RemoveAdjustmentButton
												label={removeLabel(movement)}
												onRemove={() => onRemove(movement.id)}
											/>
										</RequirePermission>
									)}
								</TableCell>
							</TableRow>
						)
					})}
					{movements.length === 0 && (
						<TableRow className="hover:bg-transparent">
							<TableCell
								colSpan={5}
								className="py-8 text-center text-muted-foreground"
							>
								Aucun mouvement sur cette période.
							</TableCell>
						</TableRow>
					)}
				</TableBody>
			</Table>
		</div>
	)
}
