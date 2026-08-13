import { Check, Copy, KeyRound, Loader2, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { RowActions } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import {
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { AuthField, AuthScheme } from '#/hooks/use-automation'
import type { CredentialFormValues } from '#/pages/settings/types'

export interface CredentialRow {
	id: string
	name: string
	kind: string
	kindLabel: string
	origin: 'supplied' | 'generated'
	updatedAt: string
}

interface FormBinding {
	values: CredentialFormValues
	isPending: boolean
	onChange: (patch: Partial<CredentialFormValues>) => void
	onSubmit: () => void
}

export interface AutomationCredentialsUIProps {
	credentials: CredentialRow[]
	authSchemes: AuthScheme[]
	isLoading: boolean
	error: string | null

	sheetOpen: boolean
	sheetMode: 'create' | 'edit'
	form: FormBinding
	formErrors: string[]
	saveError: string | null
	/** Base64, shown exactly once — right after a `generated` credential is
	 * created or rotated. `null` the rest of the time, including every
	 * later view of the same credential: the backend never returns it
	 * again, so there is nothing to restore even on reopen. */
	revealedSecret: string | null
	onOpenCreate: () => void
	onEdit: (row: CredentialRow) => void
	onOpenChange: (open: boolean) => void

	onRotate: (row: CredentialRow) => void
	rotatingId: string | null
	onDelete: (row: CredentialRow) => void
}

export function AutomationCredentialsUI({
	credentials,
	authSchemes,
	isLoading,
	error,
	sheetOpen,
	sheetMode,
	form,
	formErrors,
	saveError,
	revealedSecret,
	onOpenCreate,
	onEdit,
	onOpenChange,
	onRotate,
	rotatingId,
	onDelete,
}: AutomationCredentialsUIProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Identifications (${credentials.length})`}
				description="Authentification réutilisable par plusieurs connecteurs — Odoo, un webhook sortant, une API tierce."
				actions={
					<Button onClick={onOpenCreate} className="gap-2">
						<KeyRound className="size-4" />
						Ajouter
					</Button>
				}
			/>

			{error ? (
				<div className="mx-5 mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<div className="flex items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement…
				</div>
			) : (
				<div className="overflow-x-auto">
					<table className="w-full min-w-[640px] border-collapse text-sm">
						<thead>
							<tr className="border-b bg-muted/50">
								<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
									Nom
								</th>
								<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
									Type
								</th>
								<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
									Origine
								</th>
								<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
									<span className="sr-only">Actions</span>
								</th>
							</tr>
						</thead>
						<tbody>
							{credentials.length === 0 ? (
								<tr>
									<td colSpan={4} className="px-5 py-12 text-center">
										<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
											<p className="font-medium">Aucune identification</p>
											<p className="text-sm text-muted-foreground">
												Ajoutez-en une pour l’attacher à un connecteur.
											</p>
										</div>
									</td>
								</tr>
							) : (
								credentials.map((credential) => (
									<tr
										key={credential.id}
										className="group border-b transition hover:bg-muted/35 hover:shadow-xs last:border-b-0"
									>
										<td className="px-5 py-3 align-middle font-medium">
											{credential.name}
										</td>
										<td className="px-5 py-3 align-middle text-muted-foreground">
											{credential.kindLabel}
										</td>
										<td className="px-5 py-3 align-middle">
											<StatusBadge
												tone={
													credential.origin === 'generated'
														? 'brand'
														: 'neutral'
												}
											>
												{credential.origin === 'generated'
													? 'Générée'
													: 'Fournie'}
											</StatusBadge>
										</td>
										<td className="px-5 py-3 align-middle">
											<div className="flex justify-end gap-1 opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
												{credential.origin === 'generated' ? (
													<Button
														size="icon-sm"
														variant="ghost"
														title="Régénérer le secret"
														onClick={() => onRotate(credential)}
														disabled={rotatingId === credential.id}
													>
														{rotatingId === credential.id ? (
															<Loader2 className="animate-spin" />
														) : (
															<RefreshCw />
														)}
														<span className="sr-only">Régénérer</span>
													</Button>
												) : null}
												<RowActions
													isEditing={false}
													isSaving={false}
													onEdit={() => onEdit(credential)}
													onCancel={() => {}}
													onSave={() => {}}
													onDelete={() => onDelete(credential)}
												/>
											</div>
										</td>
									</tr>
								))
							)}
						</tbody>
					</table>
				</div>
			)}

			<CredentialFormSheet
				open={sheetOpen}
				mode={sheetMode}
				authSchemes={authSchemes}
				form={form}
				errors={formErrors}
				saveError={saveError}
				revealedSecret={revealedSecret}
				onOpenChange={onOpenChange}
			/>
		</SectionCard>
	)
}

interface CredentialFormSheetProps {
	open: boolean
	mode: 'create' | 'edit'
	authSchemes: AuthScheme[]
	form: FormBinding
	errors: string[]
	saveError: string | null
	revealedSecret: string | null
	onOpenChange: (open: boolean) => void
}

function CredentialFormSheet({
	open,
	mode,
	authSchemes,
	form,
	errors,
	saveError,
	revealedSecret,
	onOpenChange,
}: CredentialFormSheetProps) {
	const scheme = authSchemes.find(
		(candidate) => candidate.kind === form.values.kind,
	)
	const canSubmit = errors.length === 0 && !form.isPending

	if (revealedSecret !== null) {
		return (
			<Sheet open={open} onOpenChange={onOpenChange}>
				<SheetContent className="w-full gap-0 sm:max-w-lg">
					<SheetHeader className="border-b">
						<SheetTitle>Secret généré</SheetTitle>
						<SheetDescription>
							Copiez-le maintenant : il ne sera plus jamais affiché.
						</SheetDescription>
					</SheetHeader>
					<div className="flex-1 space-y-4 p-4">
						<SecretReveal value={revealedSecret} />
					</div>
					<SheetFooter className="border-t bg-background">
						<Button type="button" onClick={() => onOpenChange(false)}>
							Fermer
						</Button>
					</SheetFooter>
				</SheetContent>
			</Sheet>
		)
	}

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
				<form
					className="flex min-h-0 flex-1 flex-col"
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit) form.onSubmit()
					}}
				>
					<SheetHeader className="border-b">
						<SheetTitle>
							{mode === 'create'
								? 'Nouvelle identification'
								: 'Modifier l’identification'}
						</SheetTitle>
						<SheetDescription>
							{mode === 'create'
								? 'Le type détermine les champs attendus — jamais de texte libre.'
								: 'Laissez les champs vides pour ne pas modifier les données déjà enregistrées.'}
						</SheetDescription>
					</SheetHeader>

					<div className="flex-1 space-y-5 overflow-y-auto p-4">
						<div className="flex flex-col gap-2">
							<Label htmlFor="credential-name">Nom</Label>
							<Input
								id="credential-name"
								value={form.values.name}
								onChange={(event) =>
									form.onChange({ name: event.target.value })
								}
								placeholder="Ex. Odoo production"
							/>
						</div>

						<div className="flex flex-col gap-2">
							<Label htmlFor="credential-kind">Type</Label>
							<Select
								value={form.values.kind}
								onValueChange={(kind) => form.onChange({ kind, data: {} })}
								disabled={mode === 'edit'}
							>
								<SelectTrigger id="credential-kind" className="w-full">
									<SelectValue placeholder="Choisir un type" />
								</SelectTrigger>
								<SelectContent>
									{authSchemes.map((candidate) => (
										<SelectItem key={candidate.kind} value={candidate.kind}>
											{candidate.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>

						{mode === 'create' ? (
							<div className="flex flex-col gap-2">
								<Label htmlFor="credential-origin">Origine</Label>
								<Select
									value={form.values.origin}
									onValueChange={(origin) =>
										form.onChange({
											origin: origin as CredentialFormValues['origin'],
										})
									}
								>
									<SelectTrigger id="credential-origin" className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="supplied">
											Fournie par vous (mot de passe, clé d’API…)
										</SelectItem>
										<SelectItem value="generated">
											Générée par Mestier (secret de signature)
										</SelectItem>
									</SelectContent>
								</Select>
							</div>
						) : null}

						{form.values.origin === 'supplied' && scheme
							? scheme.fields.map((field) => (
									<AuthFieldInput
										key={field.name}
										field={field}
										value={form.values.data[field.name] ?? ''}
										onChange={(value) =>
											form.onChange({
												data: { ...form.values.data, [field.name]: value },
											})
										}
									/>
								))
							: null}

						{errors.length > 0 ? (
							<ul className="flex flex-col gap-1 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{errors.map((message) => (
									<li key={message}>{message}</li>
								))}
							</ul>
						) : null}

						{saveError ? (
							<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
								{saveError}
							</p>
						) : null}
					</div>

					<SheetFooter className="border-t bg-background sm:flex-row sm:justify-end">
						<Button
							type="button"
							variant="ghost"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={!canSubmit}>
							{form.isPending ? <Loader2 className="animate-spin" /> : null}
							{mode === 'create' ? 'Créer' : 'Enregistrer'}
						</Button>
					</SheetFooter>
				</form>
			</SheetContent>
		</Sheet>
	)
}

function AuthFieldInput({
	field,
	value,
	onChange,
}: {
	field: AuthField
	value: string
	onChange: (value: string) => void
}) {
	const id = `credential-field-${field.name}`
	const kind = field.kind

	if (typeof kind === 'object' && 'Select' in kind) {
		return (
			<div className="flex flex-col gap-2">
				<Label htmlFor={id}>{field.label}</Label>
				<Select value={value} onValueChange={onChange}>
					<SelectTrigger id={id} className="w-full">
						<SelectValue placeholder="Choisir…" />
					</SelectTrigger>
					<SelectContent>
						{kind.Select.options.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
		)
	}

	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{field.label}</Label>
			<Input
				id={id}
				type={field.secret ? 'password' : kind === 'Number' ? 'number' : 'text'}
				value={value}
				onChange={(event) => onChange(event.target.value)}
				autoComplete="off"
			/>
		</div>
	)
}

function SecretReveal({ value }: { value: string }) {
	const [copied, setCopied] = useState(false)

	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor="revealed-secret">Secret</Label>
			<div className="flex gap-2">
				<Input
					id="revealed-secret"
					readOnly
					value={value}
					className="font-mono"
				/>
				<Button
					type="button"
					variant="outline"
					onClick={() => {
						void navigator.clipboard.writeText(value)
						setCopied(true)
					}}
				>
					{copied ? <Check /> : <Copy />}
					{copied ? 'Copié' : 'Copier'}
				</Button>
			</div>
		</div>
	)
}
