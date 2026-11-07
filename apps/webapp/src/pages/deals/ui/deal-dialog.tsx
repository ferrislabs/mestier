import { Link } from '@tanstack/react-router'
import { FilePlus2, RotateCcw, Trophy } from 'lucide-react'
import { useId, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { Customer } from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import { accountStatusOf } from '#/pages/customers/types'
import { hueOf, NameAvatar } from '#/pages/customers/ui/avatars'
import { Money } from '#/pages/customers/ui/money'
import { AccountStatusBadge } from '#/pages/customers/ui/status-badges'
import {
	buildDraft,
	type DealForm,
	formFromDeal,
	isFormChanged,
} from '#/pages/deals/lib/deal-form'
import type {
	Deal,
	DealDraft,
	DealQuote,
	DealStage,
} from '#/pages/deals/lib/deal-types'
import { isOpenStage } from '#/pages/deals/lib/pipeline'
import { accountOptions, reopenStage } from '#/pages/deals/lib/pipeline-filters'
import {
	DealQuoteStatusBadge,
	DealStageBadge,
} from '#/pages/deals/ui/deal-badges'
import { DealFields } from '#/pages/deals/ui/deal-fields'

function plural(count: number, one: string, many: string): string {
	return `${count} ${count > 1 ? many : one}`
}

export function DealDialog({
	deal,
	customers,
	quotes,
	organizationSlug,
	canManage,
	onClose,
	onMove,
	onSave,
	onDelete,
}: {
	deal: Deal
	customers: readonly Customer[]
	quotes: readonly DealQuote[]
	organizationSlug: string
	canManage: boolean
	onClose: () => void
	onMove: (dealId: string, stage: DealStage) => void
	onSave: (draft: DealDraft) => void
	onDelete: () => void
}) {
	const idPrefix = useId()
	const [form, setForm] = useState<DealForm>(() => formFromDeal(deal))
	const [submitted, setSubmitted] = useState(false)
	const [confirmingDelete, setConfirmingDelete] = useState(false)
	const customer = customers.find((item) => item.id === deal.customerId)
	const result = buildDraft(form)
	const open = isOpenStage(deal.stage)
	const locked = quotes.length > 0
	const deleteHintId = `${idPrefix}-delete-hint`

	function save(event: { preventDefault: () => void }) {
		event.preventDefault()
		setSubmitted(true)
		if (!result.draft) return
		onSave(result.draft)
		onClose()
	}

	function move(stage: DealStage) {
		onMove(deal.id, stage)
		onClose()
	}

	return (
		<>
			<Dialog open onOpenChange={(next) => !next && onClose()}>
				<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
					<form
						onSubmit={save}
						noValidate
						className="flex min-h-0 flex-1 flex-col"
					>
						<DialogHeader className="px-6 pt-6 pb-4">
							<DialogTitle>Opportunité</DialogTitle>
							<DialogDescription className="flex flex-wrap items-center gap-2">
								<DealStageBadge stage={deal.stage} />
								{customer && (
									<>
										<NameAvatar
											name={customer.name}
											hue={hueOf(customer.id)}
											size="sm"
										/>
										<Link
											to={buildOrgPath(
												organizationSlug,
												'/crm/clients/$customerId',
											)}
											params={{ customerId: customer.id }}
											className="font-medium text-foreground hover:underline"
										>
											{customer.name}
										</Link>
										<AccountStatusBadge
											status={accountStatusOf(customer.status)}
										/>
									</>
								)}
							</DialogDescription>
							{deal.stage === 'lost' && (
								<p className="text-sm">
									<span className="text-muted-foreground">
										Motif de perte :{' '}
									</span>
									{deal.lostReason}
								</p>
							)}
						</DialogHeader>

						<div className="grid min-h-0 flex-1 gap-6 overflow-y-auto border-t px-6 py-5">
							<fieldset disabled={!canManage} className="min-w-0">
								<DealFields
									idPrefix={idPrefix}
									form={form}
									errors={submitted ? result.errors : {}}
									customers={accountOptions(customers, deal.customerId)}
									clientLocked={locked}
									clientNote={
										locked
											? 'Le client ne peut plus changer : des devis sont liés à cette opportunité.'
											: undefined
									}
									onChange={(patch) =>
										setForm((current) => ({ ...current, ...patch }))
									}
								/>
							</fieldset>

							<section
								aria-labelledby={`${idPrefix}-quotes`}
								className="grid gap-3"
							>
								<div className="flex flex-wrap items-center justify-between gap-2">
									<h3 id={`${idPrefix}-quotes`} className="font-medium text-sm">
										Devis liés
									</h3>
									<RequirePermission permission="MANAGE_QUOTES">
										<Button asChild type="button" variant="outline" size="sm">
											<Link
												to={buildOrgPath(
													organizationSlug,
													'/crm/devis/nouveau',
												)}
												search={{
													client: deal.customerId,
													opportunite: deal.id,
												}}
											>
												<FilePlus2 aria-hidden /> Créer un devis
											</Link>
										</Button>
									</RequirePermission>
								</div>
								{quotes.length === 0 ? (
									<p className="rounded-md border border-dashed px-3 py-4 text-center text-muted-foreground text-sm">
										Aucun devis lié pour l’instant.
									</p>
								) : (
									<ul className="divide-y rounded-md border">
										{quotes.map((quote) => (
											<li
												key={quote.id}
												className="flex flex-wrap items-center gap-x-3 gap-y-1 px-3 py-2 text-sm"
											>
												<Link
													to={buildOrgPath(
														organizationSlug,
														'/crm/devis/$quoteId',
													)}
													params={{ quoteId: quote.id }}
													className="font-medium hover:underline"
												>
													{quote.number ?? 'Brouillon'}
												</Link>
												<span className="min-w-0 flex-1 truncate text-muted-foreground">
													{quote.title}
												</span>
												<DealQuoteStatusBadge status={quote.status} />
												<span className="text-right">
													<Money cents={quote.subtotal} />
													<span className="text-muted-foreground text-xs">
														{' '}
														HT
													</span>
												</span>
											</li>
										))}
									</ul>
								)}
							</section>

							{canManage && (
								<>
									<section
										aria-labelledby={`${idPrefix}-progress`}
										className="grid gap-3"
									>
										<h3
											id={`${idPrefix}-progress`}
											className="font-medium text-sm"
										>
											Avancement
										</h3>
										<div className="flex flex-wrap gap-2">
											{deal.stage !== 'won' && (
												<Button
													type="button"
													variant="outline"
													size="sm"
													onClick={() => move('won')}
												>
													<Trophy aria-hidden /> Marquer gagnée
												</Button>
											)}
											{deal.stage !== 'lost' && (
												<Button
													type="button"
													variant="outline"
													size="sm"
													onClick={() => move('lost')}
												>
													Marquer perdue
												</Button>
											)}
											{!open && (
												<Button
													type="button"
													variant="outline"
													size="sm"
													onClick={() => move(reopenStage(quotes))}
												>
													<RotateCcw aria-hidden /> Rouvrir
												</Button>
											)}
										</div>
										{deal.stage !== 'won' &&
											customer?.status === 'PROSPECT' && (
												<p className="text-[12px] text-muted-foreground">
													{`En marquant l’opportunité gagnée, ${customer.name} devient client.`}
												</p>
											)}
									</section>

									<section
										aria-label="Suppression"
										className="flex flex-wrap items-center justify-between gap-3 rounded-md border border-dashed px-3 py-2.5"
									>
										<p
											id={deleteHintId}
											className="min-w-0 flex-1 text-[12px] text-muted-foreground"
										>
											{locked
												? `Suppression impossible : ${plural(quotes.length, 'devis est lié', 'devis sont liés')} à cette opportunité.`
												: 'Supprime définitivement cette opportunité.'}
										</p>
										<Button
											type="button"
											variant="outline"
											size="sm"
											disabled={locked}
											aria-describedby={deleteHintId}
											onClick={() => setConfirmingDelete(true)}
										>
											Supprimer
										</Button>
									</section>
								</>
							)}
						</div>

						<DialogFooter className="border-t px-6 py-4">
							<Button type="button" variant="outline" onClick={onClose}>
								Fermer
							</Button>
							{canManage && (
								<Button type="submit" disabled={!isFormChanged(form, deal)}>
									Enregistrer
								</Button>
							)}
						</DialogFooter>
					</form>
				</DialogContent>
			</Dialog>
			<ConfirmDialog
				open={confirmingDelete}
				onOpenChange={setConfirmingDelete}
				title="Supprimer l’opportunité"
				description={`« ${deal.title} » sera supprimée définitivement.`}
				confirmLabel="Supprimer"
				destructive
				onConfirm={() => {
					onDelete()
					onClose()
				}}
			/>
		</>
	)
}
