import {
	AlertCircle,
	Mail,
	MoreHorizontal,
	Plus,
	Search,
	Shield,
	ShieldOff,
	Trash2,
	UserPlus,
} from 'lucide-react'
import { useMemo, useState } from 'react'
import { Button } from '#/components/ui/button'
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
	EntityAvatar,
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import type { CreateUserPayload, User } from '#/hooks/use-users'
import { userDisplayName, userInitials } from '#/pages/users/types'

interface UserListUIProps {
	users: User[]
	error?: string | null
	isLoading?: boolean
	isCreating?: boolean
	deletingId?: string | null
	onAdd?: (payload: CreateUserPayload) => Promise<unknown>
	onEdit?: (user: User) => void
	onDelete?: (user: User) => void
	onRetry?: () => void
}

export function UserListUI({
	users,
	error,
	isLoading,
	isCreating,
	deletingId,
	onAdd,
	onEdit,
	onDelete,
	onRetry,
}: UserListUIProps) {
	const [search, setSearch] = useState('')
	const [showCreate, setShowCreate] = useState(false)
	const [draft, setDraft] = useState({
		email: '',
		username: '',
		name: '',
		sendInviteEmail: true,
	})

	const counts = useMemo(
		() => ({
			total: users.length,
			active: users.filter((u) => u.enabled).length,
			disabled: users.filter((u) => !u.enabled).length,
			verified: users.filter((u) => u.email_verified).length,
		}),
		[users],
	)

	const visible = useMemo(() => {
		const q = search.trim().toLowerCase()
		return users.filter((u) => {
			if (!q) return true
			return (
				u.username.toLowerCase().includes(q) ||
				u.email.toLowerCase().includes(q) ||
				(u.name ?? '').toLowerCase().includes(q)
			)
		})
	}, [users, search])

	const canCreate = draft.email.trim() && draft.username.trim()

	const submitCreate = async () => {
		if (!canCreate) return
		await onAdd?.({
			email: draft.email.trim(),
			username: draft.username.trim(),
			name: draft.name.trim() || null,
			send_invite_email: draft.sendInviteEmail,
		})
		setDraft({ email: '', username: '', name: '', sendInviteEmail: true })
		setShowCreate(false)
	}

	return (
		<PageShell>
			<PageHeader
				title="Utilisateurs"
				description="Gérez les comptes utilisateurs du système. Les modifications sont répercutées dans FerrisKey."
				actions={
					<Button onClick={() => setShowCreate((v) => !v)}>
						<Plus />
						Nouvel utilisateur
					</Button>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					{onRetry ? (
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					) : null}
				</SectionCard>
			) : null}

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Aperçu</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard label="Total" value={counts.total} hint="Tous comptes" />
					<MetricCard
						label="Actifs"
						value={counts.active}
						hint="Comptes activés"
					/>
					<MetricCard
						label="Désactivés"
						value={counts.disabled}
						hint="Soft-deleted / disabled"
					/>
					<MetricCard
						label="Email vérifié"
						value={counts.verified}
						hint="Vérification email complète"
					/>
				</div>
			</section>

			{showCreate ? (
				<SectionCard>
					<div className="grid gap-4 p-5 md:grid-cols-3">
						<Field
							label="Email"
							type="email"
							value={draft.email}
							onChange={(email) => setDraft((v) => ({ ...v, email }))}
						/>
						<Field
							label="Nom d'utilisateur"
							value={draft.username}
							onChange={(username) => setDraft((v) => ({ ...v, username }))}
						/>
						<Field
							label="Nom complet (optionnel)"
							value={draft.name}
							onChange={(name) => setDraft((v) => ({ ...v, name }))}
						/>
					</div>
					<div className="flex items-center justify-between gap-2 border-t p-4">
						<label className="flex cursor-pointer items-center gap-2 text-sm">
							<input
								type="checkbox"
								className="accent-primary"
								checked={draft.sendInviteEmail}
								onChange={(e) =>
									setDraft((v) => ({ ...v, sendInviteEmail: e.target.checked }))
								}
							/>
							Envoyer un email d'invitation
						</label>
						<div className="flex gap-2">
							<Button
								type="button"
								variant="ghost"
								onClick={() => setShowCreate(false)}
							>
								Annuler
							</Button>
							<Button
								type="button"
								disabled={!canCreate || isCreating}
								onClick={() => void submitCreate()}
							>
								<Plus />
								Créer
							</Button>
						</div>
					</div>
				</SectionCard>
			) : null}

			<section className="flex flex-col gap-3">
				<div className="flex flex-col items-start justify-between gap-3 sm:flex-row sm:items-center">
					<h2 className="font-semibold">Comptes ({visible.length})</h2>
					<div className="relative w-full sm:w-72">
						<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
						<Input
							type="search"
							value={search}
							onChange={(e) => setSearch(e.target.value)}
							placeholder="Rechercher un utilisateur…"
							className="pl-9"
						/>
					</div>
				</div>

				{isLoading ? (
					<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
						Chargement…
					</SectionCard>
				) : visible.length === 0 ? (
					<SectionCard className="flex flex-col items-center justify-center gap-3 border-dashed p-12 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft">
							<UserPlus className="size-6 text-muted-foreground" />
						</div>
						<div>
							<p className="font-medium">Aucun utilisateur trouvé</p>
							<p className="text-sm text-muted-foreground">
								{search
									? "Essayez d'autres critères"
									: 'Commencez par créer le premier compte'}
							</p>
						</div>
						{!search && (
							<Button onClick={() => setShowCreate(true)} variant="outline">
								<Plus />
								Ajouter un utilisateur
							</Button>
						)}
					</SectionCard>
				) : (
					<ul className="divide-y overflow-hidden rounded-lg border bg-card">
						{visible.map((u) => (
							<li
								key={u.id}
								className="flex items-center gap-4 px-5 py-4 transition-colors hover:bg-muted/40"
							>
								<EntityAvatar tone={u.enabled ? 'brand' : 'neutral'}>
									{userInitials(u)}
								</EntityAvatar>

								<div className="min-w-0 flex-1">
									<div className="flex items-center gap-2">
										<p className="truncate font-semibold">
											{userDisplayName(u)}
										</p>
										{u.enabled ? (
											<StatusBadge tone="success">actif</StatusBadge>
										) : (
											<StatusBadge tone="error">désactivé</StatusBadge>
										)}
										{u.email_verified ? null : (
											<StatusBadge tone="neutral">
												email non vérifié
											</StatusBadge>
										)}
									</div>
									<p className="mt-0.5 truncate text-xs text-muted-foreground">
										@{u.username}
									</p>
								</div>

								<div className="hidden flex-col items-end gap-0.5 text-xs text-muted-foreground md:flex">
									<span className="flex items-center gap-1 truncate">
										<Mail className="size-3" />
										{u.email}
									</span>
									<span className="flex items-center gap-1">
										{u.enabled ? (
											<Shield className="size-3 text-green-500" />
										) : (
											<ShieldOff className="size-3 text-muted-foreground" />
										)}
										{u.id}
									</span>
								</div>

								<DropdownMenu>
									<DropdownMenuTrigger asChild>
										<Button
											variant="ghost"
											size="icon-sm"
											className="text-muted-foreground"
										>
											<MoreHorizontal />
											<span className="sr-only">Actions</span>
										</Button>
									</DropdownMenuTrigger>
									<DropdownMenuContent align="end">
										<DropdownMenuItem onClick={() => onEdit?.(u)}>
											Modifier
										</DropdownMenuItem>
										<DropdownMenuSeparator />
										<DropdownMenuItem
											variant="destructive"
											disabled={deletingId === u.id}
											onClick={() => onDelete?.(u)}
										>
											<Trash2 />
											Désactiver
										</DropdownMenuItem>
									</DropdownMenuContent>
								</DropdownMenu>
							</li>
						))}
					</ul>
				)}
			</section>
		</PageShell>
	)
}

export namespace UserListUI {
	export function Loading() {
		return (
			<PageShell>
				<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
					Chargement…
				</SectionCard>
			</PageShell>
		)
	}
}

interface FieldProps {
	label: string
	value: string
	onChange: (value: string) => void
	type?: string
}

function Field({ label, value, onChange, type = 'text' }: FieldProps) {
	const id = label.toLowerCase().replaceAll(/\s+/g, '-')
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<Input
				id={id}
				type={type}
				value={value}
				onChange={(event) => onChange(event.target.value)}
			/>
		</div>
	)
}
