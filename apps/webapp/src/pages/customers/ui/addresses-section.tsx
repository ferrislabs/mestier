import { Image, Pencil, Plus, Star, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import type { CustomerContext } from '#/hooks/use-customers'
import type {
	AccountExtras,
	AddressKind,
} from '#/pages/customers/feature/mocks/account-extras'
import { EMPTY_ADDRESS } from '#/pages/customers/lib/account-form'
import { contextCountry, contextKind } from '#/pages/customers/lib/accounts'
import { addressesSummary, TAB_INFO } from '#/pages/customers/lib/rubriques'
import { ADDRESS_KIND_LABELS } from '#/pages/customers/types'
import {
	type AddressDraft,
	AddressFormDialog,
} from '#/pages/customers/ui/address-form-dialog'
import { StatusBadge } from '#/pages/customers/ui/badges'
import { RecordCard } from '#/pages/customers/ui/record-card'

export function addressDraftOf(
	context?: CustomerContext,
	kind: AddressKind = 'site',
	country: string = EMPTY_ADDRESS.country,
): AddressDraft {
	return {
		label: context?.label ?? '',
		kind,
		address: {
			...EMPTY_ADDRESS,
			country,
			street: context?.address_line ?? '',
			postalCode: context?.postal_code ?? '',
			city: context?.city ?? '',
		},
		photoKey: context?.photo_key ?? '',
	}
}

export function formatContextAddress(context: CustomerContext): string {
	const town = [context.postal_code, context.city].filter(Boolean).join(' ')
	return [context.address_line, town].filter(Boolean).join(', ')
}

interface AddressActions {
	isSaving: boolean
	isUploadingPhoto: boolean
	isDeleting: (contextId: string) => boolean
	error: string | null
	onUploadPhoto: (file: File) => Promise<string>
	onSave: (
		context: CustomerContext | null,
		draft: AddressDraft,
	) => Promise<void>
	onMakeBilling: (context: CustomerContext) => void
	onDelete: (context: CustomerContext) => void
}

function AddressCard({
	context,
	kind,
	country,
	isBilling,
	actions,
}: {
	context: CustomerContext
	kind: AddressKind
	country: string
	isBilling: boolean
	actions: AddressActions
}) {
	const [editing, setEditing] = useState(false)
	const [deleting, setDeleting] = useState(false)
	const formatted = formatContextAddress(context)
	return (
		<RecordCard
			title={context.label}
			subtitle={ADDRESS_KIND_LABELS[kind]}
			badges={
				<>
					{isBilling && (
						<StatusBadge tone="info">
							Adresse de facturation par défaut
						</StatusBadge>
					)}
					{context.photo_key && (
						<StatusBadge tone="neutral">
							<Image className="size-3" aria-hidden />
							Photo
						</StatusBadge>
					)}
				</>
			}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<AddressFormDialog
						open={editing}
						onOpenChange={setEditing}
						title={`Modifier ${context.label}`}
						initial={addressDraftOf(context, kind, country)}
						isEditing
						isPending={actions.isSaving}
						isUploadingPhoto={actions.isUploadingPhoto}
						error={actions.error}
						onUploadPhoto={actions.onUploadPhoto}
						onSubmit={async (draft) => {
							await actions.onSave(context, draft)
							setEditing(false)
						}}
						trigger={
							<Button variant="ghost" size="sm">
								<Pencil aria-hidden /> Modifier
								<span className="sr-only"> {context.label}</span>
							</Button>
						}
					/>
					{!isBilling && (
						<Button
							variant="ghost"
							size="sm"
							onClick={() => actions.onMakeBilling(context)}
						>
							<Star aria-hidden /> Définir comme adresse de facturation
							<span className="sr-only"> : {context.label}</span>
						</Button>
					)}
					<Button
						variant="ghost"
						size="sm"
						className="text-state-alert hover:text-state-alert"
						disabled={actions.isDeleting(context.id)}
						onClick={() => setDeleting(true)}
					>
						<Trash2 aria-hidden /> Supprimer
						<span className="sr-only"> {context.label}</span>
					</Button>
					<ConfirmDialog
						open={deleting}
						onOpenChange={setDeleting}
						title={`Supprimer ${context.label} ?`}
						description="Les devis et factures déjà émis conservent l’adresse figée à l’émission."
						confirmLabel="Supprimer"
						destructive
						onConfirm={() => {
							actions.onDelete(context)
							setDeleting(false)
						}}
					/>
				</RequirePermission>
			}
		>
			<address className="grid gap-0.5 text-[13px] not-italic">
				{formatted ? (
					<span>{formatted}</span>
				) : (
					<span className="text-muted-foreground">
						Aucune adresse renseignée
					</span>
				)}
				{formatted && country && (
					<span className="text-muted-foreground">{country}</span>
				)}
			</address>
		</RecordCard>
	)
}

export function AddressesSection({
	contexts,
	extras,
	billingContextId,
	isLoading,
	loadError,
	onRetry,
	actions,
}: {
	contexts: readonly CustomerContext[]
	extras: Pick<AccountExtras, 'addressKinds' | 'addressCountries'>
	billingContextId: string | undefined
	isLoading: boolean
	loadError: string | null
	onRetry: () => void
	actions: AddressActions
}) {
	const [adding, setAdding] = useState(false)
	const count = contexts.length
	return (
		<RubriqueLayout
			title={TAB_INFO.adresses.label}
			description={`${addressesSummary(count)}. L’adresse de facturation par défaut est reprise sur les devis et les factures.`}
			actions={
				<RequirePermission permission="MANAGE_CUSTOMERS">
					<AddressFormDialog
						open={adding}
						onOpenChange={setAdding}
						title="Ajouter une adresse"
						initial={addressDraftOf()}
						isEditing={false}
						isPending={actions.isSaving}
						isUploadingPhoto={actions.isUploadingPhoto}
						error={actions.error}
						onUploadPhoto={actions.onUploadPhoto}
						onSubmit={async (draft) => {
							await actions.onSave(null, draft)
							setAdding(false)
						}}
						trigger={
							<Button size="sm">
								<Plus aria-hidden /> Ajouter une adresse
							</Button>
						}
					/>
				</RequirePermission>
			}
		>
			{loadError ? (
				<div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] text-state-alert">
					<p className="font-medium">{loadError}</p>
					<Button type="button" variant="outline" size="sm" onClick={onRetry}>
						Réessayer
					</Button>
				</div>
			) : null}
			{isLoading ? (
				<EmptyState title="Chargement des adresses…" />
			) : count === 0 ? (
				<EmptyState
					title="Aucune adresse"
					hint="Ajoute l’adresse du client pour qu’elle figure sur les devis et factures."
				/>
			) : (
				<ul className="grid gap-3">
					{contexts.map((context) => (
						<li key={context.id}>
							<AddressCard
								context={context}
								kind={contextKind(context.id, extras)}
								country={contextCountry(context.id, extras)}
								isBilling={context.id === billingContextId}
								actions={actions}
							/>
						</li>
					))}
				</ul>
			)}
		</RubriqueLayout>
	)
}
