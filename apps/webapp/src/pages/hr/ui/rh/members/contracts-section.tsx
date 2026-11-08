import { ArrowRight, Pencil, Plus, Trash2 } from 'lucide-react'
import { Fragment, useId, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import { mutationErrorMessage } from '#/lib/api-error'
import { cn } from '#/lib/utils'
import {
	CONTRACT_LABELS,
	contractTrail,
	hireDateOf,
	phaseOf,
	sortContracts,
} from '#/pages/hr/lib/rh/contracts'
import { formatFull, yearsBetween } from '#/pages/hr/lib/rh/dates'
import {
	contractPeriod,
	durationLabel,
	formatDate,
	tenureLabel,
} from '#/pages/hr/lib/rh/format'
import { upcomingDeadlines } from '#/pages/hr/lib/rh/members'
import { rhythmLines } from '#/pages/hr/lib/rh/schedule-mode'
import type {
	Contract,
	IsoDate,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'
import {
	ContractBadge,
	ContractPhaseBadge,
	PHASE_DOT,
} from '#/pages/hr/ui/rh/members/contract-badge'
import { ContractDeleteDialog } from '#/pages/hr/ui/rh/members/contract-delete-dialog'
import { ContractFormDialog } from '#/pages/hr/ui/rh/members/contract-form-dialog'
import { DeadlineNote } from '#/pages/hr/ui/rh/members/deadline-note'
import { DetailCard } from '#/pages/hr/ui/rh/members/detail-card'
import { useRhContractActions } from '#/pages/hr/ui/rh/shared/rh-context'

const HORIZON_DAYS = 60

function contractLabel(contract: Contract, today: IsoDate): string {
	return `${CONTRACT_LABELS[contract.type]} ${contractPeriod(contract, today)}`
}

export function ContractsSection({
	state,
	member,
	today,
}: {
	state: RhState
	member: Member
	today: IsoDate
}) {
	const actions = useRhContractActions()
	const noteId = useId()
	const [removing, setRemoving] = useState(false)
	const [removeError, setRemoveError] = useState<string | null>(null)
	const [adding, setAdding] = useState(false)
	const [editingId, setEditingId] = useState<string | null>(null)
	const [deletingId, setDeletingId] = useState<string | null>(null)
	const contracts = [...sortContracts(member.contracts)].reverse()
	const hireDate = hireDateOf(member)
	const deadlines = upcomingDeadlines(state, today, HORIZON_DAYS).filter(
		(deadline) => deadline.memberId === member.id,
	)
	const endNote = deadlines.find((deadline) => deadline.kind === 'contract_end')
	const onlyOne = contracts.length === 1
	const trail = contractTrail(member)

	return (
		<RubriqueLayout
			title="Contrats"
			description="Historique des contrats et semaines types associées."
			actions={
				<RequirePermission permission="MANAGE_MEMBERS">
					<ContractFormDialog
						open={adding}
						onOpenChange={setAdding}
						member={member}
						trigger={
							<Button variant="outline" size="sm">
								<Plus aria-hidden /> Ajouter un contrat
							</Button>
						}
					/>
				</RequirePermission>
			}
		>
			<DetailCard title="Historique">
				<div className="mb-4 grid gap-1.5">
					<ol
						aria-label="Parcours"
						className="flex flex-wrap items-center gap-1.5"
					>
						{trail.map((type, index) => (
							<Fragment
								key={`${type}-${trail.slice(0, index).filter((other) => other === type).length}`}
							>
								{index > 0 && (
									<li aria-hidden className="text-muted-foreground">
										<ArrowRight className="size-3.5" />
									</li>
								)}
								<li>
									<ContractBadge contract={type} />
								</li>
							</Fragment>
						))}
					</ol>
					{hireDate !== '' && (
						<p className="text-[13px] text-muted-foreground">
							Entrée le {formatFull(hireDate)}
							{hireDate <= today &&
								` · ${tenureLabel(yearsBetween(hireDate, today))} d’ancienneté`}
						</p>
					)}
				</div>

				<ol className="relative grid gap-4 border-l pl-5">
					{contracts.map((contract, index) => {
						const phase = phaseOf(contract, today)
						const duration = durationLabel(
							contract.startDate,
							contract.endDate,
							today,
						)
						const trialDays = deadlines.find(
							(deadline) =>
								deadline.kind === 'trial' &&
								deadline.date === contract.trialEndDate,
						)?.daysLeft
						const label = contractLabel(contract, today)
						const rhythm = rhythmLines(contract.schedule)
						return (
							<li key={contract.id} className="relative">
								<span
									aria-hidden
									className={cn(
										'-left-[calc(1.25rem+3.5px)] absolute top-1.5 size-2 rounded-full ring-4 ring-card',
										PHASE_DOT[phase],
									)}
								/>
								<div className="flex items-center gap-2">
									<ContractBadge contract={contract.type} />
									<ContractPhaseBadge phase={phase} />
									<span className="ml-auto flex items-center gap-0.5">
										<RequirePermission permission="MANAGE_MEMBERS">
											<ContractFormDialog
												open={editingId === contract.id}
												onOpenChange={(open) =>
													setEditingId(open ? contract.id : null)
												}
												member={member}
												contract={contract}
												trigger={
													<Button
														variant="ghost"
														size="icon-xs"
														aria-label={`Modifier le contrat ${label}`}
													>
														<Pencil aria-hidden />
													</Button>
												}
											/>
											<ContractDeleteDialog
												contract={contract}
												open={deletingId === contract.id}
												onOpenChange={(open) => {
													setRemoveError(null)
													setDeletingId(open ? contract.id : null)
												}}
												error={removeError}
												isPending={removing}
												onConfirm={async () => {
													setRemoving(true)
													setRemoveError(null)
													try {
														await actions.remove(member.id, contract.id)
														setDeletingId(null)
													} catch (error) {
														setRemoveError(mutationErrorMessage(error))
													} finally {
														setRemoving(false)
													}
												}}
												trigger={
													<Button
														variant="ghost"
														size="icon-xs"
														disabled={onlyOne}
														aria-label={`Supprimer le contrat ${label}`}
														aria-describedby={onlyOne ? noteId : undefined}
														title={
															onlyOne
																? 'Un membre doit garder au moins un contrat'
																: undefined
														}
													>
														<Trash2 aria-hidden />
													</Button>
												}
											/>
										</RequirePermission>
									</span>
								</div>
								<p className="mt-1.5 flex flex-wrap items-baseline gap-x-2 text-[13px]">
									<span>{contractPeriod(contract, today)}</span>
									{duration && (
										<span className="text-muted-foreground">· {duration}</span>
									)}
									{index === 0 &&
										endNote &&
										contract.endDate === endNote.date && (
											<DeadlineNote daysLeft={endNote.daysLeft} />
										)}
								</p>
								{contract.trialEndDate && (
									<p className="mt-0.5 flex flex-wrap items-baseline gap-x-2 text-[12px] text-muted-foreground">
										<span>
											Fin de période d’essai :{' '}
											{formatDate(contract.trialEndDate)}
										</span>
										{trialDays !== undefined && (
											<DeadlineNote daysLeft={trialDays} />
										)}
									</p>
								)}
								<p className="mt-0.5 text-[12px] text-muted-foreground">
									{rhythm.volume}
									{rhythm.detail && ` · ${rhythm.detail}`}
								</p>
							</li>
						)
					})}
				</ol>
				{onlyOne && (
					<p id={noteId} className="mt-3 text-[12px] text-muted-foreground">
						Un membre doit garder au moins un contrat : celui-ci ne peut pas
						être supprimé.
					</p>
				)}
			</DetailCard>
		</RubriqueLayout>
	)
}
