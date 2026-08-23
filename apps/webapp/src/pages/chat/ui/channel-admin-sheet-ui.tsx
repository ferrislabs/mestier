import { Copy, Loader2, Trash2 } from 'lucide-react'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
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
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type { Category, Channel, Overwrite, Webhook } from '#/hooks/use-chat'
import {
	type OverwriteBitName,
	readBitState,
	type TriState,
} from '#/pages/chat/lib/permission-bits'

const NO_CATEGORY_VALUE = '__none__'

export interface ChannelAdminSheetUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void

	channel: Channel | undefined
	categories: Category[]
	nameDraft: string
	topicDraft: string
	categoryIdDraft: string | null
	onChangeName: (value: string) => void
	onChangeTopic: (value: string) => void
	onChangeCategory: (value: string | null) => void
	onSaveGeneral: () => void
	isSavingGeneral: boolean

	deleteDialogOpen: boolean
	onRequestDelete: () => void
	onCancelDelete: () => void
	onConfirmDelete: () => void
	isDeleting: boolean

	isLoadingOverwrites: boolean
	everyoneOverwrite: Overwrite | undefined
	roleAndMemberOverwrites: Overwrite[]
	onChangeEveryoneBit: (bit: OverwriteBitName, state: TriState) => void
	onDeleteTargetOverwrite: (targetType: string, targetId: string) => void

	webhooks: Webhook[]
	isLoadingWebhooks: boolean
	newWebhookName: string
	onChangeNewWebhookName: (value: string) => void
	onCreateWebhook: () => void
	isCreatingWebhook: boolean
	createdWebhookToken: string | null
	onDismissCreatedToken: () => void
	onDeleteWebhook: (id: string) => void
}

export function ChannelAdminSheetUI({
	open,
	onOpenChange,
	channel,
	categories,
	nameDraft,
	topicDraft,
	categoryIdDraft,
	onChangeName,
	onChangeTopic,
	onChangeCategory,
	onSaveGeneral,
	isSavingGeneral,
	deleteDialogOpen,
	onRequestDelete,
	onCancelDelete,
	onConfirmDelete,
	isDeleting,
	isLoadingOverwrites,
	everyoneOverwrite,
	roleAndMemberOverwrites,
	onChangeEveryoneBit,
	onDeleteTargetOverwrite,
	webhooks,
	isLoadingWebhooks,
	newWebhookName,
	onChangeNewWebhookName,
	onCreateWebhook,
	isCreatingWebhook,
	createdWebhookToken,
	onDismissCreatedToken,
	onDeleteWebhook,
}: ChannelAdminSheetUIProps) {
	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full sm:max-w-md">
				<SheetHeader>
					<SheetTitle>Paramètres du canal</SheetTitle>
					<SheetDescription>
						{channel ? `#${channel.name}` : ''}
					</SheetDescription>
				</SheetHeader>

				<Tabs defaultValue="general" className="px-4">
					<TabsList>
						<TabsTrigger value="general">Général</TabsTrigger>
						<TabsTrigger value="permissions">Permissions</TabsTrigger>
						<TabsTrigger value="webhooks">Webhooks</TabsTrigger>
					</TabsList>

					<TabsContent value="general" className="flex flex-col gap-4 py-4">
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="channel-name">Nom</Label>
							<Input
								id="channel-name"
								value={nameDraft}
								onChange={(event) => onChangeName(event.target.value)}
							/>
						</div>
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="channel-topic">Sujet</Label>
							<Input
								id="channel-topic"
								value={topicDraft}
								onChange={(event) => onChangeTopic(event.target.value)}
							/>
						</div>
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="channel-category">Catégorie</Label>
							<Select
								value={categoryIdDraft ?? NO_CATEGORY_VALUE}
								onValueChange={(value) =>
									onChangeCategory(value === NO_CATEGORY_VALUE ? null : value)
								}
							>
								<SelectTrigger id="channel-category">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={NO_CATEGORY_VALUE}>
										Sans catégorie
									</SelectItem>
									{categories.map((category) => (
										<SelectItem key={category.id} value={category.id}>
											{category.name}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>

						<Button
							type="button"
							onClick={onSaveGeneral}
							disabled={isSavingGeneral}
						>
							{isSavingGeneral ? (
								<Loader2 className="size-4 animate-spin" />
							) : null}
							Enregistrer
						</Button>

						<div className="mt-4 border-t pt-4">
							<Button
								type="button"
								variant="destructive"
								onClick={onRequestDelete}
							>
								<Trash2 className="size-4" />
								Supprimer le canal
							</Button>
						</div>

						<AlertDialog
							open={deleteDialogOpen}
							onOpenChange={(next) => {
								if (!next) onCancelDelete()
							}}
						>
							<AlertDialogContent>
								<AlertDialogHeader>
									<AlertDialogTitle>Supprimer le canal ?</AlertDialogTitle>
									<AlertDialogDescription>
										Cette action supprime définitivement le canal
										{channel ? ` #${channel.name}` : ''}, tous ses messages,
										réactions et pièces jointes. Elle est irréversible.
									</AlertDialogDescription>
								</AlertDialogHeader>
								<AlertDialogFooter>
									<AlertDialogCancel onClick={onCancelDelete}>
										Annuler
									</AlertDialogCancel>
									<AlertDialogAction
										onClick={onConfirmDelete}
										disabled={isDeleting}
									>
										Supprimer
									</AlertDialogAction>
								</AlertDialogFooter>
							</AlertDialogContent>
						</AlertDialog>
					</TabsContent>

					<TabsContent value="permissions" className="flex flex-col gap-4 py-4">
						<div>
							<h3 className="text-sm font-semibold text-foreground">
								Tout le monde
							</h3>
							<p className="mt-1 text-xs text-muted-foreground">
								Permission par défaut pour ce canal.
							</p>
							<div className="mt-3 flex flex-col gap-3">
								<BitRow
									label="Voir le canal"
									state={bitStateOf(everyoneOverwrite, 'VIEW_CHANNEL')}
									onChange={(state) =>
										onChangeEveryoneBit('VIEW_CHANNEL', state)
									}
								/>
								<BitRow
									label="Envoyer des messages"
									state={bitStateOf(everyoneOverwrite, 'SEND_MESSAGES')}
									onChange={(state) =>
										onChangeEveryoneBit('SEND_MESSAGES', state)
									}
								/>
							</div>
						</div>

						<div className="border-t pt-4">
							<h3 className="text-sm font-semibold text-foreground">
								Autres remplacements
							</h3>
							{/* No REST endpoint lists organization roles, and the member
							    directory never carries a `user_id` (see
							    `libs/handlers-member/src/response.rs`) — a role or member
							    can't be picked to create a new overwrite from here.
							    Existing ones (created elsewhere) still show and can be
							    removed. */}
							<p className="mt-1 text-xs text-muted-foreground">
								Ajouter un remplacement par rôle ou par membre n’est pas
								possible depuis cet écran : l’API ne fournit pas la liste des
								rôles de l’organisation. Les remplacements déjà créés restent
								visibles et peuvent être retirés.
							</p>
							{isLoadingOverwrites ? (
								<Loader2 className="mt-3 size-4 animate-spin text-muted-foreground" />
							) : null}
							{!isLoadingOverwrites && roleAndMemberOverwrites.length === 0 ? (
								<p className="mt-3 text-xs text-muted-foreground">
									Aucun remplacement par rôle ou par membre.
								</p>
							) : null}
							<ul className="mt-3 flex flex-col gap-2">
								{roleAndMemberOverwrites.map((overwrite) => (
									<li
										key={`${overwrite.target_type}-${overwrite.target_id}`}
										className="flex items-center justify-between rounded-md border px-2 py-1.5 text-xs"
									>
										<span>
											{overwrite.target_type === 'role' ? 'Rôle' : 'Membre'}{' '}
											{overwrite.target_id}
										</span>
										<button
											type="button"
											aria-label="Retirer ce remplacement"
											onClick={() =>
												overwrite.target_id &&
												onDeleteTargetOverwrite(
													overwrite.target_type,
													overwrite.target_id,
												)
											}
										>
											<Trash2 className="size-3.5" />
										</button>
									</li>
								))}
							</ul>
						</div>
					</TabsContent>

					<TabsContent value="webhooks" className="flex flex-col gap-4 py-4">
						{createdWebhookToken ? (
							<div className="flex flex-col gap-2 rounded-md border border-warning bg-warning-soft p-3 text-xs">
								<p className="font-semibold text-warning">
									Copiez ce jeton maintenant — il ne sera plus jamais affiché.
								</p>
								<div className="flex items-center gap-2">
									<code className="flex-1 truncate rounded bg-background px-2 py-1">
										{createdWebhookToken}
									</code>
									<button
										type="button"
										aria-label="Copier le jeton"
										onClick={() =>
											navigator.clipboard?.writeText(createdWebhookToken)
										}
									>
										<Copy className="size-3.5" />
									</button>
								</div>
								<button
									type="button"
									className="self-end underline"
									onClick={onDismissCreatedToken}
								>
									J’ai copié le jeton
								</button>
							</div>
						) : null}

						<div className="flex items-end gap-2">
							<div className="flex flex-1 flex-col gap-1.5">
								<Label htmlFor="new-webhook-name">Nouveau webhook</Label>
								<Input
									id="new-webhook-name"
									placeholder="Nom du webhook"
									value={newWebhookName}
									onChange={(event) =>
										onChangeNewWebhookName(event.target.value)
									}
								/>
							</div>
							<Button
								type="button"
								onClick={onCreateWebhook}
								disabled={
									isCreatingWebhook || newWebhookName.trim().length === 0
								}
							>
								Créer
							</Button>
						</div>

						{isLoadingWebhooks ? (
							<Loader2 className="size-4 animate-spin text-muted-foreground" />
						) : null}
						{!isLoadingWebhooks && webhooks.length === 0 ? (
							<p className="text-xs text-muted-foreground">
								Aucun webhook pour ce canal.
							</p>
						) : null}
						<ul className="flex flex-col gap-2">
							{webhooks.map((webhook) => (
								<li
									key={webhook.id}
									className="flex items-center justify-between rounded-md border px-2 py-1.5 text-sm"
								>
									<span>{webhook.name}</span>
									<button
										type="button"
										aria-label={`Révoquer ${webhook.name}`}
										onClick={() => onDeleteWebhook(webhook.id)}
									>
										<Trash2 className="size-3.5" />
									</button>
								</li>
							))}
						</ul>
					</TabsContent>
				</Tabs>
			</SheetContent>
		</Sheet>
	)
}

function bitStateOf(
	overwrite: Overwrite | undefined,
	bit: OverwriteBitName,
): TriState {
	if (!overwrite) return 'inherit'
	return readBitState(overwrite.allow, overwrite.deny, bit)
}

interface BitRowProps {
	label: string
	state: TriState
	onChange: (state: TriState) => void
}

function BitRow({ label, state, onChange }: BitRowProps) {
	return (
		<div className="flex items-center justify-between text-sm">
			<span>{label}</span>
			<div className="flex overflow-hidden rounded-md border text-xs">
				{(['inherit', 'allow', 'deny'] as const).map((option) => (
					<button
						key={option}
						type="button"
						onClick={() => onChange(option)}
						aria-pressed={state === option}
						className={
							state === option
								? 'bg-primary px-2 py-1 font-medium text-primary-foreground'
								: 'px-2 py-1 text-muted-foreground hover:bg-accent'
						}
					>
						{option === 'inherit'
							? 'Hérité'
							: option === 'allow'
								? 'Autorisé'
								: 'Refusé'}
					</button>
				))}
			</div>
		</div>
	)
}
