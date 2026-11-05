import { Check, Copy, Loader2, RefreshCw } from 'lucide-react'
import { useId, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { AuthField, AuthScheme } from '#/hooks/use-automation'
import type { ExternalService } from '#/pages/automation/feature/mocks/external-services'
import type { CredentialFormValues } from '#/pages/automation/types'
import { CredentialKindAside } from '#/pages/automation/ui/credential-kind-aside'
import { CredentialKindPicker } from '#/pages/automation/ui/credential-kind-picker'
import type { CredentialRow } from '#/pages/automation/ui/credentials-table'

export interface CredentialFormBinding {
	values: CredentialFormValues
	isPending: boolean
	onChange: (patch: Partial<CredentialFormValues>) => void
	onSubmit: () => void
}

export interface CredentialDialogProps {
	open: boolean
	mode: 'create' | 'edit'
	editing: CredentialRow | null
	authSchemes: AuthScheme[]
	services: readonly ExternalService[]
	form: CredentialFormBinding
	formErrors: string[]
	saveError: string | null
	revealedSecret: string | null
	rotatingId: string | null
	onOpenChange: (open: boolean) => void
	onRotate: (row: CredentialRow) => void
	onDelete: (row: CredentialRow) => void
}

function AuthFieldInput({
	field,
	value,
	placeholder,
	onChange,
}: {
	field: AuthField
	value: string
	placeholder?: string
	onChange: (value: string) => void
}) {
	const id = `credential-field-${field.name}`
	const kind = field.kind

	if (typeof kind === 'object' && 'Select' in kind) {
		return (
			<div className="grid gap-2">
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
		<div className="grid gap-2">
			<Label htmlFor={id}>{field.label}</Label>
			<Input
				id={id}
				type={field.secret ? 'password' : kind === 'Number' ? 'number' : 'text'}
				value={value}
				placeholder={placeholder}
				onChange={(event) => onChange(event.target.value)}
				autoComplete="off"
			/>
		</div>
	)
}

function SecretReveal({ value }: { value: string }) {
	const [copied, setCopied] = useState(false)

	return (
		<div className="grid gap-2">
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

function CredentialForm({
	mode,
	editing,
	scheme,
	form,
	formErrors,
	saveError,
	rotatingId,
	onBack,
	onCancel,
	onRotate,
	onDelete,
}: {
	mode: 'create' | 'edit'
	editing: CredentialRow | null
	scheme: AuthScheme
	form: CredentialFormBinding
	formErrors: string[]
	saveError: string | null
	rotatingId: string | null
	onBack?: () => void
	onCancel: () => void
	onRotate: (row: CredentialRow) => void
	onDelete: (row: CredentialRow) => void
}) {
	const baseId = useId()
	const [showErrors, setShowErrors] = useState(false)
	const [confirmingDelete, setConfirmingDelete] = useState(false)
	const generated = form.values.origin === 'generated'

	const submit = () => {
		if (formErrors.length > 0) {
			setShowErrors(true)
			return
		}
		form.onSubmit()
	}

	return (
		<div className="fade-in slide-in-from-right-6 grid min-h-0 flex-1 animate-in duration-300 sm:grid-cols-[17rem_minmax(0,1fr)]">
			<CredentialKindAside
				scheme={scheme}
				usage="Utilisable par les connecteurs qui demandent ce type d’authentification."
				meta={
					editing
						? `Modifiée le ${new Date(editing.updatedAt).toLocaleDateString('fr-FR', { day: 'numeric', month: 'short', year: 'numeric' })}.`
						: undefined
				}
				onBack={onBack}
			/>
			<form
				className="flex min-h-0 flex-col"
				onSubmit={(event) => {
					event.preventDefault()
					submit()
				}}
			>
				<DialogHeader className="border-border border-b px-6 py-4 text-left">
					<DialogTitle className="truncate">
						{editing ? editing.name : 'Nouvelle identification'}
					</DialogTitle>
					<DialogDescription>
						{mode === 'create'
							? 'Nommez-la, puis renseignez les paramètres de connexion.'
							: 'Modifiez son nom ou remplacez ses paramètres de connexion. Laissez les champs vides pour ne pas modifier les données déjà enregistrées.'}
					</DialogDescription>
				</DialogHeader>
				<div className="grid min-h-0 flex-1 content-start gap-5 overflow-y-auto px-6 py-5">
					<div className="grid gap-4">
						<div className="grid gap-2">
							<Label htmlFor={`${baseId}-name`}>Nom</Label>
							<Input
								id={`${baseId}-name`}
								value={form.values.name}
								onChange={(event) =>
									form.onChange({ name: event.target.value })
								}
								placeholder={`Ex. ${scheme.label} – fournisseur`}
								autoFocus={mode === 'create'}
							/>
						</div>
						{mode === 'create' ? (
							<div className="grid gap-2">
								<Label htmlFor={`${baseId}-origin`}>Origine</Label>
								<Select
									value={form.values.origin}
									onValueChange={(origin) =>
										form.onChange({
											origin: origin as CredentialFormValues['origin'],
										})
									}
								>
									<SelectTrigger id={`${baseId}-origin`} className="w-full">
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
					</div>
					<section aria-label="Connexion" className="grid gap-4">
						<h3 className="font-semibold text-sm">Connexion</h3>
						{generated ? (
							<div className="grid gap-2 rounded-md border border-border bg-muted/40 p-3">
								<p className="text-muted-foreground text-sm">
									Le secret est généré par Mestier et n’est affiché qu’une seule
									fois.
								</p>
								{editing ? (
									<RequirePermission permission="MANAGE_AUTOMATION">
										<div>
											<Button
												type="button"
												variant="outline"
												size="sm"
												disabled={rotatingId === editing.id}
												onClick={() => onRotate(editing)}
											>
												{rotatingId === editing.id ? (
													<Loader2 className="animate-spin" />
												) : (
													<RefreshCw />
												)}
												Régénérer le secret
											</Button>
										</div>
									</RequirePermission>
								) : null}
							</div>
						) : (
							scheme.fields.map((field) => (
								<AuthFieldInput
									key={field.name}
									field={field}
									value={form.values.data[field.name] ?? ''}
									placeholder={mode === 'edit' ? 'Inchangé' : undefined}
									onChange={(value) =>
										form.onChange({
											data: { ...form.values.data, [field.name]: value },
										})
									}
								/>
							))
						)}
					</section>
					{showErrors && formErrors.length > 0 ? (
						<ul className="grid gap-1 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
							{formErrors.map((message) => (
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
				<DialogFooter className="border-border border-t px-6 py-3">
					{confirmingDelete && editing ? (
						<>
							<p className="mr-auto self-center text-sm">
								Supprimer « {editing.name} » ?{' '}
								<span className="text-muted-foreground">
									Cette action est définitive.
								</span>
							</p>
							<Button
								type="button"
								variant="outline"
								onClick={() => setConfirmingDelete(false)}
							>
								Annuler
							</Button>
							<Button
								type="button"
								variant="destructive"
								onClick={() => onDelete(editing)}
							>
								Supprimer définitivement
							</Button>
						</>
					) : (
						<>
							{editing && (
								<RequirePermission permission="MANAGE_AUTOMATION">
									<Button
										type="button"
										variant="ghost"
										className="text-state-alert hover:text-state-alert sm:mr-auto"
										onClick={() => setConfirmingDelete(true)}
									>
										Supprimer
									</Button>
								</RequirePermission>
							)}
							<Button type="button" variant="outline" onClick={onCancel}>
								Annuler
							</Button>
							<Button type="submit" disabled={form.isPending}>
								{form.isPending ? <Loader2 className="animate-spin" /> : null}
								{mode === 'create' ? 'Créer l’identification' : 'Enregistrer'}
							</Button>
						</>
					)}
				</DialogFooter>
			</form>
		</div>
	)
}

export function CredentialDialog({
	open,
	mode,
	editing,
	authSchemes,
	services,
	form,
	formErrors,
	saveError,
	revealedSecret,
	rotatingId,
	onOpenChange,
	onRotate,
	onDelete,
}: CredentialDialogProps) {
	const scheme = authSchemes.find(
		(candidate) => candidate.kind === form.values.kind,
	)

	if (revealedSecret !== null) {
		return (
			<Dialog open={open} onOpenChange={onOpenChange}>
				<DialogContent className="gap-0 p-0 sm:max-w-lg">
					<DialogHeader className="border-border border-b px-6 py-4">
						<DialogTitle>Secret généré</DialogTitle>
						<DialogDescription>
							Copiez-le maintenant : il ne sera plus jamais affiché.
						</DialogDescription>
					</DialogHeader>
					<div className="px-6 py-5">
						<SecretReveal value={revealedSecret} />
					</div>
					<DialogFooter className="border-border border-t px-6 py-3">
						<Button type="button" onClick={() => onOpenChange(false)}>
							Fermer
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		)
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				className="flex h-[min(48rem,90vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-6xl"
			>
				{scheme ? (
					<CredentialForm
						key={`${mode}:${editing?.id ?? 'new'}:${scheme.kind}`}
						mode={mode}
						editing={editing}
						scheme={scheme}
						form={form}
						formErrors={formErrors}
						saveError={saveError}
						rotatingId={rotatingId}
						onBack={
							mode === 'create'
								? () => form.onChange({ kind: '', data: {} })
								: undefined
						}
						onCancel={() => onOpenChange(false)}
						onRotate={onRotate}
						onDelete={onDelete}
					/>
				) : (
					<div className="fade-in slide-in-from-left-6 flex min-h-0 flex-1 animate-in flex-col duration-300">
						<DialogHeader className="border-border border-b px-6 py-4">
							<DialogTitle>Nouvelle identification</DialogTitle>
							<DialogDescription>
								Choisissez comment Mestier doit s’identifier auprès de l’autre
								système.
							</DialogDescription>
						</DialogHeader>
						<div className="min-h-0 flex-1 overflow-y-auto px-6 py-5">
							<CredentialKindPicker
								schemes={authSchemes}
								services={services}
								onPick={(kind) => form.onChange({ kind, data: {} })}
							/>
						</div>
					</div>
				)}
			</DialogContent>
		</Dialog>
	)
}
