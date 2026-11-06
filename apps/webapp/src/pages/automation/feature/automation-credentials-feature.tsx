import { useEffect, useRef, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationCredentials,
	useCreateCredential,
	useCredentialKinds,
	useDeleteCredential,
	useDisconnectCredentialOauth,
	useRefreshCredentials,
	useRotateCredential,
	useStartCredentialOauth,
	useUpdateCredential,
} from '#/hooks/use-automation'
import { useHasPermission } from '#/hooks/use-permissions'
import { buildOrgPath } from '#/modules/org-path'
import { useExternalServices } from '#/pages/automation/feature/use-external-services'
import { identificationsPath } from '#/pages/automation/lib/automation-paths'
import { credentialFailure } from '#/pages/automation/lib/credential-errors'
import {
	buildCredentialFieldErrors,
	credentialFormFromRow,
	credentialPayload,
	emptyCredentialForm,
} from '#/pages/automation/lib/credential-form'
import { findKind } from '#/pages/automation/lib/credential-kinds'
import {
	type CredentialRow,
	credentialRow,
	credentialRows,
} from '#/pages/automation/lib/credential-rows'
import {
	type OauthNotice,
	type OauthOutcome,
	oauthNotice,
	oauthRedirectUri,
	oauthReturnTo,
} from '#/pages/automation/lib/oauth-return'
import type { CredentialFormValues } from '#/pages/automation/types'
import { AutomationCredentialsUI } from '#/pages/automation/ui/automation-credentials-ui'

export interface AutomationCredentialsFeatureProps {
	oauthOutcome?: OauthOutcome
	onOauthOutcomeShown?: () => void
	navigateTo?: (url: string) => void
}

function leavePage(url: string) {
	window.location.assign(url)
}

export function AutomationCredentialsFeature(
	props: AutomationCredentialsFeatureProps,
) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<AutomationCredentialsWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function AutomationCredentialsWorkspace({
	organizationId,
	organizationSlug,
	oauthOutcome,
	onOauthOutcomeShown,
	navigateTo = leavePage,
}: AutomationCredentialsFeatureProps & {
	organizationId: string
	organizationSlug: string
}) {
	const services = useExternalServices()
	const canManage = useHasPermission('MANAGE_AUTOMATION')
	const kindsQuery = useCredentialKinds(organizationId)
	const credentials = useAutomationCredentials(organizationId)
	const refreshCredentials = useRefreshCredentials()
	const createCredential = useCreateCredential(organizationId)
	const updateCredential = useUpdateCredential()
	const deleteCredential = useDeleteCredential()
	const rotateCredential = useRotateCredential()
	const startOauth = useStartCredentialOauth()
	const disconnectOauth = useDisconnectCredentialOauth()

	const [dialogOpen, setDialogOpen] = useState(false)
	const [mode, setMode] = useState<'create' | 'edit'>('create')
	const [editingRow, setEditingRow] = useState<CredentialRow | null>(null)
	const [values, setValues] = useState<CredentialFormValues>(
		emptyCredentialForm(),
	)
	const [revealedSecret, setRevealedSecret] = useState<string | null>(null)
	const [rotatingId, setRotatingId] = useState<string | null>(null)
	const [notice, setNotice] = useState<OauthNotice | null>(null)

	const shownRef = useRef(onOauthOutcomeShown)
	shownRef.current = onOauthOutcomeShown
	useEffect(() => {
		if (!oauthOutcome) return
		setNotice(oauthNotice(oauthOutcome))
		void refreshCredentials()
		shownRef.current?.()
	}, [oauthOutcome, refreshCredentials])

	const kinds = kindsQuery.data?.data ?? []
	const rows = credentialRows(credentials.data?.data ?? [], kinds)
	const editing = editingRow
		? (rows.find((row) => row.id === editingRow.id) ?? editingRow)
		: null
	const kind = findKind(kinds, values.kind)

	const localErrors = buildCredentialFieldErrors(
		values,
		kind,
		editing?.secretsSet,
	)
	const failure = credentialFailure(
		createCredential.error ??
			updateCredential.error ??
			deleteCredential.error ??
			rotateCredential.error ??
			startOauth.error ??
			disconnectOauth.error,
		kind?.fields,
	)
	const serverErrors = failure?.field
		? { [failure.field]: failure.message }
		: {}
	const listError =
		credentialFailure(credentials.error ?? kindsQuery.error)?.message ?? null

	const resetMutations = () => {
		createCredential.reset()
		updateCredential.reset()
		deleteCredential.reset()
		rotateCredential.reset()
		startOauth.reset()
		disconnectOauth.reset()
	}

	const openEditing = (row: CredentialRow) => {
		resetMutations()
		setMode('edit')
		setEditingRow(row)
		setValues(credentialFormFromRow(row, findKind(kinds, row.kind)))
		setRevealedSecret(null)
		setDialogOpen(true)
	}

	const handleCreate = async () => {
		if (!kind) return
		let created: Awaited<ReturnType<typeof createCredential.mutateAsync>>
		try {
			created = await createCredential.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					kind: kind.kind,
					name: values.name.trim(),
					description: values.description.trim(),
					origin: values.origin,
					data:
						values.origin === 'supplied'
							? credentialPayload(kind, values.data)
							: undefined,
				},
			})
		} catch {
			return
		}
		if (values.origin === 'generated') {
			setRevealedSecret(String(created.data.secret))
		} else if (kind.connectable) {
			openEditing(credentialRow(created.data, kinds))
		} else {
			setDialogOpen(false)
		}
	}

	const handleUpdate = async () => {
		if (!editing || !kind) return
		const data =
			editing.origin === 'supplied'
				? credentialPayload(
						kind,
						values.data,
						credentialFormFromRow(editing, kind).data,
					)
				: {}
		try {
			await updateCredential.mutateAsync({
				path: { organization_id: organizationId, credential_id: editing.id },
				body: {
					name: values.name.trim(),
					description: values.description.trim(),
					data: Object.keys(data).length > 0 ? data : undefined,
				},
			})
		} catch {
			return
		}
		setDialogOpen(false)
	}

	const handleRotate = async (row: CredentialRow) => {
		setRotatingId(row.id)
		try {
			const rotated = await rotateCredential.mutateAsync({
				path: { organization_id: organizationId, credential_id: row.id },
			})
			setMode('edit')
			setEditingRow(row)
			setValues(credentialFormFromRow(row, findKind(kinds, row.kind)))
			setRevealedSecret(String(rotated.data.secret))
			setDialogOpen(true)
		} catch {
			return
		} finally {
			setRotatingId(null)
		}
	}

	const handleDelete = async (row: CredentialRow) => {
		try {
			await deleteCredential.mutateAsync({
				path: { organization_id: organizationId, credential_id: row.id },
			})
		} catch {
			return
		}
		setDialogOpen(false)
	}

	const handleConnect = async (row: CredentialRow) => {
		let started: Awaited<ReturnType<typeof startOauth.mutateAsync>>
		try {
			started = await startOauth.mutateAsync({
				path: { organization_id: organizationId, credential_id: row.id },
				body: {
					redirect_uri: oauthRedirectUri(window.apiUrl, window.location.origin),
					return_to: oauthReturnTo(
						window.apiUrl,
						window.location.origin,
						buildOrgPath(organizationSlug, identificationsPath()),
					),
				},
			})
		} catch {
			return
		}
		navigateTo(started.data.authorization_url)
	}

	const handleDisconnect = async (row: CredentialRow) => {
		try {
			await disconnectOauth.mutateAsync({
				path: { organization_id: organizationId, credential_id: row.id },
			})
		} catch {
			return
		}
	}

	return (
		<AutomationCredentialsUI
			credentials={rows}
			kinds={kinds}
			services={services.data}
			canManage={canManage}
			isLoading={credentials.isLoading || kindsQuery.isLoading}
			kindsLoading={kindsQuery.isLoading}
			error={listError}
			notice={notice}
			onDismissNotice={() => setNotice(null)}
			dialogOpen={dialogOpen}
			dialogMode={mode}
			editing={editing}
			form={{
				values,
				isPending: createCredential.isPending || updateCredential.isPending,
				onChange: (patch) => setValues((current) => ({ ...current, ...patch })),
				onSubmit: () =>
					void (mode === 'create' ? handleCreate() : handleUpdate()),
			}}
			fieldErrors={localErrors}
			serverErrors={serverErrors}
			saveError={failure && !failure.field ? failure.message : null}
			revealedSecret={revealedSecret}
			onOpenCreate={() => {
				resetMutations()
				setMode('create')
				setEditingRow(null)
				setValues(emptyCredentialForm())
				setRevealedSecret(null)
				setDialogOpen(true)
			}}
			onEdit={openEditing}
			onOpenChange={(open) => {
				setDialogOpen(open)
				if (!open) setRevealedSecret(null)
			}}
			onRotate={(row) => void handleRotate(row)}
			rotatingId={rotatingId}
			isDeleting={deleteCredential.isPending}
			isConnecting={startOauth.isPending || disconnectOauth.isPending}
			onDelete={(row) => void handleDelete(row)}
			onConnect={(row) => void handleConnect(row)}
			onDisconnect={(row) => void handleDisconnect(row)}
		/>
	)
}
