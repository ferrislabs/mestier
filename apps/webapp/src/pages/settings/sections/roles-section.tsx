import {
	AlertTriangle,
	Loader2,
	Plus,
	Save,
	Search,
	ShieldCheck,
	Trash2,
} from 'lucide-react'
import { useEffect, useState } from 'react'
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
import { Checkbox } from '#/components/ui/checkbox'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { Skeleton } from '#/components/ui/skeleton'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import {
	type Role,
	useCreateRole,
	useDeleteRole,
	useRoleMemberCounts,
	useRoles,
	useUpdateRole,
} from '#/hooks/use-roles'
import { mutationErrorMessage } from '#/lib/api-error'
import { permissionsByArea } from '#/lib/permission-catalog'

/**
 * Whether a mutation error carries a 409 — mirrors `isForbiddenError`'s own
 * status-check shape in `api-error.ts`, just for a different status: that
 * helper is 403-specific (#307) and would show the raw `"role still
 * assigned to a member"` English constraint text here instead of the
 * French sentence #308 asks for.
 */
function isConflictError(error: unknown): boolean {
	return (
		typeof error === 'object' &&
		error !== null &&
		'status' in error &&
		(error as { status?: unknown }).status === 409
	)
}

/**
 * A role delete is refused (409) when at least one member still holds it
 * (`RoleService::delete_role`) — the frontend has no cheap count of that at
 * delete-attempt time, so this is a fallback message shown after the fact,
 * never a pre-check that would hide the delete control instead.
 */
function deleteRoleErrorMessage(error: unknown): string | null {
	if (error == null) return null
	if (isConflictError(error)) {
		return 'Ce rôle est encore attribué à au moins un membre ; retirez-le de ce rôle avant de le supprimer.'
	}
	return mutationErrorMessage(error)
}

interface RoleFormValues {
	name: string
	permissions: string[]
}

function emptyRoleForm(): RoleFormValues {
	return { name: '', permissions: [] }
}

function roleToForm(role: Role): RoleFormValues {
	return { name: role.name, permissions: [...role.permissions] }
}

/**
 * The "Rôles" settings section (#308): view, create, edit and delete an
 * organization's roles. Gated on `MANAGE_ROLES` — presentation only, the
 * API refuses regardless (`RequirePermission`'s own doc), but a caller
 * without the bit sees an explanation here rather than a list every action
 * on which would 403.
 */
export function RolesSection() {
	const { activeOrganization } = useActiveOrganization()
	const canManageRoles = useHasPermission('MANAGE_ROLES')

	if (!canManageRoles) {
		return (
			<SectionCard>
				<SectionHeader
					title="Rôles"
					description="Définissez ce que chaque rôle peut faire dans l'organisation."
				/>
				<p className="p-5 text-sm text-muted-foreground">
					Vous n'avez pas la permission de gérer les rôles de cette
					organisation.
				</p>
			</SectionCard>
		)
	}

	return (
		<RolesSectionContent
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

interface RolesSectionContentProps {
	organizationId: string
}

/** Order-independent: two forms with the same bits in a different order are
 * not "unsaved changes". */
function sameForm(a: RoleFormValues, b: RoleFormValues): boolean {
	if (a.name !== b.name) return false
	if (a.permissions.length !== b.permissions.length) return false
	const sortedA = [...a.permissions].sort()
	const sortedB = [...b.permissions].sort()
	return sortedA.every((permission, index) => permission === sortedB[index])
}

function RolesSectionContent({ organizationId }: RolesSectionContentProps) {
	const roles = useRoles(organizationId)
	const createRole = useCreateRole(organizationId)
	const updateRole = useUpdateRole()
	const deleteRole = useDeleteRole()
	const members = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
	}).members
	const memberCounts = useRoleMemberCounts(
		(members.data?.data ?? []).map((member) => member.id),
	)

	const [sheetOpen, setSheetOpen] = useState(false)
	const [mode, setMode] = useState<'create' | 'edit'>('create')
	const [editingRole, setEditingRole] = useState<Role | null>(null)
	const [values, setValues] = useState<RoleFormValues>(emptyRoleForm())
	const [initialValues, setInitialValues] = useState<RoleFormValues>(
		emptyRoleForm(),
	)
	const [discardConfirmOpen, setDiscardConfirmOpen] = useState(false)

	const items = roles.data?.data ?? []
	const isDirty = !sameForm(values, initialValues)

	const openCreate = () => {
		setMode('create')
		setEditingRole(null)
		setValues(emptyRoleForm())
		setInitialValues(emptyRoleForm())
		setSheetOpen(true)
	}

	const openEdit = (role: Role) => {
		setMode('edit')
		setEditingRole(role)
		setValues(roleToForm(role))
		setInitialValues(roleToForm(role))
		setSheetOpen(true)
	}

	const requestSheetOpenChange = (next: boolean) => {
		if (!next && isDirty) {
			setDiscardConfirmOpen(true)
			return
		}
		setSheetOpen(next)
	}

	const handleSubmit = async () => {
		const body = { name: values.name.trim(), permissions: values.permissions }
		if (mode === 'create') {
			await createRole.mutateAsync({
				path: { organization_id: organizationId },
				body,
			})
		} else if (editingRole) {
			await updateRole.mutateAsync({
				path: { role_id: editingRole.id },
				body,
			})
		}
		setSheetOpen(false)
	}

	const saveError =
		mutationErrorMessage(createRole.error) ??
		mutationErrorMessage(updateRole.error)
	const deleteError = deleteRoleErrorMessage(deleteRole.error)

	return (
		<SectionCard>
			<SectionHeader
				title="Rôles"
				description="Définissez ce que chaque rôle peut faire, puis attribuez-le aux membres depuis l'équipe."
				actions={
					<Button
						type="button"
						size="sm"
						className="gap-2"
						onClick={openCreate}
					>
						<Plus className="size-4" />
						Créer un rôle
					</Button>
				}
			/>

			{deleteError ? (
				<div className="border-b border-destructive/30 bg-destructive-soft px-5 py-3 text-sm text-destructive">
					{deleteError}
				</div>
			) : null}

			{roles.isLoading ? (
				<RolesListSkeleton />
			) : roles.error ? (
				<p className="p-5 text-sm text-destructive">{roles.error.message}</p>
			) : items.length === 0 ? (
				<p className="p-5 text-sm text-muted-foreground">
					Aucun rôle pour cette organisation.
				</p>
			) : (
				<ul className="divide-y">
					{items.map((role) => (
						<RoleRow
							key={role.id}
							role={role}
							memberCount={memberCounts.get(role.id) ?? 0}
							onEdit={() => openEdit(role)}
							onDelete={() => deleteRole.mutate({ path: { role_id: role.id } })}
						/>
					))}
				</ul>
			)}

			<RoleEditorSheet
				open={sheetOpen}
				mode={mode}
				editingRole={editingRole}
				values={values}
				isPending={createRole.isPending || updateRole.isPending}
				error={saveError}
				onOpenChange={requestSheetOpenChange}
				onChangeName={(name) => setValues((current) => ({ ...current, name }))}
				onTogglePermission={(name, checked) =>
					setValues((current) => ({
						...current,
						permissions: checked
							? [...current.permissions, name]
							: current.permissions.filter((permission) => permission !== name),
					}))
				}
				onSetPermissions={(names) =>
					setValues((current) => ({ ...current, permissions: names }))
				}
				onSubmit={() => void handleSubmit()}
			/>

			<AlertDialog
				open={discardConfirmOpen}
				onOpenChange={setDiscardConfirmOpen}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Abandonner les modifications ?</AlertDialogTitle>
						<AlertDialogDescription>
							Les changements apportés à ce rôle n'ont pas été enregistrés et
							seront perdus.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Continuer l'édition</AlertDialogCancel>
						<AlertDialogAction
							onClick={() => {
								setDiscardConfirmOpen(false)
								setSheetOpen(false)
							}}
						>
							Abandonner
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</SectionCard>
	)
}

function RolesListSkeleton() {
	return (
		<div className="flex flex-col gap-4 p-5" aria-busy="true">
			{[0, 1, 2].map((row) => (
				<div key={row} className="flex items-center justify-between gap-4">
					<div className="flex flex-1 flex-col gap-2">
						<Skeleton className="h-4 w-32" />
						<Skeleton className="h-3 w-64" />
					</div>
					<Skeleton className="h-8 w-20" />
				</div>
			))}
		</div>
	)
}

interface RoleRowProps {
	role: Role
	memberCount: number
	onEdit: () => void
	onDelete: () => void
}

function RoleRow({ role, memberCount, onEdit, onDelete }: RoleRowProps) {
	const grantedByArea = permissionsByArea()
		.map((area) => ({
			...area,
			granted: area.permissions.filter((permission) =>
				role.permissions.includes(permission.name),
			),
		}))
		.filter((area) => area.granted.length > 0)

	return (
		<li className="flex flex-col gap-3 p-5 sm:flex-row sm:items-start sm:justify-between">
			<div className="min-w-0 flex-1">
				<div className="flex flex-wrap items-center gap-2">
					<span className="font-medium text-foreground">{role.name}</span>
					{role.is_seeded ? (
						<Badge variant="secondary">
							<ShieldCheck className="mr-1 size-3" />
							Rôle prédéfini
						</Badge>
					) : null}
					<Badge variant="outline">
						{memberCount === 0
							? 'Aucun membre'
							: memberCount === 1
								? '1 membre'
								: `${memberCount} membres`}
					</Badge>
				</div>
				{grantedByArea.length > 0 ? (
					<div className="mt-2 flex flex-col gap-1">
						{grantedByArea.map((area) => (
							<p key={area.area} className="text-xs text-muted-foreground">
								<span className="font-medium">{area.label} : </span>
								{area.granted.map((permission) => permission.label).join(', ')}
							</p>
						))}
					</div>
				) : (
					<p className="mt-2 text-xs text-muted-foreground">
						Aucune permission accordée.
					</p>
				)}
			</div>
			<div className="flex shrink-0 items-center gap-2">
				<Button type="button" variant="outline" size="sm" onClick={onEdit}>
					Modifier
				</Button>
				{role.is_seeded ? null : (
					<AlertDialog>
						<AlertDialogTrigger asChild>
							<Button
								type="button"
								variant="outline"
								size="sm"
								className="gap-1 text-destructive"
							>
								<Trash2 className="size-4" />
								Supprimer
							</Button>
						</AlertDialogTrigger>
						<AlertDialogContent>
							<AlertDialogHeader>
								<AlertDialogTitle>
									Supprimer le rôle « {role.name} » ?
								</AlertDialogTitle>
								<AlertDialogDescription>
									Cette action est irréversible. Un rôle encore attribué à un
									membre ne peut pas être supprimé.
								</AlertDialogDescription>
							</AlertDialogHeader>
							<AlertDialogFooter>
								<AlertDialogCancel>Annuler</AlertDialogCancel>
								<AlertDialogAction onClick={onDelete}>
									Supprimer
								</AlertDialogAction>
							</AlertDialogFooter>
						</AlertDialogContent>
					</AlertDialog>
				)}
			</div>
		</li>
	)
}

interface RoleEditorSheetProps {
	open: boolean
	mode: 'create' | 'edit'
	editingRole: Role | null
	values: RoleFormValues
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onChangeName: (name: string) => void
	onTogglePermission: (name: string, checked: boolean) => void
	/** Replaces the whole permission set — what "tout cocher"/"tout décocher"
	 * per area needs, since toggling one bit at a time can't express "add
	 * the four bits this area still lacks" in a single state update. */
	onSetPermissions: (names: string[]) => void
	onSubmit: () => void
}

/**
 * The permission editor grouped by area (#308): a labeled group per area
 * rather than 15 flat checkboxes, each with its label, a description
 * underneath, and — where the catalog carries one — a note surfaced right
 * next to the checkbox rather than tucked in a tooltip nobody opens.
 */
function RoleEditorSheet({
	open,
	mode,
	editingRole,
	values,
	isPending,
	error,
	onOpenChange,
	onChangeName,
	onTogglePermission,
	onSetPermissions,
	onSubmit,
}: RoleEditorSheetProps) {
	const [permissionSearch, setPermissionSearch] = useState('')
	const nameDisabled = mode === 'edit' && Boolean(editingRole?.is_seeded)
	const canSubmit = values.name.trim() !== ''

	const normalizedSearch = permissionSearch.trim().toLowerCase()
	const visibleAreas = permissionsByArea()
		.map((area) => ({
			...area,
			permissions: area.permissions.filter(
				(permission) =>
					permission.label.toLowerCase().includes(normalizedSearch) ||
					permission.description.toLowerCase().includes(normalizedSearch),
			),
		}))
		.filter((area) => area.permissions.length > 0)

	// Clears the search left over from the previous role each time the sheet
	// reopens.
	useEffect(() => {
		if (open) setPermissionSearch('')
	}, [open])

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
				<SheetHeader>
					<SheetTitle>
						{mode === 'create'
							? 'Créer un rôle'
							: `Modifier « ${editingRole?.name ?? ''} »`}
					</SheetTitle>
					<SheetDescription>
						Un rôle regroupe des permissions ; attribuez-le ensuite à un ou
						plusieurs membres depuis l'équipe.
					</SheetDescription>
				</SheetHeader>

				<div className="flex flex-1 flex-col gap-6 overflow-y-auto px-4 pb-4">
					{error ? (
						<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-3 py-2 text-sm text-destructive">
							{error}
						</p>
					) : null}

					<div className="flex flex-col gap-2">
						<Label htmlFor="role-name">Nom</Label>
						<Input
							id="role-name"
							value={values.name}
							onChange={(event) => onChangeName(event.target.value)}
							disabled={nameDisabled}
							placeholder="Chef de chantier"
						/>
						{nameDisabled ? (
							<p className="text-xs text-muted-foreground">
								Le nom d'un rôle prédéfini ne peut pas être modifié.
							</p>
						) : null}
					</div>

					<div className="relative">
						<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
						<Input
							value={permissionSearch}
							onChange={(event) => setPermissionSearch(event.target.value)}
							placeholder="Rechercher une permission…"
							className="pl-9"
						/>
					</div>

					{visibleAreas.length === 0 ? (
						<p className="text-sm text-muted-foreground">
							Aucune permission ne correspond à « {permissionSearch} ».
						</p>
					) : (
						visibleAreas.map((area) => {
							const allChecked = area.permissions.every((permission) =>
								values.permissions.includes(permission.name),
							)
							return (
								<div key={area.area} className="flex flex-col gap-3">
									<div className="flex items-center justify-between">
										<h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
											{area.label}
										</h3>
										<Button
											type="button"
											variant="link"
											size="sm"
											className="h-auto p-0 text-xs"
											onClick={() => {
												const areaNames: string[] = area.permissions.map(
													(permission) => permission.name,
												)
												onSetPermissions(
													allChecked
														? values.permissions.filter(
																(name) => !areaNames.includes(name),
															)
														: [
																...values.permissions.filter(
																	(name) => !areaNames.includes(name),
																),
																...areaNames,
															],
												)
											}}
										>
											{allChecked ? 'Tout décocher' : 'Tout cocher'}
										</Button>
									</div>
									<div className="flex flex-col gap-3">
										{area.permissions.map((permission) => {
											const inputId = `permission-${permission.name}`
											const checked = values.permissions.includes(
												permission.name,
											)
											return (
												<div
													key={permission.name}
													className="flex items-start gap-3"
												>
													<Checkbox
														id={inputId}
														checked={checked}
														onCheckedChange={(next) =>
															onTogglePermission(permission.name, next === true)
														}
														className="mt-0.5"
													/>
													<div className="flex flex-col gap-1">
														<Label htmlFor={inputId} className="font-normal">
															{permission.label}
														</Label>
														<p className="text-xs text-muted-foreground">
															{permission.description}
														</p>
														{permission.note ? (
															<p className="flex items-start gap-1.5 text-xs text-amber-800 dark:text-amber-300">
																<AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
																<span>{permission.note}</span>
															</p>
														) : null}
													</div>
												</div>
											)
										})}
									</div>
								</div>
							)
						})
					)}
				</div>

				<SheetFooter>
					<Button
						type="button"
						onClick={onSubmit}
						disabled={isPending || !canSubmit}
						className="gap-2"
					>
						{isPending ? (
							<Loader2 className="size-4 animate-spin" />
						) : (
							<Save className="size-4" />
						)}
						{mode === 'create' ? 'Créer' : 'Enregistrer'}
					</Button>
				</SheetFooter>
			</SheetContent>
		</Sheet>
	)
}
