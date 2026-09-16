import { ChevronDown, X } from 'lucide-react'
import {
	type MouseEvent as ReactMouseEvent,
	useEffect,
	useRef,
	useState,
} from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { withField } from '#/pages/automation/lib/connector-config'
import type { DataTreeNode } from '#/pages/automation/lib/data-tree'
import { insertExpressionAtCursor } from '#/pages/automation/lib/expression'
import {
	type ConnectorValidationError,
	connectorLevelErrors,
} from '#/pages/automation/lib/validation'
import {
	AvailableDataTree,
	type DataTreeSource,
} from '#/pages/automation/ui/available-data-tree'
import {
	ConnectorConfigForm,
	type CredentialCreationPurpose,
} from '#/pages/automation/ui/connector-config-form'
import { CredentialSecretReveal } from '#/pages/automation/ui/credential-secret-reveal'
import {
	ExpressionPreview,
	type ExpressionPreviewState,
} from '#/pages/automation/ui/expression-preview'
import { InlineCredentialForm } from '#/pages/automation/ui/inline-credential-form'

const PREVIEW_DEBOUNCE_MS = 400

type FieldControlElement = HTMLInputElement | HTMLTextAreaElement

export interface DataTreeSourceBundle {
	tree: DataTreeNode[]
	context: Schemas.EvaluateContextBody
}

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
	connectorId: string
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
	onCreateCredential: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<CreatedCredential>
	exampleData: DataTreeSourceBundle
	lastRunData: DataTreeSourceBundle | null
	onEvaluateExpression: (
		template: unknown,
		context: Schemas.EvaluateContextBody,
	) => Promise<unknown>
}

export function ConnectorConfigPanel({
	connectorId,
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
	onCreateCredential,
	exampleData,
	lastRunData,
	onEvaluateExpression,
}: ConnectorConfigPanelProps) {
	const [width, setWidth] = useState(DEFAULT_PANEL_WIDTH)
	const [createState, setCreateState] = useState<CreateState | null>(null)
	const [createPending, setCreatePending] = useState(false)
	const [createError, setCreateError] = useState<string | null>(null)
	const [dataSource, setDataSource] = useState<DataTreeSource>('example')
	const [activeField, setActiveField] = useState<Schemas.FieldResponse | null>(
		null,
	)
	const [previewState, setPreviewState] = useState<ExpressionPreviewState>({
		status: 'idle',
	})
	const fieldRefs = useRef(new Map<string, FieldControlElement>())
	const [openedConnectorId, setOpenedConnectorId] = useState(connectorId)

	if (connectorId !== openedConnectorId) {
		setOpenedConnectorId(connectorId)
		setActiveField(null)
		fieldRefs.current.clear()
	}

	const activeSource =
		dataSource === 'last_run' && lastRunData ? lastRunData : exampleData

	useEffect(() => {
		if (!activeField) {
			setPreviewState({ status: 'idle' })
			return
		}

		const template = config[activeField.name]
		if (
			template === undefined ||
			(typeof template === 'string' && template.trim() === '')
		) {
			setPreviewState({ status: 'idle' })
			return
		}

		let cancelled = false
		setPreviewState({ status: 'loading' })
		const timer = setTimeout(() => {
			onEvaluateExpression(template, activeSource.context)
				.then((value) => {
					if (!cancelled) setPreviewState({ status: 'resolved', value })
				})
				.catch((error: unknown) => {
					if (!cancelled) {
						setPreviewState({
							status: 'error',
							message:
								error instanceof Error
									? error.message
									: "L'évaluation a échoué.",
						})
					}
				})
		}, PREVIEW_DEBOUNCE_MS)

		return () => {
			cancelled = true
			clearTimeout(timer)
		}
	}, [activeField, config, activeSource, onEvaluateExpression])

	function insertAt(fieldName: string, path: string) {
		const currentValue = config[fieldName]
		const text = typeof currentValue === 'string' ? currentValue : ''
		const element = fieldRefs.current.get(fieldName)
		const selection = {
			start: element?.selectionStart ?? text.length,
			end: element?.selectionEnd ?? text.length,
		}

		const result = insertExpressionAtCursor(text, path, selection)
		onConfigChange(withField(config, fieldName, result.text))

		if (element) {
			element.focus()
			element.setSelectionRange(result.cursor, result.cursor)
		}
	}

	function handleTreeInsert(path: string) {
		if (!activeField) return
		insertAt(activeField.name, path)
	}

	function handleFieldDrop(fieldName: string, path: string) {
		const field = descriptor.fields.find((f) => f.name === fieldName)
		if (field) setActiveField(field)
		insertAt(fieldName, path)
	}

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
			className="flex min-h-0 flex-col overflow-hidden border-l bg-card"
		>
			<div className="flex min-h-0 flex-1 items-stretch">
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

					<div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto overscroll-contain p-4">
						<Collapsible defaultOpen>
							<CollapsibleTrigger className="flex w-full items-center justify-between text-sm font-medium text-muted-foreground">
								Données disponibles
								<ChevronDown className="size-4" />
							</CollapsibleTrigger>
							<CollapsibleContent className="flex flex-col gap-2 pt-2">
								<AvailableDataTree
									branches={activeSource.tree}
									source={dataSource}
									hasLastRun={lastRunData !== null}
									onSourceChange={setDataSource}
									onInsert={handleTreeInsert}
								/>
								<ExpressionPreview
									fieldLabel={activeField?.label ?? null}
									state={previewState}
								/>
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
									onOpenExpression={setActiveField}
									onFieldRef={(name, element) => {
										if (element) fieldRefs.current.set(name, element)
										else fieldRefs.current.delete(name)
									}}
									onInsertExpression={handleFieldDrop}
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
