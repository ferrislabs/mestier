import { ChevronDown, X } from 'lucide-react'
import { type MouseEvent as ReactMouseEvent, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { withField } from '#/pages/automation/lib/connector-config'
import {
	type ConnectorValidationError,
	connectorLevelErrors,
} from '#/pages/automation/lib/validation'
import {
	ConnectorConfigForm,
	type CredentialCreationPurpose,
} from '#/pages/automation/ui/connector-config-form'
import { CredentialSecretReveal } from '#/pages/automation/ui/credential-secret-reveal'
import { InlineCredentialForm } from '#/pages/automation/ui/inline-credential-form'

export const MIN_PANEL_WIDTH = 360
export const MAX_PANEL_WIDTH = 720
const DEFAULT_PANEL_WIDTH = 440

function clampWidth(width: number): number {
	return Math.min(MAX_PANEL_WIDTH, Math.max(MIN_PANEL_WIDTH, width))
}

type CreatedCredential = Schemas.CredentialResponse & { secret: unknown }

interface CreateState {
	purpose: CredentialCreationPurpose
	result: CreatedCredential | null
}

export interface ConnectorConfigPanelProps {
	label: string
	descriptor: Schemas.ConnectorDescriptorResponse
	config: Record<string, unknown>
	credentialId: string | null
	credentials: Schemas.CredentialResponse[]
	authSchemes: Schemas.AuthSchemeResponse[]
	errors: ConnectorValidationError[]
	onClose: () => void
	onConfigChange: (config: Record<string, unknown>) => void
	onCredentialChange: (credentialId: string | null) => void
	onOpenExpression: (field: Schemas.FieldResponse) => void
	onCreateCredential: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<CreatedCredential>
}

export function ConnectorConfigPanel({
	label,
	descriptor,
	config,
	credentialId,
	credentials,
	authSchemes,
	errors,
	onClose,
	onConfigChange,
	onCredentialChange,
	onOpenExpression,
	onCreateCredential,
}: ConnectorConfigPanelProps) {
	const [width, setWidth] = useState(DEFAULT_PANEL_WIDTH)
	const [createState, setCreateState] = useState<CreateState | null>(null)
	const [createPending, setCreatePending] = useState(false)
	const [createError, setCreateError] = useState<string | null>(null)

	function handleResizeStart(event: ReactMouseEvent<HTMLButtonElement>) {
		const startX = event.clientX
		const startWidth = width

		function onMove(moveEvent: globalThis.MouseEvent) {
			setWidth(clampWidth(startWidth + (startX - moveEvent.clientX)))
		}
		function onUp() {
			window.removeEventListener('mousemove', onMove)
			window.removeEventListener('mouseup', onUp)
		}
		window.addEventListener('mousemove', onMove)
		window.addEventListener('mouseup', onUp)
	}

	function applySelection(purpose: CredentialCreationPurpose, id: string) {
		if (purpose === 'typed') {
			onCredentialChange(id)
			return
		}
		onConfigChange(withField(config, 'signing_credential_id', id))
	}

	async function handleCreateSubmit(body: Schemas.CreateCredentialRequest) {
		if (!createState) return
		setCreatePending(true)
		setCreateError(null)
		try {
			const created = await onCreateCredential(body)
			if (created.origin === 'generated') {
				setCreateState({ purpose: createState.purpose, result: created })
			} else {
				applySelection(createState.purpose, created.id)
				setCreateState(null)
			}
		} catch (error) {
			setCreateError(
				error instanceof Error ? error.message : 'La création a échoué.',
			)
		} finally {
			setCreatePending(false)
		}
	}

	function handleCreateDone() {
		if (createState?.result) {
			applySelection(createState.purpose, createState.result.id)
		}
		setCreateState(null)
	}

	function handleCreateCancel() {
		setCreateState(null)
		setCreateError(null)
	}

	const levelErrors = connectorLevelErrors(errors)

	return (
		<div
			data-testid="connector-config-panel"
			style={{ width }}
			className="flex min-h-0 flex-col border-l bg-card"
		>
			<div className="flex items-stretch">
				<button
					type="button"
					aria-label="Redimensionner le panneau"
					onMouseDown={handleResizeStart}
					className="w-1.5 shrink-0 cursor-col-resize bg-transparent hover:bg-accent"
				/>
				<div className="flex min-h-0 min-w-0 flex-1 flex-col">
					<div className="flex items-center justify-between gap-2 border-b px-4 py-3">
						<span className="truncate font-medium">{label}</span>
						<Button
							variant="ghost"
							size="icon-sm"
							aria-label="Fermer"
							onClick={onClose}
						>
							<X className="size-4" />
						</Button>
					</div>

					{levelErrors.length > 0 ? (
						<div
							role="alert"
							className="border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive"
						>
							<ul>
								{levelErrors.map((message) => (
									<li key={message}>{message}</li>
								))}
							</ul>
						</div>
					) : null}

					<div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4">
						<Collapsible>
							<CollapsibleTrigger className="flex w-full items-center justify-between text-sm font-medium text-muted-foreground">
								Données disponibles
								<ChevronDown className="size-4" />
							</CollapsibleTrigger>
							<CollapsibleContent className="pt-2 text-sm text-muted-foreground">
								Bientôt disponible.
							</CollapsibleContent>
						</Collapsible>

						<div className="flex flex-col gap-3 border-t pt-4">
							<p className="text-sm font-medium text-muted-foreground">
								Paramètres
							</p>
							{createState ? (
								createState.result ? (
									<CredentialSecretReveal
										secret={String(createState.result.secret)}
										onDone={handleCreateDone}
									/>
								) : (
									<InlineCredentialForm
										authSchemes={authSchemes}
										lockOrigin={
											createState.purpose === 'signing'
												? 'generated'
												: undefined
										}
										isPending={createPending}
										error={createError}
										onSubmit={(body) => void handleCreateSubmit(body)}
										onCancel={handleCreateCancel}
									/>
								)
							) : (
								<ConnectorConfigForm
									descriptor={descriptor}
									config={config}
									credentialId={credentialId}
									credentials={credentials}
									errors={errors}
									onConfigChange={onConfigChange}
									onCredentialChange={onCredentialChange}
									onOpenExpression={onOpenExpression}
									onRequestCreateCredential={(purpose) => {
										setCreateError(null)
										setCreateState({ purpose, result: null })
									}}
								/>
							)}
						</div>

						<div className="flex flex-col gap-2 border-t pt-4 text-sm text-muted-foreground">
							<p className="font-medium text-foreground">Dernière sortie</p>
							<p>Aucune exécution.</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	)
}
