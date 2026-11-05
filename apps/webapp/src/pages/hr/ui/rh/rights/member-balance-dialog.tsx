import { Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { cn } from '#/lib/utils'
import {
	computeBalance,
	movementsFor,
	removeAdjustment,
} from '#/pages/hr/lib/rh/balances'
import { fullName } from '#/pages/hr/lib/rh/members'
import {
	acquiredTotal,
	describeBreakdown,
	describePeriod,
	formatAmount,
	LOW_BALANCE_THRESHOLD,
	orderMovements,
} from '#/pages/hr/lib/rh/rights-model'
import type {
	IsoDate,
	LeaveAccount,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'
import { AdjustmentForm } from '#/pages/hr/ui/rh/rights/adjustment-form'
import { MovementsTable } from '#/pages/hr/ui/rh/rights/movements-table'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import { useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

function Figure({
	label,
	value,
	hint,
	tone,
}: {
	label: string
	value: number
	hint: string | null
	tone?: string
}) {
	return (
		<div className="rounded-lg border bg-card px-3.5 py-3">
			<dt className="text-[13px] text-muted-foreground">{label}</dt>
			<dd
				className={cn(
					'tabular mt-0.5 font-semibold text-xl',
					value < 0 ? 'text-state-alert' : tone,
				)}
			>
				{formatAmount(value)}
				<span className="ml-1 font-normal text-muted-foreground text-sm">
					j
				</span>
			</dd>
			<dd className="mt-0.5 min-h-4 text-[12px] text-muted-foreground">
				{hint}
			</dd>
		</div>
	)
}

export function MemberBalanceDialog({
	state,
	member,
	account,
	today,
	onClose,
}: {
	state: RhState
	member: Member
	account: LeaveAccount
	today: IsoDate
	onClose: () => void
}) {
	const updateRh = useRhUpdate()
	const [adding, setAdding] = useState(false)
	const [announcement, setAnnouncement] = useState('')
	const line = computeBalance(state, member, account, today)
	const movements = orderMovements(movementsFor(state, member, account, today))
	const waiting = [
		line.planned > 0 ? `${formatAmount(line.planned)} posés` : '',
		line.pending > 0 ? `${formatAmount(line.pending)} en attente` : '',
	].filter((part) => part !== '')

	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<DialogContent
				showCloseButton={false}
				className="flex max-h-[calc(100dvh-3rem)] flex-col gap-0 overflow-hidden p-0 sm:max-w-4xl"
			>
				<DialogHeader className="flex-row items-center gap-3 border-b px-6 py-4">
					<MemberAvatar member={member} size="lg" />
					<div className="min-w-0">
						<DialogTitle>{fullName(member)}</DialogTitle>
						<DialogDescription className="mt-1">
							{member.jobTitle} · {account.name}, {describePeriod(line.period)}
						</DialogDescription>
					</div>
				</DialogHeader>
				<div className="grid gap-5 overflow-y-auto px-6 py-5">
					<dl className="grid grid-cols-2 gap-3 sm:grid-cols-4">
						<Figure
							label="Acquis"
							value={acquiredTotal(line)}
							hint={describeBreakdown(line)}
						/>
						<Figure
							label="Pris"
							value={line.taken}
							hint={waiting.length > 0 ? `et ${waiting.join(', ')}` : null}
						/>
						<Figure
							label="Solde"
							value={line.balance}
							hint="Acquis moins pris"
						/>
						<Figure
							label="Disponible"
							value={line.available}
							hint={`Fin de période : ${formatAmount(line.forecast)} j`}
							tone={
								line.available < LOW_BALANCE_THRESHOLD
									? 'text-state-today'
									: undefined
							}
						/>
					</dl>
					<section aria-label="Registre des mouvements" className="grid gap-3">
						<div className="flex items-center justify-between gap-3">
							<h3 className="font-semibold">Registre des mouvements</h3>
							{!adding && (
								<Button
									variant="outline"
									size="sm"
									onClick={() => {
										setAnnouncement('')
										setAdding(true)
									}}
								>
									<Plus /> Ajouter une régularisation
								</Button>
							)}
						</div>
						{adding && (
							<AdjustmentForm
								state={state}
								member={member}
								account={account}
								period={line.period}
								today={today}
								onCancel={() => setAdding(false)}
								onDone={(message) => {
									setAnnouncement(message)
									setAdding(false)
								}}
							/>
						)}
						<MovementsTable
							state={state}
							account={account}
							movements={movements}
							today={today}
							onRemove={(movementId) => {
								updateRh((current) => removeAdjustment(current, movementId))
								setAnnouncement('Mouvement supprimé du registre.')
							}}
						/>
						<output className="sr-only">{announcement}</output>
					</section>
				</div>
				<DialogFooter className="border-t px-6 py-3">
					<DialogClose asChild>
						<Button variant="outline">Fermer</Button>
					</DialogClose>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
