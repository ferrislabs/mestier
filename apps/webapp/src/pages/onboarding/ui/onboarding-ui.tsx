import { Building2, Loader2, LogOut } from 'lucide-react'
import { MestierAppIcon } from '#/components/brand/mestier-logo'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { normalizeSlugInput } from '#/modules/org-slug'

interface OnboardingUIProps {
	name: string
	slug: string
	nameError?: string
	slugError?: string
	globalError?: string | null
	isPending: boolean
	onNameChange: (v: string) => void
	onSlugChange: (v: string) => void
	onSubmit: () => void
	onLogout: () => void
}

export function OnboardingUI({
	name,
	slug,
	nameError,
	slugError,
	globalError,
	isPending,
	onNameChange,
	onSlugChange,
	onSubmit,
	onLogout,
}: OnboardingUIProps) {
	return (
		<div className="flex min-h-screen flex-col items-center justify-center bg-background p-6">
			<div className="flex w-full max-w-md flex-col gap-8">
				<div className="flex flex-col items-center gap-3 text-center">
					<MestierAppIcon className="size-12 rounded-xl bg-primary text-primary-foreground" />
					<div>
						<h1 className="font-semibold text-2xl tracking-tight">
							Bienvenue sur Mestier
						</h1>
						<p className="mt-1 text-sm text-muted-foreground">
							Créez votre organisation pour commencer à gérer vos clients, devis
							et factures.
						</p>
					</div>
				</div>

				<div className="rounded-lg border bg-card p-6">
					<div className="mb-5 flex items-center gap-2">
						<div className="flex size-8 items-center justify-center rounded-lg border bg-secondary">
							<Building2 className="size-4 text-secondary-foreground" />
						</div>
						<p className="font-medium">Nouvelle organisation</p>
					</div>

					<form
						onSubmit={(e) => {
							e.preventDefault()
							onSubmit()
						}}
						className="flex flex-col gap-4"
					>
						<FormField
							id="org-name"
							label="Nom de l'organisation"
							error={nameError}
						>
							<Input
								id="org-name"
								placeholder="Entreprise Dupont"
								value={name}
								onChange={(e) => onNameChange(e.target.value)}
								disabled={isPending}
								aria-invalid={nameError ? true : undefined}
							/>
						</FormField>

						<FormField
							id="org-slug"
							label="Identifiant (slug)"
							hint="Utilisé dans les URLs"
							error={slugError}
						>
							<Input
								id="org-slug"
								placeholder="entreprise-dupont"
								value={slug}
								onChange={(e) =>
									onSlugChange(normalizeSlugInput(e.target.value))
								}
								disabled={isPending}
								aria-invalid={slugError ? true : undefined}
								className="font-mono text-sm"
							/>
						</FormField>

						{globalError ? (
							<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-3 py-2 text-sm text-destructive">
								{globalError}
							</p>
						) : null}

						<Button type="submit" disabled={isPending} className="mt-1 w-full">
							{isPending ? <Loader2 className="size-4 animate-spin" /> : null}
							Créer l'organisation
						</Button>
					</form>
				</div>

				<div className="flex items-center justify-between">
					<p className="text-[12px] text-muted-foreground">
						Vous pourrez modifier ces informations dans les paramètres.
					</p>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="gap-1.5 text-xs text-muted-foreground hover:text-foreground"
						onClick={onLogout}
					>
						<LogOut className="size-3.5" />
						Se déconnecter
					</Button>
				</div>
			</div>
		</div>
	)
}
