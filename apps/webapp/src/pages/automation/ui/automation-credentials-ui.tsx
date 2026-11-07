import { CircleCheck, Loader2, Plus, TriangleAlert, X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { CredentialKind } from '#/hooks/use-automation'
import type { ExternalService } from '#/pages/automation/feature/mocks/external-services'
import type { CredentialFieldErrors } from '#/pages/automation/lib/credential-form'
import type { CredentialRow } from '#/pages/automation/lib/credential-rows'
import type { OauthNotice } from '#/pages/automation/lib/oauth-return'
import {
	CredentialDialog,
	type CredentialFormBinding,
} from '#/pages/automation/ui/credential-dialog'
import { CredentialsTable } from '#/pages/automation/ui/credentials-table'

export interface AutomationCredentialsUIProps {
	credentials: CredentialRow[]
	kinds: CredentialKind[]
	services: readonly ExternalService[]
	canManage: boolean
	isLoading: boolean
	kindsLoading: boolean
	error: string | null
	notice: OauthNotice | null
	onDismissNotice: () => void

	dialogOpen: boolean
	dialogMode: 'create' | 'edit'
	editing: CredentialRow | null
	form: CredentialFormBinding
	fieldErrors: CredentialFieldErrors
	serverErrors: CredentialFieldErrors
	saveError: string | null
	revealedSecret: string | null
	onOpenCreate: () => void
	onEdit: (row: CredentialRow) => void
	onOpenChange: (open: boolean) => void

	onRotate: (row: CredentialRow) => void
	rotatingId: string | null
	isDeleting: boolean
	isConnecting: boolean
	onDelete: (row: CredentialRow) => void
	onConnect: (row: CredentialRow) => void
	onDisconnect: (row: CredentialRow) => void
}

function OauthNoticeBanner({
	notice,
	onDismiss,
}: {
	notice: OauthNotice
	onDismiss: () => void
}) {
	const success = notice.tone === 'success'
	const Icon = success ? CircleCheck : TriangleAlert
	return (
		<div
			role={success ? 'status' : 'alert'}
			className={
				success
					? 'mb-4 flex items-start gap-2 rounded-lg border border-border bg-card px-4 py-3 text-sm text-(color:--auto-success-text)'
					: 'mb-4 flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive'
			}
		>
			<Icon className="mt-0.5 size-4 shrink-0" aria-hidden />
			<p className="flex-1">{notice.text}</p>
			<button
				type="button"
				onClick={onDismiss}
				aria-label="Fermer le message"
				className="rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
			>
				<X className="size-4" aria-hidden />
			</button>
		</div>
	)
}

export function AutomationCredentialsUI({
	credentials,
	kinds,
	services,
	canManage,
	isLoading,
	kindsLoading,
	error,
	notice,
	onDismissNotice,
	dialogOpen,
	dialogMode,
	editing,
	form,
	fieldErrors,
	serverErrors,
	saveError,
	revealedSecret,
	onOpenCreate,
	onEdit,
	onOpenChange,
	onRotate,
	rotatingId,
	isDeleting,
	isConnecting,
	onDelete,
	onConnect,
	onDisconnect,
}: AutomationCredentialsUIProps) {
	return (
		<PageShell className="gap-0">
			<PageHeader
				className="mb-6"
				title="Identifications"
				description="Les accès utilisés par vos workflows pour parler à d’autres systèmes. Les secrets ne sont jamais réaffichés."
				actions={
					<RequirePermission permission="MANAGE_AUTOMATION">
						<Button onClick={onOpenCreate}>
							<Plus />
							Nouvelle identification
						</Button>
					</RequirePermission>
				}
			/>
			{notice ? (
				<OauthNoticeBanner notice={notice} onDismiss={onDismissNotice} />
			) : null}
			{error ? (
				<div className="mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}
			{isLoading ? (
				<div className="flex min-h-40 items-center justify-center gap-3 rounded-lg border border-border bg-card p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement…
				</div>
			) : (
				<CredentialsTable
					credentials={credentials}
					onOpen={canManage ? onEdit : null}
				/>
			)}
			<CredentialDialog
				open={dialogOpen}
				mode={dialogMode}
				editing={editing}
				kinds={kinds}
				services={services}
				form={form}
				fieldErrors={fieldErrors}
				serverErrors={serverErrors}
				saveError={saveError}
				revealedSecret={revealedSecret}
				rotatingId={rotatingId}
				isDeleting={isDeleting}
				isConnecting={isConnecting}
				kindsLoading={kindsLoading}
				onOpenChange={onOpenChange}
				onRotate={onRotate}
				onDelete={onDelete}
				onConnect={onConnect}
				onDisconnect={onDisconnect}
			/>
		</PageShell>
	)
}
