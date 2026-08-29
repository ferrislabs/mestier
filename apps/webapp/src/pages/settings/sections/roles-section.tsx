import {
	AlertTriangle,
	Loader2,
	Plus,
	Save,
	ShieldCheck,
	Trash2,
} from 'lucide-react'
import { useState } from 'react'
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
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	type Role,
	useCreateRole,
	useDeleteRole,
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

function RolesSectionContent({ organizationId }: RolesSectionContentProps) {
	const roles = useRoles(organizationId)
	const createRole = useCreateRole(organizationId)
	const updateRole = useUpdateRole()
	const deleteRole = useDeleteRole()

	const [sheetOpen, setSheetOpen] = useState(false)
	const [mode, setMode] = useState<'create' | 'edit'>('create')
	const [editingRole, setEditingRole] = useState<Role | null>(null)
	const [values, setValues] = useState<RoleFormValues>(emptyRoleForm())

	const items = roles.data?.data ?? []

	const openCreate = () => {
		setMode('create')
		setEditingRole(null)
		setValues(emptyRoleForm())
		setSheetOpen(true)
	}

	const openEdit = (role: Role) => {
		setMode('edit')
		setEditingRole(role)
		setValues(roleToForm(role))
		setSheetOpen(true)
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
				<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
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
				onOpenChange={setSheetOpen}
				onChangeName={(name) => setValues((current) => ({ ...current, name }))}
				onTogglePermission={(name, checked) =>
					setValues((current) => ({
						...current,
						permissions: checked
							? [...current.permissions, name]
							: current.permissions.filter((permission) => permission !== name),
					}))
				}
				onSubmit={() => void handleSubmit()}
			/>
		</SectionCard>
	)
}

interface RoleRowProps {
	role: Role
	onEdit: () => void
	onDelete: () => void
}

function RoleRow({ role, onEdit, onDelete }: RoleRowProps) {
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
	onSubmit,
}: RoleEditorSheetProps) {
	const nameDisabled = mode === 'edit' && Boolean(editingRole?.is_seeded)
	const canSubmit = values.name.trim() !== ''

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

					{permissionsByArea().map((area) => (
						<div key={area.area} className="flex flex-col gap-3">
							<h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
								{area.label}
							</h3>
							<div className="flex flex-col gap-3">
								{area.permissions.map((permission) => {
									const inputId = `permission-${permission.name}`
									const checked = values.permissions.includes(permission.name)
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
					))}
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
