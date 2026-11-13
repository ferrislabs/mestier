import { ArrowLeft, XCircle } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, SectionCard } from '#/components/ui/surface'
import { formatDate } from '#/pages/cursus/shared/lib/format'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'
import { ClosedBadge } from '#/pages/cursus/shared/ui/status-badge'
import type { TriadDetail } from '#/pages/cursus/triads/feature/use-triads'
import { isClosed } from '#/pages/cursus/triads/lib/triad-form'

interface TriadHeaderProps {
	triad: TriadDetail
	schoolTutors: SelectOption[]
	canManage: boolean
	isUpdatingTutor: boolean
	tutorError: string | null
	onBack: () => void
	onOpenPromotion: () => void
	onTutorChange: (memberId: string | null) => void
	onClose: () => void
}

export function TriadHeader({
	triad,
	schoolTutors,
	canManage,
	isUpdatingTutor,
	tutorError,
	onBack,
	onOpenPromotion,
	onTutorChange,
	onClose,
}: TriadHeaderProps) {
	const closed = isClosed(triad)
	const editable = canManage && !closed

	return (
		<>
			<PageHeader
				leading={
					<Button
						type="button"
						variant="ghost"
						size="icon"
						aria-label="Retour aux trinômes"
						onClick={onBack}
					>
						<ArrowLeft />
					</Button>
				}
				title={triad.apprentice_name}
				description={triad.apprentice_email}
				actions={
					<>
						<ClosedBadge closed={closed} />
						{editable ? (
							<Button type="button" variant="outline" onClick={onClose}>
								<XCircle />
								Clôturer le trinôme
							</Button>
						) : null}
					</>
				}
			/>
			<SectionCard>
				<dl className="grid gap-4 p-5 text-[13px] sm:grid-cols-3">
					<div>
						<dt className="text-muted-foreground">Promotion</dt>
						<dd>
							<button
								type="button"
								className="text-left underline-offset-4 hover:underline"
								onClick={onOpenPromotion}
							>
								{triad.promotion.label}
							</button>
							<span className="block text-muted-foreground">
								{triad.training_name}
							</span>
						</dd>
					</div>
					<div>
						<dt className="text-muted-foreground">
							{editable ? (
								<label htmlFor="triad-pedagogic-tutor">Tuteur école</label>
							) : (
								'Tuteur école'
							)}
						</dt>
						<dd>
							{editable ? (
								<OptionSelect
									id="triad-pedagogic-tutor"
									value={triad.pedagogic_tutor_member_id ?? null}
									options={schoolTutors}
									placeholder="Aucun"
									noneLabel="Aucun"
									disabled={isUpdatingTutor}
									onChange={onTutorChange}
								/>
							) : (
								(triad.pedagogic_tutor_name ?? '—')
							)}
							{tutorError ? (
								<span role="alert" className="block text-state-alert">
									{tutorError}
								</span>
							) : null}
						</dd>
					</div>
					<div>
						<dt className="text-muted-foreground">Statut</dt>
						<dd>
							{closed ? (
								<>
									Clôturé le {formatDate(triad.closed_at)}
									{triad.closure_reason_label
										? ` · ${triad.closure_reason_label}`
										: ''}
									{triad.closure_note ? (
										<span className="block text-muted-foreground">
											{triad.closure_note}
										</span>
									) : null}
								</>
							) : (
								'En cours'
							)}
						</dd>
					</div>
				</dl>
			</SectionCard>
		</>
	)
}
