import { Link } from '@tanstack/react-router'
import {
	Clock,
	Loader2,
	MoreHorizontal,
	Save,
	Search,
	Trash2,
	Undo2,
	UserPlus,
	Users,
	X,
} from 'lucide-react'
import {
	CreateButton,
	MoneyCell,
	TextField,
} from '#/components/reference-table'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import type { AccessState, MemberFormValues } from '#/pages/hr/types'
import { formatDateFr, formatDurationMinutes } from '#/pages/hr/types'

const ACCESS_LABEL: Record<AccessState, string> = {
	none: 'Aucun accès',
	invited: 'Invité',
	linkedAccount: 'Compte lié',
}

const ACCESS_TONE: Record<AccessState, 'neutral' | 'warning' | 'success'> = {
	none: 'neutral',
	invited: 'warning',
	linkedAccount: 'success',
}

interface FormBinding<T> {
	values: T
	isPending: boolean
	onChange: (patch: Partial<T>) => void
	onSubmit: () => void
}

export interface TeamMemberRow {
	id: string
	displayName: string
	access: AccessState
	/** `null` when the seat has no employee profile yet. */
	hourlyRateCents: number | null
	/** `null` when the seat has no employee profile yet. */
	weeklyContractMinutes: number | null
}

export interface MemberDraft {
	id: string
	values: MemberFormValues
}

export interface PendingInvitationRow {
	id: string
	memberName: string
	expiresAt: string
}

interface TeamListUIProps {
	organizationName: string
	organizationSlug: string
	isLoading: boolean
	error: string | null
	members: TeamMemberRow[]
	search: string
	onSearchChange: (value: string) => void
	createForm: FormBinding<MemberFormValues>
	draft: MemberDraft | null
	isSaving: boolean
	onEdit: (member: TeamMemberRow) => void
	onDraftChange: (values: MemberFormValues) => void
	onCancelEdit: () => void
	onSaveEdit: () => void
	onDeleteMember: (member: TeamMemberRow) => Promise<unknown>
	onInvite: (member: TeamMemberRow) => void
	pendingInvitations: PendingInvitationRow[]
	revokingInvitationId: string | null
	onRevokeInvitation: (invitationId: string) => void
}

export function TeamListUI({
	organizationName,
	organizationSlug,
	isLoading,
	error,
	members,
	search,
	onSearchChange,
	createForm,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancelEdit,
	onSaveEdit,
	onDeleteMember,
	onInvite,
	pendingInvitations,
	revokingInvitationId,
	onRevokeInvitation,
}: TeamListUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Équipe"
				description="Gérez les membres de l’organisation, leurs accès et leurs taux horaires."
			/>

			<MetricCard
				label="Équipe"
				value={members.length}
				hint="Sièges occupés ou libres"
				icon={<Users className="size-4" />}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
				<div className="relative w-full lg:w-80">
					<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						value={search}
						onChange={(event) => onSearchChange(event.target.value)}
						placeholder="Rechercher dans l’équipe…"
						className="pl-9"
					/>
				</div>
			</div>

			<CreateMemberSection form={createForm} />

			<PendingInvitationsSection
				invitations={pendingInvitations}
				revokingId={revokingInvitationId}
				onRevoke={onRevokeInvitation}
			/>

			{isLoading ? (
				<TeamListUI.Loading />
			) : (
				<TeamTable
					data={members}
					organizationSlug={organizationSlug}
					draft={draft}
					isSaving={isSaving}
					onEdit={onEdit}
					onDraftChange={onDraftChange}
					onCancel={onCancelEdit}
					onSave={onSaveEdit}
					onDelete={onDeleteMember}
					onInvite={onInvite}
				/>
			)}
		</PageShell>
	)
}

TeamListUI.Loading = function TeamListLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement de l’équipe…
			</SectionCard>
		</PageShell>
	)
}

interface CreateMemberSectionProps {
	form: FormBinding<MemberFormValues>
}

function CreateMemberSection({ form }: CreateMemberSectionProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Ajouter une personne"
				description="Le taux horaire est optionnel : laissez-le vide pour un siège sans profil RH, renseignez-le pour lui en attacher un."
			/>
			<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
				<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
					<TextField
						label="Nom"
						value={form.values.lastName}
						onChange={(lastName) => form.onChange({ lastName })}
					/>
					<TextField
						label="Prénom"
						value={form.values.firstName}
						onChange={(firstName) => form.onChange({ firstName })}
						placeholder="Optionnel"
					/>
					<TextField
						label="Taux horaire"
						value={form.values.hourlyRate}
						onChange={(hourlyRate) => form.onChange({ hourlyRate })}
						inputMode="decimal"
						suffix="€/h"
						placeholder="Optionnel"
					/>
				</div>
				<CreateButton isPending={form.isPending} onClick={form.onSubmit} />
			</div>
		</SectionCard>
	)
}

interface PendingInvitationsSectionProps {
	invitations: PendingInvitationRow[]
	revokingId: string | null
	onRevoke: (invitationId: string) => void
}

/** Nothing to show, nothing rendered — an org with no pending invitation
 * gets no empty-state card cluttering the page. */
function PendingInvitationsSection({
	invitations,
	revokingId,
	onRevoke,
}: PendingInvitationsSectionProps) {
	if (invitations.length === 0) return null

	return (
		<SectionCard>
			<SectionHeader
				title={`Invitations en attente (${invitations.length})`}
				description="Liens générés mais pas encore acceptés."
			/>
			<ul className="divide-y">
				{invitations.map((invitation) => (
					<li
						key={invitation.id}
						className="flex items-center justify-between px-5 py-3"
					>
						<div>
							<p className="font-medium">{invitation.memberName}</p>
							<p className="text-xs text-muted-foreground">
								Expire le {formatDateFr(invitation.expiresAt.slice(0, 10))}
							</p>
						</div>
						<Button
							variant="ghost"
							size="sm"
							onClick={() => onRevoke(invitation.id)}
							disabled={revokingId === invitation.id}
						>
							{revokingId === invitation.id ? (
								<Loader2 className="animate-spin" />
							) : (
								<X />
							)}
							Révoquer
						</Button>
					</li>
				))}
			</ul>
		</SectionCard>
	)
}

interface TeamTableProps {
	data: TeamMemberRow[]
	organizationSlug: string
	draft: MemberDraft | null
	isSaving: boolean
	onEdit: (member: TeamMemberRow) => void
	onDraftChange: (values: MemberFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (member: TeamMemberRow) => Promise<unknown>
	onInvite: (member: TeamMemberRow) => void
}

function TeamTable({
	data,
	organizationSlug,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
	onInvite,
}: TeamTableProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Équipe (${data.length})`}
				description="Accès, taux horaire et base contractuelle de chaque personne."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						<tr className="border-b bg-muted/50">
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Nom
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Accès
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Taux
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Base contractuelle
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								<span className="sr-only">Actions</span>
							</th>
						</tr>
					</thead>
					<tbody>
						{data.length === 0 ? (
							<tr>
								<td colSpan={5} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">Aucune personne trouvée</p>
										<p className="text-sm text-muted-foreground">
											Ajoutez une personne pour la rendre disponible dans les
											prochains workflows opérationnels.
										</p>
									</div>
								</td>
							</tr>
						) : (
							data.map((member) => {
								const isEditing = draft?.id === member.id
								return (
									<tr
										key={member.id}
										className="group border-b transition hover:bg-muted/35 hover:shadow-xs last:border-b-0"
									>
										<td className="px-5 py-3 align-middle">
											{isEditing ? (
												<div className="flex flex-col gap-1.5">
													<Input
														aria-label="Nom"
														value={draft.values.lastName}
														onChange={(event) =>
															onDraftChange({
																...draft.values,
																lastName: event.target.value,
															})
														}
													/>
													<Input
														aria-label="Prénom"
														placeholder="Prénom (optionnel)"
														value={draft.values.firstName}
														onChange={(event) =>
															onDraftChange({
																...draft.values,
																firstName: event.target.value,
															})
														}
													/>
												</div>
											) : (
												<p className="truncate font-medium">
													{member.displayName}
												</p>
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											<StatusBadge tone={ACCESS_TONE[member.access]}>
												{ACCESS_LABEL[member.access]}
											</StatusBadge>
										</td>
										<td className="px-5 py-3 align-middle">
											{isEditing ? (
												<Input
													value={draft.values.hourlyRate}
													onChange={(event) =>
														onDraftChange({
															...draft.values,
															hourlyRate: event.target.value,
														})
													}
													inputMode="decimal"
													placeholder="Optionnel"
												/>
											) : (
												<MoneyCell value={member.hourlyRateCents} suffix="/h" />
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											{member.weeklyContractMinutes === null ? (
												<span className="text-muted-foreground italic">
													Sans profil RH
												</span>
											) : (
												<span className="font-medium tabular-nums">
													{formatDurationMinutes(member.weeklyContractMinutes)}
													<span className="text-muted-foreground">/sem.</span>
												</span>
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											<RowActions
												memberId={member.id}
												memberName={member.displayName}
												organizationSlug={organizationSlug}
												access={member.access}
												isEditing={isEditing}
												isSaving={isSaving}
												onEdit={() => onEdit(member)}
												onCancel={onCancel}
												onSave={onSave}
												onDelete={() => onDelete(member)}
												onInvite={() => onInvite(member)}
											/>
										</td>
									</tr>
								)
							})
						)}
					</tbody>
				</table>
			</div>
		</SectionCard>
	)
}

interface RowActionsProps {
	memberId: string
	memberName: string
	organizationSlug: string
	access: AccessState
	isEditing: boolean
	isSaving: boolean
	onEdit: () => void
	onCancel: () => void
	onSave: () => void
	onDelete: () => void
	onInvite: () => void
}

function RowActions({
	memberId,
	memberName,
	organizationSlug,
	access,
	isEditing,
	isSaving,
	onEdit,
	onCancel,
	onSave,
	onDelete,
	onInvite,
}: RowActionsProps) {
	if (isEditing) {
		return (
			<div className="flex justify-end gap-1">
				<Button size="icon-sm" variant="ghost" onClick={onCancel}>
					<Undo2 />
					<span className="sr-only">Annuler</span>
				</Button>
				<Button size="icon-sm" onClick={onSave} disabled={isSaving}>
					{isSaving ? <Loader2 className="animate-spin" /> : <Save />}
					<span className="sr-only">Enregistrer</span>
				</Button>
			</div>
		)
	}

	return (
		<AlertDialog>
			<div className="flex justify-end opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<Button size="icon-sm" variant="ghost">
							<MoreHorizontal />
							<span className="sr-only">Actions</span>
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end">
						<DropdownMenuItem onClick={onEdit}>Modifier</DropdownMenuItem>
						{access === 'none' ? (
							<DropdownMenuItem onClick={onInvite}>
								<UserPlus />
								Inviter
							</DropdownMenuItem>
						) : null}
						<DropdownMenuItem asChild>
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/hr/team/$memberId/work-time',
								)}
								params={{ memberId }}
							>
								<Clock />
								Temps de travail
							</Link>
						</DropdownMenuItem>
						<DropdownMenuSeparator />
						<AlertDialogTrigger asChild>
							<DropdownMenuItem
								variant="destructive"
								onSelect={(event) => event.preventDefault()}
							>
								<Trash2 />
								Supprimer
							</DropdownMenuItem>
						</AlertDialogTrigger>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Supprimer {memberName} ?</AlertDialogTitle>
					<AlertDialogDescription>
						Cette personne sera retirée de l’organisation. Cette action est
						irréversible.
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel>Annuler</AlertDialogCancel>
					<AlertDialogAction onClick={onDelete}>Supprimer</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
