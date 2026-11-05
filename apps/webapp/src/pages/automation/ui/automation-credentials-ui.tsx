import { Loader2, Plus } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { AuthScheme } from '#/hooks/use-automation'
import type { ExternalService } from '#/pages/automation/feature/mocks/external-services'
import {
	CredentialDialog,
	type CredentialFormBinding,
} from '#/pages/automation/ui/credential-dialog'
import {
	type CredentialRow,
	CredentialsTable,
} from '#/pages/automation/ui/credentials-table'

export type { CredentialRow } from '#/pages/automation/ui/credentials-table'

export interface AutomationCredentialsUIProps {
	credentials: CredentialRow[]
	authSchemes: AuthScheme[]
	services: readonly ExternalService[]
	canManage: boolean
	isLoading: boolean
	error: string | null

	dialogOpen: boolean
	dialogMode: 'create' | 'edit'
	editing: CredentialRow | null
	form: CredentialFormBinding
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
	services,
	canManage,
	isLoading,
	error,
	dialogOpen,
	dialogMode,
	editing,
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
				authSchemes={authSchemes}
				services={services}
				form={form}
				formErrors={formErrors}
				saveError={saveError}
				revealedSecret={revealedSecret}
				rotatingId={rotatingId}
				onOpenChange={onOpenChange}
				onRotate={onRotate}
				onDelete={onDelete}
			/>
		</PageShell>
	)
}
