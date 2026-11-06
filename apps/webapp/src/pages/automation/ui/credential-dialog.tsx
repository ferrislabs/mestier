import { Loader2, PlugZap, RefreshCw } from 'lucide-react'
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
import { Switch } from '#/components/ui/switch'
import type { CredentialField, CredentialKind } from '#/hooks/use-automation'
import { cn } from '#/lib/utils'
import type { ExternalService } from '#/pages/automation/feature/mocks/external-services'
import {
	type CredentialFieldErrors,
	credentialFormDefaults,
	fieldType,
	isMultiline,
	visibleCredentialFields,
} from '#/pages/automation/lib/credential-form'
import {
	allowsGeneratedOrigin,
	usageSentence,
} from '#/pages/automation/lib/credential-kinds'
import type { CredentialRow } from '#/pages/automation/lib/credential-rows'
import { formatUsage } from '#/pages/automation/lib/format'
import {
	supportsViewTransition,
	withViewTransition,
} from '#/pages/automation/lib/view-transition'
import type { CredentialFormValues } from '#/pages/automation/types'
import { CredentialKindAside } from '#/pages/automation/ui/credential-kind-aside'
import { CredentialKindPicker } from '#/pages/automation/ui/credential-kind-picker'
import { SecretRevealField } from '#/pages/automation/ui/credential-secret-reveal'
import { CredentialStatusLabel } from '#/pages/automation/ui/credential-status-label'
import { formatCredentialDate } from '#/pages/automation/ui/credentials-table'
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
	kinds: CredentialKind[]
	services: readonly ExternalService[]
	form: CredentialFormBinding
	fieldErrors: CredentialFieldErrors
	serverErrors: CredentialFieldErrors
	saveError: string | null
	revealedSecret: string | null
	rotatingId: string | null
	isDeleting: boolean
	isConnecting: boolean
	kindsLoading: boolean
	onOpenChange: (open: boolean) => void
	onRotate: (row: CredentialRow) => void
	onDelete: (row: CredentialRow) => void
	onConnect: (row: CredentialRow) => void
	onDisconnect: (row: CredentialRow) => void
}

export function CredentialFieldControl({
	field,
	controlId,
	value,
	stored,
	invalid,
	describedBy,
	onChange,
}: {
	field: CredentialField
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
	const type = fieldType(field)

	if (type === 'select') {
		return (
			<Select value={value} onValueChange={onChange}>
				<SelectTrigger id={controlId} className="w-full" {...aria}>
					<SelectValue placeholder="Choisir…" />
				</SelectTrigger>
				<SelectContent>
					{field.options.map((option) => (
						<SelectItem key={option.value} value={option.value}>
							{option.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		)
	}

	if (type === 'boolean')
		return (
			<Switch
				id={controlId}
				checked={value === 'true'}
				onCheckedChange={(checked) => onChange(String(checked))}
				{...aria}
			/>
		)

	if (field.secret)
		return (
			<SecretField
				id={controlId}
				stored={stored}
				multiline={isMultiline(field)}
				value={value}
				placeholder={field.placeholder ?? undefined}
				invalid={invalid}
				describedBy={describedBy}
				onChange={onChange}
			/>
		)

	if (isMultiline(field))
		return (
			<TextArea
				id={controlId}
				rows={4}
				value={value}
				placeholder={field.placeholder ?? undefined}
				onChange={(event) => onChange(event.target.value)}
				className="font-mono text-xs"
				{...aria}
			/>
		)

	return (
		<Input
			id={controlId}
			type={type === 'password' ? 'password' : undefined}
			value={value}
			inputMode={type === 'number' ? 'numeric' : undefined}
			placeholder={field.placeholder ?? undefined}
			onChange={(event) => onChange(event.target.value)}
			className={type === 'number' ? 'max-w-32' : undefined}
			autoComplete="off"
			{...aria}
		/>
	)
}

export function CredentialFieldRows({
	fields,
	data,
	baseId,
	storedSecrets,
	errorOf,
	onChange,
}: {
	fields: readonly CredentialField[]
	data: Readonly<Record<string, string>>
	baseId: string
	storedSecrets: readonly string[]
	errorOf: (key: string) => string | undefined
	onChange: (data: Record<string, string>) => void
}) {
	return fields.map((field) => {
		const controlId = `${baseId}-${field.name}`
		const error = errorOf(field.name)
		const type = fieldType(field)
		return (
			<div key={field.name} className="grid gap-2">
				<Label htmlFor={controlId}>
					{field.label}
					{!field.required && type !== 'select' && type !== 'boolean' && (
						<span className="font-normal text-muted-foreground">
							{' '}
							(facultatif)
						</span>
					)}
				</Label>
				<CredentialFieldControl
					field={field}
					controlId={controlId}
					value={data[field.name] ?? ''}
					stored={field.secret && storedSecrets.includes(field.name)}
					invalid={Boolean(error)}
					describedBy={error ? `${controlId}-error` : undefined}
					onChange={(value) => onChange({ ...data, [field.name]: value })}
				/>
				{field.hint && !error && (
					<p className="text-muted-foreground text-xs">{field.hint}</p>
				)}
				{error && (
					<p id={`${controlId}-error`} className="text-state-alert text-xs">
						{error}
					</p>
				)}
			</div>
		)
	})
}

function authorizationText(credential: CredentialRow): string {
	if (credential.status === 'not_connected')
		return 'Mestier vous redirige vers le fournisseur pour autoriser l’accès, puis conserve le jeton chiffré.'
	if (credential.status === 'expired')
		return 'L’autorisation a expiré. Reconnectez le compte pour la renouveler.'
	return `Compte ${credential.account ?? 'autorisé'}. Mestier renouvelle le jeton automatiquement.`
}

function AuthorizationSection({
	credential,
	pending,
	onConnect,
	onDisconnect,
}: {
	credential: CredentialRow
	pending: boolean
	onConnect: (row: CredentialRow) => void
	onDisconnect: (row: CredentialRow) => void
}) {
	const connected = credential.status !== 'not_connected'
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
							disabled={pending}
							onClick={() => onDisconnect(credential)}
						>
							Déconnecter
						</Button>
					)}
					<Button
						type="button"
						size="sm"
						disabled={pending}
						onClick={() => onConnect(credential)}
					>
						{pending ? <Loader2 className="animate-spin" /> : <PlugZap />}
						{connected ? 'Reconnecter' : 'Connecter un compte'}
					</Button>
				</div>
			</div>
			<p className="text-muted-foreground text-xs">
				{authorizationText(credential)}
			</p>
		</section>
	)
}

function CredentialForm({
	mode,
	editing,
	kind,
	form,
	fieldErrors,
	serverErrors,
	saveError,
	rotatingId,
	isDeleting,
	isConnecting,
	onBack,
	onCancel,
	onRotate,
	onDelete,
	onConnect,
	onDisconnect,
}: {
	mode: 'create' | 'edit'
	editing: CredentialRow | null
	kind: CredentialKind
	form: CredentialFormBinding
	fieldErrors: CredentialFieldErrors
	serverErrors: CredentialFieldErrors
	saveError: string | null
	rotatingId: string | null
	isDeleting: boolean
	isConnecting: boolean
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
	const visibleFields = visibleCredentialFields(kind.fields, form.values.data)
	const errorOf = (key: string) =>
		(showErrors ? fieldErrors[key] : undefined) ?? serverErrors[key]

	const submit = () => {
		setShowErrors(true)
		if (Object.keys(fieldErrors).length > 0) return
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
				kind={kind}
				usage={usageSentence(kind)}
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
							: 'Modifiez son nom, sa description ou ses paramètres de connexion.'}
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
								placeholder={`Ex. ${kind.label} – fournisseur`}
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
								aria-invalid={errorOf('description') ? true : undefined}
							/>
							{errorText('description', `${baseId}-description`)}
						</div>
						{mode === 'create' && allowsGeneratedOrigin(kind) ? (
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
								<p className="text-muted-foreground text-xs">
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
							<CredentialFieldRows
								fields={visibleFields}
								data={form.values.data}
								baseId={baseId}
								storedSecrets={editing?.secretsSet ?? []}
								errorOf={errorOf}
								onChange={(data) => form.onChange({ data })}
							/>
						)}
						{editing && kind.connectable ? (
							<AuthorizationSection
								credential={editing}
								pending={isConnecting}
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
								disabled={isDeleting}
								onClick={() => setConfirmingDelete(false)}
							>
								Annuler
							</Button>
							<Button
								type="button"
								variant="destructive"
								disabled={isDeleting}
								onClick={() => onDelete(editing)}
							>
								{isDeleting ? <Loader2 className="animate-spin" /> : null}
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

function DialogLoading() {
	return (
		<div className="flex min-h-0 flex-1 items-center justify-center gap-3 text-muted-foreground text-sm">
			<Loader2 className="size-5 animate-spin" />
			Chargement…
		</div>
	)
}

export function CredentialDialog({
	open,
	mode,
	editing,
	kinds,
	services,
	form,
	fieldErrors,
	serverErrors,
	saveError,
	revealedSecret,
	rotatingId,
	isDeleting,
	isConnecting,
	kindsLoading,
	onOpenChange,
	onRotate,
	onDelete,
	onConnect,
	onDisconnect,
}: CredentialDialogProps) {
	const kind = kinds.find((candidate) => candidate.kind === form.values.kind)

	if (revealedSecret !== null) {
		return (
			<Dialog open={open} onOpenChange={onOpenChange}>
				<DialogContent
					showCloseButton={false}
					className="gap-0 p-0 sm:max-w-lg"
				>
					<DialogHeader className="border-border border-b px-6 py-4">
						<DialogTitle>Secret généré</DialogTitle>
						<DialogDescription>
							Copiez-le maintenant : il ne sera plus jamais affiché.
						</DialogDescription>
					</DialogHeader>
					<div className="px-6 py-5">
						<SecretRevealField secret={revealedSecret} />
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
				{kind ? (
					<CredentialForm
						key={`${mode}:${editing?.id ?? 'new'}:${kind.kind}`}
						mode={mode}
						editing={editing}
						kind={kind}
						form={form}
						fieldErrors={fieldErrors}
						serverErrors={serverErrors}
						saveError={saveError}
						rotatingId={rotatingId}
						isDeleting={isDeleting}
						isConnecting={isConnecting}
						onBack={
							mode === 'create'
								? () =>
										withViewTransition(() =>
											form.onChange({ kind: '', data: {}, origin: 'supplied' }),
										)
								: undefined
						}
						onCancel={() => onOpenChange(false)}
						onRotate={onRotate}
						onDelete={onDelete}
						onConnect={onConnect}
						onDisconnect={onDisconnect}
					/>
				) : mode === 'edit' ? (
					<div className="flex min-h-0 flex-1 flex-col">
						<DialogHeader className="border-border border-b px-6 py-4 text-left">
							<DialogTitle className="truncate">
								{editing ? editing.name : 'Identification'}
							</DialogTitle>
							<DialogDescription>
								Modifiez son nom, sa description ou ses paramètres de connexion.
							</DialogDescription>
						</DialogHeader>
						{kindsLoading ? (
							<DialogLoading />
						) : (
							<p className="px-6 py-5 text-muted-foreground text-sm">
								Ce type d’identification n’est plus pris en charge.
							</p>
						)}
						<DialogFooter className="border-border border-t px-6 py-3">
							<Button
								type="button"
								variant="outline"
								onClick={() => onOpenChange(false)}
							>
								Annuler
							</Button>
						</DialogFooter>
					</div>
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
						{kindsLoading ? (
							<DialogLoading />
						) : (
							<div className="min-h-0 flex-1 overflow-y-auto px-6 py-5">
								<CredentialKindPicker
									kinds={kinds}
									services={services}
									onPick={(picked) =>
										withViewTransition(() =>
											form.onChange({
												kind: picked,
												origin: 'supplied',
												data: credentialFormDefaults(
													kinds.find((candidate) => candidate.kind === picked),
												),
											}),
										)
									}
								/>
							</div>
						)}
					</div>
				)}
			</DialogContent>
		</Dialog>
	)
}
