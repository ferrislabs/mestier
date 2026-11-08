import { Loader2, Search, ShieldPlus, X } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Popover,
	PopoverContent,
	PopoverTitle,
	PopoverTrigger,
} from '#/components/ui/popover'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAssignRole,
	useMemberRoleIds,
	useRoles,
	useUnassignRole,
} from '#/hooks/use-roles'
import { mutationErrorMessage } from '#/lib/api-error'

export function MemberRoleCell({ memberId }: { memberId: string }) {
	const { activeOrganizationId } = useActiveOrganization()
	const rolesQuery = useRoles(activeOrganizationId)
	const memberRolesQuery = useMemberRoleIds(memberId)
	const assignRole = useAssignRole()
	const unassignRole = useUnassignRole()
	const [open, setOpen] = useState(false)
	const [search, setSearch] = useState('')

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
