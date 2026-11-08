import { Mail, Pencil, Phone } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import { formatFull } from '#/pages/hr/lib/rh/dates'
import {
	directReports,
	findMemberById,
	upcomingDeadlines,
} from '#/pages/hr/lib/rh/members'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { DeadlineNote } from '#/pages/hr/ui/rh/members/deadline-note'
import {
	DetailCard,
	DetailList,
	DetailRow,
	EmptyNote,
} from '#/pages/hr/ui/rh/members/detail-card'
import { MemberFormDialog } from '#/pages/hr/ui/rh/members/member-form-dialog'
import { MemberLink } from '#/pages/hr/ui/rh/members/member-link'

const LINK =
	'inline-flex items-center gap-1.5 rounded-sm outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring'

function Empty() {
	return <span className="text-muted-foreground">—</span>
}

function EmailValue({ email }: { email: string }) {
	if (!email) return <Empty />
	return (
		<a href={`mailto:${email}`} className={`${LINK} text-primary`}>
			<Mail className="size-3.5" aria-hidden />
			{email}
		</a>
	)
}

function PhoneValue({ phone }: { phone: string }) {
	if (!phone) return <Empty />
	return (
		<a
			href={`tel:${phone.replace(/[\s.-]/g, '')}`}
			className={`${LINK} tabular`}
		>
			<Phone className="size-3.5 text-muted-foreground" aria-hidden />
			{phone}
		</a>
	)
}

function IdentityBlock({ member }: { member: Member }) {
	return (
		<DetailCard title="Identité">
			<DetailList>
				<DetailRow label="Prénom">{member.firstName || <Empty />}</DetailRow>
				<DetailRow label="Nom">{member.lastName || <Empty />}</DetailRow>
			</DetailList>
		</DetailCard>
	)
}

function ContactBlock({ member }: { member: Member }) {
	return (
		<DetailCard title="Contact">
			<DetailList>
				<DetailRow label="E-mail professionnel">
					<EmailValue email={member.workEmail} />
				</DetailRow>
				<DetailRow label="E-mail personnel">
					<EmailValue email={member.personalEmail} />
				</DetailRow>
				<DetailRow label="Mobile">
					<PhoneValue phone={member.mobile} />
				</DetailRow>
				<DetailRow label="Téléphone fixe">
					<PhoneValue phone={member.landline} />
				</DetailRow>
			</DetailList>
		</DetailCard>
	)
}

function AddressBlock({ member }: { member: Member }) {
	const { street, complement, postalCode, city, country } = member.address
	const town = [postalCode, city].filter(Boolean).join(' ')
	const lines = [street, complement, town].filter((line) => line.trim() !== '')
	return (
		<DetailCard title="Adresse postale">
			{lines.length === 0 ? (
				<EmptyNote>Adresse non renseignée.</EmptyNote>
			) : (
				<address className="grid gap-0.5 text-[13px] not-italic">
					{lines.map((line) => (
						<span key={line}>{line}</span>
					))}
					{country && <span className="text-muted-foreground">{country}</span>}
				</address>
			)}
		</DetailCard>
	)
}

function EmploymentBlock({
	state,
	member,
	today,
}: {
	state: RhState
	member: Member
	today: IsoDate
}) {
	const manager = findMemberById(state, member.managerId)
	const reports = directReports(state, member.id)
	const permit = upcomingDeadlines(state, today, 60).find(
		(deadline) =>
			deadline.memberId === member.id && deadline.kind === 'work_permit',
	)
	return (
		<DetailCard title="Poste et rattachement">
			<div className="grid gap-4">
				<DetailList>
					<DetailRow label="Poste">{member.jobTitle || <Empty />}</DetailRow>
					<DetailRow label="Fin du titre de travail">
						{member.workPermitEndDate ? (
							<span className="flex flex-wrap items-baseline justify-end gap-x-2">
								<span>{formatFull(member.workPermitEndDate)}</span>
								{permit && <DeadlineNote daysLeft={permit.daysLeft} />}
							</span>
						) : (
							<Empty />
						)}
					</DetailRow>
				</DetailList>
				<div>
					<h4 className="mb-1 text-[12px] text-muted-foreground">
						Responsable
					</h4>
					{manager ? (
						<MemberLink member={manager} />
					) : (
						<EmptyNote>Aucun responsable.</EmptyNote>
					)}
				</div>
				<div>
					<h4 className="mb-1 text-[12px] text-muted-foreground">
						Équipe directe
						{reports.length > 0 && ` (${reports.length})`}
					</h4>
					{reports.length > 0 ? (
						<ul className="grid gap-0.5">
							{reports.map((report) => (
								<li key={report.id}>
									<MemberLink member={report} />
								</li>
							))}
						</ul>
					) : (
						<EmptyNote>
							Personne n’est rattaché directement à ce membre.
						</EmptyNote>
					)}
				</div>
			</div>
		</DetailCard>
	)
}

export function IdentitySection({
	state,
	member,
	today,
	onRename,
}: {
	state: RhState
	member: Member
	today: IsoDate
	onRename: (names: { firstName: string; lastName: string }) => void
}) {
	const [editing, setEditing] = useState(false)
	return (
		<RubriqueLayout
			title="Identité"
			description="Coordonnées, adresse, poste et rattachement."
			actions={
				<RequirePermission permission="MANAGE_MEMBERS">
					<MemberFormDialog
						open={editing}
						onOpenChange={setEditing}
						member={member}
						onRename={onRename}
						trigger={
							<Button variant="outline" size="sm">
								<Pencil aria-hidden /> Modifier
							</Button>
						}
					/>
				</RequirePermission>
			}
		>
			<IdentityBlock member={member} />
			<ContactBlock member={member} />
			<AddressBlock member={member} />
			<EmploymentBlock state={state} member={member} today={today} />
		</RubriqueLayout>
	)
}
