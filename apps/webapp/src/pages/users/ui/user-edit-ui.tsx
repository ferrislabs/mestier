import { Link } from '@tanstack/react-router'
import { ArrowLeft, ShieldOff } from 'lucide-react'
import { FloatingActionBar } from '#/components/floating-action-bar'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	EntityAvatar,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { User } from '#/hooks/use-users'
import { type UserFormValues, userInitials } from '#/pages/users/types'

interface UserEditUIProps {
	user: User
	form: UserFormValues
	isDirty: boolean
	isSaving: boolean
	isDeleting: boolean
	onChange: (patch: Partial<UserFormValues>) => void
	onReset: () => void
	onSave: () => void
	onDelete: () => void
}

export function UserEditUI({
	user,
	form,
	isDirty,
	isSaving,
	isDeleting,
	onChange,
	onReset,
	onSave,
	onDelete,
}: UserEditUIProps) {
	return (
		<PageShell>
			<PageHeader
				title={form.username || user.username}
				description={user.email}
				actions={
					<Button asChild variant="ghost" size="sm">
						<Link to="/users">
							<ArrowLeft />
							Retour
						</Link>
					</Button>
				}
			/>

			<SectionCard>
				<SectionHeader title="Identité" />
				<div className="grid gap-4 p-5 md:grid-cols-2">
					<EditField
						label="Email"
						type="email"
						value={form.email}
						onChange={(email) => onChange({ email })}
					/>
					<EditField
						label="Nom d'utilisateur"
						value={form.username}
						onChange={(username) => onChange({ username })}
					/>
					<EditField
						label="Nom complet"
						value={form.name}
						onChange={(name) => onChange({ name })}
					/>
					<div className="flex flex-col gap-2">
						<Label>Statut</Label>
						<label className="flex cursor-pointer items-center gap-2 rounded-md border px-3 py-2 text-sm">
							<input
								type="checkbox"
								className="accent-primary"
								checked={form.enabled}
								onChange={(e) => onChange({ enabled: e.target.checked })}
							/>
							Compte activé
						</label>
					</div>
				</div>
			</SectionCard>

			<SectionCard>
				<SectionHeader title="Identifiants système" />
				<div className="flex items-center gap-4 p-5">
					<EntityAvatar tone={user.enabled ? 'brand' : 'neutral'}>
						{userInitials(user)}
					</EntityAvatar>
					<div className="flex flex-col gap-1 font-mono text-xs text-muted-foreground">
						<span>id: {user.id}</span>
						<span>
							email_verified:{' '}
							{user.email_verified ? (
								<StatusBadge tone="success">oui</StatusBadge>
							) : (
								<StatusBadge tone="neutral">non</StatusBadge>
							)}
						</span>
					</div>
				</div>
			</SectionCard>

			<SectionCard className="border-destructive/30">
				<SectionHeader title="Zone de danger" />
				<div className="flex items-center justify-between gap-4 p-5">
					<div>
						<p className="text-sm font-medium">Désactiver ce compte</p>
						<p className="text-xs text-muted-foreground">
							L'utilisateur sera désactivé dans FerrisKey et son accès révoqué.
							Action réversible via un webhook de réactivation.
						</p>
					</div>
					<Button
						variant="destructive"
						size="sm"
						disabled={isDeleting || !user.enabled}
						onClick={onDelete}
					>
						<ShieldOff />
						Désactiver
					</Button>
				</div>
			</SectionCard>

			{isDirty && (
				<FloatingActionBar>
					<Button
						variant="ghost"
						size="sm"
						onClick={onReset}
						disabled={isSaving}
					>
						Annuler
					</Button>
					<Button size="sm" onClick={onSave} disabled={isSaving}>
						Enregistrer
					</Button>
				</FloatingActionBar>
			)}
		</PageShell>
	)
}

export namespace UserEditUI {
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

interface EditFieldProps {
	label: string
	value: string
	onChange: (value: string) => void
	type?: string
}

function EditField({ label, value, onChange, type = 'text' }: EditFieldProps) {
	const id = label.toLowerCase().replaceAll(/\s+/g, '-')
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<Input
				id={id}
				type={type}
				value={value}
				onChange={(e) => onChange(e.target.value)}
			/>
		</div>
	)
}
