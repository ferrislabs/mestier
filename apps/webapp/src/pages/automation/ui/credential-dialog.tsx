import { Check, Copy, Loader2, PlugZap, RefreshCw } from 'lucide-react'
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
import { cn } from '#/lib/utils'
import type { ExternalService } from '#/pages/automation/feature/mocks/external-services'
import type { CredentialFieldErrors } from '#/pages/automation/lib/credential-form'
import {
	isOauthAuthorization,
	schemeUsageText,
} from '#/pages/automation/lib/credential-meta'
import { formatUsage } from '#/pages/automation/lib/format'
import {
	supportsViewTransition,
	withViewTransition,
} from '#/pages/automation/lib/view-transition'
import type { CredentialFormValues } from '#/pages/automation/types'
import { CredentialKindAside } from '#/pages/automation/ui/credential-kind-aside'
import { CredentialKindPicker } from '#/pages/automation/ui/credential-kind-picker'
import { CredentialStatusLabel } from '#/pages/automation/ui/credential-status-label'
import {
	type CredentialRow,
	formatCredentialDate,
} from '#/pages/automation/ui/credentials-table'
import { SecretField } from '#/pages/automation/ui/secret-field'
import { TextArea } from '#/pages/automation/ui/text-area'

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
	fieldErrors: CredentialFieldErrors
	saveError: string | null
	revealedSecret: string | null
	rotatingId: string | null
	onOpenChange: (open: boolean) => void
	onRotate: (row: CredentialRow) => void
	onDelete: (row: CredentialRow) => void
	onConnect: (row: CredentialRow) => void
	onDisconnect: (row: CredentialRow) => void
}

function FieldControl({
	field,
	controlId,
	value,
	stored,
	invalid,
	describedBy,
	onChange,
}: {
	field: AuthField
	controlId: string
	value: string
	stored: boolean
	invalid: boolean
	describedBy?: string
	onChange: (value: string) => void
}) {
	const aria = {
		'aria-invalid': invalid || undefined,
		'aria-describedby': describedBy,
	}
	const kind = field.kind

	if (typeof kind === 'object' && 'Select' in kind) {
		return (
			<Select value={value} onValueChange={onChange}>
				<SelectTrigger id={controlId} className="w-full" {...aria}>
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
		)
	}

	if (field.secret || kind === 'Json')
		return (
			<SecretField
				id={controlId}
				stored={stored}
				multiline={kind === 'Json'}
				value={value}
				invalid={invalid}
				describedBy={describedBy}
				onChange={onChange}
			/>
		)

	return (
		<Input
			id={controlId}
			value={value}
			inputMode={kind === 'Number' ? 'numeric' : undefined}
			placeholder={stored ? 'Inchangé' : undefined}
			onChange={(event) => onChange(event.target.value)}
			className={kind === 'Number' ? 'max-w-32' : undefined}
			autoComplete="off"
			{...aria}
		/>
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

function AuthorizationSection({
	credential,
	onConnect,
	onDisconnect,
}: {
	credential: CredentialRow
	onConnect: (row: CredentialRow) => void
	onDisconnect: (row: CredentialRow) => void
}) {
	const connected = credential.status !== 'not-connected'
	return (
		<section
			aria-label="Compte autorisé"
			className="grid gap-2 rounded-md border border-border bg-muted/40 p-3"
		>
			<div className="flex items-center justify-between gap-3">
				<CredentialStatusLabel status={credential.status} />
				<div className="flex gap-2">
					{connected && (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							onClick={() => onDisconnect(credential)}
						>
							Déconnecter
						</Button>
					)}
					<Button type="button" size="sm" onClick={() => onConnect(credential)}>
						<PlugZap />
						{connected ? 'Reconnecter' : 'Connecter un compte'}
					</Button>
				</div>
			</div>
			<p className="text-muted-foreground text-xs">
				{connected
					? `Compte ${credential.account ?? 'autorisé'}. Mestier renouvelle le jeton automatiquement.`
					: 'Mestier vous redirige vers le fournisseur pour autoriser l’accès, puis conserve le jeton chiffré.'}
			</p>
		</section>
	)
}

function CredentialForm({
	mode,
	editing,
	scheme,
	form,
	fieldErrors,
	saveError,
	rotatingId,
	onBack,
	onCancel,
	onRotate,
	onDelete,
	onConnect,
	onDisconnect,
}: {
	mode: 'create' | 'edit'
	editing: CredentialRow | null
	scheme: AuthScheme
	form: CredentialFormBinding
	fieldErrors: CredentialFieldErrors
	saveError: string | null
	rotatingId: string | null
	onBack?: () => void
	onCancel: () => void
	onRotate: (row: CredentialRow) => void
	onDelete: (row: CredentialRow) => void
	onConnect: (row: CredentialRow) => void
	onDisconnect: (row: CredentialRow) => void
}) {
	const baseId = useId()
	const [showErrors, setShowErrors] = useState(false)
	const [confirmingDelete, setConfirmingDelete] = useState(false)
	const generated = form.values.origin === 'generated'
	const errorOf = (key: string) => (showErrors ? fieldErrors[key] : undefined)

	const submit = () => {
		if (Object.keys(fieldErrors).length > 0) {
			setShowErrors(true)
			return
		}
		form.onSubmit()
	}

	const errorText = (key: string, controlId: string) => {
		const message = errorOf(key)
		return message ? (
			<p id={`${controlId}-error`} className="text-state-alert text-xs">
				{message}
			</p>
		) : null
	}

	return (
		<div
			className={cn(
				'grid min-h-0 flex-1 sm:grid-cols-[17rem_minmax(0,1fr)]',
				!supportsViewTransition() &&
					'fade-in slide-in-from-right-6 animate-in duration-300',
			)}
		>
			<CredentialKindAside
				scheme={scheme}
				usage={schemeUsageText(scheme.kind)}
				meta={
					editing
						? `Créée le ${formatCredentialDate(editing.createdAt)} · ${formatUsage(editing.usedByWorkflows).toLowerCase()}.`
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
							: 'Modifiez son nom, sa description ou ses paramètres de connexion. Laissez les champs vides pour ne pas modifier les données déjà enregistrées.'}
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
								aria-invalid={errorOf('name') ? true : undefined}
								aria-describedby={
									errorOf('name') ? `${baseId}-name-error` : undefined
								}
								autoFocus={mode === 'create'}
							/>
							{errorText('name', `${baseId}-name`)}
						</div>
						<div className="grid gap-2">
							<Label htmlFor={`${baseId}-description`}>
								Description{' '}
								<span className="font-normal text-muted-foreground">
									(facultatif)
								</span>
							</Label>
							<TextArea
								id={`${baseId}-description`}
								rows={2}
								value={form.values.description}
								onChange={(event) =>
									form.onChange({ description: event.target.value })
								}
								placeholder="À quoi sert-elle ? Qui l’a fournie ?"
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
							scheme.fields.map((field) => {
								const controlId = `${baseId}-${field.name}`
								const error = errorOf(field.name)
								const isSelect =
									typeof field.kind === 'object' && 'Select' in field.kind
								return (
									<div key={field.name} className="grid gap-2">
										<Label htmlFor={controlId}>
											{field.label}
											{!field.required && !isSelect && (
												<span className="font-normal text-muted-foreground">
													{' '}
													(facultatif)
												</span>
											)}
										</Label>
										<FieldControl
											field={field}
											controlId={controlId}
											value={form.values.data[field.name] ?? ''}
											stored={mode === 'edit'}
											invalid={Boolean(error)}
											describedBy={error ? `${controlId}-error` : undefined}
											onChange={(value) =>
												form.onChange({
													data: { ...form.values.data, [field.name]: value },
												})
											}
										/>
										{errorText(field.name, controlId)}
									</div>
								)
							})
						)}
						{editing && isOauthAuthorization(scheme.kind) ? (
							<AuthorizationSection
								credential={editing}
								onConnect={onConnect}
								onDisconnect={onDisconnect}
							/>
						) : null}
					</section>
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
									{editing.usedByWorkflows > 0
										? `${formatUsage(editing.usedByWorkflows)} : leurs nœuds échoueront.`
										: 'Cette action est définitive.'}
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
	fieldErrors,
	saveError,
	revealedSecret,
	rotatingId,
	onOpenChange,
	onRotate,
	onDelete,
	onConnect,
	onDisconnect,
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
						fieldErrors={fieldErrors}
						saveError={saveError}
						rotatingId={rotatingId}
						onBack={
							mode === 'create'
								? () =>
										withViewTransition(() =>
											form.onChange({ kind: '', data: {} }),
										)
								: undefined
						}
						onCancel={() => onOpenChange(false)}
						onRotate={onRotate}
						onDelete={onDelete}
						onConnect={onConnect}
						onDisconnect={onDisconnect}
					/>
				) : (
					<div
						className={cn(
							'flex min-h-0 flex-1 flex-col',
							!supportsViewTransition() &&
								'fade-in slide-in-from-left-6 animate-in duration-300',
						)}
					>
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
								onPick={(kind) =>
									withViewTransition(() => form.onChange({ kind, data: {} }))
								}
							/>
						</div>
					</div>
				)}
			</DialogContent>
		</Dialog>
	)
}
