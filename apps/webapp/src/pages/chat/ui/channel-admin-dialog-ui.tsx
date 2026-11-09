import { Globe, Loader2, Lock, X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { DoubleClickConfirmButton } from '#/components/double-click-confirm-button'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
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
import { Textarea } from '#/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type { Category, Channel, Webhook } from '#/hooks/use-chat'
import { ChannelPermissionsSectionUI } from '#/pages/chat/ui/channel-permissions-section-ui'
import { ChannelWebhooksSectionUI } from '#/pages/chat/ui/channel-webhooks-section-ui'
import { NEW_ITEM_KINDS } from '#/pages/chat/lib/channel-kinds'
import type {
	OverwriteBitName,
	TriState,
} from '#/pages/chat/lib/permission-bits'
import type { PermissionTarget } from '#/pages/chat/lib/permission-targets'

const NO_CATEGORY_VALUE = '__none__'

export interface ChannelAdminDialogUIProps {
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

	onConfirmDelete: () => void
	isDeleting: boolean

	isLoadingOverwrites: boolean
	isPrivate: boolean
	onTogglePrivate: (next: boolean) => void
	permissionTargets: PermissionTarget[]
	selectedPermissionKey: string
	onSelectPermissionKey: (key: string) => void
	addableRoles: { id: string; name: string }[]
	addableMembers: { id: string; name: string }[]
	onAddTarget: (type: 'role' | 'member', id: string) => void
	onChangeTargetBit: (
		key: string,
		bit: OverwriteBitName,
		state: TriState,
	) => void
	onRemoveTarget: (key: string) => void

	webhooks: Webhook[]
	isLoadingWebhooks: boolean
	newWebhookName: string
	onChangeNewWebhookName: (value: string) => void
	onCreateWebhook: () => void
	isCreatingWebhook: boolean
	createdWebhookToken: string | null
	onDismissCreatedToken: () => void
	onDeleteWebhook: (id: string) => void
	onRenameWebhook: (id: string, name: string) => void
	webhookCallUrlBase: string
	createdWebhookCallUrl: string | null
	creatorName: (userId: string) => string | undefined
}

export function ChannelAdminDialogUI({
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
	onConfirmDelete,
	isDeleting,
	isLoadingOverwrites,
	isPrivate,
	onTogglePrivate,
	permissionTargets,
	selectedPermissionKey,
	onSelectPermissionKey,
	addableRoles,
	addableMembers,
	onAddTarget,
	onChangeTargetBit,
	onRemoveTarget,
	webhooks,
	isLoadingWebhooks,
	newWebhookName,
	onChangeNewWebhookName,
	onCreateWebhook,
	isCreatingWebhook,
	createdWebhookToken,
	onDismissCreatedToken,
	onDeleteWebhook,
	onRenameWebhook,
	webhookCallUrlBase,
	createdWebhookCallUrl,
	creatorName,
}: ChannelAdminDialogUIProps) {
	const kind = NEW_ITEM_KINDS.find((k) => k.value === channel?.channel_type)
	const createdOn = channel?.created_at
		? new Date(channel.created_at).toLocaleDateString('fr-FR', {
				dateStyle: 'long',
			})
		: null
	const TRIGGER =
		'h-auto flex-none justify-start rounded-md px-2.5 py-1.5 text-left text-nav-foreground/80 hover:bg-nav-active/60 hover:text-nav-foreground data-[state=active]:bg-nav-active data-[state=active]:text-nav-active-foreground data-[state=active]:shadow-none dark:text-nav-foreground/80 dark:hover:text-nav-foreground dark:data-[state=active]:bg-nav-active dark:data-[state=active]:text-nav-active-foreground dark:data-[state=active]:border-transparent after:hidden'
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				className="h-[calc(100dvh-2.5rem)] w-[calc(100vw-2.5rem)] max-w-none gap-0 overflow-visible border-0 p-0 sm:max-w-none"
			>
				<Tabs
					defaultValue="general"
					orientation="vertical"
					className="h-full min-h-0 gap-0 overflow-hidden rounded-[inherit]"
				>
					<aside className="flex w-[37%] shrink-0 justify-end bg-nav pt-[3.25rem] text-nav-foreground">
						<div className="flex w-[13.5rem] flex-col gap-3 pr-3">
							<DialogHeader className="flex-row items-baseline gap-2 px-2.5 text-left">
								<DialogTitle className="truncate font-normal text-nav-muted text-xs">
									{channel ? `# ${channel.name}` : 'Canal'}
								</DialogTitle>
								<DialogDescription className="whitespace-nowrap font-semibold text-nav-foreground text-xs">
									Paramètres du canal
								</DialogDescription>
							</DialogHeader>
							<TabsList
								variant="line"
								className="w-full flex-col items-stretch gap-0.5 p-0"
							>
								<TabsTrigger value="general" className={TRIGGER}>
									Vue d’ensemble
								</TabsTrigger>
								<RequirePermission permission="MANAGE_CHANNELS">
									<TabsTrigger value="permissions" className={TRIGGER}>
										Permissions
									</TabsTrigger>
								</RequirePermission>
								<RequirePermission permission="MANAGE_WEBHOOKS">
									<TabsTrigger value="webhooks" className={TRIGGER}>
										Intégrations
									</TabsTrigger>
								</RequirePermission>
							</TabsList>
							<RequirePermission permission="MANAGE_CHANNELS">
								<div className="mt-1 border-nav-border border-t pt-3">
									<DoubleClickConfirmButton
										variant="ghost"
										className="w-full justify-start px-2.5 text-red-300 hover:bg-red-500/15 hover:text-red-200"
										label="Supprimer le canal"
										disabled={isDeleting}
										confirmLabel="Confirmer la suppression"
										onConfirm={onConfirmDelete}
									/>
								</div>
							</RequirePermission>
						</div>
					</aside>

					<div className="min-h-0 min-w-0 flex-1 overflow-y-auto">
						<div className="flex items-start gap-9 py-[3.25rem] pl-9">
							<div className="w-full max-w-[37rem] min-w-0">
								<TabsContent value="general" className="flex flex-col gap-6">
									<h2 className="font-semibold text-lg">Vue d’ensemble</h2>
									<div className="flex flex-col gap-1.5">
										<Label htmlFor="channel-name">Nom du canal</Label>
										<Input
											id="channel-name"
											value={nameDraft}
											onChange={(event) => onChangeName(event.target.value)}
										/>
									</div>
									<div className="flex flex-col gap-1.5">
										<Label htmlFor="channel-topic">Sujet du canal</Label>
										<Textarea
											id="channel-topic"
											rows={5}
											placeholder="Apprends à tout le monde comment utiliser ce canal !"
											value={topicDraft}
											onChange={(event) => onChangeTopic(event.target.value)}
										/>
									</div>
									<div className="flex flex-col gap-1.5">
										<Label htmlFor="channel-category">Catégorie</Label>
										<Select
											value={categoryIdDraft ?? NO_CATEGORY_VALUE}
											onValueChange={(value) =>
												onChangeCategory(
													value === NO_CATEGORY_VALUE ? null : value,
												)
											}
										>
											<SelectTrigger id="channel-category" className="w-full">
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
									<section className="flex flex-col gap-3 border-t pt-6">
										<h3 className="font-semibold text-sm">Informations</h3>
										{kind ? (
											<div className="flex items-start gap-3 rounded-lg border bg-card p-3">
												<kind.icon className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
												<div className="min-w-0">
													<p className="font-medium text-sm">
														Canal {kind.title.toLowerCase()}
													</p>
													<p className="text-muted-foreground text-xs">
														{kind.description}
													</p>
												</div>
											</div>
										) : null}
										<div className="flex items-start gap-3 rounded-lg border bg-card p-3">
											{isPrivate ? (
												<Lock className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
											) : (
												<Globe className="mt-0.5 size-5 shrink-0 text-muted-foreground" />
											)}
											<div className="min-w-0">
												<p className="font-medium text-sm">
													{isPrivate ? 'Canal privé' : 'Canal public'}
												</p>
												<p className="text-muted-foreground text-xs">
													{isPrivate
														? 'Seuls les rôles et membres autorisés le voient. Gère-les dans Permissions.'
														: 'Tous les membres de l’organisation le voient.'}
												</p>
											</div>
										</div>
										{createdOn ? (
											<p className="text-muted-foreground text-xs">
												Créé le {createdOn}
											</p>
										) : null}
									</section>
									<RequirePermission permission="MANAGE_CHANNELS">
										<Button
											type="button"
											className="self-start"
											onClick={onSaveGeneral}
											disabled={isSavingGeneral}
										>
											{isSavingGeneral ? (
												<Loader2 className="size-4 animate-spin" />
											) : null}
											Enregistrer
										</Button>
									</RequirePermission>
								</TabsContent>

								<TabsContent
									value="permissions"
									className="flex flex-col gap-6"
								>
									<ChannelPermissionsSectionUI
										isLoading={isLoadingOverwrites}
										isPrivate={isPrivate}
										onTogglePrivate={onTogglePrivate}
										targets={permissionTargets}
										selectedKey={selectedPermissionKey}
										onSelectKey={onSelectPermissionKey}
										addableRoles={addableRoles}
										addableMembers={addableMembers}
										onAddTarget={onAddTarget}
										onChangeBit={onChangeTargetBit}
										onRemoveTarget={onRemoveTarget}
									/>
								</TabsContent>

								<TabsContent value="webhooks" className="flex flex-col gap-6">
									<ChannelWebhooksSectionUI
										channelName={channel?.name}
										webhooks={webhooks}
										isLoading={isLoadingWebhooks}
										creatorName={creatorName}
										newWebhookName={newWebhookName}
										onChangeNewWebhookName={onChangeNewWebhookName}
										onCreate={onCreateWebhook}
										isCreating={isCreatingWebhook}
										createdToken={createdWebhookToken}
										callUrlBase={webhookCallUrlBase}
										createdCallUrl={createdWebhookCallUrl}
										onDismissCreatedToken={onDismissCreatedToken}
										onRename={onRenameWebhook}
										onDelete={onDeleteWebhook}
									/>
								</TabsContent>
							</div>
						</div>
					</div>
				</Tabs>
				<DialogClose asChild>
					<button
						type="button"
						aria-label="Fermer"
						className="absolute top-4 right-4 z-10 flex flex-col items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground"
					>
						<span className="flex size-8 items-center justify-center rounded-full border">
							<X className="size-4" />
						</span>
						ESC
					</button>
				</DialogClose>
			</DialogContent>
		</Dialog>
	)
}
