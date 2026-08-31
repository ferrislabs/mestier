import { Link } from '@tanstack/react-router'
import {
	Clock,
	Loader2,
	MoreHorizontal,
	Plus,
	Save,
	Search,
	ShieldPlus,
	Trash2,
	Undo2,
	UserPlus,
	Users,
	X,
} from 'lucide-react'
import { useState } from 'react'
import {
	CreateButton,
	formatMoney,
	MoneyCell,
	TextField,
} from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
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
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Popover,
	PopoverContent,
	PopoverTitle,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAssignRole,
	useMemberRoleIds,
	useRoles,
	useUnassignRole,
} from '#/hooks/use-roles'
import { mutationErrorMessage } from '#/lib/api-error'
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
	/** False both when the seat has no profile and when it has an hourly one. */
	isSalaried: boolean
	/** `null` when not set, or when the seat is not salaried. */
	monthlyCostCents: number | null
	/**
	 * What an hour of this person costs on whichever basis they are on, computed
	 * by the backend so the browser never re-implements the division. `null`
	 * means it cannot be stated, which the row has to say rather than imply.
	 */
	effectiveHourlyRateCents: number | null
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
	organizationSlug: string
	isLoading: boolean
	error: string | null
	/**
	 * True when the caller lacks the permission to read employee profiles
	 * (`member.manage`) — an expected access boundary, not a failure. Rates
	 * and contract bases come back `null` for every row in that case
	 * regardless of whether the seat actually has a profile, so the table
	 * says "not visible to you" rather than the misleading "no profile" it
	 * would otherwise show for everyone. See #371.
	 */
	hrDataRestricted: boolean
	members: TeamMemberRow[]
	search: string
	onSearchChange: (value: string) => void
	createForm: FormBinding<MemberFormValues>
	createMemberDialogOpen: boolean
	onCreateMemberDialogOpenChange: (open: boolean) => void
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
	organizationSlug,
	isLoading,
	error,
	hrDataRestricted,
	members,
	search,
	onSearchChange,
	createForm,
	createMemberDialogOpen,
	onCreateMemberDialogOpenChange,
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
				title="Équipe"
				description="Gérez les membres de l’organisation, leurs accès et leurs taux horaires."
				actions={
					<RequirePermission permission="MANAGE_MEMBERS">
						<Button onClick={() => onCreateMemberDialogOpenChange(true)}>
							<Plus />
							Ajouter une personne
						</Button>
					</RequirePermission>
				}
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

			{!error && hrDataRestricted ? (
				<div className="rounded-lg border border-border bg-muted/50 px-4 py-3 text-sm text-muted-foreground">
					Vous n’avez pas la permission de consulter les taux horaires et bases
					contractuelles de l’équipe.
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
					hrDataRestricted={hrDataRestricted}
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

			<Dialog
				open={createMemberDialogOpen}
				onOpenChange={onCreateMemberDialogOpenChange}
			>
				<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden">
					<DialogHeader className="border-b pb-4">
						<DialogTitle>Ajouter une personne</DialogTitle>
						<DialogDescription>
							Le profil RH est optionnel : laissez le taux horaire vide pour un
							siège sans profil, renseignez-le pour lui en attacher un — ou
							marquez la personne comme salariée si elle n'a pas de coût
							horaire.
						</DialogDescription>
					</DialogHeader>
					<div className="flex-1 overflow-y-auto py-4">
						<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
							<TextField
								label="Nom"
								value={createForm.values.lastName}
								onChange={(lastName) => createForm.onChange({ lastName })}
							/>
							<TextField
								label="Prénom"
								value={createForm.values.firstName}
								onChange={(firstName) => createForm.onChange({ firstName })}
								placeholder="Optionnel"
							/>
							<TextField
								label="Taux horaire"
								value={createForm.values.hourlyRate}
								onChange={(hourlyRate) => createForm.onChange({ hourlyRate })}
								inputMode="decimal"
								suffix="€/h"
								placeholder="Optionnel"
								disabled={createForm.values.isSalaried}
							/>
							{createForm.values.isSalaried ? (
								<TextField
									label="Coût employeur mensuel"
									value={createForm.values.monthlyCost}
									onChange={(monthlyCost) =>
										createForm.onChange({ monthlyCost })
									}
									inputMode="decimal"
									suffix="€/mois"
									placeholder="Salaire chargé"
								/>
							) : null}
						</div>
						<div className="mt-4 flex items-center gap-2">
							<Switch
								id="create-member-salaried"
								checked={createForm.values.isSalaried}
								onCheckedChange={(isSalaried) =>
									createForm.onChange({
										isSalaried,
										hourlyRate: '',
										monthlyCost: '',
									})
								}
							/>
							<Label
								htmlFor="create-member-salaried"
								className="text-sm font-normal"
							>
								Salarié (coût mensuel plutôt qu'horaire)
							</Label>
						</div>
						{createForm.values.isSalaried ? (
							<p className="mt-2 text-xs text-muted-foreground">
								Le coût horaire est calculé à partir de ce montant et des heures
								contractuelles. Sans montant, la rentabilité refuse de chiffrer
								son temps plutôt que de le compter gratuit.
							</p>
						) : null}
					</div>
					<DialogFooter className="border-t pt-4">
						<Button
							type="button"
							variant="ghost"
							onClick={() => onCreateMemberDialogOpenChange(false)}
						>
							Annuler
						</Button>
						<CreateButton
							isPending={createForm.isPending}
							onClick={createForm.onSubmit}
						/>
					</DialogFooter>
				</DialogContent>
			</Dialog>
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
	hrDataRestricted: boolean
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
	hrDataRestricted,
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
								Rôles
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
								<td colSpan={6} className="px-5 py-12 text-center">
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
											<MemberRoleCell memberId={member.id} />
										</td>
										<td className="px-5 py-3 align-middle">
											{isEditing ? (
												<div className="flex flex-col gap-1.5">
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
														disabled={draft.values.isSalaried}
													/>
													{draft.values.isSalaried ? (
														<Input
															aria-label="Coût employeur mensuel"
															value={draft.values.monthlyCost}
															onChange={(event) =>
																onDraftChange({
																	...draft.values,
																	monthlyCost: event.target.value,
																})
															}
															inputMode="decimal"
															placeholder="€/mois chargé"
														/>
													) : null}
													<div className="flex items-center gap-1.5">
														<Switch
															id={`salaried-${member.id}`}
															size="sm"
															checked={draft.values.isSalaried}
															onCheckedChange={(isSalaried) =>
																onDraftChange({
																	...draft.values,
																	isSalaried,
																	hourlyRate: '',
																	monthlyCost: '',
																})
															}
														/>
														<Label
															htmlFor={`salaried-${member.id}`}
															className="text-xs font-normal text-muted-foreground"
														>
															Salarié
														</Label>
													</div>
												</div>
											) : member.isSalaried ? (
												<SalariedRateCell
													monthlyCostCents={member.monthlyCostCents}
													effectiveHourlyRateCents={
														member.effectiveHourlyRateCents
													}
													memberId={member.id}
													organizationSlug={organizationSlug}
												/>
											) : (
												<MoneyCell value={member.hourlyRateCents} suffix="/h" />
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											{member.weeklyContractMinutes === null ? (
												<span className="text-muted-foreground italic">
													{hrDataRestricted
														? 'Non consultable'
														: 'Sans profil RH'}
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
						<RequirePermission permission="MANAGE_MEMBERS">
							<DropdownMenuItem onClick={onEdit}>Modifier</DropdownMenuItem>
						</RequirePermission>
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
						<RequirePermission permission="MANAGE_MEMBERS">
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
						</RequirePermission>
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

/**
 * The role(s) a member holds, plus — behind `MANAGE_ROLES` — a small popover
 * to assign one more, and a remove control on each held role's badge (#308,
 * #401, #402). `useRoles` and `useMemberRoleIds` are read directly here
 * rather than threaded down as props, the same way `RequirePermission`
 * already resolves its own permission read inside this file — a per-row,
 * per-member query has no natural place in a plain `TeamMemberRow` without
 * turning every row's shape into an N+1 fetch plan owned by the parent.
 */
function MemberRoleCell({ memberId }: { memberId: string }) {
	const { activeOrganizationId } = useActiveOrganization()
	const rolesQuery = useRoles(activeOrganizationId)
	const memberRolesQuery = useMemberRoleIds(memberId)
	const assignRole = useAssignRole()
	const unassignRole = useUnassignRole()
	const [open, setOpen] = useState(false)
	const [search, setSearch] = useState('')

	// Neither read has resolved yet (or one of them 403'd) — say nothing
	// rather than flash "Aucun rôle" and then have it change once the read
	// comes back.
	const isKnown = rolesQuery.isSuccess && memberRolesQuery.isSuccess

	const roles = rolesQuery.data?.data ?? []
	const heldIds = new Set(memberRolesQuery.data?.data.role_ids ?? [])
	const heldRoles = roles.filter((role) => heldIds.has(role.id))
	const assignableRoles = roles
		.filter((role) => !heldIds.has(role.id))
		.filter((role) =>
			role.name.toLowerCase().includes(search.trim().toLowerCase()),
		)

	return (
		<div className="flex flex-wrap items-center gap-1.5">
			{isKnown ? (
				heldRoles.length === 0 ? (
					<span className="text-sm italic text-muted-foreground">
						Aucun rôle
					</span>
				) : (
					heldRoles.map((role) => (
						<Badge key={role.id} variant="outline" className="gap-1 pr-1">
							{role.name}
							<RequirePermission permission="MANAGE_ROLES">
								<button
									type="button"
									className="rounded-full p-0.5 text-muted-foreground transition hover:bg-destructive-soft hover:text-destructive disabled:pointer-events-none disabled:opacity-50"
									disabled={unassignRole.isPending}
									aria-label={`Retirer le rôle ${role.name}`}
									onClick={() =>
										unassignRole.mutate({
											path: { member_id: memberId, role_id: role.id },
										})
									}
								>
									<X className="size-3" />
								</button>
							</RequirePermission>
						</Badge>
					))
				)
			) : (
				<Loader2 className="size-3.5 animate-spin text-muted-foreground" />
			)}
			{unassignRole.isError ? (
				<p className="text-xs text-destructive">
					{mutationErrorMessage(unassignRole.error)}
				</p>
			) : null}
			<RequirePermission permission="MANAGE_ROLES">
				{isKnown && assignableRoles.length === 0 && search === '' ? (
					<Button
						size="icon-sm"
						variant="ghost"
						disabled
						aria-label="Ce membre a déjà tous les rôles"
					>
						<ShieldPlus />
					</Button>
				) : (
					<Popover
						open={open}
						onOpenChange={(next) => {
							setOpen(next)
							if (!next) setSearch('')
						}}
					>
						<PopoverTrigger asChild>
							<Button
								size="icon-sm"
								variant="ghost"
								disabled={!isKnown}
								aria-label="Assigner un rôle"
							>
								<ShieldPlus />
							</Button>
						</PopoverTrigger>
						<PopoverContent align="start" className="w-56 p-2">
							<PopoverTitle className="px-2 pb-1">
								Assigner un rôle
							</PopoverTitle>
							{roles.length - heldRoles.length > 5 ? (
								<div className="relative px-2 pb-2">
									<Search className="pointer-events-none absolute left-4 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
									<Input
										autoFocus
										value={search}
										onChange={(event) => setSearch(event.target.value)}
										placeholder="Rechercher un rôle…"
										className="h-8 pl-7 text-sm"
									/>
								</div>
							) : null}
							<div className="flex max-h-56 flex-col overflow-y-auto">
								{assignableRoles.length === 0 ? (
									<p className="px-2 py-1.5 text-sm text-muted-foreground">
										Aucun rôle trouvé
									</p>
								) : (
									assignableRoles.map((role) => (
										<Button
											key={role.id}
											variant="ghost"
											className="justify-start"
											disabled={assignRole.isPending}
											onClick={() =>
												assignRole.mutate(
													{
														path: { member_id: memberId },
														body: { role_id: role.id },
													},
													{ onSuccess: () => setOpen(false) },
												)
											}
										>
											{role.name}
										</Button>
									))
								)}
							</div>
							{assignRole.isError ? (
								<p className="px-2 pt-1 text-xs text-destructive">
									{mutationErrorMessage(assignRole.error)}
								</p>
							) : null}
						</PopoverContent>
					</Popover>
				)}
			</RequirePermission>
		</div>
	)
}

/**
 * A salaried person's cost, on both bases at once.
 *
 * The monthly amount is what was typed; the hourly figure beside it is what
 * profitability will actually use, computed server-side. Showing only the first
 * would leave the reader unable to sanity-check the number that ends up on a
 * margin, and showing only the second would hide what they entered.
 *
 * When the equivalent cannot be stated — no amount, or no contracted hours to
 * spread it over — the cell says so outright. This column used to read just
 * "Salarié", which is how an hour of somebody's time came to show up as 0,00 €.
 */
function SalariedRateCell({
	monthlyCostCents,
	effectiveHourlyRateCents,
	memberId,
	organizationSlug,
}: {
	monthlyCostCents: number | null
	effectiveHourlyRateCents: number | null
	memberId: string
	organizationSlug: string
}) {
	if (monthlyCostCents === null) {
		return (
			<span className="text-sm text-amber-600 dark:text-amber-500">
				Salarié, coût mensuel non renseigné
			</span>
		)
	}

	return (
		<div className="flex flex-col">
			<MoneyCell value={monthlyCostCents} suffix="/mois" />
			<span className="text-xs text-muted-foreground">
				{effectiveHourlyRateCents === null ? (
					// The gap that stranded somebody: they had entered the salary and
					// the message named the salary. The contract is edited on another
					// screen, under the exact words used here — "base contractuelle",
					// not "heures contractuelles", because a pointer that renames its
					// destination is a pointer nobody can follow.
					<Link
						to={buildOrgPath(organizationSlug, '/hr/team/$memberId/work-time')}
						params={{ memberId }}
						className="text-amber-600 underline dark:text-amber-500"
					>
						Base contractuelle à renseigner
					</Link>
				) : (
					`soit ${formatMoney(effectiveHourlyRateCents)}/h`
				)}
			</span>
		</div>
	)
}
